use std::{collections::HashMap, path::Path};

use lopdf::{Document, Object};
use regex::Regex;

use crate::annotation::symbol::SPACE;

use super::{
    content::{Annotation, PageDescription},
    decoder::decode_gb18030,
    kind::{annotation_kind, AnnotationKind},
    symbol::{
        COLOR, CONTENTS, CR, DATEPART, EQUAL_SIGN, IN, NL, SLASH, SLASH_WITH_BLANK, TESTCD,
        TIMEPART, WHEN,
    },
};

pub struct AnnotationFetcher {
    page_domain_map: HashMap<String, String>,
    current_page: usize,
    current_domain_id: String,
    annotation_map: HashMap<String, Annotation>,
}

impl AnnotationFetcher {
    pub fn new() -> AnnotationFetcher {
        AnnotationFetcher {
            page_domain_map: HashMap::new(),
            annotation_map: HashMap::new(),
            current_page: 0,
            current_domain_id: String::new(),
        }
    }
    pub fn fetch(&mut self, filepath: &Path) -> anyhow::Result<()> {
        let pdf = Document::load(filepath)?;
        for (key, page_id) in pdf.page_iter().enumerate() {
            if key.eq(&0) {
                continue;
            }
            self.page_domain_map.clear();
            self.current_page = key + 1;
            let mut annotations = vec![];
            let page_annotations = pdf.get_page_annotations(page_id);
            for page_annotation in page_annotations {
                // get color property as domain id in this page
                if let Ok(color) = page_annotation.get(COLOR) {
                    let color = color
                        .as_array()?
                        .into_iter()
                        .map(|n| n.as_float().unwrap())
                        .collect::<Vec<f32>>();

                    self.current_domain_id = domain_id(&color)
                }
                // get annotation contents
                if let Ok(object) = page_annotation.get(&CONTENTS) {
                    self.object_to_annotations(&object)?
                        .into_iter()
                        .for_each(|anno| annotations.push(anno));
                }
            }
            // assign domain and id for annotations which did not own domain and id
            for annotation in annotations
                .into_iter()
                .map(|mut anno| {
                    if anno.id.is_empty() {
                        let domain = self.page_domain_map.get(&anno.domain_id);
                        if domain.is_none() {
                            return anno;
                        }
                        let domain = domain.unwrap();
                        let domain = if anno.supp {
                            format!("SUPP{}", domain)
                        } else {
                            domain.to_owned()
                        };
                        let id = format!("{}-{}", domain.trim(), anno.variable.trim());
                        anno.domain = domain.clone();
                        anno.id = id;
                    }
                    anno
                })
                .collect::<Vec<_>>()
                .into_iter()
            {
                let id = annotation.id.as_str();
                if id.is_empty() {
                    // TODO
                    continue;
                }
                if let Some(old_annotations) = self.annotation_map.get(id) {
                    let mut old_annotations = old_annotations.clone();
                    if let Some(mut last_page_description) = old_annotations.page_description.pop()
                    {
                        // new annotation must have first page description, unwrap directly
                        let current_description = annotation.page_description.first().unwrap();
                        // new page
                        if last_page_description.page.ne(&current_description.page) {
                            old_annotations.page_description.push(last_page_description);
                            old_annotations
                                .page_description
                                .push(current_description.clone());
                            self.annotation_map.insert(id.to_string(), old_annotations);
                            continue;
                        }
                        // same page
                        current_description.description.iter().for_each(|content| {
                            if !last_page_description.has_description_in_same_page(&content) {
                                last_page_description.description.push(content.to_string());
                            }
                        });
                        old_annotations.page_description.push(last_page_description);
                        self.annotation_map.insert(id.to_string(), old_annotations);
                    }
                } else {
                    // new variable
                    self.annotation_map.insert(id.to_string(), annotation);
                }
            }
        }
        Ok(())
    }

    /// export annotation result as vec
    pub fn annotations(&self) -> Vec<Annotation> {
        let mut annotations = Vec::with_capacity(self.annotation_map.len());
        for id in self.annotation_map.keys() {
            let annotation = self.annotation_map.get(id).unwrap();
            annotations.push(annotation.clone());
        }
        annotations.sort_by_key(|annotation| annotation.id.clone());
        annotations
    }

    /// handle a pdf object into annotation
    fn object_to_annotations(&mut self, object: &Object) -> anyhow::Result<Vec<Annotation>> {
        let raw = object.as_str().unwrap();
        let raw = decode_gb18030(raw)
            .trim()
            .replace(NL, SPACE)
            .replace(CR, SPACE);
        if let None = self.page_domain_map.get(&self.current_domain_id) {
            let domain_pattern_1 = Regex::new(r"^([A-Z]{2,6})\s?\(.*?\)").unwrap();
            let domain_pattern_2 = Regex::new(r"^([A-Z]{2}|RELREC)\s?=").unwrap();
            if let Some(captures) = domain_pattern_1.captures(&raw) {
                self.page_domain_map.insert(
                    self.current_domain_id.clone(),
                    captures.get(1).unwrap().as_str().to_string(),
                );
            } else if let Some(captures) = domain_pattern_2.captures(&raw) {
                self.page_domain_map.insert(
                    self.current_domain_id.clone(),
                    captures.get(1).unwrap().as_str().to_string(),
                );
            }
        }
        match annotation_kind(&raw) {
            AnnotationKind::Main => Ok(self.main_annotation(&raw)),
            AnnotationKind::Supp => Ok(self.supp_annotation(&raw)),
            AnnotationKind::Other => Ok(vec![]),
        }
    }
    /// extract main information from annotation such as:
    ///
    /// "AESTDTC"
    ///
    /// "LBTEST = Erythrocytes"
    ///
    /// "VSORRES when VSTESTCD = TEMP"
    ///
    /// "MISTAT = NOT DONE when MITESTCD = MIALL"
    ///
    /// "TRORRES / TRORRESU when TRTESTCD = SUMDIAM"
    ///
    /// "If Normal then LBNRIND1 = NORMAL"
    ///
    /// "DSTERM/DSDECOD  = ENTERED INTO TRIAL when DSCAT = PROTOCOL MILESTONE"
    ///
    /// "Datepart of ECSTDTC"
    ///
    /// "Timepart of ECSTDTC"
    ///
    /// "DSSTDTC when DSTERM/DSDECOD=知情同意签署"
    ///
    /// "DSTERM / DSDECOD = ENTERED INTO TRIAL"
    ///
    /// "PETESTCD = PEALL / PESTAT = NOT DONE when No"
    fn main_annotation(&self, raw: &str) -> Vec<Annotation> {
        let mut annotations = vec![];
        let mut testcd = vec![];

        // try to split raw content by "when"
        let raw_split = raw.split(WHEN).collect::<Vec<&str>>();

        let part_0 = raw_split.first().unwrap();
        let description = raw_split.get(1);

        // try to split by equal to separate variable names and values
        // let part_0_list = part_0.split(EQUAL_SIGN).collect::<Vec<&str>>();

        // try to split by slash, build into Vec<Option<Some(variable name), Some(value)>>
        let mut part_0_list = part_0
            .split(SLASH)
            .map(|item| {
                let variable_value = item
                    .split(EQUAL_SIGN)
                    .map(|item| item.trim().to_owned())
                    .collect::<Vec<String>>();
                let variable = variable_value.first();
                let value = variable_value.get(1);
                (
                    match variable {
                        Some(s) => Some(s.to_owned()),
                        None => None,
                    },
                    match value {
                        Some(s) => Some(s.to_owned()),
                        None => None,
                    },
                )
            })
            .collect::<Vec<(Option<String>, Option<String>)>>();

        // handle "If XXX then" prefix
        let re_if_then = Regex::new(r"If\s\w+?\sthen\s(.*)").unwrap();

        // handle none value situtation
        for i in 0..part_0_list.len() {
            let (name, value) = part_0_list.get(i).unwrap();
            if name.is_none() {
                continue;
            }
            let name = name.clone().unwrap();

            let name = match re_if_then.captures(&name) {
                Some(catpures) => catpures.get(1).unwrap().as_str().to_string(),
                None => name,
            };
            let name = name.replace(DATEPART, "").replace(TIMEPART, "");

            // handle when variable name is a part of value, such as "EXCLUSION CRITERIA"
            if i.gt(&0) && (name.len().gt(&8) || contains_chinese_char(&name)) {
                if let Some(last_item) = part_0_list.get(i - 1) {
                    if let Some(value) = last_item.1.clone() {
                        part_0_list[i - 1].1 = Some(format!("{} / {}", &value, &name));
                        part_0_list[i].0 = None;
                        continue;
                    }
                }
            }

            if value.is_none() {
                if let Some(next_item) = part_0_list.get(i + 1) {
                    if let Some(value) = next_item.1.clone() {
                        part_0_list[i].1 = Some(value);
                    }
                }
            }
        }

        part_0_list.iter().for_each(|variable| {
            let (name, value) = variable;
            if name.is_none() {
                return;
            }

            let name = name.clone().unwrap();

            // let name = match re_if_then.captures(&name) {
            //     Some(catpures) => catpures.get(1).unwrap().as_str().to_string(),
            //     None => name,
            // };

            // // handle datepart and timepart prefix
            // let name = name.replace(DATEPART, "").replace(TIMEPART, "");

            let domain = if let Some(domain) = self.page_domain_map.get(&self.current_domain_id) {
                domain.to_owned()
            } else {
                "".to_owned()
            };
            let mut descriptions = vec![];
            if let Some(value) = value {
                descriptions.push(format!("{} = {}", name.trim(), value.trim()));
            }
            if let Some(description) = description {
                let variable_value = description.split(EQUAL_SIGN).collect::<Vec<&str>>();
                if variable_value.len().gt(&1) {
                    let variables = variable_value[0].trim();

                    let value = variable_value[1].trim();

                    variables
                        .split(SLASH)
                        .collect::<Vec<&str>>()
                        .iter()
                        .for_each(|variable| {
                            if variable.ends_with(TESTCD) {
                                testcd.push((
                                    variable.to_string(),
                                    format!("{} = {}", variable, value),
                                ));
                            }
                            descriptions.push(format!("{} = {}", variable, value))
                        });
                } else {
                    descriptions.push(description.to_string());
                }
            }
            let id = if !domain.is_empty() {
                format!("{}-{}", domain.trim(), &name)
            } else {
                "".to_owned()
            };
            annotations.push(Annotation {
                id,
                domain: domain.clone(),
                domain_id: self.current_domain_id.clone(),
                variable: name,
                page_description: vec![PageDescription {
                    page: self.current_page,
                    description: descriptions,
                }],
                supp: false,
                raw: raw.into(),
            });
            testcd.iter().for_each(|(variable, description)| {
                let id = if !domain.is_empty() {
                    format!("{}-{}", domain.trim(), &variable)
                } else {
                    "".to_owned()
                };
                annotations.push(Annotation {
                    id,
                    domain: domain.clone(),
                    domain_id: self.current_domain_id.clone(),
                    variable: variable.clone(),
                    page_description: vec![PageDescription {
                        page: self.current_page,
                        description: vec![description.clone()],
                    }],
                    supp: false,
                    raw: raw.into(),
                });
            });
        });
        annotations
    }
    /// extract supp information from annotation such as:
    ///
    ///  "AESI in SUPPAE"
    ///
    /// "PECLSIG=N in SUPPPE"
    ///
    /// "If Normal then LBNRIND1 = NORMAL in SUPPLB"
    ///
    /// "DDORRES in SUPPDD when DDTESTCD = PRCDTH"
    ///
    /// "TRNEREA in SUPPTR when TRTESTCD = LDIAM/LPERP"
    fn supp_annotation(&self, raw: &str) -> Vec<Annotation> {
        let mut annotations = vec![];

        // handle situation with when xxx=xxx
        let raw_split_by_when = raw.split(WHEN).collect::<Vec<&str>>();

        let when_description = raw_split_by_when.get(1);

        let raw_split = raw_split_by_when
            .first()
            .unwrap()
            .split(IN)
            .collect::<Vec<&str>>();
        if raw_split.len() < 2 {
            return annotations;
        }
        let domain = if let Some(domain) = self.page_domain_map.get(&self.current_domain_id) {
            format!("SUPP{}", domain.to_owned())
        } else {
            "".to_owned()
        };
        // check if in description mode
        let part_0 = raw_split
            .first()
            .unwrap()
            .split(EQUAL_SIGN)
            .collect::<Vec<&str>>();
        let value_description = part_0.get(1);
        let variables = part_0.first().unwrap();
        // handle "If XXX then" prefix
        let re_if_then = Regex::new(r"If\s\w+?\sthen\s(.*)").unwrap();
        let variables = match re_if_then.captures(&variables) {
            Some(catpures) => catpures.get(1).unwrap().as_str(),
            None => variables,
        };
        variables
            .replace(SLASH_WITH_BLANK, SLASH)
            .split(SLASH)
            .collect::<Vec<&str>>()
            .iter()
            .for_each(|variable| {
                let domain = domain.clone();
                let mut descriptions = vec![];
                if let Some(value) = value_description {
                    descriptions.push(format!("{} = {}", variable.trim(), value.trim()));
                }
                if let Some(value) = when_description {
                    descriptions.push(value.to_string());
                }
                let id = if !domain.is_empty() {
                    format!("{}-{}", domain.trim(), variable.trim())
                } else {
                    "".to_owned()
                };
                annotations.push(Annotation {
                    id,
                    domain,
                    domain_id: self.current_domain_id.clone(),
                    variable: variable.trim().to_string(),
                    page_description: vec![PageDescription {
                        page: self.current_page,
                        description: descriptions,
                    }],
                    supp: true,
                    raw: raw.into(),
                });
            });

        annotations
    }
}

pub fn fetch(filepath: &Path) -> anyhow::Result<Vec<Annotation>> {
    let mut fetcher = AnnotationFetcher::new();
    fetcher.fetch(filepath)?;
    Ok(fetcher.annotations())
}

fn domain_id(pattern: &[f32]) -> String {
    let mut id = String::new();
    for i in 0..3 {
        if let Some(element) = pattern.get(i) {
            if element.eq(&1.0) {
                id.push_str("1")
            } else {
                id.push_str("0")
            }
        }
    }
    id
}

fn contains_chinese_char(sample: &str) -> bool {
    if sample.chars().any(|c| c > '\u{7F}') {
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_annotation() -> anyhow::Result<()> {
        let acrf = Path::new(r"D:\projects\rusty\acrf\105-302.pdf");
        let mut fetcher = AnnotationFetcher::new();
        fetcher.fetch(&acrf)?;
        let result = fetcher.annotations();
        result.iter().for_each(|a| {
            println!("{:?}", a);
        });
        Ok(())
    }
}

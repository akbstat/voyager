use std::{collections::HashMap, path::Path};

use lopdf::{Document, Object};
use regex::Regex;

use super::{
    content::{Annotation, PageDescription},
    decoder::decode_gb18030,
    kind::{annotation_kind, AnnotationKind},
    symbol::{
        CONTENTS, CR, DATEPART, DM, DM_VARIABLES, EQUAL_SIGN, IN, NL, RELREC, SLASH,
        SLASH_WITH_BLANK, TIMEPART, TR, VISIT, WHEN,
    },
};

pub fn fetch(filepath: &Path) -> anyhow::Result<Vec<Annotation>> {
    let mut annotations = vec![];
    let mut annotation_map: HashMap<String, Annotation> = HashMap::new();

    let pdf = Document::load(filepath)?;
    for (key, page_id) in pdf.page_iter().enumerate() {
        if key.eq(&0) {
            continue;
        }
        let page_number = key + 1;
        let page_annotations = pdf.get_page_annotations(page_id);
        for page_annotation in page_annotations {
            if let Ok(object) = page_annotation.get(&CONTENTS) {
                let annotations = object_to_annotations(&object, page_number)?;
                for annotation in annotations.into_iter() {
                    let id = annotation.id.as_str();
                    if let Some(old_annotations) = annotation_map.get(id) {
                        let mut old_annotations = old_annotations.clone();
                        if let Some(mut last_page_description) =
                            old_annotations.page_description.pop()
                        {
                            // new annotation must have first page description, unwrap directly
                            let current_description = annotation.page_description.first().unwrap();
                            // new page
                            if last_page_description.page.ne(&current_description.page) {
                                old_annotations.page_description.push(last_page_description);
                                old_annotations
                                    .page_description
                                    .push(current_description.clone());
                                annotation_map.insert(id.to_string(), old_annotations);
                                continue;
                            }
                            // same page
                            current_description.description.iter().for_each(|content| {
                                if !last_page_description.has_description_in_same_page(&content) {
                                    last_page_description.description.push(content.to_string());
                                }
                            });
                            old_annotations.page_description.push(last_page_description);
                            annotation_map.insert(id.to_string(), old_annotations);
                        }
                    } else {
                        // new variable
                        annotation_map.insert(id.to_string(), annotation);
                    }
                }
            }
        }
    }
    for id in annotation_map.keys() {
        let annotation = annotation_map.get(id).unwrap();
        annotations.push(annotation.clone());
    }
    annotations.sort_by_key(|annotation| annotation.id.clone());
    Ok(annotations)
}

fn object_to_annotations(object: &Object, page: usize) -> anyhow::Result<Vec<Annotation>> {
    let raw = object.as_str().unwrap();
    let raw = decode_gb18030(raw).trim().replace(NL, " ").replace(CR, " ");
    match annotation_kind(&raw) {
        AnnotationKind::Main => Ok(main_annotation(&raw, page)),
        AnnotationKind::Supp => Ok(supp_annotation(&raw, page)),
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
fn main_annotation(raw: &str, page: usize) -> Vec<Annotation> {
    let mut annotations = vec![];
    // try to split raw content by "when"
    let raw_split = raw.split(WHEN).collect::<Vec<&str>>();

    let part_0 = raw_split.first().unwrap();
    let description = raw_split.get(1);

    // try to split by equal to separate variable names and values
    let part_0_list = part_0.split(EQUAL_SIGN).collect::<Vec<&str>>();
    let variables = part_0_list.first().unwrap();

    // handle "If XXX then" prefix
    let re_if_then = Regex::new(r"If\s\w+?\sthen\s(.*)").unwrap();
    let variables = match re_if_then.captures(&variables) {
        Some(catpures) => catpures.get(1).unwrap().as_str(),
        None => variables,
    };

    let value_description = part_0_list.get(1);
    // handle mulitple variable declares in annotations, such as LBORRES / LBORRESU
    variables
        .replace(SLASH_WITH_BLANK, SLASH)
        .split(SLASH)
        .collect::<Vec<&str>>()
        .iter()
        .for_each(|variable| {
            let variable = variable.trim();
            // handle datepart and timepart prefix
            let variable = variable.replace(DATEPART, "").replace(TIMEPART, "");

            let domain = if let Some(domain_name) = variable.get(..2) {
                domain(domain_name, &variable)
            } else {
                return;
            };
            let mut descriptions = vec![];
            if let Some(value) = value_description {
                descriptions.push(format!("{} = {}", variable.trim(), value.trim()));
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
                            descriptions.push(format!("{} = {}", variable, value))
                        });
                } else {
                    descriptions.push(description.to_string());
                }
            }
            annotations.push(Annotation {
                id: format!("{}-{}", domain.trim(), variable.trim()).into(),
                domain,
                variable: variable.trim().to_string(),
                page_description: vec![PageDescription {
                    page,
                    description: descriptions,
                }],
                raw: raw.into(),
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
fn supp_annotation(raw: &str, page: usize) -> Vec<Annotation> {
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
    let domain = raw_split.last().unwrap().trim().to_string();
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
            annotations.push(Annotation {
                id: format!("{}-{}", domain, variable).into(),
                domain,
                variable: variable.trim().to_string(),
                page_description: vec![PageDescription {
                    page,
                    description: descriptions,
                }],
                raw: raw.into(),
            });
        });

    annotations
}

/// handle special situation that first two alphabet does not stands for domain information,
///
/// for example: AGE in DM domain
fn domain(origin: &str, variable: &str) -> String {
    if DM_VARIABLES.contains(&variable.trim()) {
        return DM.into();
    }
    if variable.eq(VISIT) {
        return TR.into();
    }
    if variable.eq(RELREC) {
        return RELREC.into();
    }
    origin.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_annotation() {
        let acrf = Path::new(r"D:\projects\rusty\acrf\104-215.pdf");
        let result = fetch(acrf);
        result.unwrap().iter().for_each(|a| {
            println!("{:?}", a);
        })
    }

    #[test]
    fn test_fetch_main() {
        let page = 1;
        let raw = "AGE";
        let annotation = main_annotation(raw, page);
        assert_eq!("AGE", annotation[0].variable);
        assert_eq!("DM", annotation[0].domain);
        let raw = "LBCAT = HEMATOLOGY";
        let annotation = main_annotation(raw, page);
        assert_eq!("LBCAT", annotation[0].variable);
        assert_eq!("LB", annotation[0].domain);
        let raw = "VSCAT=身高/体重";
        let annotation = main_annotation(raw, page);
        assert_eq!("VSCAT", annotation[0].variable);
        assert_eq!("VS", annotation[0].domain);
        let raw = "VSORRES when VSTESTCD = TEMP";
        let annotation = main_annotation(raw, page);
        assert_eq!("VSORRES", annotation[0].variable);
        assert_eq!("VS", annotation[0].domain);
        let raw = "MISTAT = NOT DONE when MITESTCD = MIALL";
        let annotation = main_annotation(raw, page);
        assert_eq!("MISTAT", annotation[0].variable);
        assert_eq!("MI", annotation[0].domain);
        let raw = "DSSTDTC when DSDECOD=知情同意签署";
        let annotation = main_annotation(raw, page);
        assert_eq!("DSSTDTC", annotation[0].variable);
        assert_eq!("DS", annotation[0].domain);
        let raw = "TRORRES / TRORRESU / TRORRESX when TRTESTCD = SUMDIAM";
        let annotation = main_annotation(raw, page);
        assert_eq!("TRORRES", annotation[0].variable);
        assert_eq!("TR", annotation[0].domain);
        assert_eq!("TRORRESU", annotation[1].variable);
        assert_eq!("TR", annotation[1].domain);
        assert_eq!("TRORRESX", annotation[2].variable);
        assert_eq!("TR", annotation[2].domain);
        let raw = "If Abnormal then LBNRIND1 / LBNRIND2 = ABNORMAL";
        let annotation = main_annotation(raw, page);
        assert_eq!("LBNRIND1", annotation[0].variable);
        assert_eq!("LB", annotation[0].domain);
        assert_eq!("LBNRIND2", annotation[1].variable);
        assert_eq!("LB", annotation[1].domain);
        let raw = "DSTERM/DSDECOD  = ENTERED INTO TRIAL when DSCAT = PROTOCOL MILESTONE";
        let annotation = main_annotation(raw, page);
        assert_eq!("DSTERM", annotation[0].variable);
        assert_eq!("DS", annotation[0].domain);
        assert_eq!("DSDECOD", annotation[1].variable);
        assert_eq!("DS", annotation[1].domain);
        let raw = "Datepart of ECSTDTC";
        let annotation = main_annotation(raw, page);
        assert_eq!("ECSTDTC", annotation[0].variable);
        assert_eq!("EC", annotation[0].domain);
        let raw = "Timepart of ECSTDTC";
        let annotation = main_annotation(raw, page);
        assert_eq!("ECSTDTC", annotation[0].variable);
        assert_eq!("EC", annotation[0].domain);
        let raw = "DTHFL = Y";
        let annotation = main_annotation(raw, page);
        assert_eq!("DTHFL", annotation[0].variable);
        assert_eq!("DM", annotation[0].domain);
        let raw = "DSSTDTC when DSTERM/DSDECOD=知情同意签署";
        let annotation = main_annotation(raw, page);
        assert_eq!("DSSTDTC", annotation[0].variable);
        assert_eq!("DS", annotation[0].domain);
        assert_eq!(2, annotation[0].page_description[0].description.len());
        let raw = "DSTERM /\nDSDECOD = ENTERED INTO TRIAL";
        let annotation = main_annotation(raw, page);
        assert_eq!("DSTERM", annotation[0].variable);
        assert_eq!("DS", annotation[0].domain);
        assert_eq!(1, annotation[0].page_description[0].description.len());
        assert_eq!("DSDECOD", annotation[1].variable);
        assert_eq!("DS", annotation[1].domain);
    }

    #[test]
    fn test_fetch_supp() {
        let page = 1;
        let raw = "PECLSIG=N in SUPPPE";
        let annotation = supp_annotation(raw, page);
        assert_eq!("PECLSIG", annotation[0].variable);
        assert_eq!("SUPPPE", annotation[0].domain);
        let raw = "AESI in SUPPAE";
        let annotation = supp_annotation(raw, page);
        assert_eq!("AESI", annotation[0].variable);
        assert_eq!("SUPPAE", annotation[0].domain);
        let raw = "If Normal then LBNRIND1 / LBNRIND2 = NORMAL in SUPPLB";
        let annotation = supp_annotation(raw, page);
        assert_eq!("LBNRIND1", annotation[0].variable);
        assert_eq!("SUPPLB", annotation[0].domain);
        let raw = "DDORRES in SUPPDD when DDTESTCD = PRCDTH";
        let annotation = supp_annotation(raw, page);
        assert_eq!("DDORRES", annotation[0].variable);
        assert_eq!("SUPPDD", annotation[0].domain);
        let raw = "TRNEREA in SUPPTR when TRTESTCD = LDIAM/LPERP";
        let annotation = supp_annotation(raw, page);
        assert_eq!("TRNEREA", annotation[0].variable);
        assert_eq!("SUPPTR", annotation[0].domain);
    }
}

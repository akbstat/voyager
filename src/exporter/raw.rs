use std::collections::HashMap;

use crate::Annotation;

use super::utils::EMPTY_CELL;

const HEADERS: &[&str] = &["Domain", "Variable", "Description", "Pages"];

#[derive(Debug, Clone)]
pub struct Raw {
    pub id: String,
    pub dataset: String,
    pub name: String,
    pub description: String,
    pub page: Vec<usize>,
}
pub struct RawSet {
    data: HashMap<String, Raw>,
}

impl RawSet {
    pub fn new() -> RawSet {
        RawSet {
            data: HashMap::new(),
        }
    }
    pub fn add_annotations(&mut self, annotations: &[Annotation]) {
        annotations
            .iter()
            .for_each(|annotation| self.add(annotation));
    }
    fn add(&mut self, annotation: &Annotation) {
        annotation.page_description.iter().for_each(|desc| {
            if desc.description.is_empty() {
                let id = format!("{}", annotation.id);
                let item = if let Some(item) = self.data.get_mut(&id) {
                    item.page.push(desc.page);
                    item.clone()
                } else {
                    Raw {
                        id: id.to_owned(),
                        dataset: annotation.domain.to_owned(),
                        name: annotation.variable.to_owned(),
                        description: EMPTY_CELL.to_owned(),
                        page: vec![desc.page],
                    }
                };
                self.data.insert(id, item);
            } else {
                desc.description.iter().for_each(|value| {
                    let id = format!("{}-{}", annotation.id, value);
                    let item = if let Some(item) = self.data.get_mut(&id) {
                        item.page.push(desc.page);
                        item.clone()
                    } else {
                        Raw {
                            id: id.to_owned(),
                            dataset: annotation.domain.to_owned(),
                            name: annotation.variable.to_owned(),
                            description: value.to_owned(),
                            page: vec![desc.page],
                        }
                    };
                    self.data.insert(id, item);
                });
            }
        })
    }
    pub fn export(&self) -> Vec<Vec<String>> {
        let mut data = self
            .data
            .iter()
            .map(|(_, variable)| {
                vec![
                    variable.dataset.clone(),
                    variable.name.clone(),
                    variable.description.clone().into(),
                    variable
                        .page
                        .iter()
                        .map(|p| format!("{}", p))
                        .collect::<Vec<String>>()
                        .join(" "),
                ]
            })
            .collect::<Vec<Vec<String>>>();
        data.sort_by_key(|item| (item[0].clone(), item[1].clone()));
        data.insert(0, header());
        data
    }
}

fn header() -> Vec<String> {
    HEADERS.iter().map(|header| header.to_string()).collect()
}

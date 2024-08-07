use std::collections::{HashMap, HashSet};

use crate::{
    exporter::utils::{CRF, EMPTY_CELL},
    Annotation,
};

use super::utils::{qval_annotation, RELREC, SUPP};

const HEADERS: &[&str] = &[
    "Order  ",
    "Dataset    ",
    "Variable	",
    "Label	",
    "Data Type	",
    "Length	",
    "Significant Digits	",
    "Format	",
    "Mandatory	",
    "Assigned Value	",
    "Codelist	",
    "Common	",
    "Origin	",
    "Source	",
    "Pages  ",
];

#[derive(Debug, Clone)]
pub struct Variable {
    pub id: String,
    pub dataset: String,
    pub name: String,
    pub page: Vec<usize>,
}
pub struct VariableSet {
    data: HashMap<String, Variable>,
}

impl VariableSet {
    pub fn new() -> VariableSet {
        VariableSet {
            data: HashMap::new(),
        }
    }
    pub fn add_annotations(&mut self, annotations: &[Annotation]) {
        annotations
            .iter()
            .for_each(|annotation| self.add(annotation));
    }

    fn add(&mut self, annotation: &Annotation) {
        let is_supp = annotation.domain.starts_with(SUPP);
        // if supp, then regard its variable name as QNAM and QVAL
        if is_supp {
            self.add_supp_variable(annotation);
        } else {
            self.add_variable(annotation);
        }
    }

    fn add_supp_variable(&mut self, annotation: &Annotation) {
        // self.add_variable(&qnam_annotation(annotation));
        self.add_variable(&qval_annotation(annotation));
    }

    fn add_variable(&mut self, annotation: &Annotation) {
        // filter out relrec
        if annotation.variable.eq(RELREC) {
            return;
        }
        let id = annotation.id.clone();
        let new_pages = annotation
            .page_description
            .iter()
            .map(|page| page.page)
            .collect::<Vec<usize>>();
        if let Some(variable) = self.data.get(&id) {
            let mut page_set = HashSet::with_capacity(variable.page.len());
            variable.page.iter().for_each(|page| {
                page_set.insert(*page);
            });
            new_pages.iter().for_each(|page| {
                page_set.insert(*page);
            });
            let mut pages = page_set
                .into_iter()
                .map(|page| page)
                .collect::<Vec<usize>>();
            pages.sort();
            self.data.insert(
                id.clone(),
                Variable {
                    id,
                    dataset: annotation.domain.clone(),
                    name: annotation.variable.clone(),
                    page: pages,
                },
            );
        } else {
            self.data.insert(
                id.clone(),
                Variable {
                    id,
                    dataset: annotation.domain.clone(),
                    name: annotation.variable.clone(),
                    page: new_pages,
                },
            );
        }
    }

    pub fn export(&self) -> Vec<Vec<String>> {
        let mut data = self
            .data
            .iter()
            .map(|(_, variable)| {
                vec![
                    EMPTY_CELL.into(),        // Order
                    variable.dataset.clone(), // Dataset
                    variable.name.clone(),    // Variable
                    EMPTY_CELL.into(),        // Label
                    EMPTY_CELL.into(),        // Data Type
                    EMPTY_CELL.into(),        // Length
                    EMPTY_CELL.into(),        // Significant Digits
                    EMPTY_CELL.into(),        // Format
                    EMPTY_CELL.into(),        // Mandatory
                    EMPTY_CELL.into(),        // Assigned Value
                    EMPTY_CELL.into(),        // Codelist
                    EMPTY_CELL.into(),        // Common
                    CRF.into(),               // Origin
                    EMPTY_CELL.into(),        // Source
                    variable
                        .page
                        .iter()
                        .map(|p| format!("{}", p))
                        .collect::<Vec<String>>()
                        .join(" "),
                ]
            })
            .collect::<Vec<Vec<String>>>();
        data.sort_by_key(|item| (item[1].clone(), item[2].clone()));
        data.insert(0, header());
        data
    }
}

fn header() -> Vec<String> {
    HEADERS.iter().map(|header| header.to_string()).collect()
}

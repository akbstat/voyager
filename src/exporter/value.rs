use std::collections::HashMap;

use crate::{
    exporter::utils::{CRF, EMPTY_CELL, EQ, EQ_SYMBOL},
    Annotation,
};

use super::utils::{qval_annotation, ORRES, SUPP, TESTCD};

const HEADERS: &[&str] = &[
    "Order	",
    "Dataset	",
    "Variable	",
    "Where Clause	",
    "Label	",
    "Data Type	",
    "Length	",
    "Significant Digits	",
    "Format	",
    "Mandatory	",
    "Assigned Value	",
    "Codelist	",
    "Origin	",
    "Source	",
    "Pages	",
];

#[derive(Debug, Clone)]
pub struct Value {
    pub id: String,
    pub dataset: String,
    pub variable: String,
    pub description: String,
    pub page: Vec<usize>,
}

pub struct ValueSet {
    data: HashMap<String, Value>,
}

impl ValueSet {
    pub fn new() -> ValueSet {
        ValueSet {
            data: HashMap::new(),
        }
    }
    pub fn add_annotations(&mut self, annotations: &[Annotation]) {
        annotations
            .iter()
            .for_each(|annotation| self.add(annotation));
    }

    /// add annotations into set if annotation satisfy one of following rules:
    ///
    /// 1. --ORRESS, with --TESTCD qualification
    ///
    /// 2. --CAT, with --CAT equals xxx qualification
    ///
    /// 3. QVAL
    fn add(&mut self, annotation: &Annotation) {
        if annotation_need_process(annotation) {
            let is_supp = annotation.domain.starts_with(SUPP);
            // if supp, then regard its variable name as QNAM and QVAL
            if is_supp {
                self.add_supp_variable(annotation);
            } else {
                self.add_variable(annotation);
            }
        }
    }

    fn add_supp_variable(&mut self, annotation: &Annotation) {
        self.add_variable(&qval_annotation(annotation));
    }

    fn add_variable(&mut self, annotation: &Annotation) {
        annotation.page_description.iter().for_each(|desc| {
            if desc.description.is_empty() {
                let id = format!("{}", annotation.id);
                let item = if let Some(item) = self.data.get_mut(&id) {
                    item.page.push(desc.page);
                    item.clone()
                } else {
                    Value {
                        id: id.to_owned(),
                        dataset: annotation.domain.to_owned(),
                        variable: annotation.variable.to_owned(),
                        description: EMPTY_CELL.to_owned(),
                        page: vec![desc.page],
                    }
                };
                self.data.insert(id, item);
            } else {
                desc.description.iter().for_each(|value| {
                    // filter the description when XXORRESS = XXX
                    if annotation.variable.ends_with(ORRES) {
                        if !value.contains(TESTCD) {
                            return;
                        }
                    }

                    let id = format!("{}-{}", annotation.id, value);
                    let item = if let Some(item) = self.data.get_mut(&id) {
                        item.page.push(desc.page);
                        item.clone()
                    } else {
                        Value {
                            id: id.to_owned(),
                            dataset: annotation.domain.to_owned(),
                            variable: annotation.variable.to_owned(),
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
        let mut data = Vec::with_capacity(self.data.len());

        for (_, value) in self.data.iter() {
            if value.description.trim().is_empty() {
                continue;
            }
            data.push(vec![
                EMPTY_CELL.into(),                        // Order
                value.dataset.clone(),                    // Dataset
                value.variable.clone(),                   // Variable
                value.description.replace(EQ_SYMBOL, EQ), // Where Clause
                EMPTY_CELL.into(),                        // Label
                EMPTY_CELL.into(),                        // Data Type
                EMPTY_CELL.into(),                        // Length
                EMPTY_CELL.into(),                        // Significant Digits
                EMPTY_CELL.into(),                        // Format
                EMPTY_CELL.into(),                        // Mandatory
                EMPTY_CELL.into(),                        // Assigned Value
                EMPTY_CELL.into(),                        // Codelist
                CRF.into(),                               // Origin
                EMPTY_CELL.into(),                        // Source
                value
                    .page
                    .iter()
                    .map(|p| format!("{}", p))
                    .collect::<Vec<String>>()
                    .join(" "),
            ]);
        }

        // let mut data = self
        //     .data
        //     .iter()
        //     .map(|(_, value)| {
        //         vec![
        //             EMPTY_CELL.into(),                        // Order
        //             value.dataset.clone(),                    // Dataset
        //             value.variable.clone(),                   // Variable
        //             value.description.replace(EQ_SYMBOL, EQ), // Where Clause
        //             EMPTY_CELL.into(),                        // Label
        //             EMPTY_CELL.into(),                        // Data Type
        //             EMPTY_CELL.into(),                        // Length
        //             EMPTY_CELL.into(),                        // Significant Digits
        //             EMPTY_CELL.into(),                        // Format
        //             EMPTY_CELL.into(),                        // Mandatory
        //             EMPTY_CELL.into(),                        // Assigned Value
        //             EMPTY_CELL.into(),                        // Codelist
        //             CRF.into(),                               // Origin
        //             EMPTY_CELL.into(),                        // Source
        //             value
        //                 .page
        //                 .iter()
        //                 .map(|p| format!("{}", p))
        //                 .collect::<Vec<String>>()
        //                 .join(" "),
        //         ]
        //     })
        //     .collect::<Vec<Vec<String>>>();
        data.sort_by_key(|item| (item[1].clone(), item[2].clone()));
        data.insert(0, header());
        data
    }
}

fn annotation_need_process(annotation: &Annotation) -> bool {
    annotation.domain.starts_with(SUPP) || annotation.variable.ends_with(ORRES)
}

fn header() -> Vec<String> {
    HEADERS.iter().map(|header| header.to_string()).collect()
}

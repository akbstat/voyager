use std::{collections::HashMap, path::Path};

use rust_xlsxwriter::{Color, Format, Workbook};

use crate::Annotation;

use super::{
    raw::RawSet,
    utils::{RAW_SHEET_NAME, VALUE_SHEET_NAME, VARIABLE_SHEET_NAME},
    value::ValueSet,
    varaible::VariableSet,
};

const DEFAULT_FILE_NAME: &str = "result.xlsx";
const TIMES_NEW_ROMAN: &str = "Times New Roman";

#[derive(Debug, Clone)]
pub struct Item {
    // pub id: String,
    // pub dataset: String,
    // pub variable: String,
    // pub description: String,
    pub page: Vec<usize>,
}

pub struct Exporter {
    workbook: Workbook,
    items: HashMap<String, Item>,
    values: ValueSet,
    variables: VariableSet,
    raws: RawSet,
}

impl Exporter {
    pub fn new() -> Exporter {
        let workbook = Workbook::new();
        Exporter {
            workbook,
            items: HashMap::new(),
            values: ValueSet::new(),
            variables: VariableSet::new(),
            raws: RawSet::new(),
        }
    }
    pub fn add_annotations(&mut self, annotations: &[Annotation]) {
        self.values.add_annotations(annotations);
        self.variables.add_annotations(annotations);
        self.raws.add_annotations(annotations);
        annotations.iter().for_each(|anno| {
            anno.page_description.iter().for_each(|desc| {
                if desc.description.is_empty() {
                    let id = format!("{}", anno.id);
                    let item = if let Some(item) = self.items.get_mut(&id) {
                        item.page.push(desc.page);
                        item.clone()
                    } else {
                        Item {
                            // id: id.to_owned(),
                            // dataset: anno.domain.to_owned(),
                            // variable: anno.variable.to_owned(),
                            // description: "".to_owned(),
                            page: vec![desc.page],
                        }
                    };
                    self.items.insert(id, item);
                } else {
                    desc.description.iter().for_each(|value| {
                        let id = format!("{}-{}", anno.id, value);
                        let item = if let Some(item) = self.items.get_mut(&id) {
                            item.page.push(desc.page);
                            item.clone()
                        } else {
                            Item {
                                // id: id.to_owned(),
                                // dataset: anno.domain.to_owned(),
                                // variable: anno.variable.to_owned(),
                                // description: value.to_owned(),
                                page: vec![desc.page],
                            }
                        };
                        self.items.insert(id, item);
                    });
                }
            })
        });
    }

    pub fn save(&mut self, dest: &Path) -> anyhow::Result<()> {
        self.save_variable_sheet()?;
        self.save_value_sheet()?;
        self.save_raw_sheet()?;

        self.workbook.save(if dest.is_dir() {
            dest.join(DEFAULT_FILE_NAME)
        } else {
            dest.into()
        })?;
        Ok(())
    }

    fn save_variable_sheet(&mut self) -> anyhow::Result<()> {
        let header_format = Format::new()
            .set_background_color(Color::Orange)
            .set_bold()
            .set_font_name(TIMES_NEW_ROMAN);
        let item_format = Format::new().set_font_name(TIMES_NEW_ROMAN);
        let worksheet = self.workbook.add_worksheet();
        worksheet.set_name(VARIABLE_SHEET_NAME)?;
        let rows = self.variables.export();
        for (index, item) in rows.iter().enumerate() {
            let format = if index.eq(&0) {
                &header_format
            } else {
                &item_format
            };
            worksheet.write_row_with_format(index as u32, 0, item.to_vec(), format)?;
        }
        worksheet.autofit();
        Ok(())
    }

    fn save_value_sheet(&mut self) -> anyhow::Result<()> {
        let header_format = Format::new()
            .set_background_color(Color::Orange)
            .set_bold()
            .set_font_name(TIMES_NEW_ROMAN);
        let item_format = Format::new().set_font_name(TIMES_NEW_ROMAN);
        let worksheet = self.workbook.add_worksheet();
        worksheet.set_name(VALUE_SHEET_NAME)?;
        let rows = self.values.export();
        for (index, item) in rows.iter().enumerate() {
            let format = if index.eq(&0) {
                &header_format
            } else {
                &item_format
            };
            worksheet.write_row_with_format(index as u32, 0, item.to_vec(), format)?;
        }
        worksheet.autofit();
        Ok(())
    }

    fn save_raw_sheet(&mut self) -> anyhow::Result<()> {
        let header_format = Format::new()
            .set_background_color(Color::Orange)
            .set_bold()
            .set_font_name(TIMES_NEW_ROMAN);
        let item_format = Format::new().set_font_name(TIMES_NEW_ROMAN);
        let worksheet = self.workbook.add_worksheet();
        worksheet.set_name(RAW_SHEET_NAME)?;
        let rows = self.raws.export();
        for (index, item) in rows.iter().enumerate() {
            let format = if index.eq(&0) {
                &header_format
            } else {
                &item_format
            };
            worksheet.write_row_with_format(index as u32, 0, item.to_vec(), format)?;
        }
        worksheet.autofit();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::fetch;

    use super::*;
    #[test]
    fn export_test() {
        let acrf = Path::new(r"D:\projects\rusty\acrf\AK111-203_aCRF v2.2.pdf");
        let annotations = fetch(acrf).unwrap();
        let dest = Path::new(r"D:\projects\rusty\acrf");
        let mut worker = Exporter::new();
        worker.add_annotations(&annotations);
        worker.save(dest).unwrap();
    }
}

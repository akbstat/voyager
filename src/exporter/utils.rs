use crate::{annotation::content::PageDescription, Annotation};

pub const RELREC: &str = "RELREC";
pub const SUPP: &str = "SUPP";
// pub const QNAM: &str = "QNAM";
pub const QVAL: &str = "QVAL";
pub const EQ: &str = "EQ";
pub const EQ_SYMBOL: &str = "=";
pub const ORRES: &str = "ORRES";
pub const TESTCD: &str = "TESTCD";
pub const EMPTY_CELL: &str = "";
pub const CRF: &str = "CRF";
pub const VARIABLE_SHEET_NAME: &str = "Variables";
pub const VALUE_SHEET_NAME: &str = "ValueLevel";
pub const RAW_SHEET_NAME: &str = "Raw";

// pub fn qnam_annotation(source: &Annotation) -> Annotation {
//     Annotation {
//         id: format!("{}-{}", source.domain, QNAM),
//         domain: source.domain.clone(),
//         variable: QNAM.to_owned(),
//         page_description: source.page_description.clone(),
//         raw: source.raw.clone(),
//     }
// }

pub fn qval_annotation(source: &Annotation) -> Annotation {
    let mut page_description = vec![];
    source.page_description.iter().for_each(|page| {
        page_description.push(PageDescription {
            page: page.page,
            description: vec![format!("QNAM = {}", source.variable)],
        });
    });
    Annotation {
        id: format!("{}-{}", source.domain, QVAL),
        domain_id: "".into(),
        domain: source.domain.clone(),
        variable: QVAL.to_owned(),
        page_description,
        raw: source.raw.clone(),
        supp: source.supp,
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub domain: String,
    pub variable: String,
    pub page_description: Vec<PageDescription>,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageDescription {
    pub page: usize,
    pub description: Vec<String>,
}

impl PageDescription {
    pub fn has_description_in_same_page(&self, desc: &str) -> bool {
        for content in self.description.iter() {
            if content.eq(&desc) {
                return true;
            }
        }
        false
    }
}

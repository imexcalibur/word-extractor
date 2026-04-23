#[derive(Debug, Clone)]
pub enum DocElement {
    Paragraph {
        text: String,
        style: Option<String>,
        is_heading: bool,
        heading_level: Option<u32>,
    },
    Table {
        rows: Vec<Vec<String>>,
    },
}

impl DocElement {
    pub fn is_paragraph(&self) -> bool {
        matches!(self, DocElement::Paragraph { .. })
    }

    pub fn is_table(&self) -> bool {
        matches!(self, DocElement::Table { .. })
    }

    pub fn is_heading(&self) -> bool {
        match self {
            DocElement::Paragraph { is_heading, .. } => *is_heading,
            _ => false,
        }
    }

    pub fn get_text(&self) -> String {
        match self {
            DocElement::Paragraph { text, .. } => text.clone(),
            DocElement::Table { rows } => {
                rows.iter()
                    .map(|row| row.join(" | "))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }
}
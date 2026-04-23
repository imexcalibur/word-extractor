use zip::ZipArchive;
use roxmltree::Document;
use crate::parsers::element::DocElement;

pub struct WordParser {
    elements: Vec<DocElement>,
}

impl WordParser {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub fn parse_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(file_path)?;
        let mut archive = ZipArchive::new(file)?;

        let mut document_xml = archive.by_name("word/document.xml")?;
        let mut content = String::new();
        use std::io::Read;
        document_xml.read_to_string(&mut content)?;

        let doc = roxmltree::Document::parse(&content)?;
        let elements = Self::parse_document(&doc);

        Ok(Self { elements })
    }

    fn parse_document(doc: &Document) -> Vec<DocElement> {
        let root = doc.root();
        let mut elements = Vec::new();

        for node in root.descendants() {
            if node.tag_name().name() == "body" {
                for child in node.children() {
                    Self::parse_body_element(child, &mut elements);
                }
            }
        }

        elements
    }

    fn parse_body_element(node: roxmltree::Node, elements: &mut Vec<DocElement>) {
        let tag_name = node.tag_name().name();

        if tag_name == "p" {
            let (text, style, is_heading, heading_level) = Self::parse_paragraph(node);
            if !text.trim().is_empty() {
                elements.push(DocElement::Paragraph {
                    text,
                    style,
                    is_heading,
                    heading_level,
                });
            }
        } else if tag_name == "tbl" {
            let rows = Self::parse_table(node);
            if !rows.is_empty() {
                elements.push(DocElement::Table { rows });
            }
        }
    }

    fn parse_paragraph(node: roxmltree::Node) -> (String, Option<String>, bool, Option<u32>) {
        let mut text = String::new();
        let mut style: Option<String> = None;
        let mut is_heading = false;
        let mut heading_level: Option<u32> = None;

        for child in node.descendants() {
            let tag = child.tag_name().name();

            if tag == "t" {
                text.push_str(child.text().unwrap_or(""));
            } else if tag == "pStyle" {
                let val = child.attribute("val");
                if let Some(v) = val {
                    style = Some(v.to_string());
                    if v.starts_with("Heading") || v.starts_with("heading") {
                        is_heading = true;
                        let level_str = v.replace("Heading", "").replace("heading", "");
                        heading_level = level_str.parse().ok();
                    }
                }
            }
        }

        (text, style, is_heading, heading_level)
    }

    fn parse_table(node: roxmltree::Node) -> Vec<Vec<String>> {
        let mut rows = Vec::new();

        for row_node in node.children() {
            if row_node.tag_name().name() == "tr" {
                let mut cells = Vec::new();
                for cell_node in row_node.children() {
                    if cell_node.tag_name().name() == "tc" {
                        let cell_text = Self::extract_cell_text(cell_node);
                        cells.push(cell_text);
                    }
                }
                if !cells.is_empty() {
                    rows.push(cells);
                }
            }
        }

        rows
    }

    fn extract_cell_text(node: roxmltree::Node) -> String {
        let mut text = String::new();
        for child in node.descendants() {
            if child.tag_name().name() == "t" {
                text.push_str(child.text().unwrap_or(""));
            }
        }
        text.trim().to_string()
    }

    pub fn get_elements(&self) -> &[DocElement] {
        &self.elements
    }

    pub fn get_paragraphs(&self) -> Vec<&DocElement> {
        self.elements.iter().filter(|e| e.is_paragraph()).collect()
    }

    pub fn get_tables(&self) -> Vec<&DocElement> {
        self.elements.iter().filter(|e| e.is_table()).collect()
    }

    pub fn get_headings(&self) -> Vec<&DocElement> {
        self.elements.iter().filter(|e| e.is_heading()).collect()
    }
}
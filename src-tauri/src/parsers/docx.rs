use zip::ZipArchive;
use roxmltree::Document;
use crate::parsers::element::{DocElement, StructuredParagraph, CellContent, TableRow, StructuredTable};
use regex::Regex;

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
        let mut table_index = 0;

        for node in root.descendants() {
            if node.tag_name().name() == "body" {
                for child in node.children() {
                    Self::parse_body_element(child, &mut elements, &mut table_index);
                }
            }
        }

        elements
    }

    fn parse_body_element(node: roxmltree::Node, elements: &mut Vec<DocElement>, table_index: &mut usize) {
        let tag_name = node.tag_name().name();

        if tag_name == "p" {
            let paragraph = Self::parse_paragraph(node);
            if !paragraph.text.trim().is_empty() || paragraph.is_any_heading() {
                elements.push(DocElement::Paragraph { paragraph });
            }
        } else if tag_name == "tbl" {
            let table = Self::parse_table(node, *table_index);
            *table_index += 1;
            if !table.rows.is_empty() {
                elements.push(DocElement::Table { table });
            }
        }
    }

    /// 解析段落（v0.2.0 改进）
    fn parse_paragraph(node: roxmltree::Node) -> StructuredParagraph {
        let mut paragraph = StructuredParagraph::new(String::new());

        // 解析段落属性
        for child in node.children() {
            if child.tag_name().name() == "pPr" {
                Self::parse_paragraph_properties(child, &mut paragraph);
            }
        }

        // 解析文本内容
        for child in node.descendants() {
            if child.tag_name().name() == "t" {
                paragraph.text.push_str(child.text().unwrap_or(""));
            }
        }

        // 解析文本格式（加粗、字号）
        Self::parse_text_format(node, &mut paragraph);

        // 检测编号模式标题
        Self::detect_number_heading(&mut paragraph);

        paragraph
    }

    /// 解析段落属性
    fn parse_paragraph_properties(node: roxmltree::Node, paragraph: &mut StructuredParagraph) {
        for child in node.children() {
            let tag = child.tag_name().name();

            // 段落样式
            if tag == "pStyle" {
                let val = child.attribute("val");
                if let Some(v) = val {
                    paragraph.style = Some(v.to_string());
                    // 检测 Word 内置标题样式
                    if v.starts_with("Heading") || v.starts_with("heading") {
                        paragraph.is_heading = true;
                        let level_str = v.replace("Heading", "").replace("heading", "");
                        paragraph.heading_level = level_str.parse().ok();
                    }
                }
            }

            // 编号属性
            if tag == "numPr" {
                paragraph.is_list_item = true;
                for num_child in child.children() {
                    if num_child.tag_name().name() == "ilvl" {
                        paragraph.list_level = num_child.attribute("val")
                            .and_then(|v| v.parse().ok());
                    }
                    if num_child.tag_name().name() == "numId" {
                        paragraph.num_id = num_child.attribute("val")
                            .and_then(|v| v.parse().ok());
                    }
                }
            }
        }
    }

    /// 解析文本格式（加粗、字号）
    fn parse_text_format(node: roxmltree::Node, paragraph: &mut StructuredParagraph) {
        for child in node.descendants() {
            if child.tag_name().name() == "rPr" {
                for rpr_child in child.children() {
                    let tag = rpr_child.tag_name().name();

                    // 加粗
                    if tag == "b" || tag == "bCs" {
                        paragraph.is_bold = true;
                    }

                    // 字号
                    if tag == "sz" {
                        paragraph.font_size = rpr_child.attribute("val")
                            .and_then(|v| v.parse().ok());
                    }
                }
            }
        }

        // 根据格式特征判断是否可能是标题
        Self::detect_format_heading(paragraph);
    }

    /// 检测格式特征标题
    fn detect_format_heading(paragraph: &mut StructuredParagraph) {
        // 条件：加粗 + 字号较大 + 内容较短（非空）
        if paragraph.is_bold
            && paragraph.font_size.is_some()
            && !paragraph.text.trim().is_empty()
            && paragraph.text.len() < 100 {

            let font_size = paragraph.font_size.unwrap();
            // 字号 >= 24半磅(12磅) 视为可能标题
            if font_size >= 24 {
                paragraph.is_format_heading = true;
            }
        }
    }

    /// 检测编号模式标题（v0.2.0 新增）
    fn detect_number_heading(paragraph: &mut StructuredParagraph) {
        let text = paragraph.text.trim();

        if text.is_empty() {
            return;
        }

        // 中式编号模式检测
        let patterns: Vec<(Regex, u32)> = vec![
            // 一级：中文数字 + 逗号
            (Regex::new(r"^[一二三四五六七八九十]+、").unwrap(), 1),
            // 一级：第X章
            (Regex::new(r"^第[一二三四五六七八九十百零\d]+章").unwrap(), 1),
            // 二级：第X节
            (Regex::new(r"^第[一二三四五六七八九十\d]+节").unwrap(), 2),
            // 二级：数字 + 点（如 1. 2.）
            (Regex::new(r"^\d+\.").unwrap(), 2),
            // 二级：括号 + 中文数字（如 （一）（二））
            (Regex::new(r"^（[一二三四五六七八九十]+）").unwrap(), 2),
            // 三级：括号 + 数字（如 (1) (2)）
            (Regex::new(r"^\(\d+\)").unwrap(), 3),
            // 三级：数字 + 括号（如 1）2））
            (Regex::new(r"^\d+\)").unwrap(), 3),
        ];

        for (regex, level) in patterns {
            if regex.is_match(text) {
                paragraph.is_number_heading = true;
                paragraph.number_heading_level = Some(level);
                break;
            }
        }
    }

    /// 解析表格（v0.2.0 改进）
    fn parse_table(node: roxmltree::Node, table_index: usize) -> StructuredTable {
        let mut table = StructuredTable::new(table_index);

        for row_node in node.children() {
            if row_node.tag_name().name() == "tr" {
                let row = Self::parse_table_row(row_node);
                if !row.cells.is_empty() {
                    table.rows.push(row);
                }
            }
        }

        table
    }

    /// 解析表格行
    fn parse_table_row(node: roxmltree::Node) -> TableRow {
        let mut row = TableRow::new();

        for cell_node in node.children() {
            if cell_node.tag_name().name() == "tc" {
                let cell = Self::parse_table_cell(cell_node);
                row.cells.push(cell);
            }
        }

        row
    }

    /// 解析表格单元格（v0.2.0 改进：深入解析段落）
    fn parse_table_cell(node: roxmltree::Node) -> CellContent {
        let mut cell = CellContent::new();

        // 解析单元格属性
        for child in node.children() {
            if child.tag_name().name() == "tcPr" {
                for tcpr_child in child.children() {
                    // 合并单元格：跨列
                    if tcpr_child.tag_name().name() == "gridSpan" {
                        cell.grid_span = tcpr_child.attribute("val")
                            .and_then(|v| v.parse().ok());
                    }
                    // 垂直合并
                    if tcpr_child.tag_name().name() == "vMerge" {
                        cell.v_merge = tcpr_child.attribute("val")
                            .map(|v| v.to_string());
                    }
                }
            }
        }

        // 解析单元格内的段落
        for child in node.children() {
            if child.tag_name().name() == "p" {
                let paragraph = Self::parse_paragraph(child);
                // 保留所有段落（包括空的，因为可能影响结构）
                cell.paragraphs.push(paragraph);
            }
        }

        cell
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
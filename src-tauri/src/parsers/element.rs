use serde::{Serialize, Deserialize};

/// 结构化段落（v0.2.0 新增）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredParagraph {
    /// 段落文本内容
    pub text: String,
    /// 是否是编号/项目符号列表项
    pub is_list_item: bool,
    /// 编号层级（0=顶级，1=子级...）
    pub list_level: Option<u32>,
    /// 编号ID（用于识别同一列表）
    pub num_id: Option<u32>,
    /// 是否加粗
    pub is_bold: bool,
    /// 字号（单位：半磅，如24表示12磅）
    pub font_size: Option<u32>,
    /// 段落样式
    pub style: Option<String>,
    /// 是否是标题（Word内置样式）
    pub is_heading: bool,
    /// 标题层级
    pub heading_level: Option<u32>,
    /// 是否是格式特征标题（v0.2.0）
    pub is_format_heading: bool,
    /// 是否是编号模式标题（v0.2.0）
    pub is_number_heading: bool,
    /// 编号模式推断的层级
    pub number_heading_level: Option<u32>,
}

impl StructuredParagraph {
    pub fn new(text: String) -> Self {
        Self {
            text,
            is_list_item: false,
            list_level: None,
            num_id: None,
            is_bold: false,
            font_size: None,
            style: None,
            is_heading: false,
            heading_level: None,
            is_format_heading: false,
            is_number_heading: false,
            number_heading_level: None,
        }
    }

    /// 获取带编号符号的完整文本
    pub fn get_full_text(&self) -> String {
        self.text.clone()
    }

    /// 判断是否是某种形式的标题
    pub fn is_any_heading(&self) -> bool {
        self.is_heading || self.is_format_heading || self.is_number_heading
    }

    /// 获取标题层级（综合三种识别方式）
    pub fn get_heading_level(&self) -> Option<u32> {
        if self.is_heading {
            self.heading_level
        } else if self.is_format_heading {
            // 格式特征标题：根据字号推断层级，大字号=高层级
            // 假设：>=28磅(56半磅)=一级，>=24磅(48半磅)=二级，>=20磅(40半磅)=三级
            self.font_size.and_then(|sz| {
                if sz >= 56 { Some(1) }
                else if sz >= 48 { Some(2) }
                else if sz >= 40 { Some(3) }
                else { Some(4) }
            })
        } else if self.is_number_heading {
            self.number_heading_level
        } else {
            None
        }
    }
}

/// 表格单元格内容（v0.2.0 新增）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellContent {
    /// 单元格内的段落列表
    pub paragraphs: Vec<StructuredParagraph>,
    /// 合并单元格信息：跨列数
    pub grid_span: Option<u32>,
    /// 合并单元格信息：垂直合并
    pub v_merge: Option<String>,
}

impl CellContent {
    pub fn new() -> Self {
        Self {
            paragraphs: Vec::new(),
            grid_span: None,
            v_merge: None,
        }
    }

    /// 获取单元格全部文本（扁平）
    pub fn get_flat_text(&self) -> String {
        self.paragraphs
            .iter()
            .map(|p| p.text.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 获取单元格全部文本（带编号）
    pub fn get_full_text(&self) -> String {
        self.paragraphs
            .iter()
            .map(|p| p.get_full_text())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// 表格行（v0.2.0 新增）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    /// 行内单元格列表
    pub cells: Vec<CellContent>,
}

impl TableRow {
    pub fn new() -> Self {
        Self { cells: Vec::new() }
    }

    /// 获取行全部文本（扁平）
    pub fn get_flat_text(&self) -> String {
        self.cells
            .iter()
            .map(|c| c.get_flat_text())
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

/// 结构化表格（v0.2.0 新增）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredTable {
    /// 表格行列表
    pub rows: Vec<TableRow>,
    /// 表格索引（在文档中的位置）
    pub table_index: usize,
}

impl StructuredTable {
    pub fn new(table_index: usize) -> Self {
        Self {
            rows: Vec::new(),
            table_index,
        }
    }

    /// 获取表格全部文本（扁平）
    pub fn get_flat_text(&self) -> String {
        self.rows
            .iter()
            .map(|r| r.get_flat_text())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 查找包含关键词的单元格位置
    pub fn find_cell_containing(&self, keyword: &str) -> Option<(usize, usize)> {
        for (row_idx, row) in self.rows.iter().enumerate() {
            for (cell_idx, cell) in row.cells.iter().enumerate() {
                if cell.get_flat_text().contains(keyword) {
                    return Some((row_idx, cell_idx));
                }
            }
        }
        None
    }

    /// 查找包含关键词的行索引
    pub fn find_row_containing(&self, keyword: &str) -> Option<usize> {
        for (row_idx, row) in self.rows.iter().enumerate() {
            if row.get_flat_text().contains(keyword) {
                return Some(row_idx);
            }
        }
        None
    }

    /// 判断某行是否是标题行（通常有合并单元格或特殊背景）
    pub fn is_heading_row(&self, row_idx: usize) -> bool {
        if let Some(row) = self.rows.get(row_idx) {
            // 合并单元格（grid_span）通常表示标题行
            row.cells.iter().any(|c| c.grid_span.is_some())
        } else {
            false
        }
    }
}

/// 文档元素（v0.2.0 改进）
#[derive(Debug, Clone)]
pub enum DocElement {
    Paragraph {
        paragraph: StructuredParagraph,
    },
    Table {
        table: StructuredTable,
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
            DocElement::Paragraph { paragraph } => paragraph.is_any_heading(),
            _ => false,
        }
    }

    pub fn get_text(&self) -> String {
        match self {
            DocElement::Paragraph { paragraph } => paragraph.text.clone(),
            DocElement::Table { table } => table.get_flat_text(),
        }
    }

    pub fn get_full_text(&self) -> String {
        match self {
            DocElement::Paragraph { paragraph } => paragraph.get_full_text(),
            DocElement::Table { table } => table.get_flat_text(),
        }
    }

    /// 获取结构化段落（如果是段落类型）
    pub fn get_paragraph(&self) -> Option<&StructuredParagraph> {
        match self {
            DocElement::Paragraph { paragraph } => Some(paragraph),
            _ => None,
        }
    }

    /// 获取结构化表格（如果是表格类型）
    pub fn get_table(&self) -> Option<&StructuredTable> {
        match self {
            DocElement::Table { table } => Some(table),
            _ => None,
        }
    }
}

// 保留旧的扁平格式兼容（用于导出等场景）
impl DocElement {
    /// 获取扁平表格行（兼容旧格式）
    pub fn get_flat_rows(&self) -> Option<Vec<Vec<String>>> {
        match self {
            DocElement::Table { table } => Some(
                table.rows.iter().map(|row| {
                    row.cells.iter().map(|cell| cell.get_flat_text()).collect()
                }).collect()
            ),
            _ => None,
        }
    }
}
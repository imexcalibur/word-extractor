use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RuleType {
    ChapterTitle,
    TableKeyword,
    ParagraphKeyword,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MatchMode {
    Prefix,
    Contains,
    Regex,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RangeMode {
    Single,
    Multi,
    UntilMarker,
}

/// 表格提取范围模式（v0.2.0 新增）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TableExtractMode {
    /// 单个单元格内容
    Cell,
    /// 单元格全部内容（保留结构）
    CellFull,
    /// 当前单元格 + 相邻格（左/右/上/下）
    CellAdjacent,
    /// 当前整行
    RowFull,
    /// 当前整列
    ColumnFull,
    /// 到下一个标题行（合并单元格）
    ToNextHeading,
    /// 整个表格
    Table,
}

/// 标题识别方式（v0.2.0 新增）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum HeadingDetectMode {
    /// Word 内置标题样式
    HeadingStyle,
    /// 格式特征（加粗+大字号+短内容）
    FormatFeature,
    /// 编号模式（一、二、/1. 2. /第X章）
    NumberPattern,
}

/// 相邻单元格方向（v0.2.0 新增）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AdjacentDirection {
    Left,
    Right,
    Above,
    Below,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ExtractMode {
    Cell,
    Row,
    Table,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphMatch {
    pub match_mode: MatchMode,
    pub keyword: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractRange {
    pub mode: RangeMode,
    pub end_marker: Option<String>,
    pub max_paragraphs: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumnMatch {
    pub header: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRowMatch {
    pub keyword: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    // 章节配置
    pub title_pattern: Option<String>,
    pub include_subsections: Option<bool>,
    pub include_tables: Option<bool>,
    /// 标题识别方式列表（可多选）
    pub heading_detect_modes: Option<Vec<HeadingDetectMode>>,
    /// 是否还原编号符号
    pub restore_numbering: Option<bool>,

    // 表格配置
    pub column_match: Option<TableColumnMatch>,
    pub row_match: Option<TableRowMatch>,
    pub extract_mode: Option<ExtractMode>,
    /// 表格提取范围模式（v0.2.0 新增）
    pub table_extract_mode: Option<TableExtractMode>,
    /// 相邻单元格方向（配合 CellAdjacent 使用）
    pub adjacent_direction: Option<AdjacentDirection>,
    /// 表格关键词搜索（v0.2.0 新增）
    pub table_keyword: Option<String>,

    // 段落配置
    pub paragraph_match: Option<ParagraphMatch>,
    pub extract_range: Option<ExtractRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRule {
    pub id: String,
    pub name: String,
    pub rule_type: RuleType,
    pub config: RuleConfig,
}

impl ExtractionRule {
    pub fn new(name: String, rule_type: RuleType, config: RuleConfig) -> Self {
        use uuid::Uuid;
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            rule_type,
            config,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub rules: Vec<ExtractionRule>,
}
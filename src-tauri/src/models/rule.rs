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

    // 表格配置
    pub column_match: Option<TableColumnMatch>,
    pub row_match: Option<TableRowMatch>,
    pub extract_mode: Option<ExtractMode>,

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
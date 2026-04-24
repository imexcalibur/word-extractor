// 数据类型定义 - 与 Rust 后端对应

export interface Document {
  id: string;
  file_path: string;
  file_name: string;
  file_size: number;
  added_at: string;
}

export enum RuleType {
  ChapterTitle = 'ChapterTitle',
  TableKeyword = 'TableKeyword',
  ParagraphKeyword = 'ParagraphKeyword',
}

export enum MatchMode {
  Prefix = 'Prefix',
  Contains = 'Contains',
  Regex = 'Regex',
}

export enum RangeMode {
  Single = 'Single',
  Multi = 'Multi',
  UntilMarker = 'UntilMarker',
}

// 表格提取范围模式（v0.2.0 新增）
export enum TableExtractMode {
  Cell = 'Cell',
  CellFull = 'CellFull',
  CellAdjacent = 'CellAdjacent',
  RowFull = 'RowFull',
  ColumnFull = 'ColumnFull',
  ToNextHeading = 'ToNextHeading',
  Table = 'Table',
}

// 标题识别方式（v0.2.0 新增）
export enum HeadingDetectMode {
  HeadingStyle = 'HeadingStyle',
  FormatFeature = 'FormatFeature',
  NumberPattern = 'NumberPattern',
}

// 相邻单元格方向（v0.2.0 新增）
export enum AdjacentDirection {
  Left = 'Left',
  Right = 'Right',
  Above = 'Above',
  Below = 'Below',
}

export enum ExtractMode {
  Cell = 'Cell',
  Row = 'Row',
  Table = 'Table',
}

export interface ParagraphMatch {
  match_mode: MatchMode;
  keyword: string;
}

export interface ExtractRange {
  mode: RangeMode;
  end_marker?: string;
  max_paragraphs?: number;
}

export interface TableColumnMatch {
  header: string;
  value?: string;
}

export interface TableRowMatch {
  keyword: string;
}

export interface RuleConfig {
  // 章节配置
  title_pattern?: string;
  include_subsections?: boolean;
  include_tables?: boolean;
  heading_detect_modes?: HeadingDetectMode[];
  restore_numbering?: boolean;

  // 表格配置
  column_match?: TableColumnMatch;
  row_match?: TableRowMatch;
  extract_mode?: ExtractMode;
  table_extract_mode?: TableExtractMode;
  adjacent_direction?: AdjacentDirection;
  table_keyword?: string;

  // 段落配置
  paragraph_match?: ParagraphMatch;
  extract_range?: ExtractRange;
}

export interface ExtractionRule {
  id: string;
  name: string;
  rule_type: RuleType;
  config: RuleConfig;
}

export interface Template {
  id: string;
  name: string;
  rules: ExtractionRule[];
}

export enum ContentType {
  Text = 'Text',
  Table = 'Table',
}

export interface ContentPosition {
  page?: number;
  table_index?: number;
  description: string;
}

export interface ExtractionResult {
  document_name: string;
  document_id: string;
  rule_id: string;
  rule_name: string;
  content_type: ContentType;
  content: string;
  position: ContentPosition;
}

export interface ExportFormat {
  type: 'excel' | 'markdown' | 'word';
  path: string;
}
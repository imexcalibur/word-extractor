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
  title_pattern?: string;
  include_subsections?: boolean;
  include_tables?: boolean;
  column_match?: TableColumnMatch;
  row_match?: TableRowMatch;
  extract_mode?: ExtractMode;
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
  type: 'excel' | 'markdown';
  path: string;
}
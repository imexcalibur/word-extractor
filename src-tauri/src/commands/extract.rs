use tauri::State;
use crate::models::{Document, ExtractionRule, ExtractionResult, TableExtractMode, HeadingDetectMode, AdjacentDirection};
use crate::parsers::WordParser;
use crate::parsers::element::{DocElement, StructuredParagraph, StructuredTable};
use crate::commands::template::AppState;
use regex::Regex;

#[tauri::command]
pub fn add_files(paths: Vec<String>, state: State<AppState>) -> Result<Vec<Document>, String> {
    let mut docs = state.documents.lock().unwrap();
    let mut new_docs = Vec::new();

    for path in paths {
        if path.ends_with(".docx") || path.ends_with(".doc") {
            let doc = Document::new(path);
            new_docs.push(doc.clone());
            docs.push(doc);
        }
    }

    Ok(new_docs)
}

#[tauri::command]
pub fn get_documents(state: State<AppState>) -> Result<Vec<Document>, String> {
    let docs = state.documents.lock().unwrap();
    Ok(docs.clone())
}

#[tauri::command]
pub fn remove_document(doc_id: String, state: State<AppState>) -> Result<(), String> {
    let mut docs = state.documents.lock().unwrap();
    docs.retain(|d| d.id != doc_id);
    Ok(())
}

#[tauri::command]
pub fn clear_documents(state: State<AppState>) -> Result<(), String> {
    let mut docs = state.documents.lock().unwrap();
    docs.clear();
    Ok(())
}

#[tauri::command]
pub fn extract_content(
    doc_ids: Vec<String>,
    rules: Vec<ExtractionRule>,
    state: State<AppState>,
) -> Result<Vec<ExtractionResult>, String> {
    let docs = state.documents.lock().unwrap();
    let mut results = Vec::new();

    for doc_id in &doc_ids {
        let doc = docs.iter().find(|d| &d.id == doc_id);
        if let Some(doc) = doc {
            let parser = WordParser::parse_file(&doc.file_path)
                .map_err(|e| format!("解析文件失败: {}", e))?;

            for rule in &rules {
                let extracted = extract_by_rule(&parser, doc, rule);
                results.extend(extracted);
            }
        }
    }

    Ok(results)
}

#[tauri::command]
pub fn preview_extraction(
    doc_id: String,
    rule: ExtractionRule,
    state: State<AppState>,
) -> Result<Vec<ExtractionResult>, String> {
    let docs = state.documents.lock().unwrap();
    let doc = docs.iter().find(|d| d.id == doc_id);

    if let Some(doc) = doc {
        let parser = WordParser::parse_file(&doc.file_path)
            .map_err(|e| format!("解析文件失败: {}", e))?;
        Ok(extract_by_rule(&parser, doc, &rule))
    } else {
        Err("文档不存在".to_string())
    }
}

fn extract_by_rule(parser: &WordParser, doc: &Document, rule: &ExtractionRule) -> Vec<ExtractionResult> {
    match rule.rule_type {
        crate::models::RuleType::ChapterTitle => extract_chapter(parser, doc, rule),
        crate::models::RuleType::TableKeyword => extract_table(parser, doc, rule),
        crate::models::RuleType::ParagraphKeyword => extract_paragraph(parser, doc, rule),
    }
}

/// 章节提取（v0.2.0 改进）
fn extract_chapter(parser: &WordParser, doc: &Document, rule: &ExtractionRule) -> Vec<ExtractionResult> {
    let empty_pattern = String::new();
    let pattern = rule.config.title_pattern.as_ref().unwrap_or(&empty_pattern);
    let include_subsections = rule.config.include_subsections.unwrap_or(true);
    let include_tables = rule.config.include_tables.unwrap_or(true);
    let restore_numbering = rule.config.restore_numbering.unwrap_or(false);

    // 获取标题识别方式
    let detect_modes = rule.config.heading_detect_modes.clone()
        .unwrap_or(vec![HeadingDetectMode::HeadingStyle, HeadingDetectMode::NumberPattern]);

    let regex = Regex::new(pattern).ok();
    let elements = parser.get_elements();
    let mut results = Vec::new();

    let mut collecting = false;
    let mut collected_text = String::new();
    let mut start_heading_level: Option<u32> = None;

    for element in elements {
        match element {
            DocElement::Paragraph { paragraph } => {
                // 检查是否匹配标题
                let is_heading = check_is_heading(paragraph, &detect_modes);

                if is_heading {
                    let matches_title = check_title_matches(paragraph, pattern, &regex);

                    if matches_title && !collecting {
                        // 开始收集
                        collecting = true;
                        start_heading_level = paragraph.get_heading_level();
                        collected_text = format_heading_text(paragraph, restore_numbering) + "\n";
                    } else if collecting {
                        // 检查是否应该停止
                        let current_level = paragraph.get_heading_level();
                        let should_stop = if include_subsections {
                            // 包含子章节：遇到同级或更高级标题才停止
                            current_level.map_or(true, |l| l <= start_heading_level.unwrap_or(1))
                        } else {
                            // 不包含子章节：遇到任何标题都停止
                            true
                        };

                        if should_stop {
                            // 结束收集，输出结果
                            if collected_text.trim().len() > 50 {
                                results.push(create_chapter_result(doc, rule, collected_text.trim(), pattern));
                            }
                            // 如果这个新标题也匹配，重新开始收集
                            if matches_title {
                                collecting = true;
                                start_heading_level = paragraph.get_heading_level();
                                collected_text = format_heading_text(paragraph, restore_numbering) + "\n";
                            } else {
                                collected_text = String::new();
                                collecting = false;
                                start_heading_level = None;
                            }
                        } else {
                            // 继续收集（子标题也收录）
                            collected_text.push_str(&format!("{}\n", format_heading_text(paragraph, restore_numbering)));
                        }
                    }
                } else if collecting {
                    // 收集普通段落
                    collected_text.push_str(&format!("{}\n", format_paragraph_text(paragraph, restore_numbering)));
                }
            }
            DocElement::Table { table } => {
                if collecting && include_tables {
                    // 收集表格内容
                    let table_text = format_table_text(table, restore_numbering);
                    collected_text.push_str(&format!("[表格]\n{}\n", table_text));
                }
            }
        }
    }

    // 处理最后一个章节
    if collecting && collected_text.trim().len() > 50 {
        results.push(create_chapter_result(doc, rule, collected_text.trim(), pattern));
    }

    results
}

/// 检查是否是标题（根据配置的识别方式）
fn check_is_heading(paragraph: &StructuredParagraph, detect_modes: &[HeadingDetectMode]) -> bool {
    for mode in detect_modes {
        match mode {
            HeadingDetectMode::HeadingStyle => {
                if paragraph.is_heading { return true; }
            }
            HeadingDetectMode::FormatFeature => {
                if paragraph.is_format_heading { return true; }
            }
            HeadingDetectMode::NumberPattern => {
                if paragraph.is_number_heading { return true; }
            }
        }
    }
    false
}

/// 检查标题文本是否匹配
fn check_title_matches(paragraph: &StructuredParagraph, pattern: &str, regex: &Option<Regex>) -> bool {
    if pattern.is_empty() {
        return true;
    }

    let text = &paragraph.text;
    if let Some(re) = regex {
        re.is_match(text)
    } else {
        text.contains(pattern)
    }
}

/// 格式化标题文本
fn format_heading_text(paragraph: &StructuredParagraph, restore_numbering: bool) -> String {
    if restore_numbering && paragraph.is_number_heading {
        // 编号已在文本中，直接返回
        paragraph.text.clone()
    } else {
        paragraph.text.clone()
    }
}

/// 格式化段落文本
fn format_paragraph_text(paragraph: &StructuredParagraph, restore_numbering: bool) -> String {
    if restore_numbering && paragraph.is_list_item {
        // 尝试还原编号符号（简化处理：根据层级推断）
        let prefix = match paragraph.list_level {
            Some(0) => "• ",
            Some(1) => "  - ",
            Some(2) => "    · ",
            _ => "",
        };
        format!("{}{}", prefix, paragraph.text)
    } else {
        paragraph.text.clone()
    }
}

/// 格式化表格文本
fn format_table_text(table: &StructuredTable, restore_numbering: bool) -> String {
    table.rows.iter().map(|row| {
        row.cells.iter().map(|cell| {
            if restore_numbering {
                cell.paragraphs.iter().map(|p| format_paragraph_text(p, true)).collect::<Vec<_>>().join("\n")
            } else {
                cell.get_flat_text()
            }
        }).collect::<Vec<_>>().join(" | ")
    }).collect::<Vec<_>>().join("\n")
}

/// 创建章节提取结果
fn create_chapter_result(doc: &Document, rule: &ExtractionRule, content: &str, pattern: &str) -> ExtractionResult {
    ExtractionResult {
        document_name: doc.file_name.clone(),
        document_id: doc.id.clone(),
        rule_id: rule.id.clone(),
        rule_name: rule.name.clone(),
        content_type: crate::models::ContentType::Text,
        content: content.to_string(),
        position: crate::models::ContentPosition {
            page: None,
            table_index: None,
            description: format!("章节: {}", pattern),
        },
    }
}

/// 表格提取（v0.2.0 改进）
fn extract_table(parser: &WordParser, doc: &Document, rule: &ExtractionRule) -> Vec<ExtractionResult> {
    let elements = parser.get_elements();
    let mut results = Vec::new();

    for element in elements {
        if let DocElement::Table { table } = element {
            // 根据配置提取
            if let Some(col_match) = &rule.config.column_match {
                extract_table_by_column(table, doc, rule, col_match, &mut results);
            }

            if let Some(row_match) = &rule.config.row_match {
                extract_table_by_row(table, doc, rule, row_match, &mut results);
            }

            // 新增：按关键词定位后扩展提取
            if let Some(keyword) = &rule.config.table_keyword {
                extract_table_by_keyword(table, doc, rule, keyword, &mut results);
            }
        }
    }

    results
}

/// 按列匹配提取表格
fn extract_table_by_column(
    table: &StructuredTable,
    doc: &Document,
    rule: &ExtractionRule,
    col_match: &crate::models::TableColumnMatch,
    results: &mut Vec<ExtractionResult>,
) {
    let keyword = &col_match.header;

    // 在表头行查找列
    if let Some(first_row) = table.rows.first() {
        for (col_idx, cell) in first_row.cells.iter().enumerate() {
            if cell.get_flat_text().contains(keyword) {
                // 找到匹配列，按配置的提取模式提取
                let extract_mode = rule.config.table_extract_mode
                    .unwrap_or(TableExtractMode::ColumnFull);

                extract_by_mode(table, doc, rule, None, Some(col_idx), extract_mode, results);
            }
        }
    }
}

/// 按行关键词提取表格
fn extract_table_by_row(
    table: &StructuredTable,
    doc: &Document,
    rule: &ExtractionRule,
    row_match: &crate::models::TableRowMatch,
    results: &mut Vec<ExtractionResult>,
) {
    let keyword = &row_match.keyword;

    // 查找包含关键词的行
    for (row_idx, row) in table.rows.iter().enumerate() {
        if row.get_flat_text().contains(keyword) {
            let extract_mode = rule.config.table_extract_mode
                .unwrap_or(TableExtractMode::RowFull);

            extract_by_mode(table, doc, rule, Some(row_idx), None, extract_mode, results);
        }
    }
}

/// 按关键词定位并扩展提取（v0.2.0 新增）
fn extract_table_by_keyword(
    table: &StructuredTable,
    doc: &Document,
    rule: &ExtractionRule,
    keyword: &str,
    results: &mut Vec<ExtractionResult>,
) {
    let restore_numbering = rule.config.restore_numbering.unwrap_or(false);

    // 在表格中查找关键词
    if let Some((row_idx, cell_idx)) = table.find_cell_containing(keyword) {
        let extract_mode = rule.config.table_extract_mode
            .unwrap_or(TableExtractMode::CellFull);

        extract_by_mode(table, doc, rule, Some(row_idx), Some(cell_idx), extract_mode, results);
    }
}

/// 按提取模式提取内容（v0.2.0 新增）
fn extract_by_mode(
    table: &StructuredTable,
    doc: &Document,
    rule: &ExtractionRule,
    row_idx: Option<usize>,
    col_idx: Option<usize>,
    mode: TableExtractMode,
    results: &mut Vec<ExtractionResult>,
) {
    let restore_numbering = rule.config.restore_numbering.unwrap_or(false);

    let content = match mode {
        TableExtractMode::Cell => {
            // 单个单元格内容
            if let (Some(r), Some(c)) = (row_idx, col_idx) {
                if let Some(cell) = table.rows.get(r).and_then(|row| row.cells.get(c)) {
                    cell.get_flat_text()
                } else { "".to_string() }
            } else { "".to_string() }
        }

        TableExtractMode::CellFull => {
            // 单元格全部内容（保留结构）
            if let (Some(r), Some(c)) = (row_idx, col_idx) {
                if let Some(cell) = table.rows.get(r).and_then(|row| row.cells.get(c)) {
                    if restore_numbering {
                        cell.paragraphs.iter()
                            .map(|p| format_paragraph_text(p, true))
                            .collect::<Vec<_>>().join("\n")
                    } else {
                        cell.get_flat_text()
                    }
                } else { "".to_string() }
            } else { "".to_string() }
        }

        TableExtractMode::CellAdjacent => {
            // 当前单元格 + 相邻格
            let direction = rule.config.adjacent_direction.unwrap_or(AdjacentDirection::Right);
            extract_adjacent_cells(table, row_idx, col_idx, direction, restore_numbering)
        }

        TableExtractMode::RowFull => {
            // 当前整行
            if let Some(r) = row_idx {
                if let Some(row) = table.rows.get(r) {
                    row.cells.iter().map(|c| {
                        if restore_numbering {
                            c.paragraphs.iter().map(|p| format_paragraph_text(p, true)).collect::<Vec<_>>().join("\n")
                        } else {
                            c.get_flat_text()
                        }
                    }).collect::<Vec<_>>().join(" | ")
                } else { "".to_string() }
            } else { "".to_string() }
        }

        TableExtractMode::ColumnFull => {
            // 当前整列
            if let Some(c) = col_idx {
                table.rows.iter().map(|row| {
                    if let Some(cell) = row.cells.get(c) {
                        if restore_numbering {
                            cell.paragraphs.iter().map(|p| format_paragraph_text(p, true)).collect::<Vec<_>>().join("\n")
                        } else {
                            cell.get_flat_text()
                        }
                    } else { "".to_string() }
                }).collect::<Vec<_>>().join("\n")
            } else { "".to_string() }
        }

        TableExtractMode::ToNextHeading => {
            // 到下一个标题行
            if let Some(r) = row_idx {
                extract_to_next_heading(table, r, restore_numbering)
            } else { "".to_string() }
        }

        TableExtractMode::Table => {
            // 整个表格
            format_table_text(table, restore_numbering)
        }
    };

    if !content.is_empty() {
        results.push(ExtractionResult {
            document_name: doc.file_name.clone(),
            document_id: doc.id.clone(),
            rule_id: rule.id.clone(),
            rule_name: rule.name.clone(),
            content_type: crate::models::ContentType::Table,
            content,
            position: crate::models::ContentPosition {
                page: None,
                table_index: Some(table.table_index as u32),
                description: format!("表格{} 模式: {:?}", table.table_index, mode),
            },
        });
    }
}

/// 提取相邻单元格内容
fn extract_adjacent_cells(
    table: &StructuredTable,
    row_idx: Option<usize>,
    col_idx: Option<usize>,
    direction: AdjacentDirection,
    restore_numbering: bool,
) -> String {
    if let (Some(r), Some(c)) = (row_idx, col_idx) {
        let current_text = table.rows.get(r)
            .and_then(|row| row.cells.get(c))
            .map(|cell| {
                if restore_numbering {
                    cell.paragraphs.iter().map(|p| format_paragraph_text(p, true)).collect::<Vec<_>>().join("\n")
                } else {
                    cell.get_flat_text()
                }
            }).unwrap_or_default();

        let adjacent_text = match direction {
            AdjacentDirection::Left => {
                if c > 0 {
                    table.rows.get(r)
                        .and_then(|row| row.cells.get(c - 1))
                        .map(|cell| cell.get_flat_text())
                } else { None }
            }
            AdjacentDirection::Right => {
                table.rows.get(r)
                    .and_then(|row| row.cells.get(c + 1))
                    .map(|cell| cell.get_flat_text())
            }
            AdjacentDirection::Above => {
                if r > 0 {
                    table.rows.get(r - 1)
                        .and_then(|row| row.cells.get(c))
                        .map(|cell| cell.get_flat_text())
                } else { None }
            }
            AdjacentDirection::Below => {
                table.rows.get(r + 1)
                    .and_then(|row| row.cells.get(c))
                    .map(|cell| cell.get_flat_text())
            }
        }.unwrap_or_default();

        format!("{} | {}", current_text, adjacent_text)
    } else {
        "".to_string()
    }
}

/// 提取到下一个标题行
fn extract_to_next_heading(
    table: &StructuredTable,
    start_row: usize,
    restore_numbering: bool,
) -> String {
    let mut content = String::new();

    for (r, row) in table.rows.iter().enumerate().skip(start_row) {
        // 检查是否是标题行（有合并单元格）
        if r > start_row && table.is_heading_row(r) {
            break;
        }

        let row_text = row.cells.iter().map(|c| {
            if restore_numbering {
                c.paragraphs.iter().map(|p| format_paragraph_text(p, true)).collect::<Vec<_>>().join("\n")
            } else {
                c.get_flat_text()
            }
        }).collect::<Vec<_>>().join(" | ");

        content.push_str(&row_text);
        content.push_str("\n");
    }

    content.trim().to_string()
}

/// 段落关键词提取（v0.2.0 改进：扩展到表格单元格）
fn extract_paragraph(parser: &WordParser, doc: &Document, rule: &ExtractionRule) -> Vec<ExtractionResult> {
    let elements = parser.get_elements();
    let mut results = Vec::new();

    if let Some(p_match) = &rule.config.paragraph_match {
        let keyword = &p_match.keyword;
        let restore_numbering = rule.config.restore_numbering.unwrap_or(false);

        // 遍历所有元素
        for element in elements {
            // 检查普通段落
            if let DocElement::Paragraph { paragraph } = element {
                if check_paragraph_match(paragraph, keyword, &p_match.match_mode) {
                    let content = format_paragraph_text(paragraph, restore_numbering);
                    results.push(create_paragraph_result(doc, rule, content, "段落"));
                }
            }

            // 检查表格单元格内的段落
            if let DocElement::Table { table } = element {
                for (row_idx, row) in table.rows.iter().enumerate() {
                    for (cell_idx, cell) in row.cells.iter().enumerate() {
                        for paragraph in &cell.paragraphs {
                            if check_paragraph_match(paragraph, keyword, &p_match.match_mode) {
                                // 根据配置决定提取范围
                                let content = if let Some(range) = &rule.config.extract_range {
                                    extract_paragraph_range(cell, paragraph, range, restore_numbering)
                                } else {
                                    format_paragraph_text(paragraph, restore_numbering)
                                };

                                results.push(create_paragraph_result(
                                    doc,
                                    rule,
                                    content,
                                    &format!("表格{} 行{} 列{}", table.table_index, row_idx, cell_idx),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    results
}

/// 检查段落是否匹配关键词
fn check_paragraph_match(paragraph: &StructuredParagraph, keyword: &str, match_mode: &crate::models::MatchMode) -> bool {
    match match_mode {
        crate::models::MatchMode::Prefix => paragraph.text.starts_with(keyword),
        crate::models::MatchMode::Contains => paragraph.text.contains(keyword),
        crate::models::MatchMode::Regex => {
            Regex::new(keyword).map(|re| re.is_match(&paragraph.text)).unwrap_or(false)
        }
    }
}

/// 按范围提取段落内容
fn extract_paragraph_range(
    cell: &crate::parsers::element::CellContent,
    matched_paragraph: &StructuredParagraph,
    range: &crate::models::ExtractRange,
    restore_numbering: bool,
) -> String {
    match range.mode {
        crate::models::RangeMode::Single => {
            format_paragraph_text(matched_paragraph, restore_numbering)
        }
        crate::models::RangeMode::Multi => {
            let max = range.max_paragraphs.unwrap_or(5) as usize;
            cell.paragraphs.iter().take(max)
                .map(|p| format_paragraph_text(p, restore_numbering))
                .collect::<Vec<_>>().join("\n")
        }
        crate::models::RangeMode::UntilMarker => {
            let marker = range.end_marker.clone().unwrap_or_default();
            cell.paragraphs.iter()
                .take_while(|p| marker.is_empty() || !p.text.contains(&marker))
                .map(|p| format_paragraph_text(p, restore_numbering))
                .collect::<Vec<_>>().join("\n")
        }
    }
}

/// 创建段落提取结果
fn create_paragraph_result(doc: &Document, rule: &ExtractionRule, content: String, location: &str) -> ExtractionResult {
    ExtractionResult {
        document_name: doc.file_name.clone(),
        document_id: doc.id.clone(),
        rule_id: rule.id.clone(),
        rule_name: rule.name.clone(),
        content_type: crate::models::ContentType::Text,
        content,
        position: crate::models::ContentPosition {
            page: None,
            table_index: None,
            description: format!("位置: {}", location),
        },
    }
}
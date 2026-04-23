use tauri::State;
use crate::models::{Document, ExtractionRule, ExtractionResult};
use crate::parsers::WordParser;
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

fn extract_chapter(parser: &WordParser, doc: &Document, rule: &ExtractionRule) -> Vec<ExtractionResult> {
    let empty_pattern = String::new();
    let pattern = rule.config.title_pattern.as_ref().unwrap_or(&empty_pattern);
    let include_subsections = rule.config.include_subsections.unwrap_or(true);
    let include_tables = rule.config.include_tables.unwrap_or(true);

    let regex = Regex::new(pattern).ok();
    let elements = parser.get_elements();
    let mut results = Vec::new();

    let mut collecting = false;
    let mut collected_text = String::new();
    let mut start_heading_level: Option<u32> = None;

    for element in elements {
        if let crate::parsers::element::DocElement::Paragraph { text, is_heading, heading_level, .. } = element {
            if *is_heading {
                let matches = if let Some(re) = &regex {
                    re.is_match(text)
                } else {
                    text.contains(pattern)
                };

                if matches && !collecting {
                    collecting = true;
                    start_heading_level = *heading_level;
                    collected_text = text.clone() + "\n";
                } else if collecting {
                    let should_stop = if include_subsections {
                        *heading_level <= start_heading_level
                    } else {
                        *heading_level <= start_heading_level
                    };

                    if should_stop {
                        if collected_text.trim().len() > 50 {
                            results.push(ExtractionResult {
                                document_name: doc.file_name.clone(),
                                document_id: doc.id.clone(),
                                rule_id: rule.id.clone(),
                                rule_name: rule.name.clone(),
                                content_type: crate::models::ContentType::Text,
                                content: collected_text.trim().to_string(),
                                position: crate::models::ContentPosition {
                                    page: None,
                                    table_index: None,
                                    description: format!("章节: {}", pattern),
                                },
                            });
                        }
                        collected_text = String::new();
                        collecting = false;
                        start_heading_level = None;
                    } else {
                        collected_text.push_str(&format!("{}\n", text));
                    }
                }
            } else if collecting {
                collected_text.push_str(&format!("{}\n", text));
            }
        } else if let crate::parsers::element::DocElement::Table { .. } = element {
            if collecting && include_tables {
                collected_text.push_str(&format!("{}\n", element.get_text()));
            }
        }
    }

    if collecting && collected_text.trim().len() > 50 {
        results.push(ExtractionResult {
            document_name: doc.file_name.clone(),
            document_id: doc.id.clone(),
            rule_id: rule.id.clone(),
            rule_name: rule.name.clone(),
            content_type: crate::models::ContentType::Text,
            content: collected_text.trim().to_string(),
            position: crate::models::ContentPosition {
                page: None,
                table_index: None,
                description: format!("章节: {}", pattern),
            },
        });
    }

    results
}

fn extract_table(parser: &WordParser, doc: &Document, rule: &ExtractionRule) -> Vec<ExtractionResult> {
    let elements = parser.get_elements();
    let mut results = Vec::new();
    let mut table_index = 0;

    for element in elements {
        if let crate::parsers::element::DocElement::Table { rows } = element {
            table_index += 1;

            if let Some(col_match) = &rule.config.column_match {
                if !rows.is_empty() {
                    let headers = &rows[0];
                    let col_idx = headers.iter().position(|h| h.contains(&col_match.header));

                    if let Some(idx) = col_idx {
                        let extract_mode = rule.config.extract_mode.clone().unwrap_or(crate::models::ExtractMode::Row);

                        for (row_idx, row) in rows.iter().enumerate().skip(1) {
                            if idx < row.len() {
                                let cell_value = &row[idx];

                                let matches = if let Some(value) = &col_match.value {
                                    cell_value.contains(value)
                                } else {
                                    true
                                };

                                if matches {
                                    let content = match extract_mode {
                                        crate::models::ExtractMode::Cell => cell_value.clone(),
                                        crate::models::ExtractMode::Row => row.join(" | "),
                                        crate::models::ExtractMode::Table => {
                                            rows.iter().map(|r| r.join(" | ")).collect::<Vec<_>>().join("\n")
                                        }
                                    };

                                    results.push(ExtractionResult {
                                        document_name: doc.file_name.clone(),
                                        document_id: doc.id.clone(),
                                        rule_id: rule.id.clone(),
                                        rule_name: rule.name.clone(),
                                        content_type: crate::models::ContentType::Table,
                                        content,
                                        position: crate::models::ContentPosition {
                                            page: None,
                                            table_index: Some(table_index),
                                            description: format!("表格{} 第{}行", table_index, row_idx),
                                        },
                                    });
                                }
                            }
                        }
                    }
                }
            }

            if let Some(row_match) = &rule.config.row_match {
                for (row_idx, row) in rows.iter().enumerate() {
                    if row.iter().any(|cell| cell.contains(&row_match.keyword)) {
                        results.push(ExtractionResult {
                            document_name: doc.file_name.clone(),
                            document_id: doc.id.clone(),
                            rule_id: rule.id.clone(),
                            rule_name: rule.name.clone(),
                            content_type: crate::models::ContentType::Table,
                            content: row.join(" | "),
                            position: crate::models::ContentPosition {
                                page: None,
                                table_index: Some(table_index),
                                description: format!("表格{} 第{}行", table_index, row_idx),
                            },
                        });
                    }
                }
            }
        }
    }

    results
}

fn extract_paragraph(parser: &WordParser, doc: &Document, rule: &ExtractionRule) -> Vec<ExtractionResult> {
    let elements = parser.get_elements();
    let mut results = Vec::new();

    if let Some(p_match) = &rule.config.paragraph_match {
        let keyword = &p_match.keyword;
        let extract_range = rule.config.extract_range.clone();
        let empty_marker = String::new();

        let mut matched_indices: Vec<usize> = Vec::new();

        for (idx, element) in elements.iter().enumerate() {
            if let crate::parsers::element::DocElement::Paragraph { text, .. } = element {
                let matches = match p_match.match_mode {
                    crate::models::MatchMode::Prefix => text.starts_with(keyword),
                    crate::models::MatchMode::Contains => text.contains(keyword),
                    crate::models::MatchMode::Regex => {
                        Regex::new(keyword).map(|re| re.is_match(text)).unwrap_or(false)
                    }
                };

                if matches {
                    matched_indices.push(idx);
                }
            }
        }

        for start_idx in matched_indices {
            let content = match &extract_range {
                Some(range) => {
                    match range.mode {
                        crate::models::RangeMode::Single => {
                            elements[start_idx].get_text()
                        }
                        crate::models::RangeMode::Multi => {
                            let max = range.max_paragraphs.unwrap_or(5) as usize;
                            let mut text = String::new();
                            for i in start_idx..std::cmp::min(start_idx + max, elements.len()) {
                                if elements[i].is_paragraph() {
                                    text.push_str(&elements[i].get_text());
                                    text.push_str("\n");
                                } else {
                                    break;
                                }
                            }
                            text.trim().to_string()
                        }
                        crate::models::RangeMode::UntilMarker => {
                            let marker = range.end_marker.as_ref().unwrap_or(&empty_marker);
                            let mut text = String::new();
                            for i in start_idx..elements.len() {
                                let elem_text = elements[i].get_text();
                                if !marker.is_empty() && elem_text.contains(marker) {
                                    break;
                                }
                                text.push_str(&elem_text);
                                text.push_str("\n");
                            }
                            text.trim().to_string()
                        }
                    }
                }
                None => elements[start_idx].get_text(),
            };

            if !content.is_empty() {
                results.push(ExtractionResult {
                    document_name: doc.file_name.clone(),
                    document_id: doc.id.clone(),
                    rule_id: rule.id.clone(),
                    rule_name: rule.name.clone(),
                    content_type: crate::models::ContentType::Text,
                    content,
                    position: crate::models::ContentPosition {
                        page: None,
                        table_index: None,
                        description: format!("段落匹配: {}", keyword),
                    },
                });
            }
        }
    }

    results
}
use std::fs::File;
use std::io::Write;
use rust_xlsxwriter::Workbook;
use crate::models::ExtractionResult;

#[tauri::command]
pub fn export_to_excel(results: Vec<ExtractionResult>, output_path: String) -> Result<String, String> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    worksheet.write_string(0, 0, "文档名称").map_err(|e| e.to_string())?;
    worksheet.write_string(0, 1, "提取规则").map_err(|e| e.to_string())?;
    worksheet.write_string(0, 2, "内容类型").map_err(|e| e.to_string())?;
    worksheet.write_string(0, 3, "提取内容").map_err(|e| e.to_string())?;
    worksheet.write_string(0, 4, "位置").map_err(|e| e.to_string())?;

    for (row_idx, result) in results.iter().enumerate() {
        let row: u32 = (row_idx + 1) as u32;

        worksheet.write_string(row, 0, &result.document_name).map_err(|e| e.to_string())?;
        worksheet.write_string(row, 1, &result.rule_name).map_err(|e| e.to_string())?;

        let type_str = match result.content_type {
            crate::models::ContentType::Text => "文本",
            crate::models::ContentType::Table => "表格",
        };
        worksheet.write_string(row, 2, type_str).map_err(|e| e.to_string())?;
        worksheet.write_string(row, 3, &result.content).map_err(|e| e.to_string())?;
        worksheet.write_string(row, 4, &result.position.description).map_err(|e| e.to_string())?;
    }

    workbook.save(&output_path).map_err(|e| e.to_string())?;

    Ok(output_path)
}

#[tauri::command]
pub fn export_to_markdown(results: Vec<ExtractionResult>, output_path: String) -> Result<String, String> {
    let mut md_content = String::new();
    md_content.push_str("# 提取结果\n\n");

    let mut current_doc = String::new();

    for result in &results {
        if current_doc != result.document_name {
            current_doc = result.document_name.clone();
            md_content.push_str(&format!("## {}\n\n", current_doc));
        }

        md_content.push_str(&format!("### {}\n", result.rule_name));
        md_content.push_str(&format!("> 类型: {} | {}\n\n",
            match result.content_type {
                crate::models::ContentType::Text => "文本",
                crate::models::ContentType::Table => "表格",
            },
            result.position.description
        ));

        if result.content_type == crate::models::ContentType::Table {
            let lines: Vec<&str> = result.content.lines().collect();
            if !lines.is_empty() {
                for line in lines {
                    let cells: Vec<&str> = line.split("|").map(|s| s.trim()).collect();
                    md_content.push_str(&format!("| {} |\n", cells.join(" | ")));
                }
            }
        } else {
            md_content.push_str(&result.content);
            md_content.push_str("\n");
        }

        md_content.push_str("\n---\n\n");
    }

    let mut file = File::create(&output_path).map_err(|e| e.to_string())?;
    file.write_all(md_content.as_bytes()).map_err(|e| e.to_string())?;

    Ok(output_path)
}

/// 导出为 Word 文档（HTML格式，v0.2.0 新增）
#[tauri::command]
pub fn export_to_word(results: Vec<ExtractionResult>, output_path: String) -> Result<String, String> {
    let mut html_content = String::new();

    // HTML 头部
    html_content.push_str("<!DOCTYPE html>\n");
    html_content.push_str("<html lang=\"zh-CN\">\n");
    html_content.push_str("<head>\n");
    html_content.push_str("  <meta charset=\"UTF-8\">\n");
    html_content.push_str("  <title>提取结果</title>\n");
    html_content.push_str("  <style>\n");
    html_content.push_str("    body { font-family: '微软雅黑', 'SimSun', sans-serif; margin: 20px; }\n");
    html_content.push_str("    h1 { color: #333; text-align: center; }\n");
    html_content.push_str("    h2 { color: #409eff; border-bottom: 1px solid #ddd; padding-bottom: 5px; }\n");
    html_content.push_str("    h3 { color: #666; }\n");
    html_content.push_str("    .location { color: #999; font-size: 12px; font-style: italic; }\n");
    html_content.push_str("    .content { margin: 10px 0; line-height: 1.6; }\n");
    html_content.push_str("    table { border-collapse: collapse; width: 100%; margin: 10px 0; }\n");
    html_content.push_str("    th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
    html_content.push_str("    th { background-color: #f5f5f5; }\n");
    html_content.push_str("    .divider { border-top: 1px solid #eee; margin: 20px 0; }\n");
    html_content.push_str("  </style>\n");
    html_content.push_str("</head>\n");
    html_content.push_str("<body>\n");

    // 标题
    html_content.push_str("  <h1>提取结果</h1>\n");

    let mut current_doc = String::new();

    for result in &results {
        // 文档名称作为章节标题
        if current_doc != result.document_name {
            current_doc = result.document_name.clone();
            html_content.push_str(&format!("  <h2>{}</h2>\n", escape_html(&current_doc)));
        }

        // 规则名称
        html_content.push_str(&format!("  <h3>【{}】</h3>\n", escape_html(&result.rule_name)));

        // 位置信息
        html_content.push_str(&format!("  <div class=\"location\">位置: {}</div>\n", escape_html(&result.position.description)));

        // 内容
        if result.content_type == crate::models::ContentType::Table {
            // 表格内容：解析并创建 HTML 表格
            html_content.push_str(&format_html_table(&result.content));
        } else {
            // 文本内容：添加段落
            html_content.push_str("  <div class=\"content\">\n");
            for line in result.content.lines() {
                html_content.push_str(&format!("    <p>{}</p>\n", escape_html(line)));
            }
            html_content.push_str("  </div>\n");
        }

        // 分隔线
        html_content.push_str("  <div class=\"divider\"></div>\n");
    }

    // HTML 尾部
    html_content.push_str("</body>\n");
    html_content.push_str("</html>\n");

    // 保存文件
    let mut file = File::create(&output_path).map_err(|e| e.to_string())?;
    file.write_all(html_content.as_bytes()).map_err(|e| e.to_string())?;

    Ok(output_path)
}

/// 转义 HTML 特殊字符
fn escape_html(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
}

/// 将表格内容转换为 HTML 表格
fn format_html_table(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return String::new();
    }

    let mut html = String::new();
    html.push_str("  <table>\n");

    for (idx, line) in lines.iter().enumerate() {
        let cells: Vec<&str> = line.split("|").map(|s| s.trim()).collect();

        if idx == 0 {
            // 第一行作为表头
            html.push_str("    <tr>\n");
            for cell in &cells {
                html.push_str(&format!("      <th>{}</th>\n", escape_html(cell)));
            }
            html.push_str("    </tr>\n");
        } else {
            html.push_str("    <tr>\n");
            for cell in &cells {
                html.push_str(&format!("      <td>{}</td>\n", escape_html(cell)));
            }
            html.push_str("    </tr>\n");
        }
    }

    html.push_str("  </table>\n");
    html
}
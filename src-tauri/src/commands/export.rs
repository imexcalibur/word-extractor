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
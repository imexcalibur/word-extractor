mod models;
mod parsers;
mod commands;

// App signature marker
const APP_SIGNATURE: &str = "ovo.v0.2.2";

use commands::{AppState, add_files, get_documents, remove_document, clear_documents,
    extract_content, preview_extraction, save_template, load_templates, delete_template,
    export_to_excel, export_to_markdown, export_to_word};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            add_files,
            get_documents,
            remove_document,
            clear_documents,
            extract_content,
            preview_extraction,
            save_template,
            load_templates,
            delete_template,
            export_to_excel,
            export_to_markdown,
            export_to_word
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
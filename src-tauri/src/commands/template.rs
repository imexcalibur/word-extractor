use tauri::State;
use std::sync::Mutex;
use crate::models::{Document, Template};
use std::fs;
use std::path::PathBuf;

pub struct AppState {
    pub documents: Mutex<Vec<Document>>,
    pub templates: Mutex<Vec<Template>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            documents: Mutex::new(Vec::new()),
            templates: Mutex::new(Vec::new()),
        }
    }
}

fn get_templates_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".word-extractor").join("templates.json")
}

#[tauri::command]
pub fn save_template(template: Template, state: State<AppState>) -> Result<(), String> {
    let mut templates = state.templates.lock().unwrap();

    templates.push(template.clone());

    let path = get_templates_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let templates_vec: Vec<Template> = templates.clone();
    let json = serde_json::to_string_pretty(&templates_vec).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn load_templates(state: State<AppState>) -> Result<Vec<Template>, String> {
    let path = get_templates_path();

    if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let templates: Vec<Template> = serde_json::from_str(&content).map_err(|e| e.to_string())?;

        let mut state_templates = state.templates.lock().unwrap();
        *state_templates = templates.clone();

        Ok(templates)
    } else {
        Ok(Vec::new())
    }
}

#[tauri::command]
pub fn delete_template(template_id: String, state: State<AppState>) -> Result<(), String> {
    let mut templates = state.templates.lock().unwrap();
    templates.retain(|t| t.id != template_id);

    let path = get_templates_path();
    let templates_vec: Vec<Template> = templates.clone();
    let json = serde_json::to_string_pretty(&templates_vec).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;

    Ok(())
}
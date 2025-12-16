mod db;

use db::{Category, RootWithCategories};
use tauri::Manager;

#[tauri::command]
fn get_categories() -> Result<Vec<Category>, String> {
    db::get_categories().map_err(|e| e.to_string())
}

#[tauri::command]
fn import_keywords(keywords: Vec<String>) -> Result<(), String> {
    db::import_keywords(keywords).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_roots(
    search: Option<String>,
    category_ids: Option<Vec<i64>>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<RootWithCategories>, i64), String> {
    db::get_roots(search, category_ids, sort_by, sort_order, page, page_size)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn update_root_translation(id: i64, translation: String) -> Result<(), String> {
    db::update_root_translation(id, translation).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_root_category(root_id: i64, category_id: i64) -> Result<(), String> {
    db::add_root_category(root_id, category_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_root_category(root_id: i64, category_id: i64) -> Result<(), String> {
    db::remove_root_category(root_id, category_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_stats() -> Result<(i64, i64), String> {
    db::get_stats().map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_all_data() -> Result<(), String> {
    db::clear_all_data().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            db::init_db(app_data_dir).expect("Failed to initialize database");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_categories,
            import_keywords,
            get_roots,
            update_root_translation,
            add_root_category,
            remove_root_category,
            get_stats,
            clear_all_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

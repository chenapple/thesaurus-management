mod db;

use db::{Category, Product, RootWithCategories};
use tauri::Manager;

// ==================== 产品管理 ====================

#[tauri::command]
fn get_products() -> Result<Vec<Product>, String> {
    db::get_products().map_err(|e| e.to_string())
}

#[tauri::command]
fn create_product(name: String, sku: Option<String>, asin: Option<String>) -> Result<i64, String> {
    db::create_product(name, sku, asin).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_product(id: i64, name: String, sku: Option<String>, asin: Option<String>) -> Result<(), String> {
    db::update_product(id, name, sku, asin).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_product(id: i64) -> Result<(), String> {
    db::delete_product(id).map_err(|e| e.to_string())
}

// ==================== 分类 ====================

#[tauri::command]
fn get_categories() -> Result<Vec<Category>, String> {
    db::get_categories().map_err(|e| e.to_string())
}

// ==================== 关键词和词根 ====================

#[tauri::command]
fn import_keywords(product_id: i64, keywords: Vec<String>) -> Result<(), String> {
    db::import_keywords(product_id, keywords).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_roots(
    product_id: Option<i64>,
    search: Option<String>,
    category_ids: Option<Vec<i64>>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<RootWithCategories>, i64), String> {
    db::get_roots(product_id, search, category_ids, sort_by, sort_order, page, page_size)
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
fn get_stats(product_id: Option<i64>) -> Result<(i64, i64), String> {
    db::get_stats(product_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_category_counts(product_id: i64) -> Result<Vec<(i64, i64)>, String> {
    db::get_category_counts(product_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_product_data(product_id: i64) -> Result<(), String> {
    db::clear_product_data(product_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_untranslated_roots(product_id: i64) -> Result<Vec<String>, String> {
    db::get_untranslated_roots(product_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn batch_update_root_analysis(
    product_id: i64,
    updates: Vec<(String, String, Vec<String>)>,
) -> Result<(), String> {
    db::batch_update_root_analysis(product_id, updates).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            db::init_db(app_data_dir).expect("Failed to initialize database");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 产品管理
            get_products,
            create_product,
            update_product,
            delete_product,
            // 分类
            get_categories,
            // 关键词和词根
            import_keywords,
            get_roots,
            update_root_translation,
            add_root_category,
            remove_root_category,
            get_stats,
            get_category_counts,
            clear_product_data,
            get_untranslated_roots,
            batch_update_root_analysis,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

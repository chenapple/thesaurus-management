mod db;

use db::{BackupInfo, Category, KeywordData, Product, RootWithCategories, TrafficLevelStats, UncategorizedKeyword, WorkflowStatus};
use tauri::Manager;

// ==================== 产品管理 ====================

#[tauri::command]
fn get_products() -> Result<Vec<Product>, String> {
    db::get_products().map_err(|e| e.to_string())
}

#[tauri::command]
fn create_product(name: String, country: Option<String>) -> Result<i64, String> {
    db::create_product(name, country).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_product(id: i64, name: String, country: Option<String>) -> Result<(), String> {
    db::update_product(id, name, country).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_product(id: i64) -> Result<(), String> {
    db::delete_product(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_product_headers(id: i64, cpc_header: Option<String>, bid_range_header: Option<String>) -> Result<(), String> {
    db::update_product_headers(id, cpc_header, bid_range_header).map_err(|e| e.to_string())
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

// ==================== 关键词完整数据 ====================

#[tauri::command]
fn import_keyword_data(product_id: i64, data_list: Vec<KeywordData>) -> Result<(), String> {
    db::import_keyword_data(product_id, data_list).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_keyword_data(
    product_id: i64,
    search: Option<String>,
    traffic_levels: Option<Vec<String>>,
    relevance_levels: Option<Vec<String>>,
    primary_categories: Option<Vec<String>>,
    orderliness_values: Option<Vec<String>>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<KeywordData>, i64), String> {
    db::get_keyword_data(product_id, search, traffic_levels, relevance_levels, primary_categories, orderliness_values, sort_by, sort_order, page, page_size)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn update_keyword_field(id: i64, field: String, value: String) -> Result<(), String> {
    db::update_keyword_field(id, &field, &value).map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_keyword_data(product_id: i64) -> Result<(), String> {
    db::clear_keyword_data(product_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_keyword_data_stats(product_id: i64) -> Result<i64, String> {
    db::get_keyword_data_stats(product_id).map_err(|e| e.to_string())
}

// ==================== 流量级别管理 ====================

#[tauri::command]
fn update_product_thresholds(id: i64, big_word_threshold: i64, medium_word_threshold: i64) -> Result<(), String> {
    db::update_product_thresholds(id, big_word_threshold, medium_word_threshold).map_err(|e| e.to_string())
}

#[tauri::command]
fn calculate_traffic_levels(product_id: i64, big_threshold: i64, medium_threshold: i64) -> Result<(), String> {
    db::calculate_traffic_levels(product_id, big_threshold, medium_threshold).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_traffic_level_stats(product_id: i64) -> Result<TrafficLevelStats, String> {
    db::get_traffic_level_stats(product_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn recommend_threshold(product_id: i64, target_big_count: i64) -> Result<i64, String> {
    db::recommend_threshold(product_id, target_big_count).map_err(|e| e.to_string())
}

// ==================== 流量占比计算 ====================

#[tauri::command]
fn calculate_traffic_share(product_id: i64) -> Result<(), String> {
    db::calculate_traffic_share(product_id).map_err(|e| e.to_string())
}

// ==================== 关键词分类管理 ====================

#[tauri::command]
fn get_uncategorized_keywords(product_id: i64) -> Result<Vec<UncategorizedKeyword>, String> {
    db::get_uncategorized_keywords(product_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn batch_update_keyword_categories(
    product_id: i64,
    updates: Vec<(String, String, String, String)>,
) -> Result<(), String> {
    db::batch_update_keyword_categories(product_id, updates).map_err(|e| e.to_string())
}

#[tauri::command]
fn calculate_phrase_tags(product_id: i64) -> Result<(), String> {
    db::calculate_phrase_tags(product_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn calculate_orderliness(product_id: i64) -> Result<(), String> {
    db::calculate_orderliness(product_id).map_err(|e| e.to_string())
}

// ==================== 流程状态 ====================

#[tauri::command]
fn get_workflow_status(product_id: i64) -> Result<WorkflowStatus, String> {
    db::get_workflow_status(product_id).map_err(|e| e.to_string())
}

// ==================== 备份管理 ====================

#[tauri::command]
fn create_backup(product_id: i64, backup_name: Option<String>) -> Result<i64, String> {
    db::create_backup(product_id, backup_name).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_backups(product_id: i64) -> Result<Vec<BackupInfo>, String> {
    db::get_backups(product_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn restore_backup(backup_id: i64) -> Result<(), String> {
    db::restore_backup(backup_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_backup(backup_id: i64) -> Result<(), String> {
    db::delete_backup(backup_id).map_err(|e| e.to_string())
}

// ==================== API Key 存储 ====================

#[tauri::command]
fn set_api_key(key_name: String, api_key: String) -> Result<(), String> {
    db::set_setting(&key_name, &api_key).map_err(|e| format!("保存 API Key 失败: {}", e))
}

#[tauri::command]
fn get_api_key(key_name: String) -> Result<Option<String>, String> {
    db::get_setting(&key_name).map_err(|e| format!("获取 API Key 失败: {}", e))
}

#[tauri::command]
fn delete_api_key(key_name: String) -> Result<(), String> {
    db::delete_setting(&key_name).map_err(|e| format!("删除 API Key 失败: {}", e))
}

#[tauri::command]
fn has_api_key(key_name: String) -> Result<bool, String> {
    match db::get_setting(&key_name) {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(false),
        Err(e) => Err(format!("检查 API Key 失败: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
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
            update_product_headers,
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
            // 关键词完整数据
            import_keyword_data,
            get_keyword_data,
            update_keyword_field,
            clear_keyword_data,
            get_keyword_data_stats,
            // 流量级别管理
            update_product_thresholds,
            calculate_traffic_levels,
            get_traffic_level_stats,
            recommend_threshold,
            // 流量占比计算
            calculate_traffic_share,
            // 关键词分类管理
            get_uncategorized_keywords,
            batch_update_keyword_categories,
            // 词组打标
            calculate_phrase_tags,
            // 有序性计算
            calculate_orderliness,
            // 流程状态
            get_workflow_status,
            // 备份管理
            create_backup,
            get_backups,
            restore_backup,
            delete_backup,
            // API Key 安全存储
            set_api_key,
            get_api_key,
            delete_api_key,
            has_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

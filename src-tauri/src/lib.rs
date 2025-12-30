mod db;
mod crawler;
mod scheduler;
mod notification;
mod installer;

use db::{BackupInfo, Category, KeywordData, KeywordMonitoring, MonitoringSparkline, MonitoringStats, Product, RankingHistory, RankingSnapshot, RootWithCategories, TrafficLevelStats, UncategorizedKeyword, WorkflowStatus};
use scheduler::{SchedulerSettings, SchedulerStatus, SCHEDULER};
use tauri::Manager;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState};
use tauri::Emitter;

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

// ==================== 关键词排名监控 ====================

#[tauri::command]
fn add_keyword_monitoring(
    product_id: i64,
    keyword: String,
    asin: String,
    country: String,
    priority: Option<String>,
) -> Result<i64, String> {
    db::add_keyword_monitoring(product_id, keyword, asin, country, priority)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_keyword_monitoring_list(
    product_id: i64,
    country: Option<String>,
    priority: Option<String>,
    is_active: Option<bool>,
    search: Option<String>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<KeywordMonitoring>, i64), String> {
    db::get_keyword_monitoring_list(product_id, country, priority, is_active, search, sort_by, sort_order, page, page_size)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn update_keyword_monitoring(
    id: i64,
    priority: Option<String>,
    is_active: Option<bool>,
) -> Result<(), String> {
    db::update_keyword_monitoring(id, priority, is_active)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn update_keyword_monitoring_tags(id: i64, tags: Option<String>) -> Result<(), String> {
    db::update_keyword_monitoring_tags(id, tags)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_keyword_monitoring(id: i64) -> Result<(), String> {
    db::delete_keyword_monitoring(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn batch_delete_keyword_monitoring(ids: Vec<i64>) -> Result<(), String> {
    db::batch_delete_keyword_monitoring(ids).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_monitoring_stats(product_id: i64) -> Result<MonitoringStats, String> {
    db::get_monitoring_stats(product_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_ranking_history(monitoring_id: i64, days: i64) -> Result<Vec<RankingHistory>, String> {
    db::get_ranking_history(monitoring_id, days).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_ranking_snapshots(keyword: String, country: String, days: i64) -> Result<Vec<RankingSnapshot>, String> {
    db::get_ranking_snapshots(&keyword, &country, days).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_monitoring_sparklines(product_id: i64, days: i64) -> Result<Vec<MonitoringSparkline>, String> {
    db::get_monitoring_sparklines(product_id, days).map_err(|e| e.to_string())
}

// 检测单个关键词排名
#[tauri::command]
async fn check_single_ranking(
    monitoring_id: i64,
    max_pages: Option<i64>,
) -> Result<crawler::RankingResult, String> {
    // 获取监控记录
    let monitoring = db::get_keyword_monitoring_by_id(monitoring_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "监控记录不存在".to_string())?;

    // 使用异步爬虫检测
    let result = crawler::search_keyword(
        monitoring.keyword.clone(),
        monitoring.asin.clone(),
        monitoring.country.clone(),
        max_pages.unwrap_or(5),
    )
    .await;

    // 更新数据库
    if result.error.is_none() {
        let product_info = result.product_info.as_ref();
        db::update_ranking_result(
            monitoring_id,
            result.organic_rank,
            result.organic_page,
            result.sponsored_rank,
            result.sponsored_page,
            product_info.and_then(|p| p.image_url.clone()),
            product_info.and_then(|p| p.price.clone()),
            product_info.and_then(|p| p.reviews_count),
            product_info.and_then(|p| p.rating),
        )
        .map_err(|e| e.to_string())?;

        // 保存竞品快照
        if !result.organic_top_50.is_empty() || !result.sponsored_top_20.is_empty() {
            db::save_ranking_snapshot(
                &monitoring.keyword,
                &monitoring.country,
                Some(serde_json::to_string(&result.organic_top_50).unwrap_or_default()),
                Some(serde_json::to_string(&result.sponsored_top_20).unwrap_or_default()),
            )
            .ok();
        }
    }

    Ok(result)
}

// 批量检测排名（带进度回调）
#[tauri::command]
async fn check_all_rankings(
    app: tauri::AppHandle,
    product_id: i64,
    max_pages: Option<i64>,
    hours_since_last_check: Option<i64>,
) -> Result<Vec<(i64, crawler::RankingResult)>, String> {
    // 获取待检测的监控记录
    // hours_since_last_check: None 默认24小时，Some(0) 表示无时间限制
    let pending = db::get_pending_monitoring_checks(product_id, hours_since_last_check.unwrap_or(24))
        .map_err(|e| e.to_string())?;

    if pending.is_empty() {
        return Ok(Vec::new());
    }

    let total = pending.len() as i64;

    // 发送开始事件
    app.emit("ranking-check-start", serde_json::json!({
        "total": total
    })).ok();

    // 准备检测数据
    let keywords: Vec<(i64, String, String, String)> = pending
        .into_iter()
        .map(|m| (m.id, m.keyword, m.asin, m.country))
        .collect();

    // 执行批量检测
    let app_clone = app.clone();
    let results = crawler::check_rankings_batch(
        keywords,
        max_pages.unwrap_or(5),
        move |current, total, message| {
            // 发送进度事件到前端
            app_clone.emit("ranking-check-progress", serde_json::json!({
                "current": current,
                "total": total,
                "message": message
            })).ok();
        },
    )
    .await;

    // 更新数据库
    for (monitoring_id, ref result) in &results {
        if result.error.is_none() {
            let product_info = result.product_info.as_ref();
            db::update_ranking_result(
                *monitoring_id,
                result.organic_rank,
                result.organic_page,
                result.sponsored_rank,
                result.sponsored_page,
                product_info.and_then(|p| p.image_url.clone()),
                product_info.and_then(|p| p.price.clone()),
                product_info.and_then(|p| p.reviews_count),
                product_info.and_then(|p| p.rating),
            )
            .ok();

            // 保存竞品快照
            if !result.organic_top_50.is_empty() || !result.sponsored_top_20.is_empty() {
                db::save_ranking_snapshot(
                    &result.keyword,
                    &result.country,
                    Some(serde_json::to_string(&result.organic_top_50).unwrap_or_default()),
                    Some(serde_json::to_string(&result.sponsored_top_20).unwrap_or_default()),
                )
                .ok();
            }
        }
    }

    // 发送完成事件
    app.emit("ranking-check-complete", serde_json::json!({
        "total": results.len(),
        "success": results.iter().filter(|(_, r)| r.error.is_none()).count(),
        "failed": results.iter().filter(|(_, r)| r.error.is_some()).count()
    })).ok();

    Ok(results)
}

// 检测选中的关键词排名
#[tauri::command]
async fn check_selected_rankings(
    app: tauri::AppHandle,
    ids: Vec<i64>,
    max_pages: Option<i64>,
) -> Result<Vec<(i64, crawler::RankingResult)>, String> {
    // 根据ID列表获取监控记录
    let pending = db::get_monitoring_by_ids(&ids)
        .map_err(|e| e.to_string())?;

    if pending.is_empty() {
        return Ok(Vec::new());
    }

    let total = pending.len() as i64;

    // 发送开始事件
    app.emit("ranking-check-start", serde_json::json!({
        "total": total
    })).ok();

    // 准备检测数据
    let keywords: Vec<(i64, String, String, String)> = pending
        .into_iter()
        .map(|m| (m.id, m.keyword, m.asin, m.country))
        .collect();

    // 执行批量检测
    let app_clone = app.clone();
    let results = crawler::check_rankings_batch(
        keywords,
        max_pages.unwrap_or(5),
        move |current, total, message| {
            // 发送进度事件到前端
            app_clone.emit("ranking-check-progress", serde_json::json!({
                "current": current,
                "total": total,
                "message": message
            })).ok();
        },
    )
    .await;

    // 更新数据库
    for (monitoring_id, ref result) in &results {
        if result.error.is_none() {
            let product_info = result.product_info.as_ref();
            db::update_ranking_result(
                *monitoring_id,
                result.organic_rank,
                result.organic_page,
                result.sponsored_rank,
                result.sponsored_page,
                product_info.and_then(|p| p.image_url.clone()),
                product_info.and_then(|p| p.price.clone()),
                product_info.and_then(|p| p.reviews_count),
                product_info.and_then(|p| p.rating),
            )
            .ok();

            // 保存竞品快照
            if !result.organic_top_50.is_empty() || !result.sponsored_top_20.is_empty() {
                db::save_ranking_snapshot(
                    &result.keyword,
                    &result.country,
                    Some(serde_json::to_string(&result.organic_top_50).unwrap_or_default()),
                    Some(serde_json::to_string(&result.sponsored_top_20).unwrap_or_default()),
                )
                .ok();
            }
        }
    }

    // 发送完成事件
    app.emit("ranking-check-complete", serde_json::json!({
        "total": results.len(),
        "success": results.iter().filter(|(_, r)| r.error.is_none()).count(),
        "failed": results.iter().filter(|(_, r)| r.error.is_some()).count()
    })).ok();

    Ok(results)
}

// 批量添加关键词监控
#[tauri::command]
fn batch_add_keyword_monitoring(
    product_id: i64,
    items: Vec<(String, String, String, Option<String>)>, // (keyword, asin, country, priority)
) -> Result<Vec<i64>, String> {
    let mut ids = Vec::new();
    for (keyword, asin, country, priority) in items {
        let id = db::add_keyword_monitoring(product_id, keyword, asin, country, priority)
            .map_err(|e| e.to_string())?;
        ids.push(id);
    }
    Ok(ids)
}

// ==================== 调度器管理 ====================

#[tauri::command]
async fn get_scheduler_settings() -> Result<SchedulerSettings, String> {
    // 从数据库加载设置
    if let Ok(Some(json)) = db::get_setting("scheduler_settings") {
        serde_json::from_str(&json).map_err(|e| e.to_string())
    } else {
        Ok(SchedulerSettings::default())
    }
}

#[tauri::command]
async fn update_scheduler_settings(settings: SchedulerSettings) -> Result<(), String> {
    // 保存到数据库
    let json = serde_json::to_string(&settings).map_err(|e| e.to_string())?;
    db::set_setting("scheduler_settings", &json).map_err(|e| e.to_string())?;

    // 更新调度器
    SCHEDULER.update_settings(settings).await;

    Ok(())
}

#[tauri::command]
async fn start_scheduler() -> Result<(), String> {
    // 加载设置
    if let Ok(Some(json)) = db::get_setting("scheduler_settings") {
        if let Ok(settings) = serde_json::from_str::<SchedulerSettings>(&json) {
            SCHEDULER.update_settings(settings).await;
        }
    }

    SCHEDULER.start().await;
    Ok(())
}

#[tauri::command]
fn stop_scheduler() -> Result<(), String> {
    SCHEDULER.stop();
    Ok(())
}

#[tauri::command]
async fn get_scheduler_status() -> Result<SchedulerStatus, String> {
    Ok(SCHEDULER.get_status().await)
}

#[tauri::command]
fn get_task_logs(limit: Option<i64>) -> Result<Vec<db::SchedulerTaskLog>, String> {
    db::get_task_logs(limit.unwrap_or(20)).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_running_task() -> Result<Option<db::SchedulerTaskLog>, String> {
    db::get_running_task().map_err(|e| e.to_string())
}

// ==================== 依赖安装 ====================

#[tauri::command]
fn check_dependencies() -> Result<installer::DependencyStatus, String> {
    Ok(installer::check_dependency_status())
}

#[tauri::command]
async fn install_all_dependencies(app: tauri::AppHandle) -> Result<installer::InstallResult, String> {
    Ok(installer::install_all_dependencies(app).await)
}

#[tauri::command]
async fn install_playwright_only(app: tauri::AppHandle) -> Result<installer::InstallResult, String> {
    Ok(installer::install_playwright_only(app).await)
}

// 优化事件管理
#[tauri::command]
fn add_optimization_event(
    product_id: i64,
    event_date: String,
    event_type: String,
    event_sub_type: String,
    title: String,
    description: Option<String>,
    target_asin: Option<String>,
    affected_keywords: Option<String>,
) -> Result<i64, String> {
    db::add_optimization_event(product_id, event_date, event_type, event_sub_type, title, description, target_asin, affected_keywords)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_optimization_events(
    product_id: i64,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<db::OptimizationEvent>, String> {
    db::get_optimization_events(product_id, start_date, end_date)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn update_optimization_event(
    id: i64,
    event_date: String,
    event_type: String,
    event_sub_type: String,
    title: String,
    description: Option<String>,
    target_asin: Option<String>,
    affected_keywords: Option<String>,
) -> Result<(), String> {
    db::update_optimization_event(id, event_date, event_type, event_sub_type, title, description, target_asin, affected_keywords)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_optimization_event(id: i64) -> Result<(), String> {
    db::delete_optimization_event(id).map_err(|e| e.to_string())
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
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            db::init_db(app_data_dir).expect("Failed to initialize database");

            // 自动启动调度器（如果已启用）
            tauri::async_runtime::spawn(async {
                // 加载设置
                if let Ok(Some(json)) = db::get_setting("scheduler_settings") {
                    if let Ok(settings) = serde_json::from_str::<scheduler::SchedulerSettings>(&json) {
                        if settings.enabled {
                            SCHEDULER.update_settings(settings).await;
                            SCHEDULER.start().await;
                            println!("[Scheduler] Auto-started on app launch");
                        }
                    }
                }
            });

            // 设置系统托盘
            let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let check_item = MenuItem::with_id(app, "check", "立即检测", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_item, &check_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "check" => {
                            app.emit("manual-check-requested", ()).ok();
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

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
            // 关键词排名监控
            add_keyword_monitoring,
            get_keyword_monitoring_list,
            update_keyword_monitoring,
            update_keyword_monitoring_tags,
            delete_keyword_monitoring,
            batch_delete_keyword_monitoring,
            get_monitoring_stats,
            get_ranking_history,
            get_ranking_snapshots,
            get_monitoring_sparklines,
            check_single_ranking,
            check_all_rankings,
            check_selected_rankings,
            batch_add_keyword_monitoring,
            // 调度器管理
            get_scheduler_settings,
            update_scheduler_settings,
            start_scheduler,
            stop_scheduler,
            get_scheduler_status,
            get_task_logs,
            get_running_task,
            // 依赖安装
            check_dependencies,
            install_all_dependencies,
            install_playwright_only,
            // 优化事件管理
            add_optimization_event,
            get_optimization_events,
            update_optimization_event,
            delete_optimization_event,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

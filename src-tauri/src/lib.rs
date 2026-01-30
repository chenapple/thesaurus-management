mod db;
mod crawler;
mod scheduler;
mod notification;
mod installer;
mod knowledge_base;
mod ai;
mod keychain;

use db::{BackupInfo, Category, KeywordData, KeywordMonitoring, MonitoringSparkline, MonitoringStats, Product, RankingHistory, RankingSnapshot, RootWithCategories, TrafficLevelStats, UncategorizedKeyword, WorkflowStatus};
use db::{KbCategory, KbDocument, KbChunk, KbSearchResult, KbConversation, KbMessage, KbDocumentLink, KbDocumentCategory};
use db::ScProject;
use db::{QuickNote, ExchangeRateCache, ExchangeRateHistory};
use db::{WeeklyReport, WeeklyReportEntry, WeeklyReportContent};
use scheduler::{SchedulerSettings, SchedulerStatus, SCHEDULER, MARKET_RESEARCH_SCHEDULER};
use crawler::{BsrResult, SubcategoryResult};
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
fn set_root_negative(id: i64, is_negative: bool) -> Result<i64, String> {
    db::set_root_negative(id, is_negative).map_err(|e| e.to_string())
}

#[tauri::command]
fn batch_set_roots_negative(ids: Vec<i64>, is_negative: bool) -> Result<i64, String> {
    db::batch_set_roots_negative(ids, is_negative).map_err(|e| e.to_string())
}

#[tauri::command]
fn batch_set_roots_negative_by_words(
    product_id: i64,
    words: Vec<String>,
    is_negative: bool,
) -> Result<i64, String> {
    db::batch_set_roots_negative_by_words(product_id, words, is_negative).map_err(|e| e.to_string())
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
// 注意：keychain 存储在某些环境下不稳定，暂时使用 SQLite 存储
// TODO: 调查 keyring 库兼容性问题后再考虑启用 keychain

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

/// 迁移 API Key（keychain 功能暂时禁用，返回空列表）
#[tauri::command]
fn migrate_api_keys() -> Result<Vec<String>, String> {
    Ok(vec![])
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

    // 获取爬虫设置
    let max_browsers = db::get_setting("max_browsers")
        .ok()
        .flatten()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(3);  // 默认3个并发浏览器

    let tabs_per_browser = db::get_setting("tabs_per_browser")
        .ok()
        .flatten()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(1);  // 默认1个标签页

    let proxy_list = db::get_setting("proxy_list")
        .ok()
        .flatten()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.lines().filter(|l| !l.trim().is_empty()).collect::<Vec<_>>().join(","));

    // 执行批量检测（并发模式）
    let app_clone = app.clone();
    let results = crawler::check_rankings_batch(
        keywords,
        max_pages.unwrap_or(5),
        max_browsers,
        tabs_per_browser,
        proxy_list,
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

    // 获取爬虫设置
    let max_browsers = db::get_setting("max_browsers")
        .ok()
        .flatten()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(3);  // 默认3个并发浏览器

    let tabs_per_browser = db::get_setting("tabs_per_browser")
        .ok()
        .flatten()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(1);  // 默认1个标签页

    let proxy_list = db::get_setting("proxy_list")
        .ok()
        .flatten()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.lines().filter(|l| !l.trim().is_empty()).collect::<Vec<_>>().join(","));

    // 执行批量检测（并发模式）
    let app_clone = app.clone();
    let results = crawler::check_rankings_batch(
        keywords,
        max_pages.unwrap_or(5),
        max_browsers,
        tabs_per_browser,
        proxy_list,
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

#[tauri::command]
fn clear_task_logs() -> Result<(), String> {
    db::clear_task_logs().map_err(|e| e.to_string())
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

#[tauri::command]
async fn install_pdf_dependencies(app: tauri::AppHandle) -> Result<installer::InstallResult, String> {
    Ok(installer::install_pdf_dependencies(app).await)
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
    screenshots: Option<String>,
) -> Result<i64, String> {
    db::add_optimization_event(product_id, event_date, event_type, event_sub_type, title, description, target_asin, affected_keywords, screenshots)
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
    screenshots: Option<String>,
) -> Result<(), String> {
    db::update_optimization_event(id, event_date, event_type, event_sub_type, title, description, target_asin, affected_keywords, screenshots)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_optimization_event(id: i64) -> Result<(), String> {
    db::delete_optimization_event(id).map_err(|e| e.to_string())
}

// 截图管理
#[tauri::command]
fn get_screenshots_dir(app: tauri::AppHandle) -> Result<String, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let screenshots_dir = app_data_dir.join("screenshots");
    std::fs::create_dir_all(&screenshots_dir).map_err(|e| e.to_string())?;
    screenshots_dir.to_str().ok_or("Invalid path".to_string()).map(|s| s.to_string())
}

#[tauri::command]
fn save_event_screenshot(
    app: tauri::AppHandle,
    event_id: i64,
    base64_data: String,
    index: usize,
) -> Result<String, String> {
    use base64::{Engine as _, engine::general_purpose};

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let screenshots_dir = app_data_dir.join("screenshots");
    std::fs::create_dir_all(&screenshots_dir).map_err(|e| e.to_string())?;

    // 生成文件名: {event_id}_{timestamp}_{index}.png
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();
    let filename = format!("{}_{}_{}_.png", event_id, timestamp, index);
    let file_path = screenshots_dir.join(&filename);

    // 解码 base64 数据（去掉 data:image/xxx;base64, 前缀）
    let base64_clean = if base64_data.contains(",") {
        base64_data.split(',').nth(1).unwrap_or(&base64_data)
    } else {
        &base64_data
    };

    let image_data = general_purpose::STANDARD
        .decode(base64_clean)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

    // 检查文件大小（最大 5MB）
    if image_data.len() > 5 * 1024 * 1024 {
        return Err("Image size exceeds 5MB limit".to_string());
    }

    std::fs::write(&file_path, image_data).map_err(|e| e.to_string())?;

    Ok(filename)
}

#[tauri::command]
fn delete_event_screenshot(
    app: tauri::AppHandle,
    filename: String,
) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let screenshots_dir = app_data_dir.join("screenshots");
    let file_path = screenshots_dir.join(&filename);

    if file_path.exists() {
        std::fs::remove_file(&file_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}

// ==================== 知识库管理 ====================

// 分类管理
#[tauri::command]
fn kb_create_category(name: String, parent_id: Option<i64>) -> Result<i64, String> {
    db::kb_create_category(name, parent_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_get_categories() -> Result<Vec<KbCategory>, String> {
    db::kb_get_categories().map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_delete_category(id: i64) -> Result<(), String> {
    db::kb_delete_category(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_update_category(id: i64, name: String) -> Result<(), String> {
    db::kb_update_category(id, name).map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_update_category_color(id: i64, color: String) -> Result<(), String> {
    db::kb_update_category_color(id, color).map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_update_categories_order(ids: Vec<i64>) -> Result<(), String> {
    db::kb_update_categories_order(ids).map_err(|e| e.to_string())
}

// 文档管理
#[tauri::command]
fn kb_add_document(
    category_id: Option<i64>,
    title: String,
    file_name: String,
    file_path: String,
    file_type: String,
    file_size: Option<i64>,
) -> Result<i64, String> {
    db::kb_add_document(category_id, title, file_name, file_path, file_type, file_size)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_update_document_status(id: i64, status: String, chunk_count: i64) -> Result<(), String> {
    db::kb_update_document_status(id, status, chunk_count).map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_update_document_category(id: i64, category_id: Option<i64>) -> Result<(), String> {
    db::kb_update_document_category(id, category_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_get_documents(category_id: Option<i64>) -> Result<Vec<KbDocument>, String> {
    db::kb_get_documents(category_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_delete_document(id: i64) -> Result<(), String> {
    db::kb_delete_document(id).map_err(|e| e.to_string())
}

// 文档分块
#[tauri::command]
fn kb_add_chunk(
    document_id: i64,
    chunk_index: i64,
    content: String,
    page_number: Option<i64>,
) -> Result<i64, String> {
    db::kb_add_chunk(document_id, chunk_index, content, page_number).map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_add_chunks_batch(
    document_id: i64,
    chunks: Vec<(String, Option<i64>)>,
) -> Result<i64, String> {
    db::kb_add_chunks_batch(document_id, chunks).map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_get_chunks(document_id: i64) -> Result<Vec<KbChunk>, String> {
    db::kb_get_chunks(document_id).map_err(|e| e.to_string())
}

// 知识库搜索
#[tauri::command]
fn kb_search(query: String, limit: i64) -> Result<Vec<KbSearchResult>, String> {
    db::kb_search(query, limit).map_err(|e| e.to_string())
}

// AI 对话管理
#[tauri::command]
fn kb_create_conversation(
    ai_provider: String,
    ai_model: Option<String>,
    title: Option<String>,
) -> Result<i64, String> {
    db::kb_create_conversation(ai_provider, ai_model, title).map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_get_conversations() -> Result<Vec<KbConversation>, String> {
    db::kb_get_conversations().map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_update_conversation_title(id: i64, title: String) -> Result<(), String> {
    db::kb_update_conversation_title(id, title).map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_delete_conversation(id: i64) -> Result<(), String> {
    db::kb_delete_conversation(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_add_message(
    conversation_id: i64,
    role: String,
    content: String,
    sources: Option<String>,
) -> Result<i64, String> {
    db::kb_add_message(conversation_id, role, content, sources).map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_get_messages(conversation_id: i64) -> Result<Vec<KbMessage>, String> {
    db::kb_get_messages(conversation_id).map_err(|e| e.to_string())
}

// ==================== 文档链接 ====================

#[tauri::command]
fn kb_add_document_link(source_id: i64, target_id: i64, link_text: Option<String>) -> Result<i64, String> {
    db::kb_add_document_link(source_id, target_id, link_text).map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_remove_document_link(source_id: i64, target_id: i64) -> Result<(), String> {
    db::kb_remove_document_link(source_id, target_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_get_document_links(doc_id: i64) -> Result<Vec<KbDocumentLink>, String> {
    db::kb_get_document_links(doc_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_get_document_backlinks(doc_id: i64) -> Result<Vec<KbDocumentLink>, String> {
    db::kb_get_document_backlinks(doc_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_get_all_links() -> Result<Vec<KbDocumentLink>, String> {
    db::kb_get_all_links().map_err(|e| e.to_string())
}

// ==================== 文档分类关联（多对多）====================

#[tauri::command]
fn kb_add_document_category(doc_id: i64, category_id: i64) -> Result<(), String> {
    db::kb_add_document_category(doc_id, category_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_remove_document_category(doc_id: i64, category_id: i64) -> Result<(), String> {
    db::kb_remove_document_category(doc_id, category_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_get_document_categories(doc_id: i64) -> Result<Vec<KbDocumentCategory>, String> {
    db::kb_get_document_categories(doc_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_get_documents_by_categories(category_id: i64) -> Result<Vec<KbDocument>, String> {
    db::kb_get_documents_by_categories(category_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn kb_set_document_categories(doc_id: i64, category_ids: Vec<i64>) -> Result<(), String> {
    db::kb_set_document_categories(doc_id, category_ids).map_err(|e| e.to_string())
}

/// 处理上传的文档：解析 + 分块 + 存储
#[tauri::command]
fn kb_process_document(document_id: i64, file_path: String) -> Result<i64, String> {
    use knowledge_base::{process_document, ChunkerConfig};

    // 配置分块参数
    let config = ChunkerConfig {
        chunk_size: 800,
        chunk_overlap: 100,
    };

    // 解析并分块文档
    let (doc, chunks) = process_document(&file_path, Some(config))
        .map_err(|e| format!("文档处理失败: {}", e))?;

    // 将分块存储到数据库
    let chunk_data: Vec<(String, Option<i64>)> = chunks
        .iter()
        .map(|c| (c.content.clone(), c.page_number))
        .collect();

    let chunk_count = db::kb_add_chunks_batch(document_id, chunk_data)
        .map_err(|e| format!("存储分块失败: {}", e))?;

    // 更新文档状态为已完成
    db::kb_update_document_status(document_id, "completed".to_string(), chunk_count)
        .map_err(|e| format!("更新状态失败: {}", e))?;

    Ok(chunk_count)
}

/// 从文档中提取嵌入的图片
#[tauri::command]
fn kb_extract_images(file_path: String) -> Result<Vec<knowledge_base::ExtractedImage>, String> {
    knowledge_base::extract_images(&file_path)
}

/// 读取文件并返回 base64 编码数据（用于 PDF OCR）
#[tauri::command]
fn kb_read_file_base64(file_path: String) -> Result<String, String> {
    use std::fs;
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    let data = fs::read(&file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    Ok(STANDARD.encode(&data))
}

/// PDF 转图片结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PdfPageImage {
    pub page_number: i32,
    pub mime_type: String,
    pub base64_data: String,
}

/// 将 PDF 转换为图片（用于 OCR）
#[tauri::command]
fn kb_pdf_to_images(file_path: String) -> Result<Vec<PdfPageImage>, String> {
    use std::process::Command;
    use std::env;

    // 获取脚本路径
    let exe_dir = env::current_exe()
        .map_err(|e| format!("获取程序路径失败: {}", e))?
        .parent()
        .ok_or("无法获取程序目录")?
        .to_path_buf();

    let script_path = exe_dir.join("scripts").join("pdf_to_images.py");

    // 如果脚本不存在，尝试开发环境路径
    let script_path = if script_path.exists() {
        script_path
    } else {
        // 开发环境：src-tauri/scripts/
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("scripts")
            .join("pdf_to_images.py")
    };

    if !script_path.exists() {
        return Err(format!("脚本不存在: {:?}", script_path));
    }

    // 查找 Python
    let python_candidates = if cfg!(target_os = "windows") {
        vec!["python", "python3", "py"]
    } else {
        vec!["python3", "python", "/opt/homebrew/bin/python3", "/usr/local/bin/python3"]
    };

    let mut python_cmd = None;
    for candidate in &python_candidates {
        if Command::new(candidate).arg("--version").output().is_ok() {
            python_cmd = Some(candidate.to_string());
            break;
        }
    }

    let python = python_cmd.ok_or("未找到 Python，请确保已安装 Python 3")?;

    // 调用 Python 脚本
    let output = Command::new(&python)
        .arg(&script_path)
        .arg(&file_path)
        .arg("200") // DPI
        .output()
        .map_err(|e| format!("执行脚本失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("脚本执行失败: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // 解析 JSON 结果
    let result: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("解析结果失败: {} - 原始输出: {}", e, stdout))?;

    if !result["success"].as_bool().unwrap_or(false) {
        let error = result["error"].as_str().unwrap_or("未知错误");
        return Err(error.to_string());
    }

    let images: Vec<PdfPageImage> = result["images"]
        .as_array()
        .ok_or("无效的图片数据")?
        .iter()
        .map(|img| PdfPageImage {
            page_number: img["page_number"].as_i64().unwrap_or(0) as i32,
            mime_type: img["mime_type"].as_str().unwrap_or("image/png").to_string(),
            base64_data: img["base64_data"].as_str().unwrap_or("").to_string(),
        })
        .collect();

    Ok(images)
}

/// 获取图片存储目录
fn get_images_dir() -> Result<std::path::PathBuf, String> {
    let home = dirs::home_dir().ok_or("无法获取用户目录")?;
    let images_dir = home.join(".thesaurus-kb").join("images");
    std::fs::create_dir_all(&images_dir)
        .map_err(|e| format!("创建图片目录失败: {}", e))?;
    Ok(images_dir)
}

/// 保存图片到本地并返回路径
fn save_image_to_disk(document_id: i64, image_name: &str, base64_data: &str) -> Result<String, String> {
    use base64::{Engine as _, engine::general_purpose};

    let images_dir = get_images_dir()?;

    // 生成唯一文件名: {document_id}_{timestamp}_{original_name}
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("获取时间戳失败: {}", e))?
        .as_millis();

    // 清理文件名中的非法字符
    let safe_name: String = image_name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '.' || c == '_' || c == '-' { c } else { '_' })
        .collect();

    let filename = format!("{}_{:x}_{}", document_id, timestamp, safe_name);
    let file_path = images_dir.join(&filename);

    // 解码 base64 并保存
    let decoded = general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| format!("Base64 解码失败: {}", e))?;

    std::fs::write(&file_path, decoded)
        .map_err(|e| format!("保存图片失败: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

/// 将图片识别结果作为 chunk 添加到文档（不保存图片）
#[tauri::command]
fn kb_add_image_chunk(document_id: i64, image_name: String, description: String) -> Result<i64, String> {
    let content = format!("[图片: {}]\n{}", image_name, description);
    db::kb_add_chunk(document_id, 0, content, None)
        .map_err(|e| format!("添加图片描述失败: {}", e))
}

/// 将图片识别结果作为 chunk 添加到文档（同时保存图片用于图文问答）
#[tauri::command]
fn kb_add_image_chunk_with_file(
    document_id: i64,
    image_name: String,
    description: String,
    base64_data: String
) -> Result<i64, String> {
    // 保存图片到本地
    let image_path = save_image_to_disk(document_id, &image_name, &base64_data)?;

    // 存储 chunk 并关联图片路径
    let content = format!("[图片: {}]\n{}", image_name, description);
    db::kb_add_chunk_with_image(document_id, 0, content, None, image_path)
        .map_err(|e| format!("添加图片描述失败: {}", e))
}

/// 更新分块的 embedding 向量
#[tauri::command]
fn kb_update_chunk_embedding(chunk_id: i64, embedding: Vec<f32>) -> Result<(), String> {
    db::kb_update_chunk_embedding(chunk_id, embedding)
        .map_err(|e| format!("更新 embedding 失败: {}", e))
}

/// 清除所有 embedding（用于迁移到新的 embedding 模型）
#[tauri::command]
fn kb_clear_all_embeddings() -> Result<i64, String> {
    db::kb_clear_all_embeddings()
        .map_err(|e| format!("清除 embedding 失败: {}", e))
}

/// 获取没有 embedding 的分块
#[tauri::command]
fn kb_get_chunks_without_embedding(document_id: i64) -> Result<Vec<db::KbChunk>, String> {
    db::kb_get_chunks_without_embedding(document_id)
        .map_err(|e| format!("获取分块失败: {}", e))
}

/// 获取文档的向量化统计（总分块数，已向量化数）
#[tauri::command]
fn kb_get_document_embedding_stats(document_id: i64) -> Result<(i64, i64), String> {
    db::kb_get_document_embedding_stats(document_id)
        .map_err(|e| format!("获取向量化统计失败: {}", e))
}

/// 向量相似度搜索（支持相关度阈值过滤）
#[tauri::command]
fn kb_vector_search(query_embedding: Vec<f32>, limit: i64, min_score: Option<f64>) -> Result<Vec<db::KbSearchResult>, String> {
    let threshold = min_score.unwrap_or(0.0); // 默认不过滤
    db::kb_vector_search(query_embedding, limit, threshold)
        .map_err(|e| format!("向量搜索失败: {}", e))
}

// ==================== 智能文案 ====================

#[tauri::command]
fn sc_create_project(name: String, scenario_type: String, marketplace: String, my_asin: Option<String>, product_id: Option<i64>) -> Result<i64, String> {
    db::sc_create_project(&name, &scenario_type, &marketplace, my_asin.as_deref(), product_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn sc_get_projects(scenario_type: Option<String>) -> Result<Vec<ScProject>, String> {
    db::sc_get_projects(scenario_type.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn sc_get_project(id: i64) -> Result<Option<ScProject>, String> {
    db::sc_get_project(id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn sc_update_project(id: i64, name: String, my_asin: Option<String>) -> Result<(), String> {
    db::sc_update_project(id, &name, my_asin.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn sc_update_project_status(id: i64, status: String) -> Result<(), String> {
    db::sc_update_project_status(id, &status)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn sc_update_my_product_info(id: i64, my_product_info: Option<String>) -> Result<(), String> {
    db::sc_update_my_product_info(id, my_product_info.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn sc_update_my_listing(
    id: i64,
    my_title: Option<String>,
    my_bullets: Option<String>,
    my_description: Option<String>,
) -> Result<(), String> {
    db::sc_update_my_listing(id, my_title.as_deref(), my_bullets.as_deref(), my_description.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn sc_delete_project(id: i64) -> Result<(), String> {
    db::sc_delete_project(id)
        .map_err(|e| e.to_string())
}

// ==================== 竞品管理 ====================

#[tauri::command]
fn sc_add_competitor(project_id: i64, asin: String, competitor_type: String) -> Result<i64, String> {
    db::sc_add_competitor(project_id, &asin, &competitor_type)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn sc_get_competitors(project_id: i64) -> Result<Vec<db::ScCompetitor>, String> {
    db::sc_get_competitors(project_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn sc_update_competitor_info(
    id: i64,
    title: Option<String>,
    price: Option<String>,
    rating: Option<String>,
    review_count: Option<i64>,
    bsr_rank: Option<String>,
    date_first_available: Option<String>,
    image_url: Option<String>,
    bullets: Option<String>,
    description: Option<String>,
) -> Result<(), String> {
    db::sc_update_competitor_info(
        id,
        title.as_deref(),
        price.as_deref(),
        rating.as_deref(),
        review_count,
        bsr_rank.as_deref(),
        date_first_available.as_deref(),
        image_url.as_deref(),
        bullets.as_deref(),
        description.as_deref(),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn sc_delete_competitor(id: i64) -> Result<(), String> {
    db::sc_delete_competitor(id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn sc_update_competitor_type(id: i64, competitor_type: String) -> Result<(), String> {
    db::sc_update_competitor_type(id, &competitor_type)
        .map_err(|e| e.to_string())
}

// 爬取竞品 Listing 信息
#[tauri::command]
async fn sc_fetch_competitor_listing(id: i64, asin: String, country: String) -> Result<crawler::ListingResult, String> {
    eprintln!("[sc_fetch_competitor_listing] 开始爬取: id={}, asin={}, country={}", id, asin, country);
    let result = crawler::fetch_listing_info(asin, country).await;
    eprintln!("[sc_fetch_competitor_listing] 爬取结果: title={:?}, price={:?}, error={:?}",
        result.title, result.price, result.error);

    // 如果成功获取数据，更新数据库
    if result.error.is_none() {
        let bullets_json = serde_json::to_string(&result.bullets).unwrap_or_else(|_| "[]".to_string());
        eprintln!("[sc_fetch_competitor_listing] 更新数据库: id={}, title={:?}", id, result.title);
        match db::sc_update_competitor_info(
            id,
            result.title.as_deref(),
            result.price.as_deref(),
            result.rating.as_deref(),
            result.review_count,
            result.bsr_rank.as_deref(),
            result.date_first_available.as_deref(),
            result.image_url.as_deref(),
            Some(&bullets_json),
            result.description.as_deref(),
        ) {
            Ok(_) => eprintln!("[sc_fetch_competitor_listing] 数据库更新成功"),
            Err(e) => eprintln!("[sc_fetch_competitor_listing] 数据库更新失败: {:?}", e),
        }
    }

    Ok(result)
}

// 批量爬取竞品 Listing 信息（复用同一个浏览器）
#[tauri::command]
async fn sc_fetch_competitors_batch(
    items: Vec<(i64, String, String)>,  // [(id, asin, country), ...]
    app: tauri::AppHandle,
) -> Result<Vec<(i64, crawler::ListingResult)>, String> {
    eprintln!("[sc_fetch_competitors_batch] 开始批量爬取: {} 个竞品", items.len());

    let app_clone = app.clone();
    let results = crawler::fetch_listings_batch(
        items,
        move |current, total, asin| {
            eprintln!("[sc_fetch_competitors_batch] 进度: {}/{} - {}", current, total, asin);
            // 发送进度事件到前端
            let _ = app_clone.emit("sc-batch-progress", serde_json::json!({
                "current": current,
                "total": total,
                "asin": asin
            }));
        }
    ).await;

    // 更新数据库
    for (id, result) in &results {
        if result.error.is_none() {
            let bullets_json = serde_json::to_string(&result.bullets).unwrap_or_else(|_| "[]".to_string());
            if let Err(e) = db::sc_update_competitor_info(
                *id,
                result.title.as_deref(),
                result.price.as_deref(),
                result.rating.as_deref(),
                result.review_count,
                result.bsr_rank.as_deref(),
                result.date_first_available.as_deref(),
                result.image_url.as_deref(),
                Some(&bullets_json),
                result.description.as_deref(),
            ) {
                eprintln!("[sc_fetch_competitors_batch] 更新数据库失败 id={}: {:?}", id, e);
            }
        }
    }

    eprintln!("[sc_fetch_competitors_batch] 批量爬取完成: {} 个结果", results.len());
    Ok(results)
}

// ==================== 评论爬取 ====================

// 爬取竞品评论
#[tauri::command]
async fn sc_fetch_competitor_reviews(id: i64, asin: String, country: String) -> Result<crawler::ReviewResult, String> {
    eprintln!("[sc_fetch_competitor_reviews] 开始爬取: id={}, asin={}, country={}", id, asin, country);
    let result = crawler::fetch_reviews(asin, country).await;
    eprintln!("[sc_fetch_competitor_reviews] 爬取结果: total={}, error={:?}",
        result.summary.total, result.error);

    // 如果成功获取数据，保存到数据库
    if result.error.is_none() && !result.reviews.is_empty() {
        let reviews: Vec<db::ReviewInput> = result.reviews.iter().map(|r| {
            db::ReviewInput {
                star_rating: r.star_rating,
                review_text: r.review_text.clone(),
                review_title: r.review_title.clone(),
                review_date: r.review_date.clone(),
                helpful_votes: r.helpful_votes,
            }
        }).collect();

        match db::sc_add_reviews_batch(id, &reviews) {
            Ok(count) => eprintln!("[sc_fetch_competitor_reviews] 保存 {} 条评论", count),
            Err(e) => eprintln!("[sc_fetch_competitor_reviews] 保存评论失败: {:?}", e),
        }
    }

    Ok(result)
}

// 获取竞品的评论列表
#[tauri::command]
fn sc_get_competitor_reviews(competitor_id: i64) -> Result<Vec<db::ScReview>, String> {
    db::sc_get_reviews(competitor_id)
        .map_err(|e| e.to_string())
}

// 获取评论统计摘要
#[tauri::command]
fn sc_get_reviews_summary(competitor_id: i64) -> Result<db::ScReviewSummary, String> {
    db::sc_get_reviews_summary(competitor_id)
        .map_err(|e| e.to_string())
}

// ==================== BSR 爬虫 ====================

#[tauri::command]
async fn fetch_category_bsr(marketplace: String, category_id: String) -> Result<BsrResult, String> {
    eprintln!("[fetch_category_bsr] 开始爬取: marketplace={}, category_id={}", marketplace, category_id);
    let result = crawler::fetch_category_bsr(marketplace, category_id).await;
    eprintln!("[fetch_category_bsr] 爬取完成: products={}, error={:?}", result.products.len(), result.error);
    Ok(result)
}

#[tauri::command]
async fn discover_subcategories(marketplace: String, parent_category: String) -> Result<SubcategoryResult, String> {
    eprintln!("[discover_subcategories] 开始发现子类目: marketplace={}, parent={}", marketplace, parent_category);
    let result = crawler::discover_subcategories(marketplace, parent_category).await;
    eprintln!("[discover_subcategories] 发现完成: count={}, error={:?}", result.subcategories.len(), result.error);
    Ok(result)
}

#[tauri::command]
async fn fetch_listing_info(asin: String, country: String) -> Result<crawler::ListingResult, String> {
    eprintln!("[fetch_listing_info] 开始爬取: asin={}, country={}", asin, country);
    let result = crawler::fetch_listing_info(asin, country).await;
    eprintln!("[fetch_listing_info] 爬取完成: title={:?}, error={:?}", result.title, result.error);
    Ok(result)
}

// ==================== 市场调研监控任务 ====================

#[tauri::command]
fn get_market_research_tasks() -> Result<Vec<db::MarketResearchTask>, String> {
    db::get_market_research_tasks().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_market_research_task(id: i64) -> Result<Option<db::MarketResearchTask>, String> {
    db::get_market_research_task(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_market_research_task(
    name: String,
    marketplace: String,
    category_id: String,
    category_name: Option<String>,
    ai_provider: String,
    ai_model: Option<String>,
    schedule_type: String,
    schedule_days: Option<String>,
    schedule_time: String,
) -> Result<i64, String> {
    db::create_market_research_task(
        &name,
        &marketplace,
        &category_id,
        category_name.as_deref(),
        &ai_provider,
        ai_model.as_deref(),
        &schedule_type,
        schedule_days.as_deref(),
        &schedule_time,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn update_market_research_task(
    id: i64,
    name: String,
    marketplace: String,
    category_id: String,
    category_name: Option<String>,
    ai_provider: String,
    ai_model: Option<String>,
    schedule_type: String,
    schedule_days: Option<String>,
    schedule_time: String,
    is_enabled: bool,
) -> Result<(), String> {
    db::update_market_research_task(
        id,
        &name,
        &marketplace,
        &category_id,
        category_name.as_deref(),
        &ai_provider,
        ai_model.as_deref(),
        &schedule_type,
        schedule_days.as_deref(),
        &schedule_time,
        is_enabled,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_market_research_task(id: i64) -> Result<(), String> {
    db::delete_market_research_task(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_pending_research_tasks() -> Result<Vec<db::MarketResearchTask>, String> {
    db::get_pending_research_tasks().map_err(|e| e.to_string())
}

// BSR 快照管理

#[tauri::command]
fn save_bsr_snapshot(
    marketplace: String,
    category_id: String,
    category_name: Option<String>,
    products_json: String,
    product_count: i64,
) -> Result<i64, String> {
    db::save_bsr_snapshot(
        &marketplace,
        &category_id,
        category_name.as_deref(),
        &products_json,
        product_count,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_bsr_history(marketplace: String, category_id: String, days: i32) -> Result<Vec<db::BsrSnapshot>, String> {
    db::get_bsr_history(&marketplace, &category_id, days).map_err(|e| e.to_string())
}

// 执行记录管理

#[tauri::command]
fn create_research_run(task_id: i64) -> Result<i64, String> {
    db::create_research_run(task_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_research_run(
    run_id: i64,
    status: String,
    summary: Option<String>,
    content: Option<String>,
    snapshot_id: Option<i64>,
) -> Result<(), String> {
    db::update_research_run(
        run_id,
        &status,
        summary.as_deref(),
        content.as_deref(),
        snapshot_id,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn fail_research_run(run_id: i64, error_message: String) -> Result<(), String> {
    db::fail_research_run(run_id, &error_message).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_latest_research_runs(limit: i32) -> Result<Vec<db::MarketResearchRun>, String> {
    db::get_latest_research_runs(limit).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_research_runs_by_task(task_id: i64, limit: i32) -> Result<Vec<db::MarketResearchRun>, String> {
    db::get_research_runs_by_task(task_id, limit).map_err(|e| e.to_string())
}

// ==================== 竞品情报 ====================

#[tauri::command]
fn get_competitor_tasks() -> Result<Vec<db::CompetitorTask>, String> {
    db::get_competitor_tasks().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_competitor_task(id: i64) -> Result<Option<db::CompetitorTask>, String> {
    db::get_competitor_task(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_competitor_task(
    name: String,
    marketplace: String,
    my_asin: Option<String>,
    ai_provider: String,
    ai_model: Option<String>,
    schedule_type: String,
    schedule_days: Option<String>,
    schedule_time: String,
) -> Result<i64, String> {
    db::create_competitor_task(
        &name,
        &marketplace,
        my_asin.as_deref(),
        &ai_provider,
        ai_model.as_deref(),
        &schedule_type,
        schedule_days.as_deref(),
        &schedule_time,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn update_competitor_task(
    id: i64,
    name: String,
    marketplace: String,
    my_asin: Option<String>,
    ai_provider: String,
    ai_model: Option<String>,
    schedule_type: String,
    schedule_days: Option<String>,
    schedule_time: String,
    is_enabled: bool,
) -> Result<(), String> {
    db::update_competitor_task(
        id,
        &name,
        &marketplace,
        my_asin.as_deref(),
        &ai_provider,
        ai_model.as_deref(),
        &schedule_type,
        schedule_days.as_deref(),
        &schedule_time,
        is_enabled,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_competitor_task(id: i64) -> Result<(), String> {
    db::delete_competitor_task(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_competitor_asins(task_id: i64) -> Result<Vec<db::CompetitorAsin>, String> {
    db::get_competitor_asins(task_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_competitor_asin(
    task_id: i64,
    asin: String,
    title: Option<String>,
    tags: Option<String>,
) -> Result<i64, String> {
    db::add_competitor_asin(task_id, &asin, title.as_deref(), tags.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_competitor_asin(task_id: i64, asin: String) -> Result<(), String> {
    db::remove_competitor_asin(task_id, &asin).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_competitor_snapshot(
    asin_id: i64,
    _asin: String,
    price: Option<String>,
    rating: Option<String>,
    review_count: Option<i64>,
    bsr_rank: Option<String>,
    _title: Option<String>,
    _image_url: Option<String>,
) -> Result<i64, String> {
    // Parse string values to numeric types
    let price_value: Option<f64> = price.and_then(|p| {
        // Extract number from price string like "$19.99" or "19.99"
        let cleaned: String = p.chars().filter(|c| c.is_numeric() || *c == '.').collect();
        cleaned.parse().ok()
    });

    let rating_value: Option<f64> = rating.and_then(|r| r.parse().ok());

    let bsr_value: Option<i64> = bsr_rank.and_then(|b| {
        // Extract number from BSR string like "#1,234 in Category"
        let cleaned: String = b.chars().filter(|c| c.is_numeric()).collect();
        cleaned.parse().ok()
    });

    db::save_competitor_snapshot(
        asin_id,
        price_value,
        bsr_value,
        rating_value,
        review_count,
        None, // availability - not provided by crawler
    ).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_competitor_snapshots(asin_id: i64, days: i32) -> Result<Vec<db::CompetitorSnapshot>, String> {
    db::get_competitor_snapshots(asin_id, days).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_competitor_run(task_id: i64) -> Result<i64, String> {
    db::create_competitor_run(task_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_competitor_run(
    run_id: i64,
    status: String,
    report_summary: Option<String>,
    report_content: Option<String>,
) -> Result<(), String> {
    db::update_competitor_run(
        run_id,
        &status,
        report_summary.as_deref(),
        report_content.as_deref(),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn fail_competitor_run(run_id: i64, error_message: String) -> Result<(), String> {
    db::fail_competitor_run(run_id, &error_message).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_latest_competitor_runs(limit: i32) -> Result<Vec<db::CompetitorRun>, String> {
    db::get_latest_competitor_runs(limit).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_competitor_runs_by_task(task_id: i64, limit: i32) -> Result<Vec<db::CompetitorRun>, String> {
    db::get_competitor_runs_by_task(task_id, limit).map_err(|e| e.to_string())
}

// 批量爬取竞品 Listing 信息（一次浏览器获取所有 ASIN）
#[tauri::command]
async fn fetch_competitor_listings_batch(
    task_id: i64,
    marketplace: String,
    app: tauri::AppHandle,
) -> Result<Vec<crawler::ListingResult>, String> {
    // 获取任务下的所有 ASIN
    let asins = db::get_competitor_asins(task_id).map_err(|e| e.to_string())?;

    if asins.is_empty() {
        return Ok(Vec::new());
    }

    eprintln!("[fetch_competitor_listings_batch] 开始批量爬取: task_id={}, {} 个 ASIN", task_id, asins.len());

    // 构建批量请求数据
    let items: Vec<(i64, String, String)> = asins.iter()
        .map(|a| (a.id, a.asin.clone(), marketplace.clone()))
        .collect();

    let app_clone = app.clone();
    let results = crawler::fetch_listings_batch(
        items,
        move |current, total, info| {
            eprintln!("[fetch_competitor_listings_batch] 进度: {}/{} - {}", current, total, info);
            let _ = app_clone.emit("competitor-batch-progress", serde_json::json!({
                "current": current,
                "total": total,
                "info": info
            }));
        }
    ).await;

    // 保存快照到数据库
    for (asin_id, result) in &results {
        if result.error.is_none() {
            // 解析价格
            let price_value: Option<f64> = result.price.as_ref().and_then(|p| {
                let cleaned: String = p.chars().filter(|c| c.is_numeric() || *c == '.').collect();
                cleaned.parse().ok()
            });
            // 解析评分
            let rating_value: Option<f64> = result.rating.as_ref().and_then(|r| r.parse().ok());
            // 解析 BSR
            let bsr_value: Option<i64> = result.bsr_rank.as_ref().and_then(|b| {
                let cleaned: String = b.chars().filter(|c| c.is_numeric()).collect();
                cleaned.parse().ok()
            });

            if let Err(e) = db::save_competitor_snapshot(
                *asin_id,
                price_value,
                bsr_value,
                rating_value,
                result.review_count,
                None,
            ) {
                eprintln!("[fetch_competitor_listings_batch] 保存快照失败 asin_id={}: {:?}", asin_id, e);
            }
        }
    }

    // 返回结果（只返回 ListingResult）
    let listing_results: Vec<crawler::ListingResult> = results.into_iter()
        .map(|(_, result)| result)
        .collect();

    eprintln!("[fetch_competitor_listings_batch] 批量爬取完成: {} 个结果", listing_results.len());
    Ok(listing_results)
}

// ==================== AI 分析 ====================

// 保存分析结果
#[tauri::command]
fn sc_save_analysis(
    project_id: i64,
    analysis_type: String,
    result_json: String,
    model_provider: Option<String>,
    model_name: Option<String>,
) -> Result<i64, String> {
    db::sc_save_analysis(
        project_id,
        &analysis_type,
        &result_json,
        model_provider.as_deref(),
        model_name.as_deref(),
    )
    .map_err(|e| e.to_string())
}

// 获取指定类型的分析结果
#[tauri::command]
fn sc_get_analysis(project_id: i64, analysis_type: String) -> Result<Option<db::ScAnalysis>, String> {
    db::sc_get_analysis(project_id, &analysis_type)
        .map_err(|e| e.to_string())
}

// 获取项目的所有分析结果
#[tauri::command]
fn sc_get_all_analysis(project_id: i64) -> Result<Vec<db::ScAnalysis>, String> {
    db::sc_get_all_analysis(project_id)
        .map_err(|e| e.to_string())
}

// 删除项目的所有分析结果
#[tauri::command]
fn sc_delete_all_analysis(project_id: i64) -> Result<(), String> {
    db::sc_delete_all_analysis(project_id)
        .map_err(|e| e.to_string())
}

// 获取项目关联的关键词数据
#[tauri::command]
fn sc_get_project_keywords(project_id: i64, limit: i64) -> Result<Vec<db::KeywordData>, String> {
    db::sc_get_project_keywords(project_id, limit)
        .map_err(|e| e.to_string())
}

// ==================== 智能广告（Smart Ads）====================

#[tauri::command]
fn ad_create_project(product_id: Option<i64>, name: String, marketplace: String, target_acos: f64) -> Result<i64, String> {
    db::ad_create_project(product_id, &name, &marketplace, target_acos)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn ad_get_projects() -> Result<Vec<db::AdProject>, String> {
    db::ad_get_projects()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn ad_get_project(id: i64) -> Result<Option<db::AdProject>, String> {
    db::ad_get_project(id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn ad_update_project(id: i64, name: String, marketplace: String, target_acos: f64) -> Result<(), String> {
    db::ad_update_project(id, &name, &marketplace, target_acos)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn ad_delete_project(id: i64) -> Result<(), String> {
    db::ad_delete_project(id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn ad_import_search_terms(project_id: i64, search_terms: Vec<db::AdSearchTerm>, mode: String) -> Result<i64, String> {
    db::ad_import_search_terms(project_id, search_terms, &mode)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn ad_get_search_terms(project_id: i64) -> Result<Vec<db::AdSearchTerm>, String> {
    db::ad_get_search_terms(project_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn ad_get_search_terms_stats(project_id: i64) -> Result<db::SearchTermsStatsResult, String> {
    db::ad_get_search_terms_stats(project_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn ad_save_analysis(project_id: i64, analysis_type: String, result_json: String, ai_provider: String, ai_model: String) -> Result<i64, String> {
    db::ad_save_analysis(project_id, &analysis_type, &result_json, &ai_provider, &ai_model)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn ad_get_analysis(project_id: i64, analysis_type: String) -> Result<Option<db::AdAnalysisResult>, String> {
    db::ad_get_analysis(project_id, &analysis_type)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn ad_get_all_analysis(project_id: i64) -> Result<Vec<db::AdAnalysisResult>, String> {
    db::ad_get_all_analysis(project_id)
        .map_err(|e| e.to_string())
}

// ==================== 快捷备忘录 ====================

#[tauri::command]
fn add_quick_note(content: String) -> Result<i64, String> {
    db::add_quick_note(content).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_quick_notes(filter: Option<String>) -> Result<Vec<QuickNote>, String> {
    db::get_quick_notes(filter).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_quick_note(id: i64, content: String) -> Result<(), String> {
    db::update_quick_note(id, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_quick_note(id: i64) -> Result<bool, String> {
    db::toggle_quick_note(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_quick_note(id: i64) -> Result<(), String> {
    db::delete_quick_note(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_quick_notes_count() -> Result<(i64, i64), String> {
    db::get_quick_notes_count().map_err(|e| e.to_string())
}

#[tauri::command]
fn update_quick_note_due_date(id: i64, due_date: Option<String>) -> Result<(), String> {
    db::update_quick_note_due_date(id, due_date).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_quick_note_repeat(id: i64, repeat_type: Option<String>, repeat_interval: i64) -> Result<(), String> {
    db::update_quick_note_repeat(id, repeat_type, repeat_interval).map_err(|e| e.to_string())
}

#[tauri::command]
fn reorder_quick_notes(ids: Vec<i64>) -> Result<(), String> {
    db::reorder_quick_notes(ids).map_err(|e| e.to_string())
}

// ==================== 汇率 ====================

#[tauri::command]
fn save_exchange_rates(rates: Vec<(String, f64)>) -> Result<(), String> {
    db::save_exchange_rates(rates).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_exchange_rates() -> Result<Vec<ExchangeRateCache>, String> {
    db::get_exchange_rates().map_err(|e| e.to_string())
}

// 从网络获取最新汇率的响应结构
#[derive(serde::Deserialize)]
struct ExchangeRateApiResponse {
    rates: std::collections::HashMap<String, f64>,
}

#[tauri::command]
async fn fetch_exchange_rates(currencies: Vec<String>) -> Result<Vec<ExchangeRateCache>, String> {
    // 使用 exchangerate-api.com 的免费 API
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.exchangerate-api.com/v4/latest/CNY")
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API 返回错误: {}", response.status()));
    }

    let data: ExchangeRateApiResponse = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    // 转换汇率：API 返回的是 1 CNY = X 外币，我们需要 1 外币 = X CNY
    let mut rates_to_save: Vec<(String, f64)> = Vec::new();
    for currency in &currencies {
        if let Some(&rate) = data.rates.get(currency) {
            if rate > 0.0 {
                let cny_rate = 1.0 / rate;
                rates_to_save.push((currency.clone(), cny_rate));
            }
        }
    }

    // 保存到数据库缓存
    if !rates_to_save.is_empty() {
        db::save_exchange_rates(rates_to_save.clone()).map_err(|e| e.to_string())?;
        // 同时保存每日历史记录
        let _ = db::save_exchange_rate_history(rates_to_save);
    }

    // 返回最新的缓存数据
    db::get_exchange_rates().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_exchange_rate_history(currency: String, days: Option<i32>) -> Result<Vec<ExchangeRateHistory>, String> {
    let days = days.unwrap_or(30);
    db::get_exchange_rate_history(&currency, days).map_err(|e| e.to_string())
}

// ==================== 工作周报 ====================

#[tauri::command]
fn create_weekly_report(week_start: String, week_end: String, title: String) -> Result<i64, String> {
    db::create_weekly_report(&week_start, &week_end, &title).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_weekly_report(week_start: String) -> Result<Option<WeeklyReport>, String> {
    db::get_weekly_report(&week_start).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_weekly_report(
    id: i64,
    title: String,
    summary: Option<String>,
    next_week_plan: Option<String>,
    status: String,
) -> Result<(), String> {
    db::update_weekly_report(id, &title, summary.as_deref(), next_week_plan.as_deref(), &status)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_weekly_reports(limit: Option<i64>, search: Option<String>) -> Result<Vec<WeeklyReport>, String> {
    db::list_weekly_reports(limit, search.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_report_entry(
    week_start: String,
    category: String,
    content: String,
    description: Option<String>,
    task_category: Option<String>,
    priority_level: Option<String>,
    progress: Option<i64>,
    source: Option<String>,
    source_id: Option<i64>,
) -> Result<i64, String> {
    db::add_report_entry(
        &week_start,
        &category,
        &content,
        description.as_deref(),
        task_category.as_deref(),
        &priority_level.unwrap_or_else(|| "medium".to_string()),
        progress.unwrap_or(100),
        &source.unwrap_or_else(|| "manual".to_string()),
        source_id,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn update_report_entry(
    id: i64,
    content: String,
    description: Option<String>,
    task_category: Option<String>,
    category: String,
    priority_level: Option<String>,
    priority: i64,
    progress: i64,
) -> Result<(), String> {
    db::update_report_entry(
        id,
        &content,
        description.as_deref(),
        task_category.as_deref(),
        &category,
        &priority_level.unwrap_or_else(|| "medium".to_string()),
        priority,
        progress,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_report_entry(id: i64) -> Result<(), String> {
    db::delete_report_entry(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_report_entries(week_start: String) -> Result<Vec<WeeklyReportEntry>, String> {
    db::get_report_entries(&week_start).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_weekly_report_data(week_start: String, week_end: String) -> Result<WeeklyReportContent, String> {
    db::get_weekly_report_data(&week_start, &week_end).map_err(|e| e.to_string())
}

#[tauri::command]
fn import_report_entries(
    week_start: String,
    category: String,
    entries: Vec<(String, String, Option<i64>)>,
) -> Result<i64, String> {
    db::import_report_entries(&week_start, &category, entries).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_weekly_report(week_start: String) -> Result<(), String> {
    db::delete_weekly_report(&week_start).map_err(|e| e.to_string())
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
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
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

                // 自动启动市场调研调度器
                MARKET_RESEARCH_SCHEDULER.start(app_handle);
                println!("[MarketResearchScheduler] Auto-started on app launch");
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
            set_root_negative,
            batch_set_roots_negative,
            batch_set_roots_negative_by_words,
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
            // API Key 存储
            set_api_key,
            get_api_key,
            delete_api_key,
            has_api_key,
            migrate_api_keys,
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
            clear_task_logs,
            // 依赖安装
            check_dependencies,
            install_all_dependencies,
            install_playwright_only,
            install_pdf_dependencies,
            // 优化事件管理
            add_optimization_event,
            get_optimization_events,
            update_optimization_event,
            delete_optimization_event,
            // 截图管理
            get_screenshots_dir,
            save_event_screenshot,
            delete_event_screenshot,
            // 知识库管理
            kb_create_category,
            kb_get_categories,
            kb_delete_category,
            kb_update_category,
            kb_update_category_color,
            kb_update_categories_order,
            kb_add_document,
            kb_update_document_status,
            kb_update_document_category,
            kb_get_documents,
            kb_delete_document,
            kb_add_chunk,
            kb_add_chunks_batch,
            kb_get_chunks,
            kb_search,
            kb_create_conversation,
            kb_get_conversations,
            kb_update_conversation_title,
            kb_delete_conversation,
            kb_add_message,
            kb_get_messages,
            // 文档链接
            kb_add_document_link,
            kb_remove_document_link,
            kb_get_document_links,
            kb_get_document_backlinks,
            kb_get_all_links,
            // 文档分类关联（多对多）
            kb_add_document_category,
            kb_remove_document_category,
            kb_get_document_categories,
            kb_get_documents_by_categories,
            kb_set_document_categories,
            kb_process_document,
            kb_extract_images,
            kb_read_file_base64,
            kb_pdf_to_images,
            kb_add_image_chunk,
            kb_add_image_chunk_with_file,
            kb_update_chunk_embedding,
            kb_clear_all_embeddings,
            kb_get_chunks_without_embedding,
            kb_get_document_embedding_stats,
            kb_vector_search,
            // 智能文案
            sc_create_project,
            sc_get_projects,
            sc_get_project,
            sc_update_project,
            sc_update_project_status,
            sc_update_my_product_info,
            sc_update_my_listing,
            sc_delete_project,
            // 竞品管理
            sc_add_competitor,
            sc_get_competitors,
            sc_update_competitor_info,
            sc_delete_competitor,
            sc_update_competitor_type,
            sc_fetch_competitor_listing,
            sc_fetch_competitors_batch,
            // 评论管理
            sc_fetch_competitor_reviews,
            sc_get_competitor_reviews,
            sc_get_reviews_summary,
            // BSR 爬虫
            fetch_category_bsr,
            discover_subcategories,
            fetch_listing_info,
            // 市场调研监控任务
            get_market_research_tasks,
            get_market_research_task,
            create_market_research_task,
            update_market_research_task,
            delete_market_research_task,
            get_pending_research_tasks,
            save_bsr_snapshot,
            get_bsr_history,
            create_research_run,
            update_research_run,
            fail_research_run,
            get_latest_research_runs,
            get_research_runs_by_task,
            // 竞品情报
            get_competitor_tasks,
            get_competitor_task,
            create_competitor_task,
            update_competitor_task,
            delete_competitor_task,
            get_competitor_asins,
            add_competitor_asin,
            remove_competitor_asin,
            save_competitor_snapshot,
            get_competitor_snapshots,
            create_competitor_run,
            update_competitor_run,
            fail_competitor_run,
            get_latest_competitor_runs,
            get_competitor_runs_by_task,
            fetch_competitor_listings_batch,
            // AI 分析
            sc_save_analysis,
            sc_get_analysis,
            sc_get_all_analysis,
            sc_delete_all_analysis,
            sc_get_project_keywords,
            // 智能广告
            ad_create_project,
            ad_get_projects,
            ad_get_project,
            ad_update_project,
            ad_delete_project,
            ad_import_search_terms,
            ad_get_search_terms,
            ad_get_search_terms_stats,
            ad_save_analysis,
            ad_get_analysis,
            ad_get_all_analysis,
            // 快捷备忘录
            add_quick_note,
            get_quick_notes,
            update_quick_note,
            toggle_quick_note,
            delete_quick_note,
            get_quick_notes_count,
            update_quick_note_due_date,
            update_quick_note_repeat,
            reorder_quick_notes,
            // 汇率
            save_exchange_rates,
            get_exchange_rates,
            fetch_exchange_rates,
            get_exchange_rate_history,
            // 工作周报
            create_weekly_report,
            get_weekly_report,
            update_weekly_report,
            list_weekly_reports,
            add_report_entry,
            update_report_entry,
            delete_report_entry,
            get_report_entries,
            get_weekly_report_data,
            import_report_entries,
            delete_weekly_report,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{Utc, Timelike, FixedOffset, Datelike};
use tauri::Emitter;

use crate::crawler;
use crate::db;
use crate::ai;

// 调度器状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SchedulerStatus {
    pub is_running: bool,
    pub last_check_time: Option<String>,
    pub next_check_time: Option<String>,
    pub current_task: Option<String>,
}

// 调度器设置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SchedulerSettings {
    pub enabled: bool,
    pub morning_start: u32,     // 8
    pub morning_end: u32,       // 10
    pub evening_start: u32,     // 18
    pub evening_end: u32,       // 21
    pub rank_change_threshold: i64,  // 排名变化阈值，默认10
    pub notify_on_enter_top10: bool,
    pub notify_on_exit_top10: bool,
    pub notify_on_new_rank: bool,
    pub notify_on_lost_rank: bool,
    pub max_pages: u32,         // 监控页数: 1/3/5
}

impl Default for SchedulerSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            morning_start: 8,
            morning_end: 10,
            evening_start: 18,
            evening_end: 21,
            rank_change_threshold: 10,
            notify_on_enter_top10: true,
            notify_on_exit_top10: true,
            notify_on_new_rank: true,
            notify_on_lost_rank: true,
            max_pages: 5,  // 默认监控前5页
        }
    }
}

// 检查当前时间是否在检测窗口内（使用北京时间 UTC+8）
pub fn is_in_check_window(_country: &str, settings: &SchedulerSettings) -> bool {
    // 北京时间 UTC+8
    let beijing_offset = FixedOffset::east_opt(8 * 3600).unwrap();
    let beijing_time = Utc::now().with_timezone(&beijing_offset);
    let hour = beijing_time.hour();

    // 检查是否在早间窗口
    if hour >= settings.morning_start && hour < settings.morning_end {
        return true;
    }

    // 检查是否在晚间窗口
    if hour >= settings.evening_start && hour < settings.evening_end {
        return true;
    }

    false
}

// 排名变化结果
#[derive(Debug, Clone)]
pub struct RankChange {
    pub keyword: String,
    pub country: String,
    pub old_rank: Option<i64>,
    pub new_rank: Option<i64>,
    pub change: i64,
    pub change_type: RankChangeType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RankChangeType {
    Improved,      // 排名上升
    Declined,      // 排名下降
    EnteredTop10,  // 进入前10
    ExitedTop10,   // 跌出前10
    NewRank,       // 新上榜
    LostRank,      // 跌出榜单
}

// 计算排名变化
pub fn calculate_rank_change(
    old_rank: Option<i64>,
    new_rank: Option<i64>,
) -> Option<(i64, RankChangeType)> {
    match (old_rank, new_rank) {
        (None, Some(new)) => {
            // 新上榜
            Some((new, RankChangeType::NewRank))
        }
        (Some(_old), None) => {
            // 跌出榜单
            Some((0, RankChangeType::LostRank))
        }
        (Some(old), Some(new)) => {
            let change = old - new; // 正数表示排名上升

            // 判断变化类型
            let change_type = if old > 10 && new <= 10 {
                RankChangeType::EnteredTop10
            } else if old <= 10 && new > 10 {
                RankChangeType::ExitedTop10
            } else if change > 0 {
                RankChangeType::Improved
            } else {
                RankChangeType::Declined
            };

            Some((change, change_type))
        }
        (None, None) => None,
    }
}

// 检查是否需要发送通知
pub fn should_notify(
    change: i64,
    change_type: &RankChangeType,
    settings: &SchedulerSettings,
) -> bool {
    match change_type {
        RankChangeType::EnteredTop10 => settings.notify_on_enter_top10,
        RankChangeType::ExitedTop10 => settings.notify_on_exit_top10,
        RankChangeType::NewRank => settings.notify_on_new_rank,
        RankChangeType::LostRank => settings.notify_on_lost_rank,
        RankChangeType::Improved | RankChangeType::Declined => {
            change.abs() >= settings.rank_change_threshold
        }
    }
}

// 调度器
pub struct Scheduler {
    running: Arc<AtomicBool>,
    settings: Arc<Mutex<SchedulerSettings>>,
    last_check: Arc<Mutex<Option<chrono::DateTime<Utc>>>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            settings: Arc::new(Mutex::new(SchedulerSettings::default())),
            last_check: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn update_settings(&self, settings: SchedulerSettings) {
        let mut current = self.settings.lock().await;
        *current = settings;
    }

    pub async fn get_settings(&self) -> SchedulerSettings {
        self.settings.lock().await.clone()
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub async fn start(&self) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }

        self.running.store(true, Ordering::SeqCst);

        let running = self.running.clone();
        let settings = self.settings.clone();
        let last_check = self.last_check.clone();

        tokio::spawn(async move {
            while running.load(Ordering::SeqCst) {
                let current_settings = settings.lock().await.clone();

                if current_settings.enabled {
                    // 检查是否需要执行检测（每小时最多一次）
                    let should_check = {
                        let last = last_check.lock().await;
                        match *last {
                            None => true,
                            Some(last_time) => {
                                let elapsed = Utc::now() - last_time;
                                elapsed.num_hours() >= 1
                            }
                        }
                    };

                    if should_check {
                        // 收集所有需要检测的关键词
                        let mut all_pending = Vec::new();

                        if let Ok(products) = db::get_products() {
                            for product in products {
                                let country = product.country.as_deref().unwrap_or("US");
                                if is_in_check_window(country, &current_settings) {
                                    if let Ok(pending) = db::get_pending_monitoring_checks(product.id, 4) {
                                        all_pending.extend(pending);
                                    }
                                }
                            }
                        }

                        if !all_pending.is_empty() {
                            let total = all_pending.len() as i64;

                            // 创建任务记录
                            let task_id = db::create_task_log("auto", total).ok();
                            println!("[Scheduler] 开始定时检测，共 {} 个关键词，任务ID: {:?}", total, task_id);

                            // 准备批量检测数据: (monitoring_id, keyword, asin, country)
                            let keywords: Vec<(i64, String, String, String)> = all_pending
                                .iter()
                                .map(|m| (m.id, m.keyword.clone(), m.asin.clone(), m.country.clone()))
                                .collect();

                            // 获取并发浏览器数量设置
                            let max_browsers = db::get_setting("max_browsers")
                                .ok()
                                .flatten()
                                .and_then(|s| s.parse::<i64>().ok())
                                .unwrap_or(3);  // 默认3个并发浏览器

                            // 使用批量模式检测（并发模式，同一站点复用浏览器）
                            let task_id_clone = task_id;
                            let results = crawler::check_rankings_batch(
                                keywords,
                                5, // max_pages
                                max_browsers,
                                move |completed, _total, msg| {
                                    println!("[Scheduler] {}", msg);
                                    // 更新任务进度（这里无法准确统计成功/失败，在结果处理时再统计）
                                    if let Some(tid) = task_id_clone {
                                        let _ = db::update_task_progress(tid, completed, 0);
                                    }
                                },
                            ).await;

                            // 处理结果
                            let mut success_count = 0i64;
                            let mut failed_count = 0i64;

                            for (monitoring_id, result) in results {
                                if result.error.is_none() {
                                    // 更新数据库
                                    let product_info = result.product_info.as_ref();
                                    let _ = db::update_ranking_result(
                                        monitoring_id,
                                        result.organic_rank,
                                        result.organic_page,
                                        result.sponsored_rank,
                                        result.sponsored_page,
                                        product_info.and_then(|p| p.image_url.clone()),
                                        product_info.and_then(|p| p.price.clone()),
                                        product_info.and_then(|p| p.reviews_count),
                                        product_info.and_then(|p| p.rating),
                                    );
                                    success_count += 1;
                                } else {
                                    println!("[Scheduler] 检测失败 (id={}): {:?}", monitoring_id, result.error);
                                    failed_count += 1;
                                }
                            }

                            // 完成任务记录
                            if let Some(tid) = task_id {
                                let _ = db::complete_task_log(tid, success_count, failed_count);
                                println!("[Scheduler] 定时检测完成，成功: {}, 失败: {}", success_count, failed_count);
                            }
                        }

                        // 更新最后检测时间
                        let mut last = last_check.lock().await;
                        *last = Some(Utc::now());
                    }
                }

                // 每分钟检查一次
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub async fn get_status(&self) -> SchedulerStatus {
        let settings = self.settings.lock().await;
        let last = self.last_check.lock().await;

        SchedulerStatus {
            is_running: self.is_running() && settings.enabled,
            last_check_time: last.map(|t| t.to_rfc3339()),
            next_check_time: None, // TODO: 计算下次检测时间
            current_task: None,
        }
    }
}

// 全局调度器实例
use once_cell::sync::Lazy;
pub static SCHEDULER: Lazy<Scheduler> = Lazy::new(|| Scheduler::new());

// ==================== 市场调研调度器 ====================

// 市场调研任务完成通知
#[derive(Debug, Clone, serde::Serialize)]
pub struct MarketResearchComplete {
    pub task_id: i64,
    pub task_name: String,
    pub run_id: i64,
    pub success: bool,
    pub summary: Option<String>,
    pub error: Option<String>,
}

// 市场调研调度器
pub struct MarketResearchScheduler {
    running: Arc<AtomicBool>,
}

impl MarketResearchScheduler {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn start(&self, app_handle: tauri::AppHandle) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }

        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        tokio::spawn(async move {
            while running.load(Ordering::SeqCst) {
                // 检查是否有需要执行的任务
                if let Ok(tasks) = db::get_pending_research_tasks() {
                    let beijing_offset = FixedOffset::east_opt(8 * 3600).unwrap();
                    let beijing_time = Utc::now().with_timezone(&beijing_offset);
                    let current_hour = beijing_time.hour();
                    let current_minute = beijing_time.minute();
                    let current_weekday = beijing_time.weekday().num_days_from_monday(); // 0=Monday

                    for task in tasks {
                        // 解析任务的运行时间
                        let parts: Vec<&str> = task.schedule_time.split(':').collect();
                        if parts.len() != 2 {
                            continue;
                        }
                        let target_hour: u32 = parts[0].parse().unwrap_or(0);
                        let target_minute: u32 = parts[1].parse().unwrap_or(0);

                        // 检查时间是否匹配（精确到分钟）
                        if current_hour != target_hour || current_minute != target_minute {
                            continue;
                        }

                        // 检查日期是否匹配
                        let should_run = if task.schedule_type == "daily" {
                            true
                        } else if task.schedule_type == "weekly" {
                            if let Some(days_json) = &task.schedule_days {
                                if let Ok(days) = serde_json::from_str::<Vec<u32>>(days_json) {
                                    // 转换：我们存的是0=周日,1=周一...
                                    // chrono 的 num_days_from_monday 返回 0=Monday
                                    let weekday_for_compare = if current_weekday == 6 { 0 } else { current_weekday + 1 };
                                    days.contains(&weekday_for_compare)
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if !should_run {
                            continue;
                        }

                        // 检查是否今天已经运行过
                        let today = beijing_time.format("%Y-%m-%d").to_string();
                        if let Some(last_run) = &task.last_run_at {
                            if last_run.starts_with(&today) {
                                continue; // 今天已运行
                            }
                        }

                        println!("[MarketResearchScheduler] 执行任务: {} (ID: {})", task.name, task.id);

                        // 创建执行记录
                        let run_id = match db::create_research_run(task.id) {
                            Ok(id) => id,
                            Err(e) => {
                                eprintln!("[MarketResearchScheduler] 创建执行记录失败: {}", e);
                                continue;
                            }
                        };

                        // 执行 BSR 爬取
                        let bsr_result = crawler::fetch_category_bsr(
                            task.marketplace.clone(),
                            task.category_id.clone(),
                        ).await;

                        if let Some(error) = &bsr_result.error {
                            // 爬取失败
                            let _ = db::fail_research_run(run_id, error);
                            let _ = db::update_task_last_run(task.id, "failed");

                            // 发送失败通知
                            let _ = app_handle.emit("market_research_complete", MarketResearchComplete {
                                task_id: task.id,
                                task_name: task.name.clone(),
                                run_id,
                                success: false,
                                summary: None,
                                error: Some(error.clone()),
                            });
                            continue;
                        }

                        // 保存 BSR 快照
                        let products_json = serde_json::to_string(&bsr_result.products).unwrap_or_default();
                        let product_count = bsr_result.products.len() as i64;
                        let snapshot_id = db::save_bsr_snapshot(
                            &task.marketplace,
                            &task.category_id,
                            task.category_name.as_deref(),
                            &products_json,
                            product_count,
                        ).ok();

                        // 生成简要报告
                        // 解析价格字符串（例如 "$19.99" -> 19.99）
                        fn parse_price(price_str: &str) -> Option<f64> {
                            price_str.trim_start_matches(|c: char| !c.is_ascii_digit())
                                .parse::<f64>()
                                .ok()
                        }

                        let prices: Vec<f64> = bsr_result.products.iter()
                            .filter_map(|p| p.price.as_ref().and_then(|s| parse_price(s)))
                            .collect();

                        let price_info = if prices.is_empty() {
                            "价格信息暂无".to_string()
                        } else {
                            let min_price = prices.iter().cloned().fold(f64::MAX, f64::min);
                            let max_price = prices.iter().cloned().fold(0.0_f64, f64::max);
                            format!("价格范围: ${:.2} - ${:.2}", min_price, max_price)
                        };

                        let summary = format!(
                            "类目: {} ({})\n获取 {} 个产品\n{}",
                            task.category_name.as_deref().unwrap_or(&task.category_id),
                            task.marketplace,
                            product_count,
                            price_info,
                        );

                        // 调用 AI 生成完整报告
                        let category_name = task.category_name.as_deref().unwrap_or(&task.category_id);
                        let report_content = match ai::generate_market_research_report(
                            &task.marketplace,
                            category_name,
                            &products_json,
                        ).await {
                            Ok(content) => {
                                println!("[MarketResearchScheduler] AI 报告生成成功，长度: {} 字符", content.len());
                                Some(content)
                            }
                            Err(e) => {
                                eprintln!("[MarketResearchScheduler] AI 报告生成失败: {}", e);
                                None
                            }
                        };

                        // 更新执行记录
                        let _ = db::update_research_run(
                            run_id,
                            "completed",
                            Some(&summary),
                            report_content.as_deref(),
                            snapshot_id,
                        );
                        let _ = db::update_task_last_run(task.id, "completed");

                        // 发送成功通知
                        let _ = app_handle.emit("market_research_complete", MarketResearchComplete {
                            task_id: task.id,
                            task_name: task.name.clone(),
                            run_id,
                            success: true,
                            summary: Some(summary),
                            error: None,
                        });

                        println!("[MarketResearchScheduler] 任务完成: {} (ID: {})", task.name, task.id);
                    }
                }

                // 每分钟检查一次
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

pub static MARKET_RESEARCH_SCHEDULER: Lazy<MarketResearchScheduler> = Lazy::new(|| MarketResearchScheduler::new());

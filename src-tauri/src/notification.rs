use tauri_plugin_notification::NotificationExt;
use crate::scheduler::{RankChange, RankChangeType};

// 国旗 emoji
fn get_country_emoji(country: &str) -> &'static str {
    match country {
        "US" => "🇺🇸",
        "UK" => "🇬🇧",
        "DE" => "🇩🇪",
        "FR" => "🇫🇷",
        "IT" => "🇮🇹",
        "ES" => "🇪🇸",
        _ => "🌐",
    }
}

// 发送排名变化通知
pub fn send_rank_change_notification<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    change: &RankChange,
) -> Result<(), String> {
    let (title, icon) = match change.change_type {
        RankChangeType::Improved | RankChangeType::EnteredTop10 => ("📈 排名上升提醒", "up"),
        RankChangeType::Declined | RankChangeType::ExitedTop10 => ("📉 排名下降提醒", "down"),
        RankChangeType::NewRank => ("🎉 新上榜提醒", "new"),
        RankChangeType::LostRank => ("⚠️ 跌出榜单提醒", "lost"),
    };

    let body = format_notification_body(change);

    app.notification()
        .builder()
        .title(title)
        .body(&body)
        .show()
        .map_err(|e| e.to_string())?;

    Ok(())
}

// 格式化通知内容
fn format_notification_body(change: &RankChange) -> String {
    let country_emoji = get_country_emoji(&change.country);

    match change.change_type {
        RankChangeType::Improved => {
            let old = change.old_rank.unwrap_or(0);
            let new = change.new_rank.unwrap_or(0);
            format!(
                "关键词: {}\n排名: {} → {} (↑{}位)\n站点: {} {}",
                change.keyword,
                old, new, change.change,
                country_emoji, change.country
            )
        }
        RankChangeType::Declined => {
            let old = change.old_rank.unwrap_or(0);
            let new = change.new_rank.unwrap_or(0);
            format!(
                "关键词: {}\n排名: {} → {} (↓{}位)\n站点: {} {}",
                change.keyword,
                old, new, change.change.abs(),
                country_emoji, change.country
            )
        }
        RankChangeType::EnteredTop10 => {
            let old = change.old_rank.unwrap_or(0);
            let new = change.new_rank.unwrap_or(0);
            format!(
                "关键词: {}\n🏆 进入Top10!\n排名: {} → {}\n站点: {} {}",
                change.keyword,
                old, new,
                country_emoji, change.country
            )
        }
        RankChangeType::ExitedTop10 => {
            let old = change.old_rank.unwrap_or(0);
            let new = change.new_rank.unwrap_or(0);
            format!(
                "关键词: {}\n❌ 跌出Top10\n排名: {} → {}\n站点: {} {}",
                change.keyword,
                old, new,
                country_emoji, change.country
            )
        }
        RankChangeType::NewRank => {
            let new = change.new_rank.unwrap_or(0);
            format!(
                "关键词: {}\n🎊 首次上榜第{}名\n站点: {} {}",
                change.keyword,
                new,
                country_emoji, change.country
            )
        }
        RankChangeType::LostRank => {
            let old = change.old_rank.unwrap_or(0);
            format!(
                "关键词: {}\n原排名第{}名已跌出前5页\n站点: {} {}",
                change.keyword,
                old,
                country_emoji, change.country
            )
        }
    }
}

// 发送批量检测完成通知
pub fn send_batch_complete_notification<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    total: usize,
    changes: usize,
) -> Result<(), String> {
    let body = if changes > 0 {
        format!("已检测 {} 个关键词\n有 {} 个关键词排名发生明显变化", total, changes)
    } else {
        format!("已检测 {} 个关键词\n排名暂无明显变化", total)
    };

    app.notification()
        .builder()
        .title("✅ 排名检测完成")
        .body(&body)
        .show()
        .map_err(|e| e.to_string())?;

    Ok(())
}

// 发送检测开始通知
pub fn send_check_start_notification<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    count: usize,
) -> Result<(), String> {
    app.notification()
        .builder()
        .title("🔄 开始自动检测")
        .body(&format!("正在检测 {} 个关键词排名...", count))
        .show()
        .map_err(|e| e.to_string())?;

    Ok(())
}

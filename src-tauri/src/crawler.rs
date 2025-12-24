use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Duration;
use rand::Rng;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// 排名检测结果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RankingResult {
    pub keyword: String,
    pub target_asin: String,
    pub country: String,

    pub organic_rank: Option<i64>,
    pub organic_page: Option<i64>,
    pub sponsored_rank: Option<i64>,
    pub sponsored_page: Option<i64>,

    pub product_info: Option<ProductInfo>,

    pub organic_top_50: Vec<String>,
    pub sponsored_top_20: Vec<String>,

    pub checked_at: String,
    pub error: Option<String>,
}

// 产品详细信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProductInfo {
    pub asin: String,
    pub title: Option<String>,
    pub price: Option<String>,
    pub rating: Option<f64>,
    pub reviews_count: Option<i64>,
    pub image_url: Option<String>,
}

// 获取 Python 脚本路径
fn get_script_path() -> Result<std::path::PathBuf, String> {
    // 尝试多个可能的路径
    let possible_paths = vec![
        // 开发环境
        std::env::current_dir()
            .map(|p| p.join("scripts").join("amazon_crawler.py"))
            .ok(),
        // Tauri 资源目录
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("scripts").join("amazon_crawler.py")),
        // macOS .app 包内
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("Resources").join("scripts").join("amazon_crawler.py")),
        // 直接在 src-tauri 目录
        Some(std::path::PathBuf::from("src-tauri/scripts/amazon_crawler.py")),
    ];

    for path in possible_paths.into_iter().flatten() {
        if path.exists() {
            return Ok(path);
        }
    }

    Err("找不到 Python 爬虫脚本 amazon_crawler.py".to_string())
}

// 检查 Python 是否可用
fn check_python() -> Result<String, String> {
    // 尝试不同的 Python 命令
    for python_cmd in &["python3", "python"] {
        let mut cmd = Command::new(python_cmd);
        cmd.arg("--version");

        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let result = cmd.output();

        if let Ok(output) = result {
            if output.status.success() {
                return Ok(python_cmd.to_string());
            }
        }
    }

    Err("未找到 Python。请确保已安装 Python 3 并添加到 PATH".to_string())
}

// 检查依赖是否安装
fn check_dependencies(python_cmd: &str) -> Result<(), String> {
    let check_script = r#"
import sys
try:
    import cloudscraper
    import bs4
    print("ok")
except ImportError as e:
    print(f"missing:{e}")
    sys.exit(1)
"#;

    let mut cmd = Command::new(python_cmd);
    cmd.arg("-c").arg(check_script);

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd.output()
        .map_err(|e| format!("检查依赖失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.trim() == "ok" {
        Ok(())
    } else {
        Err(format!(
            "缺少 Python 依赖。请运行: {} -m pip install cloudscraper beautifulsoup4",
            python_cmd
        ))
    }
}

// 调用 Python 脚本进行搜索
fn call_python_crawler(
    keyword: &str,
    target_asin: &str,
    country: &str,
    max_pages: i64,
) -> Result<RankingResult, String> {
    // 检查 Python
    let python_cmd = check_python()?;

    // 检查依赖
    check_dependencies(&python_cmd)?;

    // 获取脚本路径
    let script_path = get_script_path()?;

    // 调用 Python 脚本
    let mut cmd = Command::new(&python_cmd);
    cmd.arg(&script_path)
        .arg(keyword)
        .arg(target_asin)
        .arg(country)
        .arg(max_pages.to_string());

    // Windows: 隐藏命令行窗口
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd.output()
        .map_err(|e| format!("执行 Python 脚本失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Python 脚本执行错误: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // 解析 JSON 结果
    serde_json::from_str(&stdout).map_err(|e| {
        format!(
            "解析 Python 输出失败: {}. 输出: {}",
            e,
            stdout.chars().take(500).collect::<String>()
        )
    })
}

// 异步包装器
pub async fn search_keyword(
    keyword: String,
    target_asin: String,
    country: String,
    max_pages: i64,
) -> RankingResult {
    tokio::task::spawn_blocking(move || {
        match call_python_crawler(&keyword, &target_asin, &country, max_pages) {
            Ok(result) => result,
            Err(e) => RankingResult {
                keyword,
                target_asin,
                country,
                organic_rank: None,
                organic_page: None,
                sponsored_rank: None,
                sponsored_page: None,
                product_info: None,
                organic_top_50: Vec::new(),
                sponsored_top_20: Vec::new(),
                checked_at: chrono::Utc::now().to_rfc3339(),
                error: Some(e),
            },
        }
    })
    .await
    .unwrap_or_else(|e| RankingResult {
        keyword: String::new(),
        target_asin: String::new(),
        country: String::new(),
        organic_rank: None,
        organic_page: None,
        sponsored_rank: None,
        sponsored_page: None,
        product_info: None,
        organic_top_50: Vec::new(),
        sponsored_top_20: Vec::new(),
        checked_at: chrono::Utc::now().to_rfc3339(),
        error: Some(format!("任务执行失败: {}", e)),
    })
}

// 批量检测接口
pub async fn check_rankings_batch(
    keywords: Vec<(i64, String, String, String)>, // (monitoring_id, keyword, asin, country)
    max_pages: i64,
    progress_callback: impl Fn(i64, i64, String),
) -> Vec<(i64, RankingResult)> {
    let total = keywords.len() as i64;
    let mut results = Vec::new();

    for (index, (monitoring_id, keyword, asin, country)) in keywords.into_iter().enumerate() {
        let current = (index + 1) as i64;
        progress_callback(current, total, format!("正在检测: {}", keyword));

        let result = search_keyword(keyword, asin, country, max_pages).await;
        results.push((monitoring_id, result));

        // 关键词间延迟
        if current < total {
            let delay_ms = {
                let mut rng = rand::thread_rng();
                rng.gen_range(5000..10000)
            };
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_python() {
        // 这个测试假设系统安装了 Python
        let result = check_python();
        println!("Python check result: {:?}", result);
    }
}

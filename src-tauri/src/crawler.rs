use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader, Write};

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
    pub warning: Option<String>,  // 警告信息（如地理限制）
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
    pub availability: Option<String>,  // 商品可用性信息
}

// 获取 Python 脚本路径 (Playwright 版本)
fn get_script_path() -> Result<std::path::PathBuf, String> {
    // 尝试多个可能的路径
    let possible_paths = vec![
        // 开发环境
        std::env::current_dir()
            .map(|p| p.join("scripts").join("amazon_crawler_playwright.py"))
            .ok(),
        // Tauri 资源目录
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("scripts").join("amazon_crawler_playwright.py")),
        // macOS .app 包内
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("Resources").join("scripts").join("amazon_crawler_playwright.py")),
        // 直接在 src-tauri 目录
        Some(std::path::PathBuf::from("src-tauri/scripts/amazon_crawler_playwright.py")),
    ];

    for path in possible_paths.into_iter().flatten() {
        if path.exists() {
            return Ok(path);
        }
    }

    Err("找不到 Python 爬虫脚本 amazon_crawler_playwright.py".to_string())
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

// 检查依赖是否安装 (Playwright 版本)
fn check_dependencies(python_cmd: &str) -> Result<(), String> {
    let check_script = r#"
import sys
try:
    from playwright.async_api import async_playwright
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
            "缺少 Playwright 依赖。请运行: {} -m pip install playwright && playwright install chromium",
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
        .arg(max_pages.to_string())
        .arg("none")   // proxy
        .arg("false"); // headless=false，有头模式（窗口隐藏到屏幕外）

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
                warning: None,
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
        warning: None,
    })
}

// 批量进度消息
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum BatchMessage {
    #[serde(rename = "progress")]
    Progress {
        monitoring_id: i64,
        result: RankingResult,
    },
    #[serde(rename = "complete")]
    Complete {
        total: i64,
    },
}

// 批量检测接口 - 优化版本，复用浏览器实例，支持并发
pub async fn check_rankings_batch(
    keywords: Vec<(i64, String, String, String)>, // (monitoring_id, keyword, asin, country)
    max_pages: i64,
    max_browsers: i64,  // 并发浏览器数量
    progress_callback: impl Fn(i64, i64, String) + Send + 'static,
) -> Vec<(i64, RankingResult)> {
    let total = keywords.len() as i64;

    if total == 0 {
        return Vec::new();
    }

    // 尝试使用批量模式（并发）
    match call_python_crawler_batch(keywords.clone(), max_pages, max_browsers, total, progress_callback).await {
        Ok(results) => results,
        Err(e) => {
            // 批量模式失败，返回错误结果
            eprintln!("[Batch] 批量检测失败: {}", e);
            keywords.into_iter().map(|(id, keyword, asin, country)| {
                (id, RankingResult {
                    keyword,
                    target_asin: asin,
                    country,
                    organic_rank: None,
                    organic_page: None,
                    sponsored_rank: None,
                    sponsored_page: None,
                    product_info: None,
                    organic_top_50: Vec::new(),
                    sponsored_top_20: Vec::new(),
                    checked_at: chrono::Utc::now().to_rfc3339(),
                    error: Some(e.clone()),
                    warning: None,
                })
            }).collect()
        }
    }
}

// 调用 Python 脚本批量处理（支持并发）
async fn call_python_crawler_batch(
    keywords: Vec<(i64, String, String, String)>,
    max_pages: i64,
    max_browsers: i64,  // 并发浏览器数量
    total: i64,
    progress_callback: impl Fn(i64, i64, String) + Send + 'static,
) -> Result<Vec<(i64, RankingResult)>, String> {
    // 检查 Python
    let python_cmd = check_python()?;

    // 检查依赖
    check_dependencies(&python_cmd)?;

    // 获取脚本路径
    let script_path = get_script_path()?;

    // 准备输入数据: [[id, keyword, asin, country], ...]
    let input_data: Vec<(i64, &str, &str, &str)> = keywords.iter()
        .map(|(id, kw, asin, country)| (*id, kw.as_str(), asin.as_str(), country.as_str()))
        .collect();
    let input_json = serde_json::to_string(&input_data)
        .map_err(|e| format!("序列化输入数据失败: {}", e))?;

    // 在阻塞任务中执行
    tokio::task::spawn_blocking(move || {
        // 调用 Python 脚本 --batch 模式（带并发参数）
        let mut cmd = Command::new(&python_cmd);
        cmd.arg(&script_path)
            .arg("--batch")
            .arg("false")  // headless=false，有头模式（窗口隐藏到屏幕外）
            .arg(max_pages.to_string())
            .arg(max_browsers.to_string())  // 并发浏览器数量
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Windows: 隐藏命令行窗口
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let mut child = cmd.spawn()
            .map_err(|e| format!("启动 Python 脚本失败: {}", e))?;

        // 写入 stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input_json.as_bytes())
                .map_err(|e| format!("写入 stdin 失败: {}", e))?;
        }

        // 读取 stdout
        let stdout = child.stdout.take()
            .ok_or_else(|| "无法获取 stdout".to_string())?;
        let reader = BufReader::new(stdout);

        let mut results: Vec<(i64, RankingResult)> = Vec::new();
        let mut completed_count = 0i64;

        for line in reader.lines() {
            let line = line.map_err(|e| format!("读取输出失败: {}", e))?;
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            // 尝试解析为 JSON
            match serde_json::from_str::<BatchMessage>(line) {
                Ok(BatchMessage::Progress { monitoring_id, result }) => {
                    completed_count += 1;
                    progress_callback(completed_count, total, format!("已完成: {}", result.keyword));
                    results.push((monitoring_id, result));
                }
                Ok(BatchMessage::Complete { total: _ }) => {
                    // 批量处理完成
                    break;
                }
                Err(_) => {
                    // 可能是调试输出，忽略
                    eprintln!("[Batch] 忽略非 JSON 输出: {}", line);
                }
            }
        }

        // 等待进程结束
        let status = child.wait()
            .map_err(|e| format!("等待进程结束失败: {}", e))?;

        if !status.success() {
            eprintln!("[Batch] Python 脚本非正常退出: {:?}", status);
        }

        Ok(results)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
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

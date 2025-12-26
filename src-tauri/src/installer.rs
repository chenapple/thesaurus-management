use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// 依赖状态
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DependencyStatus {
    pub python_installed: bool,
    pub python_version: Option<String>,
    pub python_path: Option<String>,
    pub playwright_installed: bool,
    pub chromium_installed: bool,
    pub error_message: Option<String>,
}

// 安装进度
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstallProgress {
    pub step: String,
    pub step_name: String,
    pub progress: f32,
    pub message: String,
    pub is_error: bool,
}

// 安装结果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstallResult {
    pub success: bool,
    pub message: String,
    pub python_path: Option<String>,
}

/// 获取 Python 版本
fn get_python_version(python_cmd: &str) -> Result<String, String> {
    let mut cmd = Command::new(python_cmd);
    cmd.arg("--version");

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd.output().map_err(|e| e.to_string())?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // 有些版本信息在 stderr
        if version.is_empty() {
            Ok(String::from_utf8_lossy(&output.stderr).trim().to_string())
        } else {
            Ok(version)
        }
    } else {
        Err("无法获取 Python 版本".to_string())
    }
}

/// 查找可用的 Python
fn find_python() -> Result<(String, String), String> {
    #[cfg(windows)]
    let candidates: Vec<String> = {
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
        vec![
            "python".to_string(),
            "python3".to_string(),
            format!("{}\\AppData\\Local\\Programs\\Python\\Python311\\python.exe", user_profile),
            format!("{}\\AppData\\Local\\Programs\\Python\\Python310\\python.exe", user_profile),
            format!("{}\\AppData\\Local\\Programs\\Python\\Python39\\python.exe", user_profile),
            format!("{}\\Programs\\Python\\Python311\\python.exe", local_app_data),
            format!("{}\\Programs\\Python\\Python310\\python.exe", local_app_data),
            "C:\\Python311\\python.exe".to_string(),
            "C:\\Python310\\python.exe".to_string(),
        ]
    };

    #[cfg(not(windows))]
    let candidates: Vec<String> = vec![
        "python3".to_string(),
        "python".to_string(),
        "/opt/homebrew/bin/python3".to_string(),
        "/usr/local/bin/python3".to_string(),
        "/usr/bin/python3".to_string(),
    ];

    for candidate in candidates {
        if let Ok(version) = get_python_version(&candidate) {
            // 确保是 Python 3
            if version.contains("Python 3") {
                return Ok((candidate, version));
            }
        }
    }

    Err("未找到 Python 3".to_string())
}

/// 检查 Playwright 是否安装
fn check_playwright_installed(python_path: &str) -> bool {
    let check_script = r#"
import sys
try:
    from playwright.async_api import async_playwright
    print("ok")
except ImportError:
    print("missing")
    sys.exit(1)
"#;

    let mut cmd = Command::new(python_path);
    cmd.arg("-c").arg(check_script);

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    match cmd.output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim() == "ok",
        Err(_) => false,
    }
}

/// 检查 Chromium 是否安装
fn check_chromium_installed(python_path: &str) -> bool {
    let check_script = r#"
import sys
import os
try:
    from playwright._impl._driver import compute_driver_executable
    from pathlib import Path
    driver_path = compute_driver_executable()
    browsers_path = Path(driver_path).parent / '.local-browsers'
    if browsers_path.exists():
        chromium_exists = any(p.name.startswith('chromium') for p in browsers_path.iterdir())
        print("ok" if chromium_exists else "missing")
    else:
        print("missing")
except Exception as e:
    print(f"error:{e}")
    sys.exit(1)
"#;

    let mut cmd = Command::new(python_path);
    cmd.arg("-c").arg(check_script);

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    match cmd.output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim() == "ok",
        Err(_) => false,
    }
}

/// 检查所有依赖状态
pub fn check_dependency_status() -> DependencyStatus {
    let mut status = DependencyStatus {
        python_installed: false,
        python_version: None,
        python_path: None,
        playwright_installed: false,
        chromium_installed: false,
        error_message: None,
    };

    match find_python() {
        Ok((path, version)) => {
            status.python_installed = true;
            status.python_path = Some(path.clone());
            status.python_version = Some(version);

            // 检查 Playwright
            status.playwright_installed = check_playwright_installed(&path);

            // 检查 Chromium (只有 Playwright 安装了才检查)
            if status.playwright_installed {
                status.chromium_installed = check_chromium_installed(&path);
            }
        }
        Err(e) => {
            status.error_message = Some(e);
        }
    }

    status
}

/// Windows: 安装 Python
#[cfg(windows)]
pub async fn install_python(app: tauri::AppHandle) -> Result<String, String> {
    use tauri::Emitter;

    let emit_progress = |progress: f32, message: &str, is_error: bool| {
        let _ = app.emit("install-progress", InstallProgress {
            step: "python".to_string(),
            step_name: "安装 Python".to_string(),
            progress,
            message: message.to_string(),
            is_error,
        });
    };

    emit_progress(0.0, "正在检查 winget...", false);

    // 检查 winget 是否可用
    let winget_check = Command::new("winget")
        .arg("--version")
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    if winget_check.is_err() || !winget_check.unwrap().status.success() {
        emit_progress(0.0, "winget 不可用，请手动安装 Python", true);
        return Err("winget 不可用。请从 https://www.python.org/downloads/ 下载安装 Python 3.11".to_string());
    }

    emit_progress(10.0, "正在使用 winget 安装 Python 3.11...", false);

    // 使用 winget 安装 Python
    let mut child = Command::new("winget")
        .args([
            "install", "-e", "--id", "Python.Python.3.11",
            "--accept-package-agreements", "--accept-source-agreements",
            "--silent"
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| format!("启动 winget 失败: {}", e))?;

    // 读取输出显示进度
    let stdout = child.stdout.take();
    if let Some(stdout) = stdout {
        let reader = BufReader::new(stdout);
        let mut progress: f32 = 10.0;
        for line in reader.lines().map_while(Result::ok) {
            progress = (progress + 5.0).min(90.0);
            emit_progress(progress, &line, false);
        }
    }

    let status = child.wait().map_err(|e| format!("等待安装完成失败: {}", e))?;

    if status.success() {
        emit_progress(95.0, "Python 安装完成，正在查找路径...", false);

        // 等待一下让系统更新 PATH
        std::thread::sleep(std::time::Duration::from_secs(2));

        // 查找新安装的 Python
        match find_python() {
            Ok((path, _)) => {
                emit_progress(100.0, "Python 安装完成!", false);
                Ok(path)
            }
            Err(_) => {
                // 尝试常见路径
                let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
                let possible_path = format!("{}\\AppData\\Local\\Programs\\Python\\Python311\\python.exe", user_profile);
                if std::path::Path::new(&possible_path).exists() {
                    emit_progress(100.0, "Python 安装完成!", false);
                    Ok(possible_path)
                } else {
                    emit_progress(0.0, "安装完成但无法找到 Python 路径，请重启应用", true);
                    Err("安装完成但无法找到 Python 路径，请重启应用后重试".to_string())
                }
            }
        }
    } else {
        emit_progress(0.0, "Python 安装失败", true);
        Err("Python 安装失败，请手动从 https://www.python.org/downloads/ 安装".to_string())
    }
}

/// macOS: 安装 Python
#[cfg(target_os = "macos")]
pub async fn install_python(app: tauri::AppHandle) -> Result<String, String> {
    use tauri::Emitter;

    let emit_progress = |progress: f32, message: &str, is_error: bool| {
        let _ = app.emit("install-progress", InstallProgress {
            step: "python".to_string(),
            step_name: "安装 Python".to_string(),
            progress,
            message: message.to_string(),
            is_error,
        });
    };

    emit_progress(0.0, "正在检查 Homebrew...", false);

    // 检查 Homebrew 是否可用
    let brew_check = Command::new("brew")
        .arg("--version")
        .output();

    if brew_check.is_err() || !brew_check.unwrap().status.success() {
        emit_progress(0.0, "Homebrew 未安装", true);
        return Err("请先安装 Homebrew: /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"".to_string());
    }

    emit_progress(10.0, "正在使用 Homebrew 安装 Python...", false);

    // 使用 Homebrew 安装 Python
    let mut child = Command::new("brew")
        .args(["install", "python@3.11"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("启动 brew 失败: {}", e))?;

    // 读取输出显示进度
    let stderr = child.stderr.take();
    if let Some(stderr) = stderr {
        let reader = BufReader::new(stderr);
        let mut progress: f32 = 10.0;
        for line in reader.lines().map_while(Result::ok) {
            progress = (progress + 2.0).min(90.0);
            if !line.is_empty() {
                emit_progress(progress, &line, false);
            }
        }
    }

    let status = child.wait().map_err(|e| format!("等待安装完成失败: {}", e))?;

    if status.success() {
        emit_progress(95.0, "Python 安装完成，正在查找路径...", false);

        // 查找安装的 Python
        let python_paths = vec![
            "/opt/homebrew/bin/python3",
            "/usr/local/bin/python3",
        ];

        for path in python_paths {
            if std::path::Path::new(path).exists() {
                emit_progress(100.0, "Python 安装完成!", false);
                return Ok(path.to_string());
            }
        }

        // 尝试使用 which
        if let Ok(output) = Command::new("which").arg("python3").output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                emit_progress(100.0, "Python 安装完成!", false);
                return Ok(path);
            }
        }

        emit_progress(100.0, "Python 安装完成!", false);
        Ok("python3".to_string())
    } else {
        emit_progress(0.0, "Python 安装失败", true);
        Err("Python 安装失败，请手动运行: brew install python@3.11".to_string())
    }
}

/// Linux 和其他平台
#[cfg(all(not(windows), not(target_os = "macos")))]
pub async fn install_python(_app: tauri::AppHandle) -> Result<String, String> {
    Err("请使用系统包管理器安装 Python 3:\nsudo apt install python3 (Debian/Ubuntu)\nsudo dnf install python3 (Fedora)".to_string())
}

/// 安装 Playwright
pub async fn install_playwright(app: tauri::AppHandle, python_path: &str) -> Result<(), String> {
    use tauri::Emitter;

    let emit_progress = |progress: f32, message: &str, is_error: bool| {
        let _ = app.emit("install-progress", InstallProgress {
            step: "playwright".to_string(),
            step_name: "安装 Playwright".to_string(),
            progress,
            message: message.to_string(),
            is_error,
        });
    };

    emit_progress(0.0, "正在安装 Playwright...", false);

    let mut cmd = Command::new(python_path);
    cmd.args(["-m", "pip", "install", "--upgrade", "playwright"])
        .env("PYTHONUNBUFFERED", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let mut child = cmd.spawn()
        .map_err(|e| format!("启动 pip 失败: {}", e))?;

    // 读取输出显示进度
    let stdout = child.stdout.take();
    if let Some(stdout) = stdout {
        let reader = BufReader::new(stdout);
        let mut progress: f32 = 10.0;
        for line in reader.lines().map_while(Result::ok) {
            if line.contains("Downloading") || line.contains("Collecting") {
                progress = (progress + 10.0).min(80.0);
            } else if line.contains("Installing") {
                progress = 85.0;
            } else if line.contains("Successfully") {
                progress = 95.0;
            }
            if !line.is_empty() {
                emit_progress(progress, &line, false);
            }
        }
    }

    let status = child.wait().map_err(|e| format!("等待安装完成失败: {}", e))?;

    if status.success() {
        emit_progress(100.0, "Playwright 安装完成!", false);
        Ok(())
    } else {
        emit_progress(0.0, "Playwright 安装失败", true);
        Err("Playwright 安装失败".to_string())
    }
}

/// 安装 Chromium
pub async fn install_chromium(app: tauri::AppHandle, python_path: &str) -> Result<(), String> {
    use tauri::Emitter;
    use std::io::Read;

    let app_clone = app.clone();
    let emit_progress = move |progress: f32, message: &str, is_error: bool| {
        let _ = app_clone.emit("install-progress", InstallProgress {
            step: "chromium".to_string(),
            step_name: "安装 Chromium 浏览器".to_string(),
            progress,
            message: message.to_string(),
            is_error,
        });
    };

    emit_progress(0.0, "正在下载 Chromium 浏览器 (约 150MB)...", false);

    let mut cmd = Command::new(python_path);
    cmd.args(["-m", "playwright", "install", "chromium"])
        .env("PYTHONUNBUFFERED", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let mut child = cmd.spawn()
        .map_err(|e| format!("启动 playwright install 失败: {}", e))?;

    // 读取 stderr (Playwright 进度输出在 stderr)
    // 注意：Playwright 使用 \r 来更新进度，所以要按字节读取
    let stderr = child.stderr.take();
    if let Some(mut stderr) = stderr {
        let mut buffer = [0u8; 1024];
        let mut line_buffer = String::new();
        let mut last_progress: f32 = 0.0;

        loop {
            match stderr.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buffer[..n]);
                    line_buffer.push_str(&chunk);

                    // 按 \r 或 \n 分割处理
                    while let Some(pos) = line_buffer.find(|c| c == '\r' || c == '\n') {
                        let line: String = line_buffer.drain(..=pos).collect();
                        let line = line.trim();

                        if line.is_empty() {
                            continue;
                        }

                        // 解析进度，例如 "Downloading Chromium 120.0.6099.28 - 45.2 MiB / 150 MiB"
                        let progress = if line.contains("MiB") && line.contains('/') {
                            // 解析类似 "45.2 MiB / 150 MiB" 的格式
                            let parts: Vec<&str> = line.split('/').collect();
                            if parts.len() >= 2 {
                                // 从第一部分提取当前大小
                                let current = parts[0].split_whitespace()
                                    .filter_map(|s| s.parse::<f32>().ok())
                                    .last()
                                    .unwrap_or(0.0);
                                // 从第二部分提取总大小
                                let total = parts[1].split_whitespace()
                                    .filter_map(|s| s.parse::<f32>().ok())
                                    .next()
                                    .unwrap_or(150.0);
                                if total > 0.0 {
                                    (current / total * 100.0).min(99.0)
                                } else {
                                    last_progress
                                }
                            } else {
                                last_progress
                            }
                        } else if line.contains('%') {
                            // 尝试提取百分比
                            line.split_whitespace()
                                .filter_map(|s| s.trim_end_matches('%').parse::<f32>().ok())
                                .next()
                                .unwrap_or(last_progress)
                        } else if line.contains("Downloading") {
                            5.0
                        } else {
                            last_progress
                        };

                        if progress > last_progress {
                            last_progress = progress;
                        }
                        emit_progress(last_progress, line, false);
                    }
                }
                Err(_) => break,
            }
        }
    }

    let status = child.wait().map_err(|e| format!("等待安装完成失败: {}", e))?;

    let app_final = app.clone();
    if status.success() {
        let _ = app_final.emit("install-progress", InstallProgress {
            step: "chromium".to_string(),
            step_name: "安装 Chromium 浏览器".to_string(),
            progress: 100.0,
            message: "Chromium 安装完成!".to_string(),
            is_error: false,
        });
        Ok(())
    } else {
        let _ = app_final.emit("install-progress", InstallProgress {
            step: "chromium".to_string(),
            step_name: "安装 Chromium 浏览器".to_string(),
            progress: 0.0,
            message: "Chromium 安装失败".to_string(),
            is_error: true,
        });
        Err("Chromium 安装失败".to_string())
    }
}

/// 安装所有依赖
pub async fn install_all_dependencies(app: tauri::AppHandle) -> InstallResult {
    use tauri::Emitter;

    // 检查当前状态
    let mut status = check_dependency_status();
    let mut python_path = status.python_path.clone().unwrap_or_default();

    // 1. 安装 Python (如果需要)
    if !status.python_installed {
        match install_python(app.clone()).await {
            Ok(path) => {
                python_path = path;
                status.python_installed = true;
            }
            Err(e) => {
                return InstallResult {
                    success: false,
                    message: e,
                    python_path: None,
                };
            }
        }
    }

    // 2. 安装 Playwright
    if !status.playwright_installed {
        if let Err(e) = install_playwright(app.clone(), &python_path).await {
            return InstallResult {
                success: false,
                message: e,
                python_path: Some(python_path),
            };
        }
    }

    // 3. 安装 Chromium
    if !status.chromium_installed {
        if let Err(e) = install_chromium(app.clone(), &python_path).await {
            return InstallResult {
                success: false,
                message: e,
                python_path: Some(python_path),
            };
        }
    }

    // 发送完成事件
    let _ = app.emit("install-complete", serde_json::json!({ "success": true }));

    InstallResult {
        success: true,
        message: "所有依赖安装完成!".to_string(),
        python_path: Some(python_path),
    }
}

/// 仅安装 Playwright 和 Chromium (Python 已存在)
pub async fn install_playwright_only(app: tauri::AppHandle) -> InstallResult {
    let status = check_dependency_status();

    let python_path = match status.python_path {
        Some(path) => path,
        None => {
            return InstallResult {
                success: false,
                message: "未找到 Python，请先安装 Python 3".to_string(),
                python_path: None,
            };
        }
    };

    // 安装 Playwright
    if !status.playwright_installed {
        if let Err(e) = install_playwright(app.clone(), &python_path).await {
            return InstallResult {
                success: false,
                message: e,
                python_path: Some(python_path),
            };
        }
    }

    // 安装 Chromium
    if !status.chromium_installed {
        if let Err(e) = install_chromium(app.clone(), &python_path).await {
            return InstallResult {
                success: false,
                message: e,
                python_path: Some(python_path),
            };
        }
    }

    InstallResult {
        success: true,
        message: "依赖安装完成!".to_string(),
        python_path: Some(python_path),
    }
}

use crate::db;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const DEEPSEEK_API_URL: &str = "https://api.deepseek.com/chat/completions";

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Deserialize)]
struct ChatMessageResponse {
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

/// 获取 DeepSeek API Key
fn get_api_key() -> Option<String> {
    db::get_setting("deepseek").ok().flatten()
}

/// 生成市场调研报告
pub async fn generate_market_research_report(
    marketplace: &str,
    category_name: &str,
    products_json: &str,
) -> Result<String, String> {
    let api_key = get_api_key().ok_or("未配置 DeepSeek API Key")?;

    // 解析产品数据
    let products: Vec<serde_json::Value> = serde_json::from_str(products_json)
        .map_err(|e| format!("解析产品数据失败: {}", e))?;

    // 构建产品摘要（最多取前20个产品）
    let product_summary: Vec<String> = products.iter().take(20).enumerate().map(|(i, p)| {
        let title = p.get("title").and_then(|v| v.as_str()).unwrap_or("未知");
        let price = p.get("price").and_then(|v| v.as_str()).unwrap_or("N/A");
        let rating = p.get("rating").and_then(|v| v.as_str()).unwrap_or("N/A");
        let reviews = p.get("reviews_count").and_then(|v| v.as_str()).unwrap_or("N/A");
        let bsr = p.get("bsr_rank").and_then(|v| v.as_i64()).unwrap_or(0);
        format!("{}. BSR #{} | {} | {} | {} 评价 | {}", i + 1, bsr, price, rating, reviews,
            if title.len() > 60 { format!("{}...", &title[..60]) } else { title.to_string() })
    }).collect();

    let prompt = format!(
        r#"你是一位专业的亚马逊市场分析师。请基于以下 {} 站点 "{}" 类目的 BSR 榜单数据，生成一份简洁的市场调研报告。

## 榜单数据（共 {} 个产品，显示前20）:
{}

## 报告要求:
请用 Markdown 格式输出，包含以下内容：

### 1. 市场概览
- 类目整体情况
- 价格带分布

### 2. 头部产品分析
- Top 5 产品特点
- 价格策略

### 3. 市场机会
- 潜在机会点
- 进入建议

### 4. 风险提示
- 主要风险
- 注意事项

请保持报告简洁，重点突出，总字数控制在 800 字以内。"#,
        marketplace,
        category_name,
        products.len(),
        product_summary.join("\n")
    );

    let client = Client::new();
    let request = ChatRequest {
        model: "deepseek-chat".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }],
        temperature: 0.7,
        max_tokens: 2000,
    };

    let response = client
        .post(DEEPSEEK_API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("API 请求失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("API 返回错误 {}: {}", status, error_text));
    }

    let chat_response: ChatResponse = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "API 返回空响应".to_string())
}

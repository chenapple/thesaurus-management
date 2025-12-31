// 知识库文档处理模块
// 支持 PDF、Word、Excel、Markdown、纯文本

use std::fs;
use std::path::Path;

// ==================== 文档解析结果 ====================

#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub title: String,
    pub content: String,
    pub pages: Vec<PageContent>,
    pub file_type: String,
}

#[derive(Debug, Clone)]
pub struct PageContent {
    pub page_number: i64,
    pub content: String,
}

// ==================== 文档解析器 ====================

/// 根据文件类型解析文档
pub fn parse_document(file_path: &str) -> Result<ParsedDocument, String> {
    let path = Path::new(file_path);
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let file_name = path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("untitled")
        .to_string();

    match extension.as_str() {
        "pdf" => parse_pdf(file_path, &file_name),
        "docx" => parse_docx(file_path, &file_name),
        "xlsx" | "xls" => parse_excel(file_path, &file_name),
        "pptx" => parse_pptx(file_path, &file_name),
        "md" | "markdown" => parse_markdown(file_path, &file_name),
        "txt" => parse_text(file_path, &file_name),
        _ => Err(format!("不支持的文件类型: {}", extension)),
    }
}

/// 解析 PDF 文件
fn parse_pdf(file_path: &str, title: &str) -> Result<ParsedDocument, String> {
    use pdf_extract::extract_text;

    let content = extract_text(file_path).map_err(|e| format!("PDF 解析失败: {}", e))?;

    // PDF 通常没有明确的页面分隔，我们按换行符分割估算
    // 实际上 pdf-extract 不提供页面信息，所以我们将整个内容作为一页
    let pages = vec![PageContent {
        page_number: 1,
        content: content.clone(),
    }];

    Ok(ParsedDocument {
        title: title.to_string(),
        content,
        pages,
        file_type: "pdf".to_string(),
    })
}

/// 解析 Word 文档 (.docx)
fn parse_docx(file_path: &str, title: &str) -> Result<ParsedDocument, String> {
    use docx_rs::read_docx;

    let file_content = fs::read(file_path).map_err(|e| format!("读取文件失败: {}", e))?;

    let docx = read_docx(&file_content).map_err(|e| format!("DOCX 解析失败: {}", e))?;

    // 提取所有段落文本
    let mut content_parts: Vec<String> = Vec::new();

    for child in docx.document.children {
        if let docx_rs::DocumentChild::Paragraph(para) = child {
            let mut para_text = String::new();
            for run_child in para.children {
                if let docx_rs::ParagraphChild::Run(run) = run_child {
                    for run_content in run.children {
                        if let docx_rs::RunChild::Text(text) = run_content {
                            para_text.push_str(&text.text);
                        }
                    }
                }
            }
            if !para_text.is_empty() {
                content_parts.push(para_text);
            }
        }
    }

    let content = content_parts.join("\n\n");

    let pages = vec![PageContent {
        page_number: 1,
        content: content.clone(),
    }];

    Ok(ParsedDocument {
        title: title.to_string(),
        content,
        pages,
        file_type: "docx".to_string(),
    })
}

/// 解析 Excel 文件
fn parse_excel(file_path: &str, title: &str) -> Result<ParsedDocument, String> {
    use calamine::{open_workbook_auto, Reader, Data};

    let mut workbook: calamine::Sheets<std::io::BufReader<std::fs::File>> =
        open_workbook_auto(file_path).map_err(|e| format!("Excel 解析失败: {}", e))?;

    let mut all_content: Vec<String> = Vec::new();
    let mut pages: Vec<PageContent> = Vec::new();
    let mut page_number = 1;

    // 遍历所有工作表
    let sheet_names = workbook.sheet_names().to_vec();
    for sheet_name in sheet_names {
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            let mut sheet_content: Vec<String> = Vec::new();
            sheet_content.push(format!("## 工作表: {}", sheet_name));

            for row in range.rows() {
                let row_text: Vec<String> = row
                    .iter()
                    .map(|cell: &Data| match cell {
                        Data::Int(i) => i.to_string(),
                        Data::Float(f) => f.to_string(),
                        Data::String(s) => s.clone(),
                        Data::Bool(b) => b.to_string(),
                        Data::DateTime(dt) => dt.to_string(),
                        Data::DateTimeIso(dt) => dt.clone(),
                        Data::DurationIso(d) => d.clone(),
                        Data::Error(e) => format!("Error: {:?}", e),
                        Data::Empty => String::new(),
                    })
                    .filter(|s: &String| !s.is_empty())
                    .collect();

                if !row_text.is_empty() {
                    sheet_content.push(row_text.join(" | "));
                }
            }

            let sheet_text = sheet_content.join("\n");
            all_content.push(sheet_text.clone());

            pages.push(PageContent {
                page_number,
                content: sheet_text,
            });
            page_number += 1;
        }
    }

    let content = all_content.join("\n\n---\n\n");

    Ok(ParsedDocument {
        title: title.to_string(),
        content,
        pages,
        file_type: "xlsx".to_string(),
    })
}

/// 解析 PowerPoint 文件 (.pptx)
fn parse_pptx(file_path: &str, title: &str) -> Result<ParsedDocument, String> {
    use std::io::Read;
    use zip::ZipArchive;

    let file = fs::File::open(file_path).map_err(|e| format!("打开文件失败: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("PPTX 解压失败: {}", e))?;

    // 收集所有幻灯片文件名并排序
    let mut slide_names: Vec<String> = Vec::new();
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            let name = file.name().to_string();
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                slide_names.push(name);
            }
        }
    }

    // 按幻灯片编号排序 (slide1.xml, slide2.xml, ...)
    slide_names.sort_by(|a, b| {
        let num_a = extract_slide_number(a);
        let num_b = extract_slide_number(b);
        num_a.cmp(&num_b)
    });

    let mut all_content: Vec<String> = Vec::new();
    let mut pages: Vec<PageContent> = Vec::new();

    for (idx, slide_name) in slide_names.iter().enumerate() {
        let page_number = (idx + 1) as i64;

        if let Ok(mut slide_file) = archive.by_name(slide_name) {
            let mut xml_content = String::new();
            if slide_file.read_to_string(&mut xml_content).is_ok() {
                let slide_text = extract_text_from_pptx_xml(&xml_content);

                if !slide_text.is_empty() {
                    let slide_content = format!("## 幻灯片 {}\n{}", page_number, slide_text);
                    all_content.push(slide_content.clone());

                    pages.push(PageContent {
                        page_number,
                        content: slide_content,
                    });
                }
            }
        }
    }

    let content = all_content.join("\n\n---\n\n");

    Ok(ParsedDocument {
        title: title.to_string(),
        content,
        pages,
        file_type: "pptx".to_string(),
    })
}

/// 从幻灯片文件名中提取编号
fn extract_slide_number(name: &str) -> i32 {
    // ppt/slides/slide1.xml -> 1
    name.chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or(0)
}

/// 从 PPTX XML 中提取文本内容
fn extract_text_from_pptx_xml(xml: &str) -> String {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_str(xml);

    let mut texts: Vec<String> = Vec::new();
    let mut current_paragraph: Vec<String> = Vec::new();
    let mut in_text_element = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                // <a:t> 是文本元素
                if e.name().as_ref() == b"a:t" {
                    in_text_element = true;
                }
            }
            Ok(Event::Text(e)) => {
                if in_text_element {
                    if let Ok(text) = e.unescape() {
                        let text = text.trim();
                        if !text.is_empty() {
                            current_paragraph.push(text.to_string());
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"a:t" {
                    in_text_element = false;
                }
                // <a:p> 是段落结束
                if e.name().as_ref() == b"a:p" {
                    if !current_paragraph.is_empty() {
                        texts.push(current_paragraph.join(""));
                        current_paragraph.clear();
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    // 处理剩余的段落
    if !current_paragraph.is_empty() {
        texts.push(current_paragraph.join(""));
    }

    texts.join("\n")
}

/// 解析 Markdown 文件
fn parse_markdown(file_path: &str, title: &str) -> Result<ParsedDocument, String> {
    use pulldown_cmark::{Parser, Event, TagEnd};

    let markdown_content = fs::read_to_string(file_path).map_err(|e| format!("读取文件失败: {}", e))?;

    // 将 Markdown 转换为纯文本
    let parser = Parser::new(&markdown_content);
    let mut text_content = String::new();

    for event in parser {
        match event {
            Event::Text(text) => {
                text_content.push_str(&text);
            }
            Event::Code(code) => {
                text_content.push_str(&code);
            }
            Event::SoftBreak | Event::HardBreak => {
                text_content.push('\n');
            }
            Event::End(TagEnd::Paragraph) | Event::End(TagEnd::Heading(_)) => {
                text_content.push_str("\n\n");
            }
            Event::End(TagEnd::Item) => {
                text_content.push('\n');
            }
            _ => {}
        }
    }

    let content = text_content.trim().to_string();

    let pages = vec![PageContent {
        page_number: 1,
        content: content.clone(),
    }];

    Ok(ParsedDocument {
        title: title.to_string(),
        content,
        pages,
        file_type: "md".to_string(),
    })
}

/// 解析纯文本文件
fn parse_text(file_path: &str, title: &str) -> Result<ParsedDocument, String> {
    let content = fs::read_to_string(file_path).map_err(|e| format!("读取文件失败: {}", e))?;

    let pages = vec![PageContent {
        page_number: 1,
        content: content.clone(),
    }];

    Ok(ParsedDocument {
        title: title.to_string(),
        content,
        pages,
        file_type: "txt".to_string(),
    })
}

// ==================== 文本分块器 ====================

/// 文本分块配置
pub struct ChunkerConfig {
    pub chunk_size: usize,     // 目标分块大小（字符数）
    pub chunk_overlap: usize,  // 分块重叠（字符数）
}

impl Default for ChunkerConfig {
    fn default() -> Self {
        Self {
            chunk_size: 800,
            chunk_overlap: 100,
        }
    }
}

/// 文本分块结果
#[derive(Debug, Clone)]
pub struct TextChunk {
    pub index: usize,
    pub content: String,
    pub page_number: Option<i64>,
}

/// 将文档内容分块
pub fn chunk_document(doc: &ParsedDocument, config: &ChunkerConfig) -> Vec<TextChunk> {
    let mut chunks: Vec<TextChunk> = Vec::new();
    let mut chunk_index = 0;

    for page in &doc.pages {
        let page_chunks = chunk_text(&page.content, config);

        for chunk_content in page_chunks {
            chunks.push(TextChunk {
                index: chunk_index,
                content: chunk_content,
                page_number: Some(page.page_number),
            });
            chunk_index += 1;
        }
    }

    chunks
}

/// 将文本分块（递归分割算法）
fn chunk_text(text: &str, config: &ChunkerConfig) -> Vec<String> {
    // 分隔符优先级（从高到低）
    let separators = vec![
        "\n\n\n",  // 多个空行
        "\n\n",    // 段落
        "\n",      // 换行
        "。",      // 中文句号
        ".",       // 英文句号
        "！",      // 中文感叹号
        "!",       // 英文感叹号
        "？",      // 中文问号
        "?",       // 英文问号
        "；",      // 中文分号
        ";",       // 英文分号
        "，",      // 中文逗号
        ",",       // 英文逗号
        " ",       // 空格
    ];

    recursive_split(text, &separators, config)
}

/// 递归分割文本
fn recursive_split(text: &str, separators: &[&str], config: &ChunkerConfig) -> Vec<String> {
    let text = text.trim();

    // 如果文本已经足够小，直接返回
    if text.chars().count() <= config.chunk_size {
        if text.is_empty() {
            return vec![];
        }
        return vec![text.to_string()];
    }

    // 尝试使用当前分隔符分割
    for (i, separator) in separators.iter().enumerate() {
        let parts: Vec<&str> = text.split(*separator).collect();

        if parts.len() > 1 {
            let mut chunks: Vec<String> = Vec::new();
            let mut current_chunk = String::new();

            for part in parts {
                let part_with_sep = if !current_chunk.is_empty() {
                    format!("{}{}", *separator, part)
                } else {
                    part.to_string()
                };

                if current_chunk.chars().count() + part_with_sep.chars().count() <= config.chunk_size {
                    current_chunk.push_str(&part_with_sep);
                } else {
                    // 保存当前块
                    if !current_chunk.is_empty() {
                        chunks.push(current_chunk.trim().to_string());
                    }

                    // 如果单个部分超过 chunk_size，递归分割
                    if part.chars().count() > config.chunk_size && i + 1 < separators.len() {
                        let sub_chunks = recursive_split(part, &separators[i + 1..], config);
                        chunks.extend(sub_chunks);
                        current_chunk = String::new();
                    } else {
                        current_chunk = part.to_string();
                    }
                }
            }

            // 保存最后一块
            if !current_chunk.is_empty() {
                chunks.push(current_chunk.trim().to_string());
            }

            // 添加重叠
            if config.chunk_overlap > 0 {
                chunks = add_overlap(chunks, config.chunk_overlap);
            }

            return chunks.into_iter().filter(|c| !c.is_empty()).collect();
        }
    }

    // 如果没有分隔符可用，按字符强制分割
    force_split(text, config)
}

/// 添加分块重叠
fn add_overlap(chunks: Vec<String>, overlap: usize) -> Vec<String> {
    if chunks.len() <= 1 {
        return chunks;
    }

    let mut result: Vec<String> = Vec::new();

    for (i, chunk) in chunks.iter().enumerate() {
        if i == 0 {
            result.push(chunk.clone());
        } else {
            // 从前一个块获取重叠部分
            let prev_chunk = &chunks[i - 1];
            let prev_chars: Vec<char> = prev_chunk.chars().collect();
            let overlap_start = if prev_chars.len() > overlap {
                prev_chars.len() - overlap
            } else {
                0
            };
            let overlap_text: String = prev_chars[overlap_start..].iter().collect();

            // 合并重叠部分和当前块
            let merged = format!("{}{}", overlap_text, chunk);
            result.push(merged);
        }
    }

    result
}

/// 强制按字符分割
fn force_split(text: &str, config: &ChunkerConfig) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut chunks: Vec<String> = Vec::new();

    let mut start = 0;
    while start < chars.len() {
        let end = (start + config.chunk_size).min(chars.len());
        let chunk: String = chars[start..end].iter().collect();
        chunks.push(chunk);
        start = end;
    }

    chunks
}

// ==================== 完整处理流程 ====================

/// 处理文档：解析 + 分块
pub fn process_document(file_path: &str, chunk_config: Option<ChunkerConfig>) -> Result<(ParsedDocument, Vec<TextChunk>), String> {
    let config = chunk_config.unwrap_or_default();

    // 解析文档
    let doc = parse_document(file_path)?;

    // 分块
    let chunks = chunk_document(&doc, &config);

    Ok((doc, chunks))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text() {
        let config = ChunkerConfig {
            chunk_size: 100,
            chunk_overlap: 20,
        };

        let text = "这是第一段内容。这是第一段的更多内容。\n\n这是第二段内容。这是第二段的更多内容。\n\n这是第三段内容。";
        let chunks = chunk_text(text, &config);

        assert!(!chunks.is_empty());
        for chunk in &chunks {
            println!("Chunk: {}", chunk);
        }
    }
}

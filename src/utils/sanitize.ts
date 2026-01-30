/**
 * XSS 防护工具模块
 * 使用 DOMPurify 清理 HTML，防止 XSS 攻击
 */
import DOMPurify from 'dompurify';
import { marked } from 'marked';

// 允许的 HTML 标签
const ALLOWED_TAGS = [
  'p', 'br', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
  'strong', 'em', 'b', 'i', 'u', 's', 'del',
  'ul', 'ol', 'li',
  'code', 'pre',
  'table', 'thead', 'tbody', 'tr', 'th', 'td',
  'a', 'img',
  'blockquote',
  'div', 'span',
  'hr',
];

// 允许的 HTML 属性
const ALLOWED_ATTR = [
  'class', 'style',
  'href', 'src', 'alt', 'title',
  'target', 'rel',
  'colspan', 'rowspan',
];

// DOMPurify 配置
const PURIFY_CONFIG = {
  ALLOWED_TAGS,
  ALLOWED_ATTR,
  ALLOW_DATA_ATTR: false,
  // 禁止所有 javascript: URLs
  FORBID_ATTR: ['onerror', 'onload', 'onclick', 'onmouseover', 'onfocus', 'onblur'],
};

/**
 * 清理任意 HTML，移除潜在的 XSS 攻击代码
 * @param dirty 不可信的 HTML 字符串
 * @returns 安全的 HTML 字符串
 */
export function sanitizeHtml(dirty: string): string {
  if (!dirty) return '';
  return DOMPurify.sanitize(dirty, PURIFY_CONFIG);
}

/**
 * 安全渲染 Markdown 为 HTML
 * @param content Markdown 内容
 * @returns 安全的 HTML 字符串
 */
export function renderMarkdownSafe(content: string): string {
  if (!content) return '';
  const html = marked(content, { breaks: true }) as string;
  return sanitizeHtml(html);
}

// Callout 类型配置（用于知识库）
const CALLOUT_TYPES: Record<string, { icon: string; color: string; label: string }> = {
  note: { icon: '📝', color: '#3B82F6', label: '笔记' },
  tip: { icon: '💡', color: '#10B981', label: '提示' },
  important: { icon: '⚠️', color: '#F59E0B', label: '重要' },
  warning: { icon: '⚠️', color: '#F97316', label: '警告' },
  caution: { icon: '🔴', color: '#EF4444', label: '注意' },
  info: { icon: 'ℹ️', color: '#6366F1', label: '信息' },
  quote: { icon: '💬', color: '#8B5CF6', label: '引用' },
  example: { icon: '📋', color: '#14B8A6', label: '示例' },
  question: { icon: '❓', color: '#EC4899', label: '问题' },
  success: { icon: '✅', color: '#67C23A', label: '成功' },
  failure: { icon: '❎', color: '#F56C6C', label: '失败' },
};

/**
 * 安全渲染 Markdown，支持 Obsidian Callout 语法（用于知识库）
 * @param content Markdown 内容
 * @returns 安全的 HTML 字符串
 */
export function renderMarkdownWithCallouts(content: string): string {
  if (!content) return '';

  // 预处理 Callouts: > [!type] 标题
  let processedContent = content;

  // 分割内容为块（按 Obsidian Callout 语法分割）
  const blocks = processedContent.split(/\n(?=>\s*\[!)/);
  const processedBlocks = blocks.map(block => {
    const calloutMatch = block.match(/^>\s*\[!(\w+)\]\s*(.*?)(?:\n|$)/);
    if (calloutMatch) {
      const type = calloutMatch[1].toLowerCase();
      const title = calloutMatch[2] || '';
      const config = CALLOUT_TYPES[type] || CALLOUT_TYPES['note'];

      // 提取 callout 内容（去掉首行和 > 前缀）
      const lines = block.split('\n');
      const contentLines = lines.slice(1).map(line => {
        return line.replace(/^>\s?/, '');
      }).join('\n');

      // 先渲染内容的 Markdown，再整体清理
      const renderedContent = marked(contentLines, { breaks: true }) as string;

      return `<div class="callout callout-${sanitizeHtml(type)}" style="--callout-color: ${config.color}">
        <div class="callout-header">
          <span class="callout-icon">${config.icon}</span>
          <span class="callout-title">${sanitizeHtml(title) || config.label}</span>
        </div>
        <div class="callout-content">${sanitizeHtml(renderedContent)}</div>
      </div>`;
    }
    return block;
  });

  processedContent = processedBlocks.join('\n');

  let html = marked(processedContent, { breaks: true }) as string;

  // 将 [来源X] 包裹在 span 中以便样式化
  html = html.replace(/\[来源(\d+)\]/g, '<span class="source-ref">[来源$1]</span>');

  // 处理 [[文档标题]] 双向链接语法
  html = html.replace(/\[\[([^\]]+)\]\]/g, '<span class="wiki-link">$1</span>');

  return sanitizeHtml(html);
}

/**
 * 简单 Markdown 渲染（用于 Agent 和 Dashboard）
 * 支持基本 Markdown 语法和表格
 * @param text Markdown 文本
 * @returns 安全的 HTML 字符串
 */
export function renderSimpleMarkdown(text: string): string {
  if (!text) return '';

  let result = text
    // 代码块
    .replace(/```(\w*)\n([\s\S]*?)```/g, '<pre><code>$2</code></pre>')
    // 标题（从最多井号开始处理，避免误匹配）
    .replace(/^#### (.+)$/gm, '<h5>$1</h5>')
    .replace(/^### (.+)$/gm, '<h4>$1</h4>')
    .replace(/^## (.+)$/gm, '<h3>$1</h3>')
    .replace(/^# (.+)$/gm, '<h2>$1</h2>')
    // 粗体
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    // 斜体
    .replace(/\*(.+?)\*/g, '<em>$1</em>')
    // 列表
    .replace(/^- (.+)$/gm, '<li>$1</li>')
    .replace(/(<li>.*<\/li>\n?)+/g, '<ul>$&</ul>');

  // 解析 Markdown 表格
  result = result.replace(/(?:^|\n)(\|.+\|)\n(\|[-:\s|]+\|)\n((?:\|.+\|\n?)+)/g, (_match, header, _separator, body) => {
    const headerCells = header.split('|').filter((c: string) => c.trim()).map((c: string) =>
      `<th style="padding:10px 14px;text-align:left;font-weight:600;background:#f8fafc;border-bottom:2px solid #e5e7eb;color:#374151;font-size:13px;">${sanitizeHtml(c.trim())}</th>`
    ).join('');
    const bodyRows = body.trim().split('\n').map((row: string) => {
      const cells = row.split('|').filter((c: string) => c.trim()).map((c: string) => {
        const content = c.trim();
        // 检测变化列（包含 ↑ 或 ↓）
        const isUp = content.includes('↑');
        const isDown = content.includes('↓');
        const color = isUp ? '#059669' : isDown ? '#dc2626' : '#374151';
        const fontWeight = (isUp || isDown) ? '600' : '400';
        return `<td style="padding:10px 14px;border-bottom:1px solid #e5e7eb;color:${color};font-weight:${fontWeight};font-size:13px;">${sanitizeHtml(content)}</td>`;
      }).join('');
      return `<tr>${cells}</tr>`;
    }).join('');
    return `<table style="width:100%;border-collapse:collapse;margin:12px 0;"><thead><tr>${headerCells}</tr></thead><tbody>${bodyRows}</tbody></table>`;
  });

  // 换行：连续两个换行变成段落，单个换行变成 <br>
  // 但先排除 HTML 标签后的换行
  result = result
    .replace(/>\n</g, '><')
    .replace(/\n\n+/g, '</p><p>')
    .replace(/\n/g, '<br>');

  // 包裹在段落中
  if (!result.startsWith('<')) {
    result = `<p>${result}</p>`;
  }

  return sanitizeHtml(result);
}

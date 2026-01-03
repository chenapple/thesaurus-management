// 错误信息转换工具
// 将技术性的 API 错误信息转换为用户友好的提示

/**
 * HTTP 状态码到用户友好消息的映射
 */
const HTTP_STATUS_MESSAGES: Record<number, string> = {
  400: '请求参数错误，请检查输入内容',
  401: 'API Key 无效或已过期，请检查配置',
  403: 'API Key 权限不足，请检查账户权限',
  404: 'API 服务不可用，请稍后重试',
  429: '请求过于频繁，请稍后重试',
  500: 'AI 服务内部错误，请稍后重试',
  502: 'AI 服务暂时不可用，请稍后重试',
  503: 'AI 服务繁忙，请稍后重试',
  504: '请求超时，请稍后重试',
};

/**
 * 常见错误模式到用户友好消息的映射
 */
const ERROR_PATTERNS: { pattern: RegExp; message: string }[] = [
  // 认证错误
  { pattern: /invalid.?api.?key/i, message: 'API Key 无效，请检查配置' },
  { pattern: /authentication/i, message: 'API Key 认证失败，请检查配置' },
  { pattern: /unauthorized/i, message: 'API Key 未授权，请检查配置' },
  { pattern: /auth.*fail/i, message: 'API Key 认证失败，请检查配置' },

  // 配额/限流
  { pattern: /rate.?limit/i, message: '请求过于频繁，请稍后重试' },
  { pattern: /quota/i, message: 'API 配额不足，请检查账户余额' },
  { pattern: /insufficient.*balance/i, message: 'API 账户余额不足，请充值' },
  { pattern: /exceeded/i, message: 'API 使用量超限，请检查配额' },

  // 网络错误
  { pattern: /network/i, message: '网络连接失败，请检查网络' },
  { pattern: /timeout/i, message: '请求超时，请重试' },
  { pattern: /timed?\s*out/i, message: '请求超时，请重试' },
  { pattern: /ECONNREFUSED/i, message: '无法连接 AI 服务，请检查网络' },
  { pattern: /ENOTFOUND/i, message: '无法访问 AI 服务，请检查网络' },
  { pattern: /fetch.*fail/i, message: '网络请求失败，请检查网络' },
  { pattern: /connection.*refused/i, message: '无法连接 AI 服务，请检查网络' },

  // 内容问题
  { pattern: /content.?filter/i, message: '内容被安全策略过滤，请修改后重试' },
  { pattern: /blocked/i, message: '请求被拒绝，请修改内容后重试' },
  { pattern: /too.?long/i, message: '内容过长，请缩短后重试' },
  { pattern: /token.?limit/i, message: '内容过长，请缩短后重试' },

  // 服务错误
  { pattern: /server.*error/i, message: 'AI 服务内部错误，请稍后重试' },
  { pattern: /internal.*error/i, message: 'AI 服务内部错误，请稍后重试' },
  { pattern: /service.*unavailable/i, message: 'AI 服务暂时不可用，请稍后重试' },
  { pattern: /overloaded/i, message: 'AI 服务繁忙，请稍后重试' },

  // 模型相关
  { pattern: /model.*not.*found/i, message: '指定的模型不可用，请检查模型配置' },
  { pattern: /invalid.*model/i, message: '无效的模型配置，请检查设置' },
];

/**
 * 解析 HTTP 响应错误，返回用户友好的错误信息
 * @param status HTTP 状态码
 * @param errorText 错误响应文本
 * @param providerName 服务提供商名称（可选，用于增强错误信息）
 * @returns 用户友好的错误信息
 */
export function parseHttpError(
  status: number,
  errorText: string,
  providerName?: string
): string {
  const provider = providerName ? `${providerName} ` : '';

  // 1. 先检查常见错误模式
  for (const { pattern, message } of ERROR_PATTERNS) {
    if (pattern.test(errorText)) {
      return message;
    }
  }

  // 2. 根据状态码返回通用消息
  if (HTTP_STATUS_MESSAGES[status]) {
    return HTTP_STATUS_MESSAGES[status];
  }

  // 3. 默认消息
  if (status >= 500) {
    return `${provider}服务暂时不可用，请稍后重试`;
  }
  if (status >= 400) {
    return `${provider}请求失败，请检查配置后重试`;
  }

  return `${provider}请求异常，请稍后重试`;
}

/**
 * 解析通用错误，返回用户友好的错误信息
 * @param error 错误对象或错误信息
 * @param context 错误上下文（可选）
 * @returns 用户友好的错误信息
 */
export function parseError(error: unknown, context?: string): string {
  const prefix = context ? `${context}: ` : '';

  // 处理字符串错误
  if (typeof error === 'string') {
    // 检查是否是已经友好的错误信息
    if (isFriendlyMessage(error)) {
      return error;
    }

    // 尝试从错误模式中匹配
    for (const { pattern, message } of ERROR_PATTERNS) {
      if (pattern.test(error)) {
        return prefix + message;
      }
    }

    // 尝试提取 HTTP 状态码
    const statusMatch = error.match(/(\d{3})/);
    if (statusMatch) {
      const status = parseInt(statusMatch[1]);
      if (HTTP_STATUS_MESSAGES[status]) {
        return prefix + HTTP_STATUS_MESSAGES[status];
      }
    }

    return prefix + '请求失败，请稍后重试';
  }

  // 处理 Error 对象
  if (error instanceof Error) {
    // 特殊处理中断错误
    if (error.name === 'AbortError') {
      return '操作已取消';
    }

    return parseError(error.message, context);
  }

  return prefix + '未知错误，请稍后重试';
}

/**
 * 检查消息是否已经是用户友好的
 */
function isFriendlyMessage(message: string): boolean {
  const friendlyPatterns = [
    /^请先/,      // 请先...
    /^请检查/,    // 请检查...
    /^请稍后/,    // 请稍后...
    /^无法/,      // 无法...
    /^操作/,      // 操作...
    /配置$/,      // ...配置
    /重试$/,      // ...重试
    /不足$/,      // ...不足
  ];

  return friendlyPatterns.some(pattern => pattern.test(message));
}

/**
 * 创建带服务商名称的错误信息解析器
 * @param providerName 服务商名称
 * @returns 错误解析函数
 */
export function createErrorParser(providerName: string) {
  return {
    parseHttp: (status: number, errorText: string) =>
      parseHttpError(status, errorText, providerName),
    parse: (error: unknown) => parseError(error, providerName),
  };
}

// 预定义的服务商错误解析器
export const deepseekError = createErrorParser('DeepSeek');
export const geminiError = createErrorParser('Gemini');
export const qwenError = createErrorParser('通义千问');
export const openaiError = createErrorParser('OpenAI');

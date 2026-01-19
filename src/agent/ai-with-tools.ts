// 支持 Tool Calling 的 AI 服务
// 基于现有 ai-service.ts 扩展

import { getApiKey } from '../api';
import type { AIProvider } from '../types';
import { AI_PROVIDERS } from '../types';
import { parseHttpError } from '../error-utils';
import type { Tool, ToolCall, AgentMessage } from './types';
import { toOpenAITool, toGeminiTool } from './Tool';

// API 端点配置
const API_ENDPOINTS: Record<AIProvider, string> = {
  deepseek: 'https://api.deepseek.com/chat/completions',
  openai: 'https://api.openai.com/v1/chat/completions',
  gemini: 'https://generativelanguage.googleapis.com/v1beta/models',
  qwen: 'https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions',
};

// ==================== 类型定义 ====================

export interface ChatWithToolsOptions {
  provider: AIProvider;
  model?: string;
  temperature?: number;
  maxTokens?: number;
  signal?: AbortSignal;
}

export interface ChatWithToolsResponse {
  content: string | null;
  toolCalls: ToolCall[] | null;
  finishReason: 'stop' | 'tool_calls' | 'length' | 'error';
}

// ==================== 工具函数 ====================

async function getProviderApiKey(provider: AIProvider): Promise<string> {
  const config = AI_PROVIDERS[provider];
  const apiKey = await getApiKey(config.apiKeyName);
  if (!apiKey) {
    throw new Error(`请先在设置中配置 ${config.name} API Key`);
  }
  return apiKey;
}

// 将 AgentMessage 转换为 OpenAI 格式
function toOpenAIMessages(messages: AgentMessage[]): any[] {
  return messages.map(msg => {
    if (typeof msg.content === 'string') {
      return { role: msg.role, content: msg.content };
    }

    // 处理复杂内容（tool_use, tool_result 等）
    if (Array.isArray(msg.content)) {
      // Assistant 消息带 tool_calls
      if (msg.role === 'assistant') {
        const toolUses = msg.content.filter(c => c.type === 'tool_use');
        const textContent = msg.content.find(c => c.type === 'text');

        if (toolUses.length > 0) {
          return {
            role: 'assistant',
            content: textContent ? (textContent as any).text : null,
            tool_calls: toolUses.map(tu => ({
              id: (tu as any).id,
              type: 'function',
              function: {
                name: (tu as any).name,
                arguments: JSON.stringify((tu as any).input),
              },
            })),
          };
        }
      }

      // Tool 结果消息
      if (msg.role === 'tool') {
        const toolResult = msg.content.find(c => c.type === 'tool_result');
        if (toolResult) {
          return {
            role: 'tool',
            tool_call_id: (toolResult as any).tool_use_id,
            content: (toolResult as any).content,
          };
        }
      }
    }

    return { role: msg.role, content: msg.content };
  });
}

// 将 AgentMessage 转换为 Gemini 格式
function toGeminiMessages(messages: AgentMessage[]): { contents: any[]; systemInstruction?: any } {
  const systemMessage = messages.find(m => m.role === 'system');
  const chatMessages = messages.filter(m => m.role !== 'system');

  const contents = chatMessages.map(msg => {
    const role = msg.role === 'assistant' ? 'model' : msg.role === 'tool' ? 'function' : 'user';

    if (typeof msg.content === 'string') {
      return {
        role,
        parts: [{ text: msg.content }],
      };
    }

    // 处理复杂内容
    if (Array.isArray(msg.content)) {
      const parts: any[] = [];

      for (const c of msg.content) {
        if (c.type === 'text') {
          parts.push({ text: (c as any).text });
        } else if (c.type === 'tool_use') {
          parts.push({
            functionCall: {
              name: (c as any).name,
              args: (c as any).input,
            },
          });
        } else if (c.type === 'tool_result') {
          parts.push({
            functionResponse: {
              name: (c as any).tool_use_id.split('_')[0], // 从 ID 提取 tool name
              response: { result: (c as any).content },
            },
          });
        }
      }

      return { role, parts };
    }

    return { role, parts: [{ text: String(msg.content) }] };
  });

  const result: { contents: any[]; systemInstruction?: any } = { contents };

  if (systemMessage) {
    result.systemInstruction = {
      parts: [{ text: typeof systemMessage.content === 'string' ? systemMessage.content : '' }],
    };
  }

  return result;
}

// ==================== OpenAI 兼容接口（带 Tools）====================

async function chatWithToolsOpenAI(
  messages: AgentMessage[],
  tools: Tool[],
  options: ChatWithToolsOptions
): Promise<ChatWithToolsResponse> {
  const { provider, model, temperature = 0.7, maxTokens = 8000, signal } = options;
  const apiKey = await getProviderApiKey(provider);
  const endpoint = API_ENDPOINTS[provider];
  const actualModel = model || AI_PROVIDERS[provider].defaultModel;

  const openAIMessages = toOpenAIMessages(messages);
  const openAITools = tools.map(toOpenAITool);

  const requestBody: any = {
    model: actualModel,
    messages: openAIMessages,
    temperature,
    max_tokens: maxTokens,
  };

  // 只有在有 tools 时才添加
  if (openAITools.length > 0) {
    requestBody.tools = openAITools;
    requestBody.tool_choice = 'auto';
  }

  const response = await fetch(endpoint, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${apiKey}`,
    },
    body: JSON.stringify(requestBody),
    signal,
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(parseHttpError(response.status, errorText));
  }

  const data = await response.json();
  const choice = data.choices[0];
  const message = choice.message;

  // 解析 tool_calls
  let toolCalls: ToolCall[] | null = null;
  if (message.tool_calls && message.tool_calls.length > 0) {
    toolCalls = message.tool_calls.map((tc: any) => {
      let args = {};
      try {
        args = JSON.parse(tc.function.arguments || '{}');
      } catch (e) {
        console.error('Failed to parse tool call arguments:', tc.function.arguments, e);
        // 尝试修复常见的 JSON 截断问题
        const argStr = tc.function.arguments || '';
        if (argStr && !argStr.endsWith('}')) {
          try {
            args = JSON.parse(argStr + '"}');
          } catch {
            try {
              args = JSON.parse(argStr + '}');
            } catch {
              // 最后尝试：如果还是失败，使用空对象
              args = {};
            }
          }
        }
      }
      return {
        id: tc.id,
        name: tc.function.name,
        arguments: args,
      };
    });
  }

  // 确定 finish_reason
  let finishReason: ChatWithToolsResponse['finishReason'] = 'stop';
  if (choice.finish_reason === 'tool_calls') {
    finishReason = 'tool_calls';
  } else if (choice.finish_reason === 'length') {
    finishReason = 'length';
  }

  return {
    content: message.content,
    toolCalls,
    finishReason,
  };
}

// ==================== Gemini 接口（带 Tools）====================

async function chatWithToolsGemini(
  messages: AgentMessage[],
  tools: Tool[],
  options: ChatWithToolsOptions
): Promise<ChatWithToolsResponse> {
  const { model, temperature = 0.7, maxTokens = 8000, signal } = options;
  const apiKey = await getProviderApiKey('gemini');
  const actualModel = model || AI_PROVIDERS.gemini.defaultModel;
  const endpoint = `${API_ENDPOINTS.gemini}/${actualModel}:generateContent?key=${apiKey}`;

  const { contents, systemInstruction } = toGeminiMessages(messages);

  const requestBody: any = {
    contents,
    generationConfig: {
      temperature,
      maxOutputTokens: maxTokens,
    },
  };

  if (systemInstruction) {
    requestBody.systemInstruction = systemInstruction;
  }

  // 添加 tools
  if (tools.length > 0) {
    requestBody.tools = [
      {
        functionDeclarations: tools.map(toGeminiTool),
      },
    ];
  }

  const response = await fetch(endpoint, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(requestBody),
    signal,
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(parseHttpError(response.status, errorText, 'Gemini'));
  }

  const data = await response.json();
  const candidate = data.candidates?.[0];
  const content = candidate?.content;

  // 解析响应
  let textContent: string | null = null;
  let toolCalls: ToolCall[] | null = null;

  if (content?.parts) {
    for (const part of content.parts) {
      if (part.text) {
        textContent = (textContent || '') + part.text;
      }
      if (part.functionCall) {
        if (!toolCalls) toolCalls = [];
        toolCalls.push({
          id: `${part.functionCall.name}_${Date.now()}`,
          name: part.functionCall.name,
          arguments: part.functionCall.args || {},
        });
      }
    }
  }

  // 确定 finish_reason
  let finishReason: ChatWithToolsResponse['finishReason'] = 'stop';
  if (toolCalls && toolCalls.length > 0) {
    finishReason = 'tool_calls';
  } else if (candidate?.finishReason === 'MAX_TOKENS') {
    finishReason = 'length';
  }

  return {
    content: textContent,
    toolCalls,
    finishReason,
  };
}

// ==================== 统一接口 ====================

/**
 * 带 Tool Calling 的聊天接口
 */
export async function chatWithTools(
  messages: AgentMessage[],
  tools: Tool[],
  options: ChatWithToolsOptions
): Promise<ChatWithToolsResponse> {
  const { provider } = options;

  if (provider === 'gemini') {
    return chatWithToolsGemini(messages, tools, options);
  } else {
    // OpenAI 兼容：deepseek, openai, qwen
    return chatWithToolsOpenAI(messages, tools, options);
  }
}

/**
 * 检查 Provider 是否支持 Tool Calling
 */
export function supportsToolCalling(provider: AIProvider): boolean {
  // 目前所有主流 provider 都支持
  return ['openai', 'deepseek', 'gemini', 'qwen'].includes(provider);
}

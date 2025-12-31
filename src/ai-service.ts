// 多 AI 服务抽象层
// 支持 DeepSeek, OpenAI, Claude, Gemini

import { getApiKey } from "./api";
import type { AIProvider, KbSearchResult } from "./types";
import { AI_PROVIDERS } from "./types";

// ==================== API 配置 ====================

const API_ENDPOINTS: Record<AIProvider, string> = {
  deepseek: "https://api.deepseek.com/chat/completions",
  openai: "https://api.openai.com/v1/chat/completions",
  claude: "https://api.anthropic.com/v1/messages",
  gemini: "https://generativelanguage.googleapis.com/v1beta/models",
};

// ==================== 消息类型 ====================

export interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

export interface ChatOptions {
  provider: AIProvider;
  model?: string;
  temperature?: number;
  maxTokens?: number;
  signal?: AbortSignal;
}

export interface StreamChunk {
  content: string;
  done: boolean;
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

// 带超时的 fetch
async function fetchWithTimeout(
  url: string,
  options: RequestInit,
  timeoutMs: number = 60000
): Promise<Response> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), timeoutMs);

  const originalSignal = options.signal;
  if (originalSignal) {
    originalSignal.addEventListener("abort", () => controller.abort());
  }

  try {
    const response = await fetch(url, {
      ...options,
      signal: controller.signal,
    });
    return response;
  } finally {
    clearTimeout(timeoutId);
  }
}

// ==================== 通用聊天接口 ====================

export async function chat(
  messages: ChatMessage[],
  options: ChatOptions
): Promise<string> {
  const { provider, model, temperature = 0.7, maxTokens = 2000, signal } = options;
  const apiKey = await getProviderApiKey(provider);
  const endpoint = API_ENDPOINTS[provider];
  const actualModel = model || AI_PROVIDERS[provider].defaultModel;

  if (provider === 'claude') {
    return chatClaude(messages, apiKey, actualModel, temperature, maxTokens, signal);
  } else if (provider === 'gemini') {
    return chatGemini(messages, apiKey, actualModel, temperature, maxTokens, signal);
  } else {
    return chatOpenAICompatible(messages, apiKey, endpoint, actualModel, temperature, maxTokens, signal);
  }
}

// OpenAI 兼容接口（DeepSeek 和 OpenAI）
async function chatOpenAICompatible(
  messages: ChatMessage[],
  apiKey: string,
  endpoint: string,
  model: string,
  temperature: number,
  maxTokens: number,
  signal?: AbortSignal
): Promise<string> {
  const response = await fetchWithTimeout(
    endpoint,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${apiKey}`,
      },
      body: JSON.stringify({
        model,
        messages,
        temperature,
        max_tokens: maxTokens,
      }),
      signal,
    },
    120000
  );

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`API 错误: ${error}`);
  }

  const data = await response.json();
  return data.choices[0]?.message?.content || "";
}

// Claude 接口
async function chatClaude(
  messages: ChatMessage[],
  apiKey: string,
  model: string,
  temperature: number,
  maxTokens: number,
  signal?: AbortSignal
): Promise<string> {
  // Claude 需要单独处理 system 消息
  const systemMessage = messages.find(m => m.role === 'system');
  const chatMessages = messages.filter(m => m.role !== 'system').map(m => ({
    role: m.role as 'user' | 'assistant',
    content: m.content,
  }));

  const response = await fetchWithTimeout(
    API_ENDPOINTS.claude,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "x-api-key": apiKey,
        "anthropic-version": "2023-06-01",
      },
      body: JSON.stringify({
        model,
        max_tokens: maxTokens,
        system: systemMessage?.content,
        messages: chatMessages,
        temperature,
      }),
      signal,
    },
    120000
  );

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Claude API 错误: ${error}`);
  }

  const data = await response.json();
  return data.content[0]?.text || "";
}

// Gemini 接口
async function chatGemini(
  messages: ChatMessage[],
  apiKey: string,
  model: string,
  temperature: number,
  maxTokens: number,
  signal?: AbortSignal
): Promise<string> {
  // Gemini 使用不同的消息格式
  const systemMessage = messages.find(m => m.role === 'system');
  const chatMessages = messages.filter(m => m.role !== 'system').map(m => ({
    role: m.role === 'assistant' ? 'model' : 'user',
    parts: [{ text: m.content }],
  }));

  const endpoint = `${API_ENDPOINTS.gemini}/${model}:generateContent?key=${apiKey}`;

  const requestBody: any = {
    contents: chatMessages,
    generationConfig: {
      temperature,
      maxOutputTokens: maxTokens,
    },
  };

  // 添加系统指令
  if (systemMessage) {
    requestBody.systemInstruction = {
      parts: [{ text: systemMessage.content }],
    };
  }

  const response = await fetchWithTimeout(
    endpoint,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(requestBody),
      signal,
    },
    120000
  );

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Gemini API 错误: ${error}`);
  }

  const data = await response.json();
  return data.candidates?.[0]?.content?.parts?.[0]?.text || "";
}

// ==================== 流式聊天接口 ====================

export async function* chatStream(
  messages: ChatMessage[],
  options: ChatOptions
): AsyncGenerator<StreamChunk> {
  const { provider, model, temperature = 0.7, maxTokens = 2000, signal } = options;
  const apiKey = await getProviderApiKey(provider);
  const endpoint = API_ENDPOINTS[provider];
  const actualModel = model || AI_PROVIDERS[provider].defaultModel;

  if (provider === 'claude') {
    yield* chatStreamClaude(messages, apiKey, actualModel, temperature, maxTokens, signal);
  } else if (provider === 'gemini') {
    yield* chatStreamGemini(messages, apiKey, actualModel, temperature, maxTokens, signal);
  } else {
    yield* chatStreamOpenAICompatible(messages, apiKey, endpoint, actualModel, temperature, maxTokens, signal);
  }
}

// OpenAI 兼容流式接口
async function* chatStreamOpenAICompatible(
  messages: ChatMessage[],
  apiKey: string,
  endpoint: string,
  model: string,
  temperature: number,
  maxTokens: number,
  signal?: AbortSignal
): AsyncGenerator<StreamChunk> {
  const response = await fetch(endpoint, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${apiKey}`,
    },
    body: JSON.stringify({
      model,
      messages,
      temperature,
      max_tokens: maxTokens,
      stream: true,
    }),
    signal,
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`API 错误: ${error}`);
  }

  const reader = response.body?.getReader();
  if (!reader) {
    throw new Error("无法读取响应流");
  }

  const decoder = new TextDecoder();
  let buffer = "";

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split("\n");
    buffer = lines.pop() || "";

    for (const line of lines) {
      if (line.startsWith("data: ")) {
        const data = line.slice(6);
        if (data === "[DONE]") {
          yield { content: "", done: true };
          return;
        }
        try {
          const json = JSON.parse(data);
          const content = json.choices[0]?.delta?.content || "";
          if (content) {
            yield { content, done: false };
          }
        } catch {
          // 忽略解析错误
        }
      }
    }
  }

  yield { content: "", done: true };
}

// Claude 流式接口
async function* chatStreamClaude(
  messages: ChatMessage[],
  apiKey: string,
  model: string,
  temperature: number,
  maxTokens: number,
  signal?: AbortSignal
): AsyncGenerator<StreamChunk> {
  const systemMessage = messages.find(m => m.role === 'system');
  const chatMessages = messages.filter(m => m.role !== 'system').map(m => ({
    role: m.role as 'user' | 'assistant',
    content: m.content,
  }));

  const response = await fetch(API_ENDPOINTS.claude, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "x-api-key": apiKey,
      "anthropic-version": "2023-06-01",
    },
    body: JSON.stringify({
      model,
      max_tokens: maxTokens,
      system: systemMessage?.content,
      messages: chatMessages,
      temperature,
      stream: true,
    }),
    signal,
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Claude API 错误: ${error}`);
  }

  const reader = response.body?.getReader();
  if (!reader) {
    throw new Error("无法读取响应流");
  }

  const decoder = new TextDecoder();
  let buffer = "";

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split("\n");
    buffer = lines.pop() || "";

    for (const line of lines) {
      if (line.startsWith("data: ")) {
        const data = line.slice(6);
        try {
          const json = JSON.parse(data);
          if (json.type === "content_block_delta" && json.delta?.text) {
            yield { content: json.delta.text, done: false };
          } else if (json.type === "message_stop") {
            yield { content: "", done: true };
            return;
          }
        } catch {
          // 忽略解析错误
        }
      }
    }
  }

  yield { content: "", done: true };
}

// Gemini 流式接口
async function* chatStreamGemini(
  messages: ChatMessage[],
  apiKey: string,
  model: string,
  temperature: number,
  maxTokens: number,
  signal?: AbortSignal
): AsyncGenerator<StreamChunk> {
  const systemMessage = messages.find(m => m.role === 'system');
  const chatMessages = messages.filter(m => m.role !== 'system').map(m => ({
    role: m.role === 'assistant' ? 'model' : 'user',
    parts: [{ text: m.content }],
  }));

  const endpoint = `${API_ENDPOINTS.gemini}/${model}:streamGenerateContent?key=${apiKey}&alt=sse`;

  const requestBody: any = {
    contents: chatMessages,
    generationConfig: {
      temperature,
      maxOutputTokens: maxTokens,
    },
  };

  if (systemMessage) {
    requestBody.systemInstruction = {
      parts: [{ text: systemMessage.content }],
    };
  }

  const response = await fetch(endpoint, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(requestBody),
    signal,
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Gemini API 错误: ${error}`);
  }

  const reader = response.body?.getReader();
  if (!reader) {
    throw new Error("无法读取响应流");
  }

  const decoder = new TextDecoder();
  let buffer = "";

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split("\n");
    buffer = lines.pop() || "";

    for (const line of lines) {
      if (line.startsWith("data: ")) {
        const data = line.slice(6);
        try {
          const json = JSON.parse(data);
          const text = json.candidates?.[0]?.content?.parts?.[0]?.text || "";
          if (text) {
            yield { content: text, done: false };
          }
          // 检查是否完成
          if (json.candidates?.[0]?.finishReason) {
            yield { content: "", done: true };
            return;
          }
        } catch {
          // 忽略解析错误
        }
      }
    }
  }

  yield { content: "", done: true };
}

// ==================== RAG 相关 ====================

// 构建 RAG 系统提示词
export function buildRAGSystemPrompt(sources: KbSearchResult[]): string {
  if (sources.length === 0) {
    return `你是一个企业知识库助手。当前知识库中没有找到相关信息，请根据你的知识尽力回答用户的问题。如果不确定，请明确告知。`;
  }

  const sourceTexts = sources.map((source, index) => {
    const pageInfo = source.page_number ? ` (第${source.page_number}页)` : "";
    return `[来源${index + 1}: ${source.document_title}${pageInfo}]\n${source.content}`;
  }).join("\n\n---\n\n");

  return `你是一个企业知识库助手。请根据以下参考资料回答用户问题。

回答要求：
1. 优先使用参考资料中的信息回答问题
2. 如果资料中没有相关信息，可以结合你的知识回答，但请明确告知
3. 回答时请标注信息来源，如 [来源1]、[来源2]
4. 保持回答简洁、专业

参考资料：
${sourceTexts}`;
}

// 解析回答中的来源引用
export function parseSourceReferences(
  answer: string,
  sources: KbSearchResult[]
): { document_id: number; document_title: string; snippet: string }[] {
  const refs: { document_id: number; document_title: string; snippet: string }[] = [];
  const pattern = /\[来源(\d+)\]/g;
  const matches = answer.matchAll(pattern);
  const usedIndices = new Set<number>();

  for (const match of matches) {
    const index = parseInt(match[1]) - 1;
    if (index >= 0 && index < sources.length && !usedIndices.has(index)) {
      usedIndices.add(index);
      const source = sources[index];
      refs.push({
        document_id: source.document_id,
        document_title: source.document_title,
        snippet: source.content.substring(0, 100) + "...",
      });
    }
  }

  return refs;
}

// 检查 API Key 是否已配置
export async function checkApiKeyConfigured(provider: AIProvider): Promise<boolean> {
  const config = AI_PROVIDERS[provider];
  const apiKey = await getApiKey(config.apiKeyName);
  return !!apiKey;
}

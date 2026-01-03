// 多 AI 服务抽象层
// 支持 DeepSeek, OpenAI, Gemini, 通义千问

import { getApiKey } from "./api";
import type { AIProvider, KbSearchResult } from "./types";
import { AI_PROVIDERS } from "./types";
import { parseHttpError, parseError } from "./error-utils";

// ==================== API 配置 ====================

const API_ENDPOINTS: Record<AIProvider, string> = {
  deepseek: "https://api.deepseek.com/chat/completions",
  openai: "https://api.openai.com/v1/chat/completions",
  gemini: "https://generativelanguage.googleapis.com/v1beta/models",
  qwen: "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions",
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

  if (provider === 'gemini') {
    return chatGemini(messages, apiKey, actualModel, temperature, maxTokens, signal);
  } else {
    // OpenAI 兼容格式: deepseek, openai, qwen
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
    const errorText = await response.text();
    throw new Error(parseHttpError(response.status, errorText));
  }

  const data = await response.json();
  return data.choices[0]?.message?.content || "";
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
    const errorText = await response.text();
    throw new Error(parseHttpError(response.status, errorText, 'Gemini'));
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

  if (provider === 'gemini') {
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
    const errorText = await response.text();
    throw new Error(parseHttpError(response.status, errorText));
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
    const errorText = await response.text();
    throw new Error(parseHttpError(response.status, errorText, 'Gemini'));
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

// 格式化来源文本（供多个模式使用）
function formatSourceTexts(sources: KbSearchResult[]): string {
  return sources.map((source, index) => {
    const pageInfo = source.page_number ? ` (第${source.page_number}页)` : "";
    return `[来源${index + 1}: ${source.document_title}${pageInfo}]\n${source.content}`;
  }).join("\n\n---\n\n");
}

// 严格模式：只基于知识库回答，不添加 AI 分析
export function buildStrictModePrompt(sources: KbSearchResult[]): string {
  if (sources.length === 0) {
    return `你是一个企业知识库助手。当前知识库中没有找到相关信息。

请告知用户"知识库中未找到相关内容"，不要自行编造答案。`;
  }

  const sourceTexts = formatSourceTexts(sources);
  return `你是一个企业知识库助手。请严格基于以下参考资料回答用户问题。

回答要求：
1. 只能使用参考资料中的信息回答
2. 如果资料中没有相关信息，明确告知"知识库中未找到相关内容"
3. 回答时标注信息来源，如 [来源1]、[来源2]
4. 不要添加任何资料之外的内容或个人分析

参考资料：
${sourceTexts}

【重要】当前为严格模式，请忽略对话历史中可能存在的分析性回答风格，严格只使用上述参考资料回答。`;
}

// 分析模式：知识库 + AI 分析（原 buildRAGSystemPrompt）
export function buildAnalysisModePrompt(sources: KbSearchResult[]): string {
  if (sources.length === 0) {
    return `你是一个专业的企业知识库助手，同时也是一个善于思考和分析的顾问。

当前知识库中没有找到与问题直接相关的信息。请根据你的专业知识回答用户的问题：
1. 如果你有把握，可以直接回答并提供有价值的分析和建议
2. 如果不确定，请明确告知，并说明可能需要补充哪些资料到知识库
3. 始终保持专业、务实的态度`;
  }

  const sourceTexts = formatSourceTexts(sources);
  return `你是一个专业的企业知识库助手，同时也是一个善于思考和分析的顾问。

你的回答方式：
1. **知识库内容**：首先基于参考资料中的信息回答问题，引用时标注 [来源1]、[来源2] 等
2. **延伸分析**：在知识库内容的基础上，结合你的专业知识进行补充分析、提供建议或延伸思考
3. **清晰区分**：当内容来自知识库时标注来源；当是你的分析或建议时，可以用"根据我的分析"、"此外建议"等方式说明
4. **务实有用**：不要机械地复述资料，而是理解用户真正的需求，给出有价值的回答

回答格式建议：
- 先回答用户的具体问题（基于知识库）
- 再提供你的分析或补充见解（基于你的思考）
- 如有必要，给出可行的建议或下一步行动

参考资料：
${sourceTexts}

【当前模式】分析模式 - 请结合知识库内容与你的专业分析回答。`;
}

// 直接对话模式：纯 AI 对话，不检索知识库
export function buildDirectChatPrompt(): string {
  return `你是一个专业的 AI 助手，可以回答各种问题、提供建议和帮助分析。

请直接与用户对话，不需要引用任何文档来源。根据你的知识和理解，尽力提供有价值的回答。

【当前模式】对话模式 - 直接对话，无需引用知识库内容。`;
}

// 兼容旧代码：保留原函数名
export const buildRAGSystemPrompt = buildAnalysisModePrompt;

// 解析回答中的来源引用
export function parseSourceReferences(
  answer: string,
  sources: KbSearchResult[]
): { document_id: number; document_title: string; snippet: string; image_path?: string | null }[] {
  const refs: { document_id: number; document_title: string; snippet: string; image_path?: string | null }[] = [];
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
        image_path: source.image_path,  // 添加图片路径用于图文问答
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

// ==================== 图片识别 ====================

export interface ImageRecognitionResult {
  success: boolean;
  description: string;
  error?: string;
}

/**
 * 使用 Gemini Vision API 识别图片内容
 * @param base64Data Base64 编码的图片数据
 * @param mimeType 图片 MIME 类型 (如 "image/png", "image/jpeg")
 * @param prompt 识别提示词
 * @param retryCount 重试次数
 * @returns 图片内容描述
 */
export async function recognizeImageWithGemini(
  base64Data: string,
  mimeType: string,
  prompt: string = "请详细描述这张图片中的所有内容，包括文字、数字、表格、图表等。如果图片中有文字，请完整提取出来。",
  retryCount: number = 0
): Promise<ImageRecognitionResult> {
  const MAX_RETRIES = 3;

  try {
    const apiKey = await getApiKey("gemini");
    if (!apiKey) {
      return {
        success: false,
        description: "",
        error: "Gemini API Key 未配置",
      };
    }

    // 使用 Gemini 2.5 Flash 模型（支持视觉）
    const model = "gemini-2.5-flash";
    const endpoint = `${API_ENDPOINTS.gemini}/${model}:generateContent?key=${apiKey}`;

    const requestBody = {
      contents: [
        {
          parts: [
            {
              inline_data: {
                mime_type: mimeType,
                data: base64Data,
              },
            },
            {
              text: prompt,
            },
          ],
        },
      ],
      generationConfig: {
        temperature: 0.2,
        maxOutputTokens: 4096,
      },
    };

    const response = await fetch(endpoint, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(requestBody),
    });

    // 处理速率限制 (429)
    if (response.status === 429 && retryCount < MAX_RETRIES) {
      const retryAfter = 25; // 默认等待 25 秒
      console.log(`Gemini API 速率限制，${retryAfter} 秒后重试 (${retryCount + 1}/${MAX_RETRIES})...`);
      await sleep(retryAfter * 1000);
      return recognizeImageWithGemini(base64Data, mimeType, prompt, retryCount + 1);
    }

    if (!response.ok) {
      const errorText = await response.text();
      console.error("Gemini Vision API 错误:", errorText);
      return {
        success: false,
        description: "",
        error: parseHttpError(response.status, errorText, 'Gemini'),
      };
    }

    const data = await response.json();
    const text = data.candidates?.[0]?.content?.parts?.[0]?.text || "";

    if (!text) {
      return {
        success: false,
        description: "",
        error: "未能识别图片内容",
      };
    }

    return {
      success: true,
      description: text,
    };
  } catch (error) {
    console.error("图片识别失败:", error);
    return {
      success: false,
      description: "",
      error: parseError(error, 'Gemini 图片识别'),
    };
  }
}

/**
 * 使用通义千问视觉模型识别图片内容
 * @param base64Data 图片的 base64 数据
 * @param mimeType 图片 MIME 类型 (如 image/png, image/jpeg)
 * @param prompt 识别提示词
 * @param retryCount 重试次数
 */
export async function recognizeImageWithQwen(
  base64Data: string,
  mimeType: string,
  prompt: string = "请完整提取图片中的所有文字内容，并保留表格和公式的结构。如果有表格请用 Markdown 表格格式输出，如果有公式请用 LaTeX 格式。",
  retryCount: number = 0
): Promise<ImageRecognitionResult> {
  const MAX_RETRIES = 3;

  try {
    const apiKey = await getApiKey("qwen");
    if (!apiKey) {
      return {
        success: false,
        description: "",
        error: "通义千问 API Key 未配置",
      };
    }

    // 使用通义千问视觉模型 (qwen-vl-max 基于 Qwen3-VL-32B，OCR 能力更强)
    const model = "qwen-vl-max";
    const endpoint = API_ENDPOINTS.qwen;

    // 构建 base64 URL
    const imageUrl = `data:${mimeType};base64,${base64Data}`;

    const requestBody = {
      model: model,
      messages: [
        {
          role: "user",
          content: [
            {
              type: "image_url",
              image_url: {
                url: imageUrl,
              },
            },
            {
              type: "text",
              text: prompt,
            },
          ],
        },
      ],
      max_tokens: 4096,
    };

    const response = await fetch(endpoint, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Authorization": `Bearer ${apiKey}`,
      },
      body: JSON.stringify(requestBody),
    });

    // 处理速率限制 (429)
    if (response.status === 429 && retryCount < MAX_RETRIES) {
      const retryAfter = 10; // 默认等待 10 秒
      console.log(`通义千问 API 速率限制，${retryAfter} 秒后重试 (${retryCount + 1}/${MAX_RETRIES})...`);
      await sleep(retryAfter * 1000);
      return recognizeImageWithQwen(base64Data, mimeType, prompt, retryCount + 1);
    }

    if (!response.ok) {
      const errorText = await response.text();
      console.error("通义千问 Vision API 错误:", errorText);
      return {
        success: false,
        description: "",
        error: parseHttpError(response.status, errorText, '通义千问'),
      };
    }

    const data = await response.json();
    const text = data.choices?.[0]?.message?.content || "";

    if (!text) {
      return {
        success: false,
        description: "",
        error: "未能识别图片内容",
      };
    }

    return {
      success: true,
      description: text,
    };
  } catch (error) {
    console.error("通义千问图片识别失败:", error);
    return {
      success: false,
      description: "",
      error: parseError(error, '通义千问图片识别'),
    };
  }
}

/**
 * 统一图片识别接口 - 自动选择可用的 AI 服务
 * 优先使用通义千问，失败则尝试 Gemini
 */
export async function recognizeImage(
  base64Data: string,
  mimeType: string,
  prompt?: string
): Promise<ImageRecognitionResult> {
  // 先尝试通义千问
  const hasQwenKey = await checkApiKeyConfigured('qwen');
  if (hasQwenKey) {
    console.log('[图片识别] 使用通义千问...');
    const result = await recognizeImageWithQwen(base64Data, mimeType, prompt);
    if (result.success) {
      return result;
    }
    console.log('[图片识别] 通义千问失败，尝试 Gemini...', result.error);
  }

  // 再尝试 Gemini
  const hasGeminiKey = await checkApiKeyConfigured('gemini');
  if (hasGeminiKey) {
    console.log('[图片识别] 使用 Gemini...');
    return recognizeImageWithGemini(base64Data, mimeType, prompt);
  }

  return {
    success: false,
    description: "",
    error: "请先配置通义千问或 Gemini API Key",
  };
}

// 辅助函数：延迟
function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

// ==================== Embedding 向量化 ====================

export interface EmbeddingResult {
  success: boolean;
  embedding: number[];
  error?: string;
}

/**
 * 使用 DeepSeek Embedding API 获取文本向量
 */
async function getEmbeddingFromDeepSeek(
  text: string,
  apiKey: string,
  retryCount: number = 0
): Promise<EmbeddingResult> {
  const MAX_RETRIES = 3;

  try {
    const response = await fetch("https://api.deepseek.com/v1/embeddings", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Authorization": `Bearer ${apiKey}`,
      },
      body: JSON.stringify({
        model: "deepseek-embedding",
        input: text,
      }),
    });

    // 处理速率限制 (429)
    if (response.status === 429 && retryCount < MAX_RETRIES) {
      const retryAfter = 10;
      console.log(`DeepSeek Embedding API 速率限制，${retryAfter} 秒后重试 (${retryCount + 1}/${MAX_RETRIES})...`);
      await sleep(retryAfter * 1000);
      return getEmbeddingFromDeepSeek(text, apiKey, retryCount + 1);
    }

    if (!response.ok) {
      const errorText = await response.text();
      console.error("DeepSeek Embedding API 错误:", errorText);
      return {
        success: false,
        embedding: [],
        error: parseHttpError(response.status, errorText, 'DeepSeek'),
      };
    }

    const data = await response.json();
    const embedding = data.data?.[0]?.embedding || [];

    if (embedding.length === 0) {
      return {
        success: false,
        embedding: [],
        error: "DeepSeek 未能获取向量",
      };
    }

    return {
      success: true,
      embedding,
    };
  } catch (error) {
    console.error("DeepSeek 获取向量失败:", error);
    return {
      success: false,
      embedding: [],
      error: parseError(error, 'DeepSeek 向量化'),
    };
  }
}

/**
 * 使用千问 Embedding API 获取文本向量
 */
async function getEmbeddingFromQwen(
  text: string,
  apiKey: string,
  retryCount: number = 0
): Promise<EmbeddingResult> {
  const MAX_RETRIES = 3;

  try {
    const response = await fetch("https://dashscope.aliyuncs.com/compatible-mode/v1/embeddings", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Authorization": `Bearer ${apiKey}`,
      },
      body: JSON.stringify({
        model: "text-embedding-v4",
        input: text,
      }),
    });

    // 处理速率限制 (429)
    if (response.status === 429 && retryCount < MAX_RETRIES) {
      const retryAfter = 10;
      console.log(`千问 Embedding API 速率限制，${retryAfter} 秒后重试 (${retryCount + 1}/${MAX_RETRIES})...`);
      await sleep(retryAfter * 1000);
      return getEmbeddingFromQwen(text, apiKey, retryCount + 1);
    }

    if (!response.ok) {
      const errorText = await response.text();
      console.error("千问 Embedding API 错误:", errorText);
      return {
        success: false,
        embedding: [],
        error: parseHttpError(response.status, errorText, '通义千问'),
      };
    }

    const data = await response.json();
    const embedding = data.data?.[0]?.embedding || [];

    if (embedding.length === 0) {
      return {
        success: false,
        embedding: [],
        error: "千问未能获取向量",
      };
    }

    return {
      success: true,
      embedding,
    };
  } catch (error) {
    console.error("千问获取向量失败:", error);
    return {
      success: false,
      embedding: [],
      error: parseError(error, '通义千问向量化'),
    };
  }
}

/**
 * 获取文本向量（优先使用千问，DeepSeek embedding 暂不可用）
 * @param text 要向量化的文本
 * @param retryCount 重试次数
 * @returns 向量数组
 */
export async function getTextEmbedding(
  text: string,
  retryCount: number = 0
): Promise<EmbeddingResult> {
  // 优先使用千问（DeepSeek embedding API 目前不可用）
  const qwenKey = await getApiKey("qwen");
  if (qwenKey) {
    console.log("[Embedding] 使用千问 text-embedding-v4");
    return getEmbeddingFromQwen(text, qwenKey, retryCount);
  }

  // 备选：尝试 DeepSeek（如果未来支持的话）
  const deepseekKey = await getApiKey("deepseek");
  if (deepseekKey) {
    console.log("[Embedding] 尝试 DeepSeek");
    const result = await getEmbeddingFromDeepSeek(text, deepseekKey, retryCount);
    if (result.success) {
      return result;
    }
    console.log("[Embedding] DeepSeek 失败，无其他备选");
  }

  return {
    success: false,
    embedding: [],
    error: "未配置千问 API Key（用于 Embedding）",
  };
}

/**
 * 批量获取文本向量（带延迟避免速率限制）- 串行版本（已弃用）
 */
export async function getTextEmbeddingsBatch(
  texts: { id: number; content: string }[],
  onProgress?: (current: number, total: number) => void
): Promise<Map<number, number[]>> {
  const results = new Map<number, number[]>();

  for (let i = 0; i < texts.length; i++) {
    const { id, content } = texts[i];

    // 报告进度
    if (onProgress) {
      onProgress(i + 1, texts.length);
    }

    // 获取向量
    const result = await getTextEmbedding(content);
    if (result.success) {
      results.set(id, result.embedding);
    } else {
      console.warn(`Chunk ${id} 向量化失败:`, result.error);
    }

    // 添加延迟避免速率限制
    if (i < texts.length - 1) {
      await sleep(500); // 500ms 延迟
    }
  }

  return results;
}

/**
 * 批量获取文本向量 - 并行版本（推荐使用）
 * 通过并发请求大幅提升处理速度
 */
export async function getTextEmbeddingsBatchParallel(
  texts: { id: number; content: string }[],
  onProgress?: (current: number, total: number) => void,
  concurrency: number = 5  // 并发数
): Promise<Map<number, number[]>> {
  const results = new Map<number, number[]>();
  let completed = 0;

  // 分批处理
  for (let i = 0; i < texts.length; i += concurrency) {
    const batch = texts.slice(i, i + concurrency);

    // 并行请求
    const promises = batch.map(async ({ id, content }) => {
      const result = await getTextEmbedding(content);
      if (result.success) {
        results.set(id, result.embedding);
      } else {
        console.warn(`Chunk ${id} 向量化失败:`, result.error);
      }
      completed++;
      onProgress?.(completed, texts.length);
    });

    await Promise.all(promises);

    // 批次间延迟（避免限流）
    if (i + concurrency < texts.length) {
      await sleep(200);
    }
  }

  return results;
}

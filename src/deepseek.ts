import { getApiKey } from "./api";
import { parseHttpError } from "./error-utils";

// DeepSeek API 配置
const DEEPSEEK_API_URL = "https://api.deepseek.com/chat/completions";

// 获取 API Key（从系统密钥链）
async function getDeepSeekApiKey(): Promise<string> {
  const apiKey = await getApiKey("deepseek");
  if (!apiKey) {
    throw new Error("请先在设置中配置 DeepSeek API Key");
  }
  return apiKey;
}

// 分类列表
const CATEGORIES = [
  "品类词",
  "品牌",
  "颜色",
  "形状",
  "功能",
  "适用人群",
  "材质",
  "尺寸",
  "使用场景",
  "情绪价值",
  "使用地点",
  "节假日",
  "适配",
  "其他",
];

export interface AnalysisResult {
  word: string;
  translation: string;
  categories: string[];
  is_negative?: boolean;  // 是否为与产品不相关的否词
}

// 关键词分类结果
export interface KeywordCategoryResult {
  keyword: string;
  primary_category: string;    // 一级分类
  secondary_category: string;  // 二级分类
  search_intent: string;       // 搜索意图
}

// 带超时的fetch
async function fetchWithTimeout(
  url: string,
  options: RequestInit,
  timeoutMs: number = 60000
): Promise<Response> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), timeoutMs);

  // 合并外部signal和超时signal
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

// 单批次分析（支持取消）
export async function analyzeWords(
  words: string[],
  signal?: AbortSignal,
  productName?: string  // 产品名称，用于判断否词
): Promise<AnalysisResult[]> {
  // 获取 API Key
  const apiKey = await getDeepSeekApiKey();

  // 根据是否有产品名构建不同的prompt
  const negativeInstruction = productName
    ? `
3. 是否为否词（is_negative）：判断该词根是否与产品"${productName}"不相关。如果该词根明显指向其他品类、其他人群、竞品品牌等与该产品无关的概念，则标记为true。`
    : "";

  const negativeExample = productName
    ? `, "is_negative": false`
    : "";

  const negativeExamples = productName
    ? `
  {"word": "dog", "translation": "狗", "categories": ["其他"], "is_negative": true},
  {"word": "men", "translation": "男士", "categories": ["适用人群"], "is_negative": true}`
    : "";

  const prompt = `你是一个电商关键词分析专家。请分析以下英文词根，为每个词提供：
1. 中文翻译（简洁准确）
2. 分类标签（从以下分类中选择1-3个最合适的）${negativeInstruction}

可选分类：${CATEGORIES.join("、")}
${productName ? `\n当前产品：${productName}` : ""}

词根列表：
${words.join("\n")}

请严格按照以下JSON格式返回，不要有其他内容：
[
  {"word": "helmet", "translation": "头盔", "categories": ["品类词"]${negativeExample}},
  {"word": "kids", "translation": "儿童", "categories": ["适用人群"]${negativeExample}},
  {"word": "blue", "translation": "蓝色", "categories": ["颜色"]${negativeExample}}${negativeExamples}
]`;

  const response = await fetchWithTimeout(
    DEEPSEEK_API_URL,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${apiKey}`,
      },
      body: JSON.stringify({
        model: "deepseek-chat",
        messages: [
          {
            role: "user",
            content: prompt,
          },
        ],
        temperature: 0.1,
      }),
      signal,
    },
    60000 // 60秒超时
  );

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(parseHttpError(response.status, errorText, 'DeepSeek'));
  }

  const data = await response.json();
  const content = data.choices[0]?.message?.content || "";

  // 解析JSON响应
  try {
    // 提取JSON部分（处理可能的markdown代码块）
    let jsonStr = content;
    const jsonMatch = content.match(/\[[\s\S]*\]/);
    if (jsonMatch) {
      jsonStr = jsonMatch[0];
    }
    return JSON.parse(jsonStr);
  } catch (e) {
    console.error("解析响应失败:", content);
    throw new Error("AI 响应格式异常，请重试");
  }
}

// 并发控制器
async function runWithConcurrency<T>(
  tasks: (() => Promise<T>)[],
  concurrency: number,
  signal?: AbortSignal
): Promise<T[]> {
  const results: T[] = [];
  let index = 0;

  async function runNext(): Promise<void> {
    while (index < tasks.length) {
      if (signal?.aborted) {
        throw new DOMException("Aborted", "AbortError");
      }
      const currentIndex = index++;
      const result = await tasks[currentIndex]();
      results[currentIndex] = result;
    }
  }

  // 启动并发workers
  const workers = Array(Math.min(concurrency, tasks.length))
    .fill(null)
    .map(() => runNext());

  await Promise.all(workers);
  return results;
}

export interface BatchAnalyzeOptions {
  batchSize?: number;
  concurrency?: number;
  onProgress?: (current: number, total: number) => void;
  onBatchComplete?: (results: AnalysisResult[]) => Promise<void>;
  signal?: AbortSignal;
  productName?: string;  // 产品名称，用于判断否词
}

// 批量分析（支持并发和取消）
export async function batchAnalyzeWords(
  words: string[],
  options: BatchAnalyzeOptions = {}
): Promise<AnalysisResult[]> {
  const {
    batchSize = 30,
    concurrency = 3,
    onProgress,
    onBatchComplete,
    signal,
    productName,
  } = options;

  const allResults: AnalysisResult[] = [];
  const batches: string[][] = [];

  // 分批
  for (let i = 0; i < words.length; i += batchSize) {
    batches.push(words.slice(i, i + batchSize));
  }

  let completedWords = 0;

  // 创建任务
  const tasks = batches.map((batch) => async () => {
    if (signal?.aborted) {
      throw new DOMException("Aborted", "AbortError");
    }

    const batchResults = await analyzeWords(batch, signal, productName);

    // 更新进度
    completedWords += batch.length;
    if (onProgress) {
      onProgress(completedWords, words.length);
    }

    // 每批完成后回调（用于渐进式保存）
    if (onBatchComplete) {
      await onBatchComplete(batchResults);
    }

    // 添加小延迟避免API限流（并发时也需要）
    await new Promise((resolve) => setTimeout(resolve, 200));

    return batchResults;
  });

  try {
    // 并发执行
    const batchResults = await runWithConcurrency(tasks, concurrency, signal);
    for (const results of batchResults) {
      allResults.push(...results);
    }
  } catch (e) {
    if (e instanceof DOMException && e.name === "AbortError") {
      // 取消时返回已完成的结果
      console.log("分析已取消，返回已完成的结果");
    } else {
      throw e;
    }
  }

  return allResults;
}

// ==================== 关键词分类分析 ====================

// 一级分类列表
const PRIMARY_CATEGORIES = [
  "品类词",
  "功能词",
  "场景词",
  "属性词",
  "品牌词",
  "人群词",
  "受众词",
  "其他",
];

// 单批次关键词分类分析
export async function analyzeKeywordCategories(
  keywords: { keyword: string; translation: string | null }[],
  signal?: AbortSignal
): Promise<KeywordCategoryResult[]> {
  // 获取 API Key
  const apiKey = await getDeepSeekApiKey();

  const keywordList = keywords
    .map((k) => `${k.keyword} | ${k.translation || ""}`)
    .join("\n");

  const prompt = `你是一个专业的Amazon关键词分析专家，请为我提供的所有关键词以一级分类、二级分类、用户搜索意图进行打标。

1. 一级分类：${PRIMARY_CATEGORIES.join("、")}（尽量在我给定的一级分类里进行打标，除非不符合，则为其他）
2. 二级分类是在一级分类的基础上再进行细分
3. 用户搜索意图请明确用户的需求：比如背部疼痛、放松等
4. 输出内容为中文

关键词列表（格式：关键词 | 翻译）：
${keywordList}

请严格按照以下JSON格式返回，不要有其他内容：
[
  {"keyword": "back massager", "primary_category": "品类词", "secondary_category": "按摩器", "search_intent": "缓解背部疼痛"},
  {"keyword": "neck pillow", "primary_category": "品类词", "secondary_category": "枕头", "search_intent": "颈部支撑舒适"}
]`;

  const response = await fetchWithTimeout(
    DEEPSEEK_API_URL,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${apiKey}`,
      },
      body: JSON.stringify({
        model: "deepseek-chat",
        messages: [
          {
            role: "user",
            content: prompt,
          },
        ],
        temperature: 0.1,
      }),
      signal,
    },
    60000 // 60秒超时
  );

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(parseHttpError(response.status, errorText, 'DeepSeek'));
  }

  const data = await response.json();
  const content = data.choices[0]?.message?.content || "";

  // 解析JSON响应
  try {
    let jsonStr = content;
    const jsonMatch = content.match(/\[[\s\S]*\]/);
    if (jsonMatch) {
      jsonStr = jsonMatch[0];
    }
    return JSON.parse(jsonStr);
  } catch (e) {
    console.error("解析响应失败:", content);
    throw new Error("AI 响应格式异常，请重试");
  }
}

export interface BatchKeywordCategoryOptions {
  batchSize?: number;
  concurrency?: number;
  onProgress?: (current: number, total: number) => void;
  onBatchComplete?: (results: KeywordCategoryResult[]) => Promise<void>;
  signal?: AbortSignal;
}

// 批量关键词分类分析
export async function batchAnalyzeKeywordCategories(
  keywords: { keyword: string; translation: string | null }[],
  options: BatchKeywordCategoryOptions = {}
): Promise<KeywordCategoryResult[]> {
  const {
    batchSize = 30,
    concurrency = 3,
    onProgress,
    onBatchComplete,
    signal,
  } = options;

  const allResults: KeywordCategoryResult[] = [];
  const batches: { keyword: string; translation: string | null }[][] = [];

  // 分批
  for (let i = 0; i < keywords.length; i += batchSize) {
    batches.push(keywords.slice(i, i + batchSize));
  }

  let completedKeywords = 0;

  // 创建任务
  const tasks = batches.map((batch) => async () => {
    if (signal?.aborted) {
      throw new DOMException("Aborted", "AbortError");
    }

    const batchResults = await analyzeKeywordCategories(batch, signal);

    // 更新进度
    completedKeywords += batch.length;
    if (onProgress) {
      onProgress(completedKeywords, keywords.length);
    }

    // 每批完成后回调
    if (onBatchComplete) {
      await onBatchComplete(batchResults);
    }

    // 添加小延迟避免API限流
    await new Promise((resolve) => setTimeout(resolve, 200));

    return batchResults;
  });

  try {
    const batchResults = await runWithConcurrency(tasks, concurrency, signal);
    for (const results of batchResults) {
      allResults.push(...results);
    }
  } catch (e) {
    if (e instanceof DOMException && e.name === "AbortError") {
      console.log("分类分析已取消，返回已完成的结果");
    } else {
      throw e;
    }
  }

  return allResults;
}

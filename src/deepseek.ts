// DeepSeek API 配置
const DEEPSEEK_API_KEY = "sk-260241b985f243a78114c8f8d360c34c";
const DEEPSEEK_API_URL = "https://api.deepseek.com/chat/completions";

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
}

export async function analyzeWords(words: string[]): Promise<AnalysisResult[]> {
  const prompt = `你是一个电商关键词分析专家。请分析以下英文词根，为每个词提供：
1. 中文翻译（简洁准确）
2. 分类标签（从以下分类中选择1-3个最合适的）

可选分类：${CATEGORIES.join("、")}

词根列表：
${words.join("\n")}

请严格按照以下JSON格式返回，不要有其他内容：
[
  {"word": "helmet", "translation": "头盔", "categories": ["品类词"]},
  {"word": "kids", "translation": "儿童", "categories": ["适用人群"]},
  {"word": "blue", "translation": "蓝色", "categories": ["颜色"]}
]`;

  const response = await fetch(DEEPSEEK_API_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${DEEPSEEK_API_KEY}`,
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
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`DeepSeek API 错误: ${error}`);
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
    throw new Error("解析AI响应失败");
  }
}

// 批量分析（分批处理）
export async function batchAnalyzeWords(
  words: string[],
  batchSize: number = 30,
  onProgress?: (current: number, total: number) => void
): Promise<AnalysisResult[]> {
  const results: AnalysisResult[] = [];
  const totalBatches = Math.ceil(words.length / batchSize);

  for (let i = 0; i < words.length; i += batchSize) {
    const batch = words.slice(i, i + batchSize);
    const batchResults = await analyzeWords(batch);
    results.push(...batchResults);

    if (onProgress) {
      onProgress(Math.min(i + batchSize, words.length), words.length);
    }

    // 添加延迟避免API限流
    if (i + batchSize < words.length) {
      await new Promise((resolve) => setTimeout(resolve, 500));
    }
  }

  return results;
}

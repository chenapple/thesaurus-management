/**
 * 智能文案 - AI Prompt 模板
 *
 * 用于生成 Listing 优化建议的 AI Prompt
 */

import type { ScCompetitor, ScReview, KeywordData, ReviewInsights, ListingAnalysis, MyProductInfo } from './types';

// ==================== 评论洞察分析 ====================

export function buildReviewInsightsPrompt(
  reviews: Array<{ asin: string; reviews: ScReview[] }>,
  marketplace: string
): string {
  // 组织评论数据
  const reviewTexts: string[] = [];

  reviews.forEach(({ asin, reviews: compReviews }) => {
    if (compReviews.length > 0) {
      reviewTexts.push(`\n## 竞品 ${asin} 的评论 (${compReviews.length}条):\n`);
      compReviews.forEach((r, i) => {
        const stars = '★'.repeat(r.star_rating) + '☆'.repeat(5 - r.star_rating);
        reviewTexts.push(`[${i + 1}] ${stars} ${r.review_text}`);
      });
    }
  });

  return `你是一位亚马逊产品分析专家。请分析以下竞品评论数据，提取有价值的用户洞察。

## 任务
分析评论，提取以下信息：
1. **使用场景** (5-10条): 买家实际使用产品的场景
2. **爽点/卖点** (5-10条): 买家喜欢的产品优点
3. **痛点/问题** (5-10条): 买家抱怨的问题或不满

## 要求
- 使用场景要具体，如"户外露营时使用"而非"户外使用"
- 卖点要从买家角度描述，用买家的语言
- 痛点关注差评(1-3星)中的高频问题
- 每条都要标注出现频次和示例评论

## 站点
${marketplace}

## 评论数据
${reviewTexts.join('\n')}

## 输出格式
请严格按以下 JSON 格式输出（不要输出其他内容）：
\`\`\`json
{
  "usage_scenarios": [
    { "scenario": "使用场景描述", "source_count": 出现次数, "example_review": "示例评论原文片段" }
  ],
  "praise_points": [
    { "point": "卖点描述", "frequency": 出现次数, "example_review": "示例评论原文片段" }
  ],
  "pain_points": [
    { "point": "痛点描述", "frequency": 出现次数, "star_distribution": "主要出现在几星评论", "example_review": "示例评论原文片段" }
  ],
  "summary": "综合洞察总结（100字以内）"
}
\`\`\``;
}

// ==================== 竞品文案分析 ====================

export function buildListingAnalysisPrompt(
  competitors: ScCompetitor[],
  marketplace: string
): string {
  // 组织竞品文案数据
  const listingTexts: string[] = [];

  competitors.forEach((comp, i) => {
    if (comp.title || comp.bullets) {
      listingTexts.push(`
## 竞品 ${i + 1}: ${comp.asin}
**标题**: ${comp.title || '(未获取)'}
**五点描述**:
${comp.bullets ? JSON.parse(comp.bullets).map((b: string, j: number) => `${j + 1}. ${b}`).join('\n') : '(未获取)'}
**商品描述**: ${comp.description || '(未获取)'}
`);
    }
  });

  return `你是一位亚马逊 Listing 优化专家。请分析以下竞品的文案结构。

## 任务
分析竞品文案，提取以下信息：
1. **标题结构**: 分析标题的组成结构（品牌词、核心词、属性词、场景词的排列方式）
2. **五点主题**: 提取五点描述中的共同主题和卖点
3. **关键词使用**: 分析竞品文案中使用的高频词汇

## 站点
${marketplace}

## 竞品文案数据
${listingTexts.join('\n---\n')}

## 输出格式
请严格按以下 JSON 格式输出（不要输出其他内容）：
\`\`\`json
{
  "title_analysis": {
    "common_structure": "竞品标题的共同结构模式描述",
    "high_frequency_words": ["高频词1", "高频词2", "高频词3"],
    "competitors": [
      {
        "asin": "ASIN",
        "title": "完整标题",
        "structure_breakdown": {
          "brand": "品牌词",
          "core": "核心关键词",
          "attributes": "属性词",
          "scenarios": "场景词"
        }
      }
    ]
  },
  "bullet_analysis": {
    "common_themes": ["共同主题1", "共同主题2"],
    "best_practices": ["最佳实践1", "最佳实践2"]
  },
  "keyword_coverage": {
    "covered": ["竞品普遍使用的关键词"],
    "patterns": ["关键词使用模式"]
  }
}
\`\`\``;
}

// ==================== 优化建议生成 ====================

export function buildOptimizationPrompt(
  reviewInsights: ReviewInsights,
  listingAnalysis: ListingAnalysis,
  keywords: KeywordData[],
  scenarioType: 'new' | 'optimize',
  marketplace: string,
  myListing?: { title?: string; bullets?: string[]; description?: string },
  myProductInfo?: MyProductInfo | null
): string {
  // 准备关键词数据（Top 50 高搜索量）
  const topKeywords = keywords
    .slice(0, 50)
    .map(k => ({
      keyword: k.keyword,
      translation: k.translation,
      search_volume: k.avg_search_volume,
      relevance: k.relevance_level,
      search_intent: k.search_intent,
    }));

  // 按搜索意图分组统计
  const intentGroups: Record<string, string[]> = {};
  topKeywords.forEach(k => {
    if (k.search_intent) {
      if (!intentGroups[k.search_intent]) {
        intentGroups[k.search_intent] = [];
      }
      intentGroups[k.search_intent].push(k.keyword);
    }
  });

  const intentSummary = Object.keys(intentGroups).length > 0
    ? `\n\n## 搜索意图分布\n${Object.entries(intentGroups).map(([intent, kws]) => `- **${intent}**: ${kws.slice(0, 5).join('、')}${kws.length > 5 ? ` 等${kws.length}个` : ''}`).join('\n')}`
    : '';

  const keywordSection = topKeywords.length > 0
    ? `## 关键词数据 (按搜索量排序)
${topKeywords.map((k, i) => `${i + 1}. "${k.keyword}" ${k.translation ? `(${k.translation})` : ''} - 搜索量: ${k.search_volume || 'N/A'}, 相关性: ${k.relevance || 'N/A'}${k.search_intent ? `, 意图: ${k.search_intent}` : ''}`).join('\n')}${intentSummary}`
    : '## 关键词数据\n(未关联关键词数据)';

  // 老品优化时的现有文案
  const myListingSection = scenarioType === 'optimize' && myListing
    ? `## 我的现有文案
**标题**: ${myListing.title || '(未获取)'}
**五点描述**:
${myListing.bullets ? myListing.bullets.map((b, i) => `${i + 1}. ${b}`).join('\n') : '(未获取)'}
**商品描述**: ${myListing.description || '(未获取)'}`
    : '';

  // 新品打造时的产品信息
  const myProductSection = myProductInfo
    ? `## 我的产品信息
**品牌名称**: ${myProductInfo.brand_name}
**产品名称**: ${myProductInfo.product_name}
**核心卖点**:
${myProductInfo.key_features.map((f, i) => `${i + 1}. ${f}`).join('\n')}
${myProductInfo.differentiators ? `**差异化特点**: ${myProductInfo.differentiators}` : ''}
${myProductInfo.specifications ? `**规格参数**: ${myProductInfo.specifications}` : ''}
${myProductInfo.target_audience ? `**目标人群**: ${myProductInfo.target_audience}` : ''}
${myProductInfo.package_contents ? `**包装配件**: ${myProductInfo.package_contents}` : ''}
${myProductInfo.additional_notes ? `**补充说明**: ${myProductInfo.additional_notes}` : ''}`
    : '';

  const taskDescription = scenarioType === 'new'
    ? (myProductInfo
        ? '基于我的产品信息和竞品分析，生成针对性的 Listing 文案建议。请确保文案突出我的产品优势和差异化特点。'
        : '生成全新的 Listing 文案建议')
    : '基于现有文案生成优化建议，保留品牌调性，重点优化关键词布局和卖点表达';

  return `你是一位资深亚马逊 Listing 优化专家，精通 A9 搜索算法、COSMO 场景算法和 Rufus AI 购物助手的工作原理。

## 任务
${taskDescription}

## 优化原则
1. **A9 算法**: 在标题前部放置核心关键词，确保完全匹配
2. **COSMO 算法**: 覆盖多种使用场景，匹配用户搜索意图
3. **Rufus 算法**: 预设性解答买家常见问题，消除购买顾虑
4. **搜索意图匹配**: 根据关键词的搜索意图（如品牌、功能、场景、人群等）合理布局文案

## 站点
${marketplace}

## 评论洞察
**使用场景**: ${reviewInsights.usage_scenarios.map(s => s.scenario).join('、')}
**卖点/爽点**: ${reviewInsights.praise_points.map(p => p.point).join('、')}
**痛点/问题**: ${reviewInsights.pain_points.map(p => p.point).join('、')}
**综合洞察**: ${reviewInsights.summary}

## 竞品文案分析
**标题结构**: ${listingAnalysis.title_analysis.common_structure}
**高频词汇**: ${listingAnalysis.title_analysis.high_frequency_words.join('、')}
**五点主题**: ${listingAnalysis.bullet_analysis.common_themes.join('、')}

${keywordSection}

${myProductSection}

${myListingSection}

## 输出要求

### 标题要求
1. 每个标题建议都要说明**理由**和**数据来源**
2. 关键词选择要说明搜索量和竞品使用情况

### 五点描述要求（重要）
1. **必须生成5条**五点描述
2. **关键词埋入**：5条五点**总共**埋入4-5个关键词（从关键词数据中选择），关键词要自然融入句子，不要重复
3. **主题参考**（根据产品特点灵活调整顺序和侧重）：
   - 核心卖点/主要功能
   - 解决用户痛点（针对评论中的负面反馈）
   - 使用场景覆盖（匹配 COSMO 场景算法）
   - 规格参数/品质保障
   - 售后服务/包装配件
4. **字符限制**：每条五点控制在200-500字符（英文）或100-250字符（中文）

### 后台关键词要求
1. 选择前台（标题+五点）未使用的长尾关键词
2. 避免与前台关键词重复

### A+内容建议要求
1. **主图文案建议**：主图上应展示的核心卖点文案（3-5条），简洁有力
2. **辅图建议**（5-7张）：
   - 每张图的主题（如：功能展示、场景使用、规格对比、包装配件等）
   - 图片上的文案建议
3. **A+模块推荐**（3-5个模块）：
   - 推荐的模块类型（对比表、场景图、品牌故事、FAQ等）
   - 每个模块的内容要点

## 输出格式
请严格按以下 JSON 格式输出（不要输出其他内容）：
\`\`\`json
{
  "title_suggestions": [
    {
      "version": 1,
      "content": "完整的标题建议",
      "reasons": [
        { "word": "关键词", "reason": "选择该词的原因", "source": "数据来源(keyword_data/listing_analysis/review_insights)" }
      ]
    }
  ],
  "bullet_suggestions": [
    {
      "index": 1,
      "focus": "这条五点的重点(核心卖点/解决痛点/使用场景/规格参数/售后保障)",
      "content": "五点描述内容（必须自然融入关键词）",
      "embedded_keywords": ["埋入的关键词1", "埋入的关键词2"],
      "reason": "为什么这样写，以及为什么选择这些关键词",
      "source": "数据来源"
    }
  ],
  "backend_keywords": [
    { "keyword": "后台关键词", "reason": "选择原因(避免前台重复/长尾词/竞品遗漏词等)", "search_volume": 搜索量数字或null }
  ],
  "keyword_distribution_summary": "关键词分布总结：说明哪些高搜索量关键词被埋入了哪条五点",
  "aplus_suggestions": {
    "main_image": {
      "key_points": ["主图卖点文案1", "主图卖点文案2", "主图卖点文案3"]
    },
    "secondary_images": [
      { "index": 1, "theme": "图片主题", "copy_suggestion": "图片文案建议" }
    ],
    "module_recommendations": [
      { "module_type": "comparison_chart", "module_name": "产品对比表", "content_points": ["对比维度1", "对比维度2"] }
    ]
  }
}
\`\`\``;
}

// ==================== 辅助函数 ====================

/**
 * 解析 AI 返回的 JSON 结果
 */
export function parseAIResponse<T>(response: string): T | null {
  try {
    // 尝试提取 JSON 代码块
    const jsonMatch = response.match(/```json\s*([\s\S]*?)\s*```/);
    if (jsonMatch) {
      return JSON.parse(jsonMatch[1]);
    }
    // 尝试直接解析
    return JSON.parse(response);
  } catch (e) {
    console.error('Failed to parse AI response:', e);
    return null;
  }
}

/**
 * 验证评论洞察结果
 */
export function validateReviewInsights(data: unknown): data is ReviewInsights {
  if (!data || typeof data !== 'object') return false;
  const d = data as Record<string, unknown>;
  return (
    Array.isArray(d.usage_scenarios) &&
    Array.isArray(d.praise_points) &&
    Array.isArray(d.pain_points) &&
    typeof d.summary === 'string'
  );
}

/**
 * 验证文案分析结果
 */
export function validateListingAnalysis(data: unknown): data is ListingAnalysis {
  if (!data || typeof data !== 'object') return false;
  const d = data as Record<string, unknown>;
  return Boolean(
    d.title_analysis && typeof d.title_analysis === 'object' &&
    d.bullet_analysis && typeof d.bullet_analysis === 'object' &&
    d.keyword_coverage && typeof d.keyword_coverage === 'object'
  );
}

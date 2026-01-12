/**
 * 智能广告多智能体 Prompt 模板
 * 基于 visible_manus 多智能体架构设计
 */

import type { AdSearchTerm } from './types';

// 货币信息接口
export interface CurrencyInfo {
  symbol: string;
  code: string;
}

// ==================== 搜索词分析师 ====================

export function buildSearchTermAnalystPrompt(
  searchTerms: AdSearchTerm[],
  targetAcos: number,
  country?: string,
  currency?: CurrencyInfo
): string {
  // 计算统计数据
  const avgSpend = searchTerms.reduce((s, t) => s + t.spend, 0) / searchTerms.length;
  const currencySymbol = currency?.symbol || '$';
  const countryInfo = country ? `\n\n## 市场/国家\n${country} (${currency?.code || 'USD'})` : '';

  return `你是一位资深的亚马逊 PPC 搜索词分析师。

## 重要：输出语言要求
**无论搜索词是什么语言（英语、日语、德语等），你的分析结果必须全部使用中文输出。**
- reason、suggestion、insights 等说明性字段必须是中文
- search_term 保留原文

## 任务
分析以下搜索词数据，识别需要否定的无效词和高潜力词。${countryInfo}

## 目标 ACOS
${targetAcos}%

## 平均花费参考
${currencySymbol}${avgSpend.toFixed(2)}

## 搜索词数据（按花费排序前 200 条）
${JSON.stringify(
  searchTerms
    .sort((a, b) => b.spend - a.spend)
    .slice(0, 200)
    .map(t => ({
      search_term: t.customer_search_term,
      campaign: t.campaign_name,
      ad_group: t.ad_group_name,
      targeting: t.targeting,
      match_type: t.match_type,
      sku: t.sku,
      impressions: t.impressions,
      clicks: t.clicks,
      spend: `${currencySymbol}${t.spend.toFixed(2)}`,
      sales: `${currencySymbol}${t.sales.toFixed(2)}`,
      orders: t.orders,
      acos: t.acos,
      ctr: t.ctr,
      conversion_rate: t.conversion_rate
    })),
  null, 2
)}

## 分析维度

1. **无效搜索词识别**
   - 高花费零转化（spend > 平均值 && orders = 0）
   - 极低转化率（conversion_rate < 1% && clicks > 10）
   - 不相关词（通过语义判断与投放词不相关）
   - ACOS 极高（acos > 200%）

2. **高潜力词识别**
   - 高转化低 ACOS（acos < 目标 && orders >= 2）
   - 低展示高转化（需要加大投放）
   - 精准匹配候选词（已有好表现的广泛/词组匹配词）

3. **否定原因分类（reason_category）- 必须为每个否定词指定**
   - wrong_category: 产品品类不匹配（如搜"大鼠"卖"小鼠"）
   - wrong_scenario: 使用场景不匹配（如搜"实验室用"卖家用产品）
   - non_target_customer: 非目标客户群（如搜"批发"但只零售）
   - low_intent: 低购买意图词（如含"免费"、"怎么做"、"DIY"）
   - competitor: 竞品品牌词
   - other: 纯数据驱动（高花费零转化等，无法判断具体原因）

4. **否定层级建议（negation_level）- 必须为每个否定词指定**
   - ad_group: 仅在该广告组中不相关，其他广告组可能有效
   - campaign: 在整个活动中都不相关，建议活动级否定
   - account: 全账户都应否定（谨慎使用，仅用于明确的不相关词如竞品词）

## 输出格式（必须是有效的 JSON）

**重要：为控制输出长度，请限制数量：**
- negative_candidates: 最多 15 条（按浪费金额排序，优先返回最严重的）
- high_potential: 最多 10 条（按潜力排序）
- insights: 最多 3 条

**match_type_suggestion 字段规则：**
- "exact"（精准否定）：只否定该搜索词的完全匹配。适用于大多数情况，特别是高花费零转化的具体搜索词
- "phrase"（词组否定）：否定包含该词组的所有搜索词。仅当该词组的所有变体都表现差时使用
- **注意：reason 中的描述必须与 match_type_suggestion 保持一致，不要出现"否定精确"但建议"phrase"的矛盾情况**

\`\`\`json
{
  "negative_candidates": [
    {
      "search_term": "搜索词",
      "campaign_name": "具体广告活动名称",
      "ad_group_name": "具体广告组名称",
      "targeting": "投放词",
      "sku": "SKU（如有）",
      "reason": "高花费零转化，花费${currencySymbol}XX，建议精准否定",
      "risk_level": "high",
      "spend_wasted": 12.5,
      "match_type_suggestion": "exact",
      "campaigns_affected": ["Campaign A"],
      "reason_category": "other",
      "negation_level": "campaign",
      "negation_level_reason": "该词在整个活动中都不相关"
    }
  ],
  "high_potential": [
    {
      "search_term": "搜索词",
      "campaign_name": "广告活动名称",
      "ad_group_name": "广告组名称",
      "targeting": "投放词",
      "performance": { "orders": 5, "acos": 15, "conversion_rate": 8.5 },
      "suggestion": "建议精准投放",
      "current_match_type": "broad"
    }
  ],
  "insights": ["洞察1", "洞察2"]
}
\`\`\`

请严格按照上述 JSON 格式输出，不要添加额外的解释文字。确保 JSON 完整有效。`;
}

// ==================== ACOS 专家 ====================

export function buildAcosExpertPrompt(
  searchTerms: AdSearchTerm[],
  targetAcos: number,
  country?: string,
  currency?: CurrencyInfo
): string {
  const totalSpend = searchTerms.reduce((s, t) => s + t.spend, 0);
  const totalSales = searchTerms.reduce((s, t) => s + t.sales, 0);
  const overallAcos = totalSales > 0 ? (totalSpend / totalSales) * 100 : 0;
  const currencySymbol = currency?.symbol || '$';
  const countryInfo = country ? ` (${country} - ${currency?.code || 'USD'})` : '';

  // 按 ACOS 分组
  const excellentTerms = searchTerms.filter(t => t.acos > 0 && t.acos <= targetAcos * 0.7);
  const goodTerms = searchTerms.filter(t => t.acos > targetAcos * 0.7 && t.acos <= targetAcos);
  const marginalTerms = searchTerms.filter(t => t.acos > targetAcos && t.acos <= targetAcos * 1.5);
  const poorTerms = searchTerms.filter(t => t.acos > targetAcos * 1.5 && t.acos <= 100);
  const veryPoorTerms = searchTerms.filter(t => t.acos > 100);
  const noSalesTerms = searchTerms.filter(t => t.spend > 0 && t.sales === 0);

  return `你是一位亚马逊广告 ACOS 优化专家。

## 重要：输出语言要求
**无论搜索词是什么语言（英语、日语、德语等），你的分析结果必须全部使用中文输出。**
- issue、suggestion、insights 等说明性字段必须是中文
- search_term、targeting 保留原文

## 任务
分析广告效率，识别 ACOS 异常的投放词，并提供优化建议。${countryInfo}

## 目标 ACOS: ${targetAcos}%

## 数据摘要
- 总花费: ${currencySymbol}${totalSpend.toFixed(2)}
- 总销售: ${currencySymbol}${totalSales.toFixed(2)}
- 整体 ACOS: ${overallAcos.toFixed(1)}%
- 搜索词总数: ${searchTerms.length}

## ACOS 分布统计
- 优秀 (< ${(targetAcos * 0.7).toFixed(0)}%): ${excellentTerms.length} 个, 花费 ${currencySymbol}${excellentTerms.reduce((s, t) => s + t.spend, 0).toFixed(2)}
- 良好 (${(targetAcos * 0.7).toFixed(0)}% - ${targetAcos}%): ${goodTerms.length} 个, 花费 ${currencySymbol}${goodTerms.reduce((s, t) => s + t.spend, 0).toFixed(2)}
- 边缘 (${targetAcos}% - ${(targetAcos * 1.5).toFixed(0)}%): ${marginalTerms.length} 个, 花费 ${currencySymbol}${marginalTerms.reduce((s, t) => s + t.spend, 0).toFixed(2)}
- 较差 (${(targetAcos * 1.5).toFixed(0)}% - 100%): ${poorTerms.length} 个, 花费 ${currencySymbol}${poorTerms.reduce((s, t) => s + t.spend, 0).toFixed(2)}
- 极差 (> 100%): ${veryPoorTerms.length} 个, 花费 ${currencySymbol}${veryPoorTerms.reduce((s, t) => s + t.spend, 0).toFixed(2)}
- 无销售: ${noSalesTerms.length} 个, 花费 ${currencySymbol}${noSalesTerms.reduce((s, t) => s + t.spend, 0).toFixed(2)}

## 需要重点关注的搜索词

### 超高 ACOS (>100%) - 前 30 条
${JSON.stringify(veryPoorTerms.slice(0, 30).map(t => ({
  search_term: t.customer_search_term,
  campaign: t.campaign_name,
  ad_group: t.ad_group_name,
  targeting: t.targeting,
  sku: t.sku,
  spend: `${currencySymbol}${t.spend.toFixed(2)}`,
  sales: `${currencySymbol}${t.sales.toFixed(2)}`,
  acos: t.acos,
  orders: t.orders
})), null, 2)}

### 高花费无销售 - 前 30 条
${JSON.stringify(noSalesTerms.sort((a, b) => b.spend - a.spend).slice(0, 30).map(t => ({
  search_term: t.customer_search_term,
  campaign: t.campaign_name,
  ad_group: t.ad_group_name,
  targeting: t.targeting,
  sku: t.sku,
  spend: `${currencySymbol}${t.spend.toFixed(2)}`,
  clicks: t.clicks,
  impressions: t.impressions
})), null, 2)}

## 输出格式（必须是有效的 JSON）

**重要：为控制输出长度，请限制数量：**
- optimization_priorities: 最多 15 条（按优先级和花费排序）
- insights: 最多 3 条

\`\`\`json
{
  "efficiency_analysis": {
    "profitable_keywords_count": 10,
    "losing_keywords_count": 25,
    "break_even_keywords_count": 15,
    "no_sales_keywords_count": 20
  },
  "acos_distribution": {
    "excellent": { "count": 5, "total_spend": 100, "total_sales": 500 },
    "good": { "count": 10, "total_spend": 200, "total_sales": 600 },
    "marginal": { "count": 15, "total_spend": 300, "total_sales": 400 },
    "poor": { "count": 20, "total_spend": 500, "total_sales": 300 },
    "very_poor": { "count": 10, "total_spend": 200, "total_sales": 50 }
  },
  "optimization_priorities": [
    {
      "search_term": "搜索词",
      "campaign_name": "广告活动名称",
      "ad_group_name": "广告组名称",
      "targeting": "投放词",
      "current_acos": 85,
      "spend": 50,
      "issue": "ACOS 过高但有转化",
      "suggestion": "降低竞价 20%",
      "priority": "high"
    }
  ],
  "insights": ["洞察1", "洞察2", "洞察3"]
}
\`\`\`

请严格按照上述 JSON 格式输出，不要添加额外的解释文字。确保 JSON 完整有效。`;
}

// ==================== 竞价策略师 ====================

export function buildBidStrategistPrompt(
  searchTerms: AdSearchTerm[],
  targetAcos: number,
  country?: string,
  currency?: CurrencyInfo
): string {
  const currencySymbol = currency?.symbol || '$';
  const countryInfo = country ? ` (${country} - ${currency?.code || 'USD'})` : '';

  // 按投放词聚合数据
  const targetingMap = new Map<string, {
    campaigns: Set<string>;
    adGroups: Set<string>;
    totalSpend: number;
    totalSales: number;
    totalOrders: number;
    totalClicks: number;
    totalImpressions: number;
    avgCpc: number;
    count: number;
  }>();

  searchTerms.forEach(t => {
    const key = t.targeting || t.customer_search_term || 'unknown';
    if (!targetingMap.has(key)) {
      targetingMap.set(key, {
        campaigns: new Set(),
        adGroups: new Set(),
        totalSpend: 0,
        totalSales: 0,
        totalOrders: 0,
        totalClicks: 0,
        totalImpressions: 0,
        avgCpc: 0,
        count: 0
      });
    }
    const data = targetingMap.get(key)!;
    if (t.campaign_name) data.campaigns.add(t.campaign_name);
    if (t.ad_group_name) data.adGroups.add(t.ad_group_name);
    data.totalSpend += t.spend;
    data.totalSales += t.sales;
    data.totalOrders += t.orders;
    data.totalClicks += t.clicks;
    data.totalImpressions += t.impressions;
    data.avgCpc = data.totalClicks > 0 ? data.totalSpend / data.totalClicks : 0;
    data.count++;
  });

  const aggregatedData = Array.from(targetingMap.entries())
    .map(([targeting, data]) => ({
      targeting,
      campaigns: Array.from(data.campaigns),
      ad_groups: Array.from(data.adGroups),
      spend: `${currencySymbol}${data.totalSpend.toFixed(2)}`,
      sales: `${currencySymbol}${data.totalSales.toFixed(2)}`,
      orders: data.totalOrders,
      clicks: data.totalClicks,
      impressions: data.totalImpressions,
      acos: data.totalSales > 0 ? (data.totalSpend / data.totalSales) * 100 : 0,
      conversion_rate: data.totalClicks > 0 ? (data.totalOrders / data.totalClicks) * 100 : 0,
      cpc: `${currencySymbol}${data.avgCpc.toFixed(2)}`
    }))
    .sort((a, b) => parseFloat(a.spend.slice(1)) - parseFloat(b.spend.slice(1)))
    .reverse()
    .slice(0, 100);

  return `你是一位亚马逊广告竞价策略专家。

## 重要：输出语言要求
**无论搜索词是什么语言（英语、日语、德语等），你的分析结果必须全部使用中文输出。**
- reason、insights 等说明性字段必须是中文
- targeting、campaign_name 保留原文

## 任务
基于搜索词表现数据，生成具体的竞价调整建议。${countryInfo}

## 目标 ACOS: ${targetAcos}%

## 投放词汇总表现数据（按花费排序前 100）
${JSON.stringify(aggregatedData, null, 2)}

## 竞价调整原则

1. **加价条件**（suggestion: "increase"）
   - ACOS < 目标 * 0.7 且转化率 > 5% 且展示量偏低（< 1000）
   - 表现优秀但市场份额可能不足
   - 调整幅度: +10% 到 +30%

2. **降价条件**（suggestion: "decrease"）
   - ACOS > 目标 * 1.3 且有转化
   - 需要保持曝光但控制成本
   - 调整幅度: -10% 到 -30%

3. **暂停条件**（suggestion: "pause"）
   - ACOS > 150%
   - 花费 > ${currencySymbol}15 且零转化
   - 持续亏损无改善迹象

4. **维持条件**（suggestion: "maintain"）
   - ACOS 在目标 ± 30% 范围内
   - 表现稳定

5. **调整等级（adjustment_level）- 必须为每个建议指定**
   - L1（轻微试探）: 点击 < 20 或数据不足，建议小幅调整观察
   - L2（明确调整）: 点击 >= 20，数据支持明确，可直接执行
   - L3（激进策略）: 点击 >= 50 且趋势极端（ACOS > 200% 或 ACOS < 目标*0.5），需立即行动

6. **信心值（confidence）- 0到1的数值**
   - 基于点击量: 点击越多信心越高（20次+为0.7基础，50次+为0.85基础）
   - 基于ACOS偏离度: 偏离目标越远，判断越确定（+0.1）
   - 基于转化率稳定性: 有转化记录加分（+0.05~0.1）

## 输出格式（必须是有效的 JSON）

**重要：为控制输出长度，请限制数量：**
- bid_adjustments: 最多 20 条（优先返回需要调整的，按影响金额排序）
- insights: 最多 3 条

\`\`\`json
{
  "bid_adjustments": [
    {
      "targeting": "投放词",
      "campaign_name": "广告活动名称",
      "ad_group_name": "广告组名称",
      "current_performance": {
        "acos": 45,
        "conversion_rate": 8,
        "impressions": 1000,
        "clicks": 50,
        "spend": 25.5,
        "cpc": 0.51
      },
      "suggestion": "decrease",
      "adjustment_percent": -20,
      "reason": "ACOS 超出目标 50%，建议降价观察",
      "priority": "high",
      "adjustment_level": "L2",
      "confidence": 0.85,
      "confidence_factors": ["基于50次点击", "ACOS持续高于目标"]
    }
  ],
  "summary": {
    "increase_count": 5,
    "decrease_count": 15,
    "pause_count": 8,
    "maintain_count": 72,
    "estimated_savings": 150.00
  },
  "insights": ["洞察1", "洞察2"]
}
\`\`\`

**重要提示：**
- 每条建议必须包含 campaign_name 和 ad_group_name，从输入数据的 campaigns 和 ad_groups 数组中选取
- 如果一个投放词存在多个活动/广告组，选择花费最高的那个

请严格按照上述 JSON 格式输出，不要添加额外的解释文字。确保 JSON 完整有效。`;
}

// ==================== 建议整合器 ====================

export function buildSuggestionIntegratorPrompt(
  searchTermAnalysis: any,
  acosAnalysis: any,
  bidStrategy: any,
  targetAcos: number,
  totalSpend: number,
  totalSales: number,
  country?: string,
  currency?: CurrencyInfo
): string {
  const currencySymbol = currency?.symbol || '$';
  const countryInfo = country ? ` (${country} - ${currency?.code || 'USD'})` : '';

  return `你是广告优化建议整合专家。

## 重要：输出语言要求
**无论搜索词是什么语言（英语、日语、德语等），你的分析结果必须全部使用中文输出。**
- reason、suggestion、key_insights、executive_summary 等说明性字段必须是中文
- search_term、targeting、campaign_name 保留原文

## 任务
整合三位专家的分析结果，生成最终的优化建议报告。${countryInfo}

## 基础数据
- 目标 ACOS: ${targetAcos}%
- 总花费: ${currencySymbol}${totalSpend.toFixed(2)}
- 总销售: ${currencySymbol}${totalSales.toFixed(2)}
- 整体 ACOS: ${totalSales > 0 ? ((totalSpend / totalSales) * 100).toFixed(1) : 0}%

## 搜索词分析师结果
${JSON.stringify(searchTermAnalysis, null, 2)}

## ACOS 专家结果
${JSON.stringify(acosAnalysis, null, 2)}

## 竞价策略师结果
${JSON.stringify(bidStrategy, null, 2)}

## 整合要求
1. 合并去重否定词建议，按风险级别和浪费花费排序
2. 整合竞价调整建议并按优先级排序
3. 提取关键词机会并评估潜力
4. 生成执行摘要和关键洞察

## 新增字段要求

### 否定词必须包含：
- reason_category: 原因分类（wrong_category/wrong_scenario/non_target_customer/low_intent/competitor/other）
- negation_level: 否定层级建议（ad_group/campaign/account）
- negation_level_reason: 为什么建议这个层级

### 竞价调整必须包含：
- adjustment_level: 调整等级（L1轻微试探/L2明确调整/L3激进策略）
- confidence: 信心值（0-1）
- confidence_factors: 信心来源说明数组

### 新词机会必须包含：
- opportunity_type: 机会类型
  - expansion（扩量词）: ACOS < 目标*0.8 且订单>=3，已验证可扩量
  - testing（测试词）: 订单1-2但转化率高，需单独测试验证
  - structure（结构词）: 当前是Broad匹配但表现好，适合升级匹配方式
- recommended_match_type: 推荐匹配方式（exact/phrase/broad）
- match_type_reason: 推荐原因

## 输出格式（必须是有效的 JSON）

**重要：为控制输出长度，请限制数量：**
- negative_words: 最多 15 条（按浪费金额排序，优先返回最严重的）
- bid_adjustments: 最多 15 条（按优先级排序）
- keyword_opportunities: 最多 10 条（按潜力排序）
- key_insights: 最多 5 条

**match_type_suggestion 字段规则（必须严格遵守）：**
- "exact"（精准否定）：只否定该搜索词的完全匹配。适用于大多数情况，特别是高花费零转化的具体搜索词
- "phrase"（词组否定）：否定包含该词组的所有搜索词。仅当该词组的所有变体都表现差时使用
- **重要：reason 中的描述必须与 match_type_suggestion 保持一致！如果建议"exact"则 reason 中应写"精准否定"，如果建议"phrase"则 reason 中应写"词组否定"**

\`\`\`json
{
  "negative_words": [
    {
      "search_term": "搜索词",
      "campaign_name": "广告活动名称",
      "ad_group_name": "广告组名称",
      "targeting": "投放词",
      "sku": "SKU（如有）",
      "reason": "高花费零转化，建议精准否定",
      "risk_level": "high",
      "spend_wasted": 25.00,
      "match_type_suggestion": "exact",
      "campaigns_affected": ["Campaign A", "Campaign B"],
      "reason_category": "other",
      "negation_level": "campaign",
      "negation_level_reason": "该词在整个活动中都不相关"
    }
  ],
  "bid_adjustments": [
    {
      "targeting": "投放词",
      "campaign_name": "广告活动名称",
      "ad_group_name": "广告组名称",
      "current_performance": {
        "acos": 45,
        "conversion_rate": 8,
        "impressions": 1000,
        "clicks": 50
      },
      "suggestion": "decrease",
      "adjustment_percent": -20,
      "reason": "ACOS 超标，需要降价",
      "priority": "high",
      "adjustment_level": "L2",
      "confidence": 0.85,
      "confidence_factors": ["基于50次点击", "ACOS持续高于目标"]
    }
  ],
  "keyword_opportunities": [
    {
      "search_term": "高潜力搜索词",
      "campaign_name": "广告活动名称",
      "ad_group_name": "广告组名称",
      "targeting": "投放词",
      "performance": { "orders": 5, "acos": 12, "conversion_rate": 10 },
      "suggestion": "添加为精准匹配关键词",
      "match_type": "exact",
      "estimated_potential": "高",
      "opportunity_type": "expansion",
      "recommended_match_type": "exact",
      "match_type_reason": "该词已有5单转化，ACOS仅12%，建议精准匹配锁定"
    }
  ],
  "summary": {
    "total_spend_analyzed": 5000.00,
    "potential_savings": 800.00,
    "optimization_score": 75,
    "key_insights": [
      "发现 XX 个高风险否定词，预计节省 ${currencySymbol}XXX/月",
      "XX 个关键词需要降价，当前 ACOS 超标",
      "发现 XX 个高潜力新词机会"
    ]
  }
}
\`\`\`

**optimization_score 计算规则（0-100分）：**
- 基础分 50 分
- 整体 ACOS 低于目标：+20 分
- 整体 ACOS 高于目标 50% 以上：-20 分
- 高风险否定词占比 < 5%：+10 分
- 高风险否定词占比 > 20%：-10 分
- 存在高潜力关键词机会：+10 分
- 大部分关键词 ACOS 在目标范围内：+10 分

请根据实际分析数据计算评分，不要使用示例值。

请严格按照上述 JSON 格式输出，不要添加额外的解释文字。确保 JSON 完整有效，必须包含 summary 字段。`;
}

// ==================== 智能体状态类型 ====================

export type AgentStatus = 'pending' | 'running' | 'completed' | 'error';

export interface AgentState {
  id: string;
  name: string;
  status: AgentStatus;
  progress: number;
  message?: string;           // 当前状态消息
  streamingContent?: string;  // 流式输出内容预览
  result?: any;
  error?: string;
  startTime?: number;
  endTime?: number;
}

export interface AnalysisSession {
  id: string;
  projectId: number;
  targetAcos: number;
  agents: {
    searchTermAnalyst: AgentState;
    acosExpert: AgentState;
    bidStrategist: AgentState;
    suggestionIntegrator: AgentState;
  };
  startTime: number;
  endTime?: number;
  status: 'idle' | 'running' | 'completed' | 'error' | 'partial' | 'cancelled';
  finalResult?: any;
  // 多国家分析相关
  currentCountry?: string;
  countryProgress?: {
    total: number;
    completed: number;
    countries: string[];
    failedCountries?: string[];  // 失败的国家列表
  };
  // 增量结果（每完成一个国家就更新）
  partialResults?: any[];
}

// ==================== 辅助函数 ====================

/**
 * 解析 AI 返回的 JSON（处理可能的 markdown 代码块）
 */
export function parseAIResponse(response: string): any {
  // 移除 markdown 代码块标记
  let cleaned = response.trim();

  // 处理 ```json ... ``` 格式
  const jsonBlockMatch = cleaned.match(/```(?:json)?\s*([\s\S]*?)```/);
  if (jsonBlockMatch) {
    cleaned = jsonBlockMatch[1].trim();
  }

  // 尝试找到 JSON 对象的开始和结束
  const jsonStart = cleaned.indexOf('{');
  const jsonEnd = cleaned.lastIndexOf('}');

  if (jsonStart !== -1 && jsonEnd !== -1) {
    cleaned = cleaned.substring(jsonStart, jsonEnd + 1);
  }

  try {
    return JSON.parse(cleaned);
  } catch (parseError) {
    console.error('JSON 解析失败，原始响应:', response);
    console.error('清理后的内容:', cleaned);

    // 尝试修复常见的 JSON 问题
    try {
      // 修复可能的尾部逗号问题
      let fixed = cleaned
        .replace(/,\s*}/g, '}')
        .replace(/,\s*]/g, ']');

      // 修复可能的截断数组（添加缺失的 ']' 和 '}'）
      const openBrackets = (fixed.match(/\[/g) || []).length;
      const closeBrackets = (fixed.match(/]/g) || []).length;
      const openBraces = (fixed.match(/{/g) || []).length;
      const closeBraces = (fixed.match(/}/g) || []).length;

      // 补充缺失的闭合符号
      for (let i = 0; i < openBrackets - closeBrackets; i++) {
        fixed += ']';
      }
      for (let i = 0; i < openBraces - closeBraces; i++) {
        fixed += '}';
      }

      return JSON.parse(fixed);
    } catch (fixError) {
      // 如果修复也失败了，返回一个基础结构
      console.error('JSON 修复也失败，返回默认结构');
      throw new Error(`AI 返回的 JSON 格式无效: ${(parseError as Error).message}。请尝试重新分析。`);
    }
  }
}

/**
 * 验证分析结果结构
 */
export function validateAnalysisResult(result: any): boolean {
  if (!result) return false;

  // 检查必要字段
  const requiredFields = ['negative_words', 'bid_adjustments', 'keyword_opportunities', 'summary'];
  for (const field of requiredFields) {
    if (!(field in result)) {
      console.warn(`Missing required field: ${field}`);
      return false;
    }
  }

  // 检查数组类型
  if (!Array.isArray(result.negative_words)) return false;
  if (!Array.isArray(result.bid_adjustments)) return false;
  if (!Array.isArray(result.keyword_opportunities)) return false;

  // 检查 summary 结构
  if (!result.summary.key_insights || !Array.isArray(result.summary.key_insights)) {
    return false;
  }

  return true;
}

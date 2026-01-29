/**
 * 智能广告多智能体分析引擎
 * 基于 visible_manus 多智能体架构，使用流式输出实现实时反馈
 * 支持按国家分别分析，确保货币正确处理
 */

import type { AdSearchTerm, AdAnalysisResult, AIProvider, CountryAnalysisResult } from './types';
import { COUNTRY_CURRENCY_MAP } from './types';
import { chatStream } from './ai-service';
import {
  buildSearchTermAnalystPrompt,
  buildAcosExpertPrompt,
  buildBidStrategistPrompt,
  buildSuggestionIntegratorPrompt,
  parseAIResponse,
  validateAnalysisResult,
  type AnalysisSession,
  type AgentState,
  type CurrencyInfo,
} from './ad-prompts';

// 全局 AbortController 用于中断分析
let currentAbortController: AbortController | null = null;

/**
 * 停止当前正在进行的分析
 * @returns 是否成功停止（如果没有分析在进行则返回 false）
 */
export function stopAnalysis(): boolean {
  if (currentAbortController) {
    console.log('[MultiAgent] 用户请求停止分析');
    currentAbortController.abort();
    currentAbortController = null;
    return true;
  }
  return false;
}

/**
 * 检查分析是否正在进行
 */
export function isAnalysisRunning(): boolean {
  return currentAbortController !== null;
}

/**
 * 智能采样 - 确保各类问题都能被 AI 发现
 * 当数据量超过 maxCount 时，按分类采样而非只取高花费词
 *
 * @param terms 原始搜索词数据
 * @param targetAcos 目标 ACOS
 * @param maxCount 最大采样数量（默认 200）
 * @returns 采样后的搜索词数据和采样信息
 */
export function smartSampleTerms(
  terms: AdSearchTerm[],
  targetAcos: number,
  maxCount: number = 200
): { sampledTerms: AdSearchTerm[]; originalCount: number; wasSampled: boolean } {
  const originalCount = terms.length;

  if (terms.length <= maxCount) {
    return { sampledTerms: terms, originalCount, wasSampled: false };
  }

  const selected = new Set<number>(); // 存储已选中的 ID
  const result: AdSearchTerm[] = [];

  // 1. 高花费词 (40%) - 按花费降序
  const highSpend = [...terms].sort((a, b) => b.spend - a.spend);
  const highSpendCount = Math.floor(maxCount * 0.4);
  for (let i = 0; i < highSpendCount && i < highSpend.length; i++) {
    selected.add(highSpend[i].id);
    result.push(highSpend[i]);
  }

  // 2. 高 ACOS 词 (20%) - ACOS > 目标 ACOS，按 ACOS 降序
  const highAcos = terms
    .filter(t => t.acos > targetAcos && !selected.has(t.id))
    .sort((a, b) => b.acos - a.acos);
  const highAcosCount = Math.floor(maxCount * 0.2);
  for (let i = 0; i < highAcosCount && i < highAcos.length; i++) {
    selected.add(highAcos[i].id);
    result.push(highAcos[i]);
  }

  // 3. 低转化词 (20%) - 点击 > 10 且订单 = 0，按花费降序
  const lowConversion = terms
    .filter(t => t.clicks > 10 && t.orders === 0 && !selected.has(t.id))
    .sort((a, b) => b.spend - a.spend);
  const lowConversionCount = Math.floor(maxCount * 0.2);
  for (let i = 0; i < lowConversionCount && i < lowConversion.length; i++) {
    selected.add(lowConversion[i].id);
    result.push(lowConversion[i]);
  }

  // 4. 高潜力词 (20%) - ACOS < 目标 ACOS 且订单 > 0，按销售额降序
  const highPotential = terms
    .filter(t => t.acos > 0 && t.acos < targetAcos && t.orders > 0 && !selected.has(t.id))
    .sort((a, b) => b.sales - a.sales);
  const highPotentialCount = Math.floor(maxCount * 0.2);
  for (let i = 0; i < highPotentialCount && i < highPotential.length; i++) {
    selected.add(highPotential[i].id);
    result.push(highPotential[i]);
  }

  // 5. 如果还不够，补充剩余高花费词
  if (result.length < maxCount) {
    const remaining = highSpend.filter(t => !selected.has(t.id));
    for (let i = 0; result.length < maxCount && i < remaining.length; i++) {
      selected.add(remaining[i].id);
      result.push(remaining[i]);
    }
  }

  console.log(`[SmartSample] 智能采样完成: ${originalCount} → ${result.length} 条`);
  console.log(`[SmartSample] 分布: 高花费 ${highSpendCount}, 高ACOS ${Math.min(highAcosCount, highAcos.length)}, 低转化 ${Math.min(lowConversionCount, lowConversion.length)}, 高潜力 ${Math.min(highPotentialCount, highPotential.length)}`);

  return { sampledTerms: result, originalCount, wasSampled: true };
}

// 按国家分组搜索词
function groupSearchTermsByCountry(searchTerms: AdSearchTerm[]): Map<string, AdSearchTerm[]> {
  const grouped = new Map<string, AdSearchTerm[]>();

  searchTerms.forEach(term => {
    // 处理 null、undefined、空字符串的情况
    const country = (term.country && term.country.trim()) ? term.country.trim() : 'Unknown';
    if (!grouped.has(country)) {
      grouped.set(country, []);
    }
    grouped.get(country)!.push(term);
  });

  // 调试日志：输出各国家数据量
  console.log('[MultiAgent] 按国家分组结果:');
  grouped.forEach((terms, country) => {
    console.log(`  - ${country}: ${terms.length} 条`);
  });

  return grouped;
}

// 获取国家对应的货币信息
function getCurrencyForCountry(country: string): CurrencyInfo {
  return COUNTRY_CURRENCY_MAP[country] || { symbol: '$', code: 'USD' };
}

// 根据 AI 服务商获取最大 token 限制
function getMaxTokensForProvider(provider: AIProvider, model?: string): number {
  switch (provider) {
    case 'gemini':
      // Gemini API 支持最大 65536，但为安全起见使用较低值
      return 16384;
    case 'openai':
      return 16384;  // GPT-4 系列
    case 'deepseek':
      return 8192;   // DeepSeek 最大 8192
    case 'qwen':
      // 根据模型调整
      if (model?.includes('plus')) return 16384;  // Qwen-Plus 支持更高
      if (model?.includes('max')) return 8192;    // Qwen-Max
      return 8192;   // Qwen-Turbo 等
    default:
      return 4096;   // 保守默认值
  }
}

// 使用流式 API 调用 AI
async function callAIStream(
  agentName: string,
  prompt: string,
  provider: AIProvider,
  model: string,
  onProgress?: (progress: number, message: string, streaming: string) => void,
  signal?: AbortSignal
): Promise<string> {
  console.log(`[${agentName}] 开始流式调用 AI...`);
  onProgress?.(5, '准备请求...', '');

  let fullResponse = '';
  let charCount = 0;
  const maxTokens = getMaxTokensForProvider(provider, model);

  // 节流控制：限制 UI 更新频率，避免过度渲染
  let lastUpdateTime = 0;
  const UPDATE_INTERVAL = 200; // 每 200ms 最多更新一次 UI

  try {
    for await (const chunk of chatStream(
      [
        {
          role: 'system',
          content: '你是一位专业的亚马逊广告优化专家。请严格按照用户要求的 JSON 格式输出结果，不要添加额外的解释文字。确保返回有效的 JSON。',
        },
        {
          role: 'user',
          content: prompt,
        },
      ],
      {
        provider,
        model,
        temperature: 0.3,
        maxTokens,
        signal,
      }
    )) {
      fullResponse += chunk.content;
      charCount += chunk.content.length;

      // 节流：只在间隔足够时才更新 UI
      const now = Date.now();
      if (now - lastUpdateTime >= UPDATE_INTERVAL) {
        lastUpdateTime = now;
        const estimatedProgress = Math.min(90, Math.round((charCount / 3000) * 85) + 10);
        const preview = fullResponse.slice(-150);
        onProgress?.(estimatedProgress, `正在生成... ${charCount} 字符`, preview);
      }
    }

    console.log(`[${agentName}] 流式响应完成，总长度: ${fullResponse.length}`);
    onProgress?.(100, '分析完成', '');

    return fullResponse;
  } catch (error) {
    console.error(`[${agentName}] AI 调用失败:`, error);
    throw error;
  }
}

// 创建初始会话
function createSession(projectId: number, targetAcos: number): AnalysisSession {
  const createAgent = (id: string, name: string): AgentState => ({
    id,
    name,
    status: 'pending',
    progress: 0,
  });

  return {
    id: `session_${Date.now()}`,
    projectId,
    targetAcos,
    agents: {
      searchTermAnalyst: createAgent('search_term_analyst', '搜索词分析师'),
      acosExpert: createAgent('acos_expert', 'ACOS 专家'),
      bidStrategist: createAgent('bid_strategist', '竞价策略师'),
      suggestionIntegrator: createAgent('suggestion_integrator', '建议整合器'),
    },
    startTime: Date.now(),
    status: 'idle',
  };
}

// 深拷贝 session 并触发更新
function cloneSession(session: AnalysisSession): AnalysisSession {
  return JSON.parse(JSON.stringify(session));
}

// 更新单个 agent 状态
function updateAgentState(
  session: AnalysisSession,
  agentKey: keyof AnalysisSession['agents'],
  updates: Partial<AgentState>,
  onUpdate?: () => void
) {
  const agent = session.agents[agentKey];
  Object.assign(agent, updates);
  onUpdate?.();
}

// 运行单个智能体（使用流式输出）
async function runAgentStream(
  session: AnalysisSession,
  agentKey: keyof AnalysisSession['agents'],
  prompt: string,
  provider: AIProvider,
  model: string,
  onUpdate: () => void,
  signal?: AbortSignal
): Promise<any> {
  const agent = session.agents[agentKey];

  // 设置为运行状态
  updateAgentState(session, agentKey, {
    status: 'running',
    progress: 0,
    startTime: Date.now(),
    message: '启动中...',
  }, onUpdate);

  try {
    const response = await callAIStream(
      agent.name,
      prompt,
      provider,
      model,
      (progress, message, streaming) => {
        updateAgentState(session, agentKey, {
          progress,
          message,
          streamingContent: streaming,
        }, onUpdate);
      },
      signal
    );

    const result = parseAIResponse(response);

    // 设置为完成状态
    updateAgentState(session, agentKey, {
      status: 'completed',
      progress: 100,
      result,
      endTime: Date.now(),
      message: '分析完成',
      streamingContent: undefined,
    }, onUpdate);

    return result;
  } catch (error) {
    // 设置为错误状态
    updateAgentState(session, agentKey, {
      status: 'error',
      progress: 0,
      error: (error as Error).message,
      endTime: Date.now(),
      message: '分析失败: ' + (error as Error).message,
      streamingContent: undefined,
    }, onUpdate);
    throw error;
  }
}

/**
 * 去重否定词建议
 * 相同搜索词合并：累加浪费金额，合并影响活动，取最高风险级别
 */
function deduplicateNegativeWords(negativeWords: any[]): any[] {
  const wordMap = new Map<string, any>();

  for (const word of negativeWords) {
    const key = word.search_term?.toLowerCase?.() || '';
    if (!key) continue;

    if (wordMap.has(key)) {
      const existing = wordMap.get(key)!;
      // 累加浪费金额
      existing.spend_wasted = (existing.spend_wasted || 0) + (word.spend_wasted || 0);
      // 合并影响活动（去重）
      const campaigns = new Set([
        ...(existing.campaigns_affected || []),
        ...(word.campaigns_affected || []),
      ]);
      existing.campaigns_affected = Array.from(campaigns);
      // 合并广告组（记录所有涉及的广告组）
      if (word.ad_group_name && word.ad_group_name !== existing.ad_group_name) {
        existing.ad_group_name = existing.ad_group_name
          ? `${existing.ad_group_name}, ${word.ad_group_name}`
          : word.ad_group_name;
      }
      // 取最高风险级别
      const riskOrder = { high: 3, medium: 2, low: 1 };
      const existingRisk = riskOrder[existing.risk_level as keyof typeof riskOrder] || 0;
      const newRisk = riskOrder[word.risk_level as keyof typeof riskOrder] || 0;
      if (newRisk > existingRisk) {
        existing.risk_level = word.risk_level;
      }
      // 优先使用精准否定
      if (word.match_type_suggestion === 'exact') {
        existing.match_type_suggestion = 'exact';
      }
    } else {
      wordMap.set(key, { ...word });
    }
  }

  // 转回数组，按浪费金额降序排列
  return Array.from(wordMap.values()).sort((a, b) => (b.spend_wasted || 0) - (a.spend_wasted || 0));
}

/**
 * 去重新词机会
 * 相同搜索词合并：累加订单数，取最佳 ACOS，合并广告组
 */
function deduplicateKeywordOpportunities(opportunities: any[]): any[] {
  const wordMap = new Map<string, any>();

  for (const opp of opportunities) {
    const key = opp.search_term?.toLowerCase?.() || '';
    if (!key) continue;

    if (wordMap.has(key)) {
      const existing = wordMap.get(key)!;
      // 累加订单数
      if (existing.performance && opp.performance) {
        existing.performance.orders = (existing.performance.orders || 0) + (opp.performance.orders || 0);
        // 取最低 ACOS（更好的表现）
        if (opp.performance.acos < existing.performance.acos) {
          existing.performance.acos = opp.performance.acos;
          existing.performance.conversion_rate = opp.performance.conversion_rate;
        }
      }
      // 合并广告组
      if (opp.ad_group_name && opp.ad_group_name !== existing.ad_group_name) {
        existing.ad_group_name = existing.ad_group_name
          ? `${existing.ad_group_name}, ${opp.ad_group_name}`
          : opp.ad_group_name;
      }
      // 合并活动
      if (opp.campaign_name && opp.campaign_name !== existing.campaign_name) {
        existing.campaign_name = existing.campaign_name
          ? `${existing.campaign_name}, ${opp.campaign_name}`
          : opp.campaign_name;
      }
    } else {
      wordMap.set(key, { ...opp });
    }
  }

  // 转回数组，按订单数降序排列
  return Array.from(wordMap.values()).sort((a, b) =>
    (b.performance?.orders || 0) - (a.performance?.orders || 0)
  );
}

/**
 * 运行单个国家的分析
 */
async function runSingleCountryAnalysis(
  searchTerms: AdSearchTerm[],
  targetAcos: number,
  provider: AIProvider,
  model: string,
  country: string,
  currency: CurrencyInfo,
  session: AnalysisSession,
  notifyUpdate: () => void,
  signal: AbortSignal
): Promise<CountryAnalysisResult> {
  console.log(`[MultiAgent] 分析国家: ${country} (${currency.code})`);
  console.log(`[MultiAgent] 该国家搜索词数量: ${searchTerms.length}`);

  // 智能采样 - 确保各类问题都能被发现
  const { sampledTerms, originalCount, wasSampled } = smartSampleTerms(searchTerms, targetAcos);
  if (wasSampled) {
    console.log(`[MultiAgent] 已智能采样: ${originalCount} → ${sampledTerms.length} 条`);
  }
  const termsForAnalysis = sampledTerms;
  const samplingInfo = wasSampled ? { originalCount, sampledCount: sampledTerms.length } : null;

  // 重置所有 agent 状态
  const resetAgents = () => {
    const agentKeys: (keyof AnalysisSession['agents'])[] = [
      'searchTermAnalyst', 'acosExpert', 'bidStrategist', 'suggestionIntegrator'
    ];
    agentKeys.forEach(key => {
      updateAgentState(session, key, {
        status: 'pending',
        progress: 0,
        message: undefined,
        streamingContent: undefined,
        result: undefined,
        error: undefined,
      }, notifyUpdate);
    });
  };

  resetAgents();

  // 构建带国家/货币信息的 prompts（使用采样后的数据）
  const prompts = {
    searchTermAnalyst: buildSearchTermAnalystPrompt(termsForAnalysis, targetAcos, country, currency, samplingInfo),
    acosExpert: buildAcosExpertPrompt(termsForAnalysis, targetAcos, country, currency, samplingInfo),
    bidStrategist: buildBidStrategistPrompt(termsForAnalysis, targetAcos, country, currency, samplingInfo),
  };

  // 并行执行三个智能体
  const [searchTermResult, acosResult, bidResult] = await Promise.all([
    runAgentStream(session, 'searchTermAnalyst', prompts.searchTermAnalyst, provider, model, notifyUpdate, signal),
    runAgentStream(session, 'acosExpert', prompts.acosExpert, provider, model, notifyUpdate, signal),
    runAgentStream(session, 'bidStrategist', prompts.bidStrategist, provider, model, notifyUpdate, signal),
  ]);

  // 运行整合器
  const totalSpend = searchTerms.reduce((s, t) => s + t.spend, 0);
  const totalSales = searchTerms.reduce((s, t) => s + t.sales, 0);

  const integrationPrompt = buildSuggestionIntegratorPrompt(
    searchTermResult,
    acosResult,
    bidResult,
    targetAcos,
    totalSpend,
    totalSales,
    country,
    currency
  );

  const finalResult = await runAgentStream(
    session,
    'suggestionIntegrator',
    integrationPrompt,
    provider,
    model,
    notifyUpdate,
    signal
  );

  // 验证并修复结果
  if (!validateAnalysisResult(finalResult)) {
    console.warn(`[MultiAgent] ${country} 结果验证失败，尝试修复...`);
    if (!finalResult.negative_words) finalResult.negative_words = [];
    if (!finalResult.bid_adjustments) finalResult.bid_adjustments = [];
    if (!finalResult.keyword_opportunities) finalResult.keyword_opportunities = [];
    if (!finalResult.summary) {
      finalResult.summary = {
        total_spend_analyzed: totalSpend,
        potential_savings: 0,
        optimization_score: 50,
        key_insights: [`${country} 分析完成，但结果可能不完整`],
      };
    }
  }

  // 去重否定词：相同搜索词合并，累加浪费金额，合并影响活动
  const deduplicatedNegativeWords = deduplicateNegativeWords(finalResult.negative_words || []);

  // 去重新词机会：相同搜索词合并，累加订单数，取最佳 ACOS
  const deduplicatedOpportunities = deduplicateKeywordOpportunities(finalResult.keyword_opportunities || []);

  // 基于实际 spend_wasted 计算 potential_savings（而非依赖 AI 估算）
  const calculatedPotentialSavings = deduplicatedNegativeWords.reduce(
    (sum, w) => sum + (w.spend_wasted || 0), 0
  );

  return {
    country,
    currency,
    negative_words: deduplicatedNegativeWords,
    bid_adjustments: finalResult.bid_adjustments || [],
    keyword_opportunities: deduplicatedOpportunities,
    summary: {
      total_spend_analyzed: totalSpend,
      potential_savings: parseFloat(calculatedPotentialSavings.toFixed(2)),
      optimization_score: finalResult.summary?.optimization_score || 50,
      key_insights: finalResult.summary?.key_insights || [],
    },
  };
}

/**
 * 合并多个国家的分析结果
 */
function mergeCountryResults(countryResults: CountryAnalysisResult[]): AdAnalysisResult {
  // 合并所有国家的结果（用于概览视图）
  const allNegativeWords = countryResults.flatMap(r =>
    r.negative_words.map(w => ({ ...w, country: r.country }))
  );
  const allBidAdjustments = countryResults.flatMap(r =>
    r.bid_adjustments.map(b => ({ ...b, country: r.country }))
  );
  const allKeywordOpportunities = countryResults.flatMap(r =>
    r.keyword_opportunities.map(k => ({ ...k, country: r.country }))
  );

  // 合并摘要（注意：金额不能直接相加，因为货币不同）
  const totalOptimizationScore = countryResults.reduce((s, r) => s + r.summary.optimization_score, 0) / countryResults.length;
  const allInsights = countryResults.flatMap(r =>
    r.summary.key_insights.map(insight => `[${r.country}] ${insight}`)
  );

  return {
    negative_words: allNegativeWords,
    bid_adjustments: allBidAdjustments,
    keyword_opportunities: allKeywordOpportunities,
    summary: {
      total_spend_analyzed: 0, // 不同货币无法相加
      potential_savings: 0,    // 不同货币无法相加
      optimization_score: Math.round(totalOptimizationScore),
      key_insights: allInsights.slice(0, 10), // 限制数量
    },
    by_country: countryResults,
  };
}

/**
 * 运行多智能体分析（流式版本）
 * 使用流式输出让用户实时看到 AI 响应
 * 支持按国家分别分析，确保货币正确处理
 * 支持增量显示结果、容错和断点续传
 */
export async function runMultiAgentAnalysis(
  searchTerms: AdSearchTerm[],
  targetAcos: number,
  provider: AIProvider,
  model: string,
  onSessionUpdate?: (session: AnalysisSession) => void,
  onCountryComplete?: (country: string, result: CountryAnalysisResult) => void,
  skipCountries?: string[]  // 跳过已完成的国家（用于断点续传）
): Promise<AdAnalysisResult | null> {
  // 创建分析会话
  const session = createSession(0, targetAcos);
  session.status = 'running';
  session.partialResults = [];

  // 创建 AbortController 用于取消（保存到全局变量以支持外部中断）
  const abortController = new AbortController();
  currentAbortController = abortController;

  // 节流更新：避免频繁的深拷贝和 Vue 响应式更新
  let lastNotifyTime = 0;
  let pendingNotify = false;
  const NOTIFY_INTERVAL = 250; // 每 250ms 最多通知一次

  const notifyUpdate = () => {
    const now = Date.now();
    if (now - lastNotifyTime >= NOTIFY_INTERVAL) {
      lastNotifyTime = now;
      pendingNotify = false;
      onSessionUpdate?.(cloneSession(session));
    } else if (!pendingNotify) {
      // 如果距离上次更新不够久，延迟到下一个间隔点更新
      pendingNotify = true;
      setTimeout(() => {
        if (pendingNotify) {
          lastNotifyTime = Date.now();
          pendingNotify = false;
          onSessionUpdate?.(cloneSession(session));
        }
      }, NOTIFY_INTERVAL - (now - lastNotifyTime));
    }
  };

  // 强制立即通知（用于初始化和完成时）
  const forceNotifyUpdate = () => {
    lastNotifyTime = Date.now();
    pendingNotify = false;
    onSessionUpdate?.(cloneSession(session));
  };

  forceNotifyUpdate();

  console.log(`[MultiAgent] ========== 开始分析 ==========`);
  console.log(`[MultiAgent] 使用 ${provider} - ${model}`);
  console.log(`[MultiAgent] 总搜索词数量: ${searchTerms.length}`);

  // 按国家分组
  const byCountry = groupSearchTermsByCountry(searchTerms);
  let countries = Array.from(byCountry.keys());

  // 过滤掉 Unknown 国家（没有国家信息的数据无法准确分析）
  const unknownCount = byCountry.get('Unknown')?.length || 0;
  if (unknownCount > 0) {
    console.log(`[MultiAgent] 跳过 ${unknownCount} 条无国家信息的数据`);
    countries = countries.filter(c => c !== 'Unknown');
  }

  // 如果有跳过的国家（断点续传），过滤掉
  if (skipCountries && skipCountries.length > 0) {
    console.log(`[MultiAgent] 跳过已完成的国家: ${skipCountries.join(', ')}`);
    countries = countries.filter(c => !skipCountries.includes(c));
  }

  console.log(`[MultiAgent] 待分析国家: ${countries.join(', ')}`);

  // 初始化国家进度（不包括 Unknown）
  const validCountries = Array.from(byCountry.keys()).filter(c => c !== 'Unknown');
  session.countryProgress = {
    total: validCountries.length,
    completed: skipCountries?.length || 0,
    countries: validCountries,
    failedCountries: [],
  };
  forceNotifyUpdate();

  // 如果没有需要分析的国家
  if (countries.length === 0) {
    session.status = 'completed';
    session.endTime = Date.now();
    forceNotifyUpdate();
    return null;
  }

  // 收集结果
  const countryResults: CountryAnalysisResult[] = [];
  const failedCountries: string[] = [];

  // 逐个国家分析（支持容错）
  for (let i = 0; i < countries.length; i++) {
    const country = countries[i];
    const terms = byCountry.get(country)!;
    const currency = getCurrencyForCountry(country);

    console.log(`[MultiAgent] 开始分析国家 ${i + 1}/${countries.length}: ${country}`);

    // 更新当前分析的国家
    session.currentCountry = country;
    forceNotifyUpdate();

    try {
      const result = await runSingleCountryAnalysis(
        terms, targetAcos, provider, model, country, currency,
        session, notifyUpdate, abortController.signal
      );

      countryResults.push(result);
      session.partialResults!.push(result);
      session.countryProgress!.completed++;

      // 触发单个国家完成回调
      console.log(`[MultiAgent] ${country} 分析完成`);
      onCountryComplete?.(country, result);

      // 更新增量结果到 session
      const currentMerged = mergeCountryResults([...countryResults]);
      session.finalResult = currentMerged;
      forceNotifyUpdate();

    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);

      // 检查是否是用户主动取消
      if (error instanceof Error && error.name === 'AbortError') {
        console.log('[MultiAgent] 分析被用户取消');
        session.status = 'cancelled';
        session.currentCountry = undefined;
        session.endTime = Date.now();
        currentAbortController = null;
        forceNotifyUpdate();

        // 如果已有部分结果，返回它们
        if (countryResults.length > 0) {
          const partialMerged = mergeCountryResults(countryResults);
          session.finalResult = partialMerged;
          forceNotifyUpdate();
          return partialMerged;
        }
        return null;
      }

      // 单个国家失败，记录但继续下一个
      console.error(`[MultiAgent] ${country} 分析失败:`, errorMsg);

      // 记录更详细的错误信息
      if (errorMsg.includes('timeout') || errorMsg.includes('Timeout')) {
        console.error(`[MultiAgent] ${country}: API 请求超时，可能是网络问题或数据量过大`);
      } else if (errorMsg.includes('rate') || errorMsg.includes('429')) {
        console.error(`[MultiAgent] ${country}: API 限流，请稍后重试`);
      } else if (errorMsg.includes('token') || errorMsg.includes('context')) {
        console.error(`[MultiAgent] ${country}: 数据量过大，超出模型上下文限制`);
      }

      failedCountries.push(country);
      session.countryProgress!.failedCountries = failedCountries;
      forceNotifyUpdate();
      // 不抛出错误，继续下一个国家
    }
  }

  // 清理 AbortController
  currentAbortController = null;

  // 分析完成
  session.currentCountry = undefined;
  session.endTime = Date.now();

  if (countryResults.length === 0) {
    // 所有国家都失败了
    session.status = 'error';
    forceNotifyUpdate();
    throw new Error('所有国家分析均失败');
  } else if (failedCountries.length > 0) {
    // 部分成功
    session.status = 'partial';
    console.log(`[MultiAgent] 部分完成: ${countryResults.length} 成功, ${failedCountries.length} 失败`);
  } else {
    // 全部成功
    session.status = 'completed';
    console.log('[MultiAgent] 所有国家分析完成');
  }

  // 合并结果
  const mergedResult = mergeCountryResults(countryResults);
  session.finalResult = mergedResult;
  session.countryProgress!.completed = (skipCountries?.length || 0) + countryResults.length;
  forceNotifyUpdate();

  return mergedResult;
}

/**
 * 重试失败的国家分析
 */
export async function retryFailedCountries(
  searchTerms: AdSearchTerm[],
  targetAcos: number,
  provider: AIProvider,
  model: string,
  failedCountries: string[],
  existingResults: CountryAnalysisResult[],
  onSessionUpdate?: (session: AnalysisSession) => void,
  onCountryComplete?: (country: string, result: CountryAnalysisResult) => void
): Promise<AdAnalysisResult | null> {
  console.log(`[MultiAgent] ========== 重试模式 ==========`);
  console.log(`[MultiAgent] 重试失败的国家: ${failedCountries.join(', ')}`);
  console.log(`[MultiAgent] 已完成的国家数量: ${existingResults.length}`);

  // 过滤出失败国家的搜索词
  const failedTerms = searchTerms.filter(t => failedCountries.includes(t.country || 'Unknown'));
  console.log(`[MultiAgent] 失败国家的搜索词数量: ${failedTerms.length}`);

  // 运行分析
  const newResult = await runMultiAgentAnalysis(
    failedTerms,
    targetAcos,
    provider,
    model,
    onSessionUpdate,
    onCountryComplete,
    [] // 不跳过任何国家，因为 failedTerms 已经只包含失败国家的数据
  );

  if (!newResult) {
    return null;
  }

  // 合并新旧结果
  const allResults = [...existingResults, ...(newResult.by_country || [])];
  return mergeCountryResults(allResults);
}

/**
 * 单独运行某个智能体（用于调试或重试）
 */
export async function runSingleAgent(
  agentType: 'searchTermAnalyst' | 'acosExpert' | 'bidStrategist',
  searchTerms: AdSearchTerm[],
  targetAcos: number,
  provider: AIProvider,
  model: string
): Promise<any> {
  let prompt: string;

  switch (agentType) {
    case 'searchTermAnalyst':
      prompt = buildSearchTermAnalystPrompt(searchTerms, targetAcos);
      break;
    case 'acosExpert':
      prompt = buildAcosExpertPrompt(searchTerms, targetAcos);
      break;
    case 'bidStrategist':
      prompt = buildBidStrategistPrompt(searchTerms, targetAcos);
      break;
    default:
      throw new Error(`未知的智能体类型: ${agentType}`);
  }

  const response = await callAIStream(agentType, prompt, provider, model);
  return parseAIResponse(response);
}

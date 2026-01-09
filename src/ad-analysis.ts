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

      // 基于响应长度估算进度（假设平均响应约 3000 字符）
      const estimatedProgress = Math.min(90, Math.round((charCount / 3000) * 85) + 10);
      const preview = fullResponse.slice(-150);
      onProgress?.(estimatedProgress, `正在生成... ${charCount} 字符`, preview);
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

  // 构建带国家/货币信息的 prompts
  const prompts = {
    searchTermAnalyst: buildSearchTermAnalystPrompt(searchTerms, targetAcos, country, currency),
    acosExpert: buildAcosExpertPrompt(searchTerms, targetAcos, country, currency),
    bidStrategist: buildBidStrategistPrompt(searchTerms, targetAcos, country, currency),
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

  return {
    country,
    currency,
    negative_words: finalResult.negative_words || [],
    bid_adjustments: finalResult.bid_adjustments || [],
    keyword_opportunities: finalResult.keyword_opportunities || [],
    summary: finalResult.summary || {
      total_spend_analyzed: totalSpend,
      potential_savings: 0,
      optimization_score: 50,
      key_insights: [],
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

  // 创建 AbortController 用于取消
  const abortController = new AbortController();

  const notifyUpdate = () => {
    onSessionUpdate?.(cloneSession(session));
  };

  notifyUpdate();

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
  notifyUpdate();

  // 如果没有需要分析的国家
  if (countries.length === 0) {
    session.status = 'completed';
    session.endTime = Date.now();
    notifyUpdate();
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
    notifyUpdate();

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
      notifyUpdate();

    } catch (error) {
      // 单个国家失败，记录但继续下一个
      const errorMsg = error instanceof Error ? error.message : String(error);
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
      notifyUpdate();
      // 不抛出错误，继续下一个国家
    }
  }

  // 分析完成
  session.currentCountry = undefined;
  session.endTime = Date.now();

  if (countryResults.length === 0) {
    // 所有国家都失败了
    session.status = 'error';
    notifyUpdate();
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
  notifyUpdate();

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

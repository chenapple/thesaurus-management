// 竞品情报 Agent - 深度追踪竞品动态

import { Agent, createAgent } from '../Agent';
import type { AgentConfig, TaskConfig } from '../types';
import { competitorIntelligenceTools } from '../tools/competitor-intelligence-tools';
import type { AIProvider } from '../../types';

// ==================== Agent 配置 ====================

export const COMPETITOR_INTELLIGENCE_AGENT_CONFIG: Omit<AgentConfig, 'provider' | 'model'> = {
  role: '竞品情报分析师',
  goal: '深度追踪竞品动态，监控价格、BSR、评论变化，提供有价值的竞争情报',
  backstory: `你是一位资深的 Amazon 竞争情报分析师，拥有丰富的市场监控经验。
你擅长：
- 监控竞品价格变化和促销策略
- 追踪竞品 BSR 排名趋势
- 分析评论增长和评分变化
- 识别竞品的市场动作和战略意图

你的分析基于数据，结论清晰务实。你能够从细微的变化中发现竞争对手的策略调整，
为卖家提供及时的竞争情报和应对建议。`,
  tools: competitorIntelligenceTools,
  maxIterations: 10,
  temperature: 0.6,
};

// ==================== 创建 Agent 实例 ====================

/**
 * 创建竞品情报 Agent
 */
export function createCompetitorIntelligenceAgent(
  provider: AIProvider = 'deepseek',
  model?: string
): Agent {
  return createAgent({
    ...COMPETITOR_INTELLIGENCE_AGENT_CONFIG,
    provider,
    model,
  });
}

// ==================== 预定义任务 ====================

/**
 * 竞品监控任务配置
 * 抓取最新数据并与历史对比，生成情报报告
 */
export function createCompetitorMonitorTask(
  taskId: number,
  taskName: string,
  marketplace: string,
  myAsin?: string
): TaskConfig {
  return {
    description: `请为竞品监控任务"${taskName}"执行情报收集和分析。

**操作步骤：**
1. 调用 fetch_competitors_batch 工具获取所有竞品的当前数据
   - task_id: ${taskId}
   - marketplace: "${marketplace}"

2. 调用 compare_competitor_history 工具对比历史数据（7天）
   - task_id: ${taskId}
   - days: 7

3. 调用 generate_competitor_report 工具生成分析报告
   - task_id: ${taskId}
   - task_name: "${taskName}"
   - marketplace: "${marketplace}"
   ${myAsin ? `- my_asin: "${myAsin}"` : ''}
   - current_data: 第1步的结果
   - history_changes: 第2步的结果

**完成后：** 返回生成的 HTML 报告内容。`,

    expectedOutput: '包含竞品数据和变化分析的 HTML 报告',

    context: {
      task_id: taskId,
      task_name: taskName,
      marketplace,
      my_asin: myAsin,
      report_date: new Date().toISOString().split('T')[0],
    },
  };
}

/**
 * 快速检查任务配置
 * 只检查是否有重大变化，不生成完整报告
 */
export function createQuickCheckTask(
  taskId: number,
  taskName: string,
  marketplace: string
): TaskConfig {
  return {
    description: `快速检查竞品监控任务"${taskName}"是否有重大变化。

**操作步骤：**
1. 调用 compare_competitor_history 工具对比最近3天的数据
   - task_id: ${taskId}
   - days: 3

2. 分析变化结果，判断是否有需要关注的重大变化：
   - 价格变动超过 10%
   - BSR 排名大幅波动
   - 评论数突然增加
   - 评分明显变化

**完成后：** 简要说明检查结果，指出是否有需要关注的变化。`,

    expectedOutput: '简短的检查摘要（50-100字），说明是否有重大变化及具体内容',

    context: {
      task_id: taskId,
      task_name: taskName,
      marketplace,
    },
  };
}

/**
 * 单品深度分析任务
 * 针对单个 ASIN 进行深度分析
 */
export function createSingleProductAnalysisTask(
  asin: string,
  marketplace: string
): TaskConfig {
  return {
    description: `请对 ASIN "${asin}" 进行深度分析。

**操作步骤：**
1. 调用 fetch_competitor_listing 工具获取该产品的详细信息
   - asin: "${asin}"
   - marketplace: "${marketplace}"

2. 分析获取到的数据，包括：
   - 定价策略（价格区间、是否有折扣）
   - 市场表现（BSR排名、评分、评论数）
   - 产品特点（从标题和描述推断）

**完成后：** 提供该产品的综合分析，包括优势、劣势和可能的竞争策略。`,

    expectedOutput: '产品的综合分析报告（200-300字），包含关键数据和战略建议',

    context: {
      asin,
      marketplace,
    },
  };
}

/**
 * 快速竞品分析任务（不依赖数据库任务）
 * 直接传入 ASIN 列表进行分析
 */
export function createQuickCompetitorAnalysisTask(
  marketplace: string,
  myAsin?: string,
  competitorAsins?: string[]
): TaskConfig {
  const asinList = competitorAsins?.length ? competitorAsins : [];
  const allAsins = myAsin ? [myAsin, ...asinList] : asinList;

  if (allAsins.length === 0) {
    return {
      description: '没有提供任何 ASIN，无法执行分析。',
      expectedOutput: '错误提示',
      context: { marketplace },
    };
  }

  const asinListStr = allAsins.map(a => `"${a}"`).join(', ');

  return {
    description: `请对以下 ASIN 进行竞品分析：${asinListStr}

**操作步骤：**
1. 对每个 ASIN 调用 fetch_competitor_listing 工具获取详细信息
   - marketplace: "${marketplace}"
   - 依次获取每个 ASIN 的数据

2. 汇总分析所有产品数据，包括：
   - 价格对比（最高、最低、平均）
   - BSR 排名对比
   - 评分和评论数对比
   ${myAsin ? `- 重点分析 ${myAsin} 与其他竞品的差异` : ''}

3. 调用 generate_quick_report 工具生成分析报告
   - marketplace: "${marketplace}"
   - products: 获取到的所有产品数据
   ${myAsin ? `- my_asin: "${myAsin}"` : ''}

**完成后：** 返回生成的 HTML 报告内容。`,

    expectedOutput: '包含所有竞品数据和对比分析的 HTML 报告',

    context: {
      marketplace,
      my_asin: myAsin,
      competitor_asins: competitorAsins,
      report_date: new Date().toISOString().split('T')[0],
    },
  };
}

// 市场调研 Agent - 自主发现市场机会和威胁

import { Agent, createAgent } from '../Agent';
import type { AgentConfig, TaskResult } from '../types';
import { marketResearchTools } from '../tools/market-research-tools';
import type { AIProvider } from '../../types';

// ==================== Agent 配置 ====================

export const MARKET_RESEARCH_AGENT_CONFIG: Omit<AgentConfig, 'provider' | 'model'> = {
  role: '市场调研员',
  goal: '主动发现市场机会和威胁，生成有价值的市场洞察',
  backstory: `你是一位资深的 Amazon 市场分析师，拥有 10 年跨境电商经验。
你擅长从海量数据中发现趋势和机会，能够识别：
- 快速崛起的新品和潜力产品
- 市场竞争格局的变化
- 价格策略调整的信号
- 季节性需求变化

你的分析总是数据驱动，结论务实可行。你不会给出空泛的建议，
而是基于具体数据指出明确的机会点和风险点。`,
  tools: marketResearchTools,
  maxIterations: 15,
  temperature: 0.7,
};

// ==================== 创建 Agent 实例 ====================

/**
 * 创建市场调研 Agent
 */
export function createMarketResearchAgent(
  provider: AIProvider = 'deepseek',
  model?: string
): Agent {
  return createAgent({
    ...MARKET_RESEARCH_AGENT_CONFIG,
    provider,
    model,
  });
}

// ==================== 预定义任务 ====================

/**
 * 生成周报任务配置
 */
export function createWeeklyReportTask(
  marketplace: string,
  categoryId: string,
  categoryName: string
) {
  return {
    description: `请为 ${marketplace} 站点的 "${categoryName}" 类目生成本周的市场调研周报。

请按以下步骤进行：

1. **获取当前 BSR 数据**
   - 使用 fetch_bsr_data 工具获取该类目 Top 100 产品数据

2. **分析排名变化**
   - 使用 compare_bsr_history 工具对比上周数据
   - 识别快速上升（排名提升 >20 位）和快速下降的产品

3. **识别新品**
   - 使用 identify_new_products 工具找出本周新进入 Top 100 的产品
   - 评估这些新品的潜力（基于价格、评论数、评分）

4. **分析价格趋势**
   - 使用 analyze_price_trends 工具分析价格变动
   - 标注显著降价/涨价的产品

5. **生成周报**
   - 整合以上数据，生成结构化的 Markdown 周报
   - 包含：BSR 变化概览、新品观察、价格动态、行动建议

6. **保存报告**
   - 使用 save_report 工具保存周报到数据库`,

    expectedOutput: `一份完整的 Markdown 格式周报，包含：
- 标题和日期
- BSR 变化 Top 10（上升/下降）
- 本周新品列表及潜力评估
- 价格变动分析
- 具体可行的行动建议`,

    context: {
      marketplace,
      category_id: categoryId,
      category_name: categoryName,
      report_date: new Date().toISOString().split('T')[0],
    },
  };
}

/**
 * 快速扫描任务配置（不生成完整周报）
 */
export function createQuickScanTask(
  marketplace: string,
  categoryId: string,
  categoryName: string
) {
  return {
    description: `快速扫描 ${marketplace} 站点的 "${categoryName}" 类目，识别本周最值得关注的变化。

只需要：
1. 获取 BSR 数据
2. 对比历史，找出排名变化最大的 5 个产品
3. 简要总结发现

不需要生成完整周报，只需要一个简短的摘要。`,

    expectedOutput: `简短的市场变化摘要（100-200 字），包含：
- 最值得关注的 2-3 个变化
- 是否有需要立即关注的异常`,

    context: {
      marketplace,
      category_id: categoryId,
      category_name: categoryName,
    },
  };
}

// ==================== 批量执行 ====================

/**
 * 批量生成多个类目的周报
 */
export async function generateMultipleCategoryReports(
  categories: { marketplace: string; categoryId: string; categoryName: string }[],
  provider: AIProvider = 'deepseek',
  model?: string,
  onProgress?: (current: number, total: number, categoryName: string) => void
): Promise<{ category: string; result: TaskResult }[]> {
  const agent = createMarketResearchAgent(provider, model);
  const results: { category: string; result: TaskResult }[] = [];

  for (let i = 0; i < categories.length; i++) {
    const { marketplace, categoryId, categoryName } = categories[i];

    onProgress?.(i + 1, categories.length, categoryName);

    const task = createWeeklyReportTask(marketplace, categoryId, categoryName);
    const result = await agent.execute(task);

    results.push({
      category: `${marketplace}/${categoryName}`,
      result,
    });

    // 添加延迟避免 API 限流
    if (i < categories.length - 1) {
      await new Promise(resolve => setTimeout(resolve, 2000));
    }
  }

  return results;
}

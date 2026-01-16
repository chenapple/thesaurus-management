// Agent 框架导出

// 类型
export type {
  Tool,
  ToolDefinition,
  ToolParameter,
  ToolCall,
  ToolResult,
  AgentConfig,
  AgentState,
  AgentMessage,
  TaskConfig,
  TaskResult,
  AgentEvent,
  AgentEventType,
} from './types';

// Agent 基类
export { Agent, createAgent } from './Agent';

// Tool 相关
export {
  ToolExecutor,
  createTool,
  createToolDefinition,
  toOpenAITool,
  toGeminiTool,
} from './Tool';

// AI 服务
export {
  chatWithTools,
  supportsToolCalling,
} from './ai-with-tools';

// 预定义 Agents
export {
  createMarketResearchAgent,
  createWeeklyReportTask,
  createQuickScanTask,
  generateMultipleCategoryReports,
  MARKET_RESEARCH_AGENT_CONFIG,
} from './agents';

// Tools
export { marketResearchTools } from './tools';

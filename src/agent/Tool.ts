// Tool 定义和执行器

import type { Tool, ToolDefinition, ToolCall, ToolResult } from './types';

/**
 * 创建 Tool 定义（用于 AI API 的 tools 参数）
 */
export function createToolDefinition(tool: Tool): ToolDefinition {
  return {
    name: tool.name,
    description: tool.description,
    parameters: tool.parameters,
  };
}

/**
 * 将 Tool 转换为 OpenAI 格式
 */
export function toOpenAITool(tool: Tool): {
  type: 'function';
  function: {
    name: string;
    description: string;
    parameters: object;
  };
} {
  return {
    type: 'function',
    function: {
      name: tool.name,
      description: tool.description,
      parameters: tool.parameters,
    },
  };
}

/**
 * 将 Tool 转换为 Gemini 格式
 */
export function toGeminiTool(tool: Tool): {
  name: string;
  description: string;
  parameters: object;
} {
  return {
    name: tool.name,
    description: tool.description,
    parameters: tool.parameters,
  };
}

/**
 * Tool 执行器
 */
export class ToolExecutor {
  private tools: Map<string, Tool> = new Map();

  constructor(tools: Tool[]) {
    for (const tool of tools) {
      this.tools.set(tool.name, tool);
    }
  }

  /**
   * 获取所有 Tool 定义
   */
  getDefinitions(): ToolDefinition[] {
    return Array.from(this.tools.values()).map(createToolDefinition);
  }

  /**
   * 获取 OpenAI 格式的 tools
   */
  getOpenAITools(): ReturnType<typeof toOpenAITool>[] {
    return Array.from(this.tools.values()).map(toOpenAITool);
  }

  /**
   * 获取 Gemini 格式的 tools
   */
  getGeminiTools(): ReturnType<typeof toGeminiTool>[] {
    return Array.from(this.tools.values()).map(toGeminiTool);
  }

  /**
   * 检查 Tool 是否存在
   */
  hasTool(name: string): boolean {
    return this.tools.has(name);
  }

  /**
   * 执行单个 Tool
   */
  async execute(toolCall: ToolCall): Promise<ToolResult> {
    const tool = this.tools.get(toolCall.name);

    if (!tool) {
      return {
        toolCallId: toolCall.id,
        result: null,
        error: `Tool "${toolCall.name}" not found`,
      };
    }

    try {
      const result = await tool.execute(toolCall.arguments);
      return {
        toolCallId: toolCall.id,
        result,
      };
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      return {
        toolCallId: toolCall.id,
        result: null,
        error: errorMessage,
      };
    }
  }

  /**
   * 批量执行 Tools（串行）
   */
  async executeAll(toolCalls: ToolCall[]): Promise<ToolResult[]> {
    const results: ToolResult[] = [];
    for (const call of toolCalls) {
      const result = await this.execute(call);
      results.push(result);
    }
    return results;
  }

  /**
   * 批量执行 Tools（并行）
   */
  async executeAllParallel(toolCalls: ToolCall[]): Promise<ToolResult[]> {
    return Promise.all(toolCalls.map(call => this.execute(call)));
  }
}

/**
 * 辅助函数：创建简单的 Tool
 */
export function createTool(
  name: string,
  description: string,
  parameters: Tool['parameters'],
  execute: Tool['execute']
): Tool {
  return {
    name,
    description,
    parameters,
    execute,
  };
}

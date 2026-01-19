// Agent 基类 - ReAct 循环实现

import type {
  AgentConfig,
  AgentState,
  AgentMessage,
  TaskConfig,
  TaskResult,
  Tool,
  ToolCall,
  AgentEvent,
} from './types';
import { ToolExecutor } from './Tool';
import { chatWithTools } from './ai-with-tools';
import type { AIProvider } from '../types';

/**
 * Agent 基类
 * 实现 ReAct (Reasoning + Acting) 循环
 */
export class Agent {
  readonly role: string;
  readonly goal: string;
  readonly backstory: string;
  readonly tools: Tool[];
  readonly provider: AIProvider;
  readonly model?: string;
  readonly maxIterations: number;
  readonly temperature: number;

  private toolExecutor: ToolExecutor;
  private state: AgentState;
  private eventListeners: ((event: AgentEvent) => void)[] = [];
  private lastToolResults: Map<string, any> = new Map();

  constructor(config: AgentConfig) {
    this.role = config.role;
    this.goal = config.goal;
    this.backstory = config.backstory;
    this.tools = config.tools;
    this.provider = config.provider || 'deepseek';
    this.model = config.model;
    this.maxIterations = config.maxIterations || 10;
    this.temperature = config.temperature || 0.7;

    this.toolExecutor = new ToolExecutor(this.tools);
    this.state = this.createInitialState();
  }

  /**
   * 创建初始状态
   */
  private createInitialState(): AgentState {
    return {
      messages: [],
      iteration: 0,
      status: 'idle',
    };
  }

  /**
   * 构建系统提示词
   */
  private buildSystemPrompt(task: TaskConfig): string {
    const toolDescriptions = this.tools
      .map(t => `- ${t.name}: ${t.description}`)
      .join('\n');

    return `你是 ${this.role}。

## 你的目标
${this.goal}

## 你的背景
${this.backstory}

## 当前任务
${task.description}

${task.expectedOutput ? `## 期望输出\n${task.expectedOutput}` : ''}

## 可用工具
${toolDescriptions || '（无可用工具）'}

## 工作方式
1. 仔细分析任务需求
2. 思考需要哪些信息来完成任务
3. 如果需要数据，调用相应的工具获取
4. 基于获取的数据进行分析和推理
5. 给出完整的回答或建议

## 重要提示
- 每次只调用必要的工具，避免重复调用
- 获取到数据后要进行分析和总结，不要只是罗列原始数据
- 如果工具调用失败，分析原因并尝试其他方法
- 任务完成后，直接给出最终答案，不要再调用工具`;
  }

  /**
   * 添加事件监听器
   */
  onEvent(listener: (event: AgentEvent) => void): () => void {
    this.eventListeners.push(listener);
    return () => {
      const index = this.eventListeners.indexOf(listener);
      if (index > -1) {
        this.eventListeners.splice(index, 1);
      }
    };
  }

  /**
   * 发送事件
   */
  private emit(type: AgentEvent['type'], data: any): void {
    const event: AgentEvent = {
      type,
      data,
      timestamp: Date.now(),
    };
    for (const listener of this.eventListeners) {
      try {
        listener(event);
      } catch (e) {
        console.error('Event listener error:', e);
      }
    }
  }

  /**
   * 执行任务（主入口）
   */
  async execute(task: TaskConfig, signal?: AbortSignal): Promise<TaskResult> {
    // 重置状态
    this.state = this.createInitialState();
    this.lastToolResults.clear();
    const toolsUsed: string[] = [];

    try {
      // 构建初始消息
      const systemPrompt = this.buildSystemPrompt(task);
      this.state.messages = [
        { role: 'system', content: systemPrompt },
      ];

      // 如果有上下文，添加到消息中
      if (task.context && Object.keys(task.context).length > 0) {
        this.state.messages.push({
          role: 'user',
          content: `参考上下文信息：\n${JSON.stringify(task.context, null, 2)}`,
        });
      }

      // 添加用户任务
      this.state.messages.push({
        role: 'user',
        content: task.description,
      });

      // ReAct 循环
      while (this.state.iteration < this.maxIterations) {
        // 检查是否被取消
        if (signal?.aborted) {
          throw new Error('任务被取消');
        }

        this.state.iteration++;
        this.state.status = 'thinking';
        this.emit('thinking_start', { iteration: this.state.iteration });

        // 调用 LLM
        const response = await chatWithTools(
          this.state.messages,
          this.tools,
          {
            provider: this.provider,
            model: this.model,
            temperature: this.temperature,
            signal,
          }
        );

        // 处理响应
        if (response.finishReason === 'tool_calls' && response.toolCalls) {
          // 需要执行工具
          this.emit('thinking_end', { content: response.content || '' });

          // 添加 assistant 消息（包含 tool_calls）
          this.state.messages.push({
            role: 'assistant',
            content: this.buildToolCallContent(response.content, response.toolCalls),
          });

          // 执行工具
          for (const toolCall of response.toolCalls) {
            this.state.status = 'executing_tool';
            this.state.currentToolCall = toolCall;
            this.emit('tool_call_start', {
              toolName: toolCall.name,
              arguments: toolCall.arguments,
            });

            // 记录使用的工具
            if (!toolsUsed.includes(toolCall.name)) {
              toolsUsed.push(toolCall.name);
            }

            // 执行工具
            const result = await this.toolExecutor.execute(toolCall);

            // 保存 generate_weekly_report 的报告内容，用于直接输出
            if (toolCall.name === 'generate_weekly_report') {
              if (result.result?.report_content) {
                this.lastToolResults.set('report_content', result.result.report_content);
                console.log('[Agent] 已捕获报告内容，长度:', result.result.report_content.length);
              } else if (result.error) {
                console.error('[Agent] generate_weekly_report 执行失败:', result.error);
              } else {
                console.warn('[Agent] generate_weekly_report 返回结果中没有 report_content', result.result);
              }
            }

            this.emit('tool_call_end', {
              toolName: toolCall.name,
              result: result.result,
              error: result.error,
            });

            // 添加工具结果消息
            this.state.messages.push({
              role: 'tool',
              content: [
                {
                  type: 'tool_result',
                  tool_use_id: toolCall.id,
                  content: result.error
                    ? `Error: ${result.error}`
                    : JSON.stringify(result.result, null, 2),
                  is_error: !!result.error,
                },
              ],
            });
          }
        } else {
          // 任务完成（没有更多工具调用）
          this.state.status = 'completed';

          // 优先使用工具返回的报告内容（避免 AI 重复输出被截断）
          const reportContent = this.lastToolResults.get('report_content');
          const finalOutput = reportContent || response.content || '';

          // 日志：检查最终输出
          if (!finalOutput || finalOutput.trim().length === 0) {
            console.warn('[Agent] 任务完成但输出为空', {
              hasReportContent: !!reportContent,
              hasResponseContent: !!response.content,
              toolsUsed,
            });
          } else {
            console.log('[Agent] 任务完成，输出长度:', finalOutput.length, '来源:', reportContent ? 'tool' : 'response');
          }

          this.emit('thinking_end', { content: response.content || '' });

          const taskResult: TaskResult = {
            success: true,
            output: finalOutput,
            toolsUsed,
            iterations: this.state.iteration,
          };

          this.emit('task_complete', taskResult);
          return taskResult;
        }
      }

      // 达到最大迭代次数
      const lastAssistantMsg = this.state.messages
        .filter(m => m.role === 'assistant')
        .pop();

      const output = typeof lastAssistantMsg?.content === 'string'
        ? lastAssistantMsg.content
        : '达到最大迭代次数，任务未完成';

      const taskResult: TaskResult = {
        success: false,
        output,
        error: '达到最大迭代次数',
        toolsUsed,
        iterations: this.state.iteration,
      };

      this.emit('task_complete', taskResult);
      return taskResult;

    } catch (error) {
      this.state.status = 'error';
      const errorMessage = error instanceof Error ? error.message : String(error);

      this.emit('error', { message: errorMessage });

      return {
        success: false,
        output: '',
        error: errorMessage,
        toolsUsed,
        iterations: this.state.iteration,
      };
    }
  }

  /**
   * 构建包含 tool_calls 的消息内容
   */
  private buildToolCallContent(text: string | null, toolCalls: ToolCall[]): AgentMessage['content'] {
    const content: any[] = [];

    if (text) {
      content.push({ type: 'text', text });
    }

    for (const tc of toolCalls) {
      content.push({
        type: 'tool_use',
        id: tc.id,
        name: tc.name,
        input: tc.arguments,
      });
    }

    return content;
  }

  /**
   * 获取当前状态（用于调试）
   */
  getState(): AgentState {
    return { ...this.state };
  }

  /**
   * 获取消息历史
   */
  getMessages(): AgentMessage[] {
    return [...this.state.messages];
  }
}

/**
 * 创建 Agent 的工厂函数
 */
export function createAgent(config: AgentConfig): Agent {
  return new Agent(config);
}

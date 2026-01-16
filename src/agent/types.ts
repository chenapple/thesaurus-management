// Agent 框架类型定义

import type { AIProvider } from '../types';

// ==================== Tool 相关类型 ====================

export interface ToolParameter {
  type: 'string' | 'number' | 'boolean' | 'object' | 'array';
  description: string;
  required?: boolean;
  enum?: string[];
  items?: ToolParameter;  // 用于 array 类型
  properties?: Record<string, ToolParameter>;  // 用于 object 类型
}

export interface ToolDefinition {
  name: string;
  description: string;
  parameters: {
    type: 'object';
    properties: Record<string, ToolParameter>;
    required?: string[];
  };
}

export interface Tool extends ToolDefinition {
  execute: (params: Record<string, any>) => Promise<any>;
}

export interface ToolCall {
  id: string;
  name: string;
  arguments: Record<string, any>;
}

export interface ToolResult {
  toolCallId: string;
  result: any;
  error?: string;
}

// ==================== Message 相关类型 ====================

export interface TextContent {
  type: 'text';
  text: string;
}

export interface ToolUseContent {
  type: 'tool_use';
  id: string;
  name: string;
  input: Record<string, any>;
}

export interface ToolResultContent {
  type: 'tool_result';
  tool_use_id: string;
  content: string;
  is_error?: boolean;
}

export type MessageContent = TextContent | ToolUseContent | ToolResultContent;

export interface AgentMessage {
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string | MessageContent[];
}

// ==================== Agent 相关类型 ====================

export interface AgentConfig {
  role: string;
  goal: string;
  backstory: string;
  tools: Tool[];
  provider?: AIProvider;
  model?: string;
  maxIterations?: number;
  temperature?: number;
}

export interface AgentState {
  messages: AgentMessage[];
  iteration: number;
  status: 'idle' | 'thinking' | 'executing_tool' | 'completed' | 'error';
  currentToolCall?: ToolCall;
}

export interface TaskConfig {
  description: string;
  expectedOutput?: string;
  context?: Record<string, any>;
}

export interface TaskResult {
  success: boolean;
  output: string;
  error?: string;
  toolsUsed: string[];
  iterations: number;
}

// ==================== Event 相关类型（用于流式输出）====================

export type AgentEventType =
  | 'thinking_start'
  | 'thinking_chunk'
  | 'thinking_end'
  | 'tool_call_start'
  | 'tool_call_end'
  | 'task_complete'
  | 'error';

export interface AgentEvent {
  type: AgentEventType;
  data: any;
  timestamp: number;
}

export interface ThinkingStartEvent extends AgentEvent {
  type: 'thinking_start';
  data: { iteration: number };
}

export interface ThinkingChunkEvent extends AgentEvent {
  type: 'thinking_chunk';
  data: { content: string };
}

export interface ThinkingEndEvent extends AgentEvent {
  type: 'thinking_end';
  data: { content: string };
}

export interface ToolCallStartEvent extends AgentEvent {
  type: 'tool_call_start';
  data: { toolName: string; arguments: Record<string, any> };
}

export interface ToolCallEndEvent extends AgentEvent {
  type: 'tool_call_end';
  data: { toolName: string; result: any; error?: string };
}

export interface TaskCompleteEvent extends AgentEvent {
  type: 'task_complete';
  data: TaskResult;
}

export interface ErrorEvent extends AgentEvent {
  type: 'error';
  data: { message: string; details?: any };
}

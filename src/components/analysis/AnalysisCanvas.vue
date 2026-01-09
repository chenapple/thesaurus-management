<template>
  <div class="analysis-canvas-container">
    <VueFlow
      :nodes="nodes"
      :edges="edges"
      :default-viewport="{ x: 0, y: 0, zoom: 0.9 }"
      :min-zoom="0.5"
      :max-zoom="1.5"
      fit-view-on-init
      class="analysis-flow"
    >
      <Background :gap="20" :size="1" />
      <Controls :show-interactive="false" position="bottom-right" />

      <!-- 自定义节点 -->
      <template #node-dataSource="nodeProps">
        <DataSourceNode v-bind="nodeProps" />
      </template>
      <template #node-agent="nodeProps">
        <AgentNode v-bind="nodeProps" />
      </template>
      <template #node-output="nodeProps">
        <OutputNode v-bind="nodeProps" />
      </template>
    </VueFlow>

    <!-- 流式输出预览 -->
    <div v-if="streamingContent" class="streaming-preview">
      <div class="streaming-header">
        <span class="streaming-title">AI 输出中...</span>
        <span class="typing-cursor">▋</span>
      </div>
      <pre class="streaming-content">{{ streamingContent.slice(-300) }}</pre>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { VueFlow } from '@vue-flow/core';
import { Background } from '@vue-flow/background';
import { Controls } from '@vue-flow/controls';
import '@vue-flow/core/dist/style.css';
import '@vue-flow/core/dist/theme-default.css';
import '@vue-flow/controls/dist/style.css';

import DataSourceNode from './nodes/DataSourceNode.vue';
import AgentNode from './nodes/AgentNode.vue';
import OutputNode from './nodes/OutputNode.vue';

export type AnalysisStatus = {
  step: number;  // 0=未开始, 1=评论洞察, 2=文案分析, 3=优化建议
  reviewInsightsCompleted: boolean;
  listingAnalysisCompleted: boolean;
  optimizationCompleted: boolean;
  hasReviews: boolean;
  hasCompetitors: boolean;
  hasKeywords: boolean;
  isParallel?: boolean;  // 是否并行模式
  parallelStep1Done?: boolean;
  parallelStep2Done?: boolean;
};

const props = defineProps<{
  status: AnalysisStatus;
  streamingContent: string;
}>();

// 节点状态计算
function getNodeStatus(nodeId: string): 'pending' | 'running' | 'completed' | 'disabled' {
  const { step, reviewInsightsCompleted, listingAnalysisCompleted, optimizationCompleted, isParallel, parallelStep1Done, parallelStep2Done } = props.status;

  switch (nodeId) {
    case 'reviews':
      return props.status.hasReviews ? 'completed' : 'disabled';
    case 'competitors':
      return props.status.hasCompetitors ? 'completed' : 'disabled';
    case 'keywords':
      return props.status.hasKeywords ? 'completed' : 'disabled';

    case 'review-analyst':
      if (reviewInsightsCompleted) return 'completed';
      if (isParallel) {
        if (step >= 1 && !parallelStep1Done) return 'running';
        if (parallelStep1Done) return 'completed';
      } else {
        if (step === 1) return 'running';
        if (step > 1) return 'completed';
      }
      return 'pending';

    case 'listing-expert':
      if (listingAnalysisCompleted) return 'completed';
      if (isParallel) {
        if (step >= 1 && !parallelStep2Done) return 'running';
        if (parallelStep2Done) return 'completed';
      } else {
        if (step === 2) return 'running';
        if (step > 2) return 'completed';
      }
      return 'pending';

    case 'optimizer':
      if (optimizationCompleted) return 'completed';
      if (step === 3) return 'running';
      return 'pending';

    case 'output':
      if (optimizationCompleted) return 'completed';
      return 'pending';

    default:
      return 'pending';
  }
}

// 节点定义
const nodes = computed(() => [
  // 数据源节点 - 顶部
  {
    id: 'reviews',
    type: 'dataSource',
    position: { x: 50, y: 0 },
    data: {
      label: '竞品评论池',
      icon: 'comment',
      status: getNodeStatus('reviews'),
    },
  },
  {
    id: 'competitors',
    type: 'dataSource',
    position: { x: 250, y: 0 },
    data: {
      label: '竞品 Listing',
      icon: 'document',
      status: getNodeStatus('competitors'),
    },
  },
  {
    id: 'keywords',
    type: 'dataSource',
    position: { x: 450, y: 0 },
    data: {
      label: '关键词数据',
      icon: 'key',
      status: getNodeStatus('keywords'),
    },
  },

  // 智能体节点 - 中部
  {
    id: 'review-analyst',
    type: 'agent',
    position: { x: 50, y: 150 },
    data: {
      label: '评论分析师',
      description: '提取使用场景、痛点、爽点',
      status: getNodeStatus('review-analyst'),
      stepNumber: 1,
    },
  },
  {
    id: 'listing-expert',
    type: 'agent',
    position: { x: 300, y: 150 },
    data: {
      label: '文案专家',
      description: '分析标题结构、五点主题',
      status: getNodeStatus('listing-expert'),
      stepNumber: 2,
    },
  },

  // 优化生成器 - 下方
  {
    id: 'optimizer',
    type: 'agent',
    position: { x: 175, y: 300 },
    data: {
      label: '优化策略生成器',
      description: '整合生成标题、五点、描述建议',
      status: getNodeStatus('optimizer'),
      stepNumber: 3,
    },
  },

  // 输出节点 - 底部
  {
    id: 'output',
    type: 'output',
    position: { x: 175, y: 450 },
    data: {
      label: '最终输出',
      description: '标题 / 五点 / 描述建议',
      status: getNodeStatus('output'),
    },
  },
]);

// 获取边的动画状态
function getEdgeAnimated(sourceStatus: string, targetStatus: string): boolean {
  return sourceStatus === 'completed' && targetStatus === 'running';
}

// 边定义
const edges = computed(() => {
  const reviewStatus = getNodeStatus('review-analyst');
  const listingStatus = getNodeStatus('listing-expert');
  const optimizerStatus = getNodeStatus('optimizer');

  return [
    // 数据源 → 智能体
    {
      id: 'e-reviews-analyst',
      source: 'reviews',
      target: 'review-analyst',
      animated: getEdgeAnimated('completed', reviewStatus),
      style: { stroke: reviewStatus === 'running' ? 'var(--el-color-primary)' : 'var(--el-border-color)' },
    },
    {
      id: 'e-competitors-analyst',
      source: 'competitors',
      target: 'review-analyst',
      animated: getEdgeAnimated('completed', reviewStatus),
      style: { stroke: reviewStatus === 'running' ? 'var(--el-color-primary)' : 'var(--el-border-color)' },
    },
    {
      id: 'e-competitors-expert',
      source: 'competitors',
      target: 'listing-expert',
      animated: getEdgeAnimated('completed', listingStatus),
      style: { stroke: listingStatus === 'running' ? 'var(--el-color-primary)' : 'var(--el-border-color)' },
    },

    // 智能体 → 优化器
    {
      id: 'e-analyst-optimizer',
      source: 'review-analyst',
      target: 'optimizer',
      animated: getEdgeAnimated(reviewStatus, optimizerStatus),
      style: { stroke: optimizerStatus === 'running' ? 'var(--el-color-primary)' : 'var(--el-border-color)' },
    },
    {
      id: 'e-expert-optimizer',
      source: 'listing-expert',
      target: 'optimizer',
      animated: getEdgeAnimated(listingStatus, optimizerStatus),
      style: { stroke: optimizerStatus === 'running' ? 'var(--el-color-primary)' : 'var(--el-border-color)' },
    },
    {
      id: 'e-keywords-optimizer',
      source: 'keywords',
      target: 'optimizer',
      animated: getEdgeAnimated('completed', optimizerStatus),
      style: { stroke: optimizerStatus === 'running' ? 'var(--el-color-primary)' : 'var(--el-border-color)' },
    },

    // 优化器 → 输出
    {
      id: 'e-optimizer-output',
      source: 'optimizer',
      target: 'output',
      animated: optimizerStatus === 'completed',
      style: { stroke: optimizerStatus === 'completed' ? 'var(--el-color-success)' : 'var(--el-border-color)' },
    },
  ];
});
</script>

<style scoped>
.analysis-canvas-container {
  position: relative;
  width: 100%;
  height: 550px;
  border: 1px solid var(--el-border-color);
  border-radius: 8px;
  background: var(--el-bg-color);
  overflow: hidden;
}

.analysis-flow {
  width: 100%;
  height: 100%;
}

/* 流式输出预览 */
.streaming-preview {
  position: absolute;
  bottom: 16px;
  left: 16px;
  right: 16px;
  max-height: 120px;
  background: var(--el-bg-color-overlay);
  border: 1px solid var(--el-border-color);
  border-radius: 8px;
  padding: 12px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
  z-index: 10;
}

.streaming-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.streaming-title {
  font-size: 12px;
  color: var(--el-color-primary);
  font-weight: 500;
}

.typing-cursor {
  animation: blink 1s infinite;
  color: var(--el-color-primary);
}

@keyframes blink {
  0%, 50% { opacity: 1; }
  51%, 100% { opacity: 0; }
}

.streaming-content {
  margin: 0;
  font-size: 11px;
  line-height: 1.4;
  color: var(--el-text-color-secondary);
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 60px;
  overflow-y: auto;
}

/* Vue Flow 控制按钮样式覆盖 */
:deep(.vue-flow__controls) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  border-radius: 6px;
  overflow: hidden;
}

:deep(.vue-flow__controls-button) {
  background: var(--el-bg-color);
  border: none;
  color: var(--el-text-color-primary);
}

:deep(.vue-flow__controls-button:hover) {
  background: var(--el-fill-color-light);
}

:deep(.vue-flow__background) {
  background: var(--el-bg-color-page);
}
</style>

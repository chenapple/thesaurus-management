<template>
  <div :class="['agent-node', `status-${data.status}`]">
    <Handle type="target" :position="Position.Top" />
    <Handle type="source" :position="Position.Bottom" />

    <div class="node-header">
      <span class="step-badge">{{ data.stepNumber }}</span>
      <span class="node-title">{{ data.label }}</span>
    </div>

    <div class="node-description">{{ data.description }}</div>

    <div class="status-bar">
      <div v-if="data.status === 'running'" class="running-indicator">
        <span class="pulse-dot"></span>
        <span class="status-text">执行中...</span>
      </div>
      <div v-else-if="data.status === 'completed'" class="completed-indicator">
        <span class="check-icon">✓</span>
        <span class="status-text">已完成</span>
      </div>
      <div v-else class="pending-indicator">
        <span class="wait-icon">○</span>
        <span class="status-text">等待中</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Handle, Position } from '@vue-flow/core';

defineProps<{
  data: {
    label: string;
    description: string;
    status: 'pending' | 'running' | 'completed' | 'disabled';
    stepNumber: number;
  };
}>();
</script>

<style scoped>
.agent-node {
  display: flex;
  flex-direction: column;
  padding: 14px 16px;
  background: var(--el-bg-color);
  border: 2px solid var(--el-border-color);
  border-radius: 12px;
  min-width: 180px;
  max-width: 200px;
  transition: all 0.3s ease;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.05);
}

.agent-node.status-running {
  border-color: var(--el-color-primary);
  box-shadow: 0 0 0 3px var(--el-color-primary-light-8), 0 4px 16px rgba(0, 0, 0, 0.1);
  animation: pulse-border 2s ease-in-out infinite;
}

.agent-node.status-completed {
  border-color: var(--el-color-success);
  background: var(--el-color-success-light-9);
}

.agent-node.status-pending {
  border-style: dashed;
}

@keyframes pulse-border {
  0%, 100% {
    box-shadow: 0 0 0 3px var(--el-color-primary-light-8), 0 4px 16px rgba(0, 0, 0, 0.1);
  }
  50% {
    box-shadow: 0 0 0 6px var(--el-color-primary-light-9), 0 4px 16px rgba(0, 0, 0, 0.1);
  }
}

.node-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.step-badge {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: var(--el-color-primary);
  color: white;
  font-size: 11px;
  font-weight: bold;
  display: flex;
  align-items: center;
  justify-content: center;
}

.status-completed .step-badge {
  background: var(--el-color-success);
}

.node-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.node-description {
  font-size: 11px;
  color: var(--el-text-color-secondary);
  line-height: 1.4;
  margin-bottom: 10px;
}

.status-bar {
  padding-top: 8px;
  border-top: 1px solid var(--el-border-color-lighter);
}

.running-indicator,
.completed-indicator,
.pending-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
}

.status-text {
  color: var(--el-text-color-secondary);
}

.running-indicator .status-text {
  color: var(--el-color-primary);
}

.completed-indicator .status-text {
  color: var(--el-color-success);
}

.pulse-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--el-color-primary);
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    transform: scale(1);
    opacity: 1;
  }
  50% {
    transform: scale(1.3);
    opacity: 0.7;
  }
}

.check-icon {
  color: var(--el-color-success);
  font-weight: bold;
}

.wait-icon {
  color: var(--el-text-color-disabled);
}

:deep(.vue-flow__handle) {
  width: 10px;
  height: 10px;
  background: var(--el-border-color);
  border: 2px solid var(--el-bg-color);
}

.status-running :deep(.vue-flow__handle) {
  background: var(--el-color-primary);
}

.status-completed :deep(.vue-flow__handle) {
  background: var(--el-color-success);
}
</style>

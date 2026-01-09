<template>
  <div :class="['output-node', `status-${data.status}`]">
    <Handle type="target" :position="Position.Top" />

    <div class="node-icon">
      <span v-if="data.status === 'completed'">🎉</span>
      <span v-else>📋</span>
    </div>
    <div class="node-label">{{ data.label }}</div>
    <div class="node-description">{{ data.description }}</div>

    <div v-if="data.status === 'completed'" class="completed-badge">
      生成完毕
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
  };
}>();
</script>

<style scoped>
.output-node {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 16px 20px;
  background: var(--el-bg-color);
  border: 2px dashed var(--el-border-color);
  border-radius: 16px;
  min-width: 160px;
  transition: all 0.3s ease;
}

.output-node.status-completed {
  border-style: solid;
  border-color: var(--el-color-success);
  background: linear-gradient(135deg, var(--el-color-success-light-9), var(--el-color-primary-light-9));
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
}

.node-icon {
  font-size: 28px;
  margin-bottom: 8px;
}

.node-label {
  font-size: 14px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin-bottom: 4px;
}

.node-description {
  font-size: 11px;
  color: var(--el-text-color-secondary);
  text-align: center;
}

.completed-badge {
  margin-top: 10px;
  padding: 4px 12px;
  background: var(--el-color-success);
  color: white;
  font-size: 11px;
  font-weight: 500;
  border-radius: 12px;
  animation: pop-in 0.3s ease;
}

@keyframes pop-in {
  0% {
    transform: scale(0.8);
    opacity: 0;
  }
  100% {
    transform: scale(1);
    opacity: 1;
  }
}

:deep(.vue-flow__handle) {
  width: 10px;
  height: 10px;
  background: var(--el-border-color);
  border: 2px solid var(--el-bg-color);
}

.status-completed :deep(.vue-flow__handle) {
  background: var(--el-color-success);
}
</style>

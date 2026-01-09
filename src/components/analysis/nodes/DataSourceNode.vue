<template>
  <div :class="['data-source-node', `status-${data.status}`]">
    <Handle type="source" :position="Position.Bottom" />
    <div class="node-icon">
      <span v-if="data.icon === 'comment'">💬</span>
      <span v-else-if="data.icon === 'document'">📄</span>
      <span v-else-if="data.icon === 'key'">🔑</span>
    </div>
    <div class="node-label">{{ data.label }}</div>
    <div class="status-indicator">
      <span v-if="data.status === 'completed'" class="icon-done">✓</span>
      <span v-else-if="data.status === 'disabled'" class="icon-disabled">—</span>
      <span v-else class="icon-ready">○</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Handle, Position } from '@vue-flow/core';

defineProps<{
  data: {
    label: string;
    icon: string;
    status: 'pending' | 'running' | 'completed' | 'disabled';
  };
}>();
</script>

<style scoped>
.data-source-node {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px 16px;
  background: var(--el-bg-color);
  border: 2px solid var(--el-border-color);
  border-radius: 12px;
  min-width: 120px;
  transition: all 0.3s ease;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

.data-source-node.status-completed {
  border-color: var(--el-color-success);
  background: var(--el-color-success-light-9);
}

.data-source-node.status-disabled {
  border-color: var(--el-border-color-lighter);
  opacity: 0.6;
}

.node-icon {
  font-size: 24px;
  margin-bottom: 6px;
}

.node-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--el-text-color-primary);
  text-align: center;
}

.status-indicator {
  position: absolute;
  top: -6px;
  right: -6px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color);
}

.icon-done {
  color: var(--el-color-success);
  font-weight: bold;
}

.icon-disabled {
  color: var(--el-text-color-disabled);
}

.icon-ready {
  color: var(--el-text-color-secondary);
}

:deep(.vue-flow__handle) {
  width: 8px;
  height: 8px;
  background: var(--el-border-color);
  border: none;
}

.status-completed :deep(.vue-flow__handle) {
  background: var(--el-color-success);
}
</style>

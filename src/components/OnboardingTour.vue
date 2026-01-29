<script setup lang="ts">
import { onUnmounted, watch } from 'vue';
import { useOnboardingTour, type ViewMode } from '../composables/useOnboardingTour';

const props = defineProps<{
  trigger?: boolean;
}>();

const emit = defineEmits<{
  (e: 'started'): void;
  (e: 'completed'): void;
  (e: 'switch-view', mode: ViewMode): void;
}>();

const { isRunning, startTour, stopTour, restartTour } = useOnboardingTour({
  onSwitchView: (mode) => {
    emit('switch-view', mode);
  },
});

// 监听 trigger 变化来启动教程
watch(() => props.trigger, (newVal) => {
  if (newVal) {
    startTour();
    emit('started');
  }
});

// 监听教程运行状态
watch(isRunning, (running, wasRunning) => {
  if (wasRunning && !running) {
    emit('completed');
  }
});

// 暴露方法给父组件
defineExpose({
  startTour,
  stopTour,
  restartTour,
});

onUnmounted(() => {
  stopTour();
});
</script>

<template>
  <!-- 无需渲染任何内容，Driver.js 会自动创建覆盖层 -->
</template>

<style>
/* Driver.js 自定义样式 */
.driver-popover.onboarding-popover {
  background: var(--bg-secondary, #fff);
  color: var(--text-primary, #303133);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
  max-width: 360px;
}

.driver-popover.onboarding-popover .driver-popover-title {
  font-size: 18px;
  font-weight: 600;
  font-family: 'Poppins', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  color: var(--text-primary, #303133);
  margin-bottom: 8px;
}

.driver-popover.onboarding-popover .driver-popover-description {
  font-size: 14px;
  line-height: 1.6;
  color: var(--text-secondary, #606266);
}

.driver-popover.onboarding-popover .driver-popover-progress-text {
  font-size: 12px;
  color: var(--text-muted, #909399);
}

.driver-popover.onboarding-popover .driver-popover-navigation-btns {
  gap: 8px;
}

.driver-popover.onboarding-popover .driver-popover-prev-btn {
  background: var(--bg-hover, #f5f7fa);
  color: var(--text-primary, #303133);
  border: 1px solid var(--border-color, #e4e7ed);
  border-radius: 6px;
  padding: 8px 16px;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
}

.driver-popover.onboarding-popover .driver-popover-prev-btn:hover {
  background: var(--bg-active, #ecf5ff);
  border-color: var(--el-color-primary-light-5, #a0cfff);
}

.driver-popover.onboarding-popover .driver-popover-next-btn {
  background: var(--el-color-primary, #409eff);
  color: #fff;
  border: none;
  border-radius: 6px;
  padding: 8px 16px;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
}

.driver-popover.onboarding-popover .driver-popover-next-btn:hover {
  background: var(--el-color-primary-light-3, #79bbff);
}

.driver-popover.onboarding-popover .driver-popover-close-btn {
  color: var(--text-muted, #909399);
}

.driver-popover.onboarding-popover .driver-popover-close-btn:hover {
  color: var(--text-primary, #303133);
}

/* 暗色模式适配 */
html.dark .driver-popover.onboarding-popover {
  background: var(--bg-secondary, #242424);
  border: 1px solid var(--border-color, #3a3a3a);
}

html.dark .driver-popover.onboarding-popover .driver-popover-title {
  color: var(--text-primary, #e5e5e5);
}

html.dark .driver-popover.onboarding-popover .driver-popover-description {
  color: var(--text-secondary, #a3a3a3);
}

html.dark .driver-popover.onboarding-popover .driver-popover-prev-btn {
  background: var(--bg-hover, #2c2c2c);
  border-color: var(--border-color, #3a3a3a);
  color: var(--text-primary, #e5e5e5);
}

html.dark .driver-popover.onboarding-popover .driver-popover-prev-btn:hover {
  background: var(--bg-active, #1a3a5c);
}

/* Driver.js 高亮区域样式 */
.driver-overlay {
  background: rgba(0, 0, 0, 0.6) !important;
}

html.dark .driver-overlay {
  background: rgba(0, 0, 0, 0.75) !important;
}
</style>

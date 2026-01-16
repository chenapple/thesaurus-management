<template>
  <Teleport to="body">
    <Transition name="notification-slide">
      <div v-if="visible" class="global-notification" :class="{ success: notification?.success, error: !notification?.success }">
        <div class="notification-header">
          <div class="header-left">
            <el-icon :size="20" :class="notification?.success ? 'success-icon' : 'error-icon'">
              <component :is="notification?.success ? CircleCheck : CircleClose" />
            </el-icon>
            <span class="notification-title">
              {{ notification?.success ? '市场调研完成' : '市场调研失败' }}
            </span>
          </div>
          <el-button class="close-btn" text @click="close">
            <el-icon><Close /></el-icon>
          </el-button>
        </div>
        <div class="notification-body">
          <div class="task-name">{{ notification?.taskName }}</div>
          <div v-if="notification?.success && notification?.summary" class="summary">
            <pre>{{ notification.summary }}</pre>
          </div>
          <div v-else-if="notification?.error" class="error-message">
            {{ notification.error }}
          </div>
        </div>
        <div class="notification-footer">
          <el-button @click="close">关闭</el-button>
          <el-button v-if="notification?.success" type="primary" @click="viewDetails">
            查看详情
          </el-button>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { Close, CircleCheck, CircleClose } from '@element-plus/icons-vue';
import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';

interface MarketResearchNotification {
  taskId: number;
  taskName: string;
  runId: number;
  success: boolean;
  summary?: string;
  error?: string;
}

const emit = defineEmits<{
  (e: 'viewDetails', notification: MarketResearchNotification): void;
}>();

const visible = ref(false);
const notification = ref<MarketResearchNotification | null>(null);
let unlisten: UnlistenFn | null = null;

function show(data: MarketResearchNotification) {
  notification.value = data;
  visible.value = true;
}

function close() {
  visible.value = false;
  notification.value = null;
}

function viewDetails() {
  if (notification.value) {
    emit('viewDetails', notification.value);
  }
  close();
}

onMounted(async () => {
  unlisten = await listen<{
    task_id: number;
    task_name: string;
    run_id: number;
    success: boolean;
    summary?: string;
    error?: string;
  }>('market_research_complete', (event) => {
    show({
      taskId: event.payload.task_id,
      taskName: event.payload.task_name,
      runId: event.payload.run_id,
      success: event.payload.success,
      summary: event.payload.summary,
      error: event.payload.error,
    });
  });
});

onUnmounted(() => {
  if (unlisten) {
    unlisten();
  }
});
</script>

<style scoped>
.global-notification {
  position: fixed;
  top: 20px;
  right: 20px;
  width: 380px;
  background: var(--el-bg-color);
  border-radius: 12px;
  box-shadow: 0 6px 24px rgba(0, 0, 0, 0.15);
  z-index: 9999;
  overflow: hidden;
}

.global-notification.success {
  border-left: 4px solid var(--el-color-success);
}

.global-notification.error {
  border-left: 4px solid var(--el-color-danger);
}

.notification-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.success-icon {
  color: var(--el-color-success);
}

.error-icon {
  color: var(--el-color-danger);
}

.notification-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.close-btn {
  padding: 4px;
  margin: -4px;
}

.notification-body {
  padding: 16px;
}

.task-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 8px;
}

.summary {
  background: var(--el-fill-color-light);
  padding: 10px 12px;
  border-radius: 6px;
  font-size: 13px;
  color: var(--el-text-color-regular);
}

.summary pre {
  margin: 0;
  font-family: inherit;
  white-space: pre-wrap;
  line-height: 1.5;
}

.error-message {
  color: var(--el-color-danger);
  font-size: 13px;
  padding: 10px 12px;
  background: var(--el-color-danger-light-9);
  border-radius: 6px;
}

.notification-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--el-border-color-lighter);
  background: var(--el-fill-color-light);
}

/* Animation */
.notification-slide-enter-active,
.notification-slide-leave-active {
  transition: all 0.3s ease-out;
}

.notification-slide-enter-from {
  opacity: 0;
  transform: translateX(100%);
}

.notification-slide-leave-to {
  opacity: 0;
  transform: translateX(100%);
}
</style>

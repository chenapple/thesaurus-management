<template>
  <el-dialog
    :model-value="modelValue"
    title="自动检测设置"
    width="500px"
    @update:model-value="$emit('update:modelValue', $event)"
    @close="handleClose"
  >
    <el-form :model="settings" label-width="120px" v-loading="loading">
      <!-- 启用开关 -->
      <el-form-item label="自动检测">
        <el-switch
          v-model="settings.enabled"
          active-text="启用"
          inactive-text="关闭"
        />
        <div class="form-tip">
          启用后将在指定时间自动检测关键词排名
        </div>
      </el-form-item>

      <el-divider content-position="left">检测时间</el-divider>

      <!-- 早间时间窗口 -->
      <el-form-item label="早间检测">
        <div class="time-range">
          <el-input-number
            v-model="settings.morning_start"
            :min="0"
            :max="12"
            :step="1"
            size="small"
            style="width: 80px"
          />
          <span class="time-separator">至</span>
          <el-input-number
            v-model="settings.morning_end"
            :min="settings.morning_start"
            :max="12"
            :step="1"
            size="small"
            style="width: 80px"
          />
          <span class="time-unit">点</span>
        </div>
        <div class="form-tip">目标站点当地时间</div>
      </el-form-item>

      <!-- 晚间时间窗口 -->
      <el-form-item label="晚间检测">
        <div class="time-range">
          <el-input-number
            v-model="settings.evening_start"
            :min="12"
            :max="23"
            :step="1"
            size="small"
            style="width: 80px"
          />
          <span class="time-separator">至</span>
          <el-input-number
            v-model="settings.evening_end"
            :min="settings.evening_start"
            :max="24"
            :step="1"
            size="small"
            style="width: 80px"
          />
          <span class="time-unit">点</span>
        </div>
        <div class="form-tip">目标站点当地时间</div>
      </el-form-item>

      <el-divider content-position="left">通知设置</el-divider>

      <!-- 排名变化阈值 -->
      <el-form-item label="变化阈值">
        <el-input-number
          v-model="settings.rank_change_threshold"
          :min="1"
          :max="100"
          :step="5"
          size="small"
          style="width: 120px"
        />
        <span class="threshold-unit">位</span>
        <div class="form-tip">排名变化超过此值时发送通知</div>
      </el-form-item>

      <!-- 通知类型 -->
      <el-form-item label="通知类型">
        <div class="notify-options">
          <el-checkbox v-model="settings.notify_on_enter_top10">
            进入 Top 10
          </el-checkbox>
          <el-checkbox v-model="settings.notify_on_exit_top10">
            跌出 Top 10
          </el-checkbox>
          <el-checkbox v-model="settings.notify_on_new_rank">
            新上榜
          </el-checkbox>
          <el-checkbox v-model="settings.notify_on_lost_rank">
            跌出榜单
          </el-checkbox>
        </div>
      </el-form-item>

      <!-- 状态显示 -->
      <el-divider content-position="left">运行状态</el-divider>

      <el-form-item label="调度器状态">
        <el-tag :type="status.is_running ? 'success' : 'info'" size="small">
          {{ status.is_running ? '运行中' : '已停止' }}
        </el-tag>
      </el-form-item>

      <el-form-item v-if="status.last_check_time" label="上次检测">
        {{ formatDateTime(status.last_check_time) }}
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button type="primary" :loading="saving" @click="handleSave">
        保存设置
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue';
import { ElMessage } from 'element-plus';
import {
  getSchedulerSettings,
  updateSchedulerSettings,
  getSchedulerStatus,
  startScheduler,
  stopScheduler,
} from '../api';
import type { SchedulerSettings, SchedulerStatus } from '../types';
import { DEFAULT_SCHEDULER_SETTINGS } from '../types';

const props = defineProps<{
  modelValue: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>();

const loading = ref(false);
const saving = ref(false);

const settings = reactive<SchedulerSettings>({ ...DEFAULT_SCHEDULER_SETTINGS });

const status = reactive<SchedulerStatus>({
  is_running: false,
  last_check_time: null,
  next_check_time: null,
  current_task: null,
});

// 加载设置
async function loadSettings() {
  loading.value = true;
  try {
    const [savedSettings, savedStatus] = await Promise.all([
      getSchedulerSettings(),
      getSchedulerStatus(),
    ]);

    Object.assign(settings, savedSettings);
    Object.assign(status, savedStatus);
  } catch (e) {
    console.error('加载设置失败:', e);
  } finally {
    loading.value = false;
  }
}

// 保存设置
async function handleSave() {
  saving.value = true;
  try {
    await updateSchedulerSettings({ ...settings });

    // 根据设置启动或停止调度器
    if (settings.enabled) {
      await startScheduler();
    } else {
      await stopScheduler();
    }

    // 刷新状态
    const newStatus = await getSchedulerStatus();
    Object.assign(status, newStatus);

    ElMessage.success('设置已保存');
    emit('update:modelValue', false);
  } catch (e) {
    ElMessage.error(`保存失败: ${e}`);
  } finally {
    saving.value = false;
  }
}

// 关闭
function handleClose() {
  emit('update:modelValue', false);
}

// 格式化时间
function formatDateTime(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}

// 监听对话框打开
watch(() => props.modelValue, (val) => {
  if (val) {
    loadSettings();
  }
});
</script>

<style scoped>
.form-tip {
  font-size: 12px;
  color: var(--el-text-color-placeholder);
  margin-top: 4px;
}

.time-range {
  display: flex;
  align-items: center;
  gap: 8px;
}

.time-separator {
  color: var(--el-text-color-regular);
}

.time-unit {
  color: var(--el-text-color-secondary);
  font-size: 13px;
}

.threshold-unit {
  margin-left: 8px;
  color: var(--el-text-color-secondary);
  font-size: 13px;
}

.notify-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.el-divider {
  margin: 20px 0 16px;
}

.el-divider :deep(.el-divider__text) {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}
</style>

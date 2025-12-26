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

      <!-- 任务记录 -->
      <el-divider content-position="left">
        任务记录
        <el-button link type="primary" size="small" @click="loadTaskLogs" style="margin-left: 8px;">
          刷新
        </el-button>
      </el-divider>

      <div class="task-logs" v-loading="loadingLogs">
        <el-empty v-if="taskLogs.length === 0" description="暂无任务记录" :image-size="60" />
        <el-table v-else :data="taskLogs" size="small" max-height="200">
          <el-table-column label="时间" width="140">
            <template #default="{ row }">
              {{ formatDateTime(row.started_at) }}
            </template>
          </el-table-column>
          <el-table-column label="状态" width="80">
            <template #default="{ row }">
              <el-tag
                :type="row.status === 'completed' ? 'success' : row.status === 'running' ? 'warning' : 'danger'"
                size="small"
              >
                {{ row.status === 'completed' ? '完成' : row.status === 'running' ? '进行中' : '失败' }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="关键词" width="80">
            <template #default="{ row }">
              {{ row.total_keywords }}
            </template>
          </el-table-column>
          <el-table-column label="成功/失败" width="90">
            <template #default="{ row }">
              <span class="success-count">{{ row.success_count }}</span>
              /
              <span class="failed-count">{{ row.failed_count }}</span>
            </template>
          </el-table-column>
          <el-table-column label="耗时">
            <template #default="{ row }">
              {{ row.ended_at ? formatDuration(row.started_at, row.ended_at) : '-' }}
            </template>
          </el-table-column>
        </el-table>
      </div>
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
  getTaskLogs,
} from '../api';
import type { SchedulerSettings, SchedulerStatus, TaskLog } from '../types';
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

const loadingLogs = ref(false);
const taskLogs = ref<TaskLog[]>([]);

// 加载任务记录
async function loadTaskLogs() {
  loadingLogs.value = true;
  try {
    taskLogs.value = await getTaskLogs(10);
  } catch (e) {
    console.error('加载任务记录失败:', e);
  } finally {
    loadingLogs.value = false;
  }
}

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

    // 同时加载任务记录
    loadTaskLogs();
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

// 格式化耗时
function formatDuration(startStr: string, endStr: string): string {
  const start = new Date(startStr);
  const end = new Date(endStr);
  const diffMs = end.getTime() - start.getTime();
  const diffSec = Math.floor(diffMs / 1000);

  if (diffSec < 60) {
    return `${diffSec}秒`;
  } else if (diffSec < 3600) {
    const min = Math.floor(diffSec / 60);
    const sec = diffSec % 60;
    return `${min}分${sec}秒`;
  } else {
    const hour = Math.floor(diffSec / 3600);
    const min = Math.floor((diffSec % 3600) / 60);
    return `${hour}小时${min}分`;
  }
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

.task-logs {
  margin-top: 8px;
  margin-bottom: 8px;
}

.success-count {
  color: var(--el-color-success);
}

.failed-count {
  color: var(--el-color-danger);
}
</style>

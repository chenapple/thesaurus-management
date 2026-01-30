<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import {
  TrendCharts,
  CircleCheck,
  Warning,
  Clock,
  Refresh,
  Edit,
  VideoPlay,
  Loading,
} from '@element-plus/icons-vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import type { MarketResearchTask, MarketResearchRun } from '../types';
import { getCountryLabel, getCountryFlag } from '../types';
import { renderSimpleMarkdown } from '../utils/sanitize';

// Emits
const emit = defineEmits<{
  (e: 'runTask', task: MarketResearchTask): void;
  (e: 'editTask', task: MarketResearchTask): void;
}>();

// 数据状态
const loading = ref(false);
const tasks = ref<MarketResearchTask[]>([]);
const recentRuns = ref<MarketResearchRun[]>([]);

// 抽屉相关
const drawerVisible = ref(false);
const selectedTask = ref<MarketResearchTask | null>(null);
const taskRuns = ref<MarketResearchRun[]>([]);
const selectedRunId = ref<number | null>(null);
const loadingRuns = ref(false);

// 加载所有数据
async function loadData() {
  loading.value = true;
  try {
    const [tasksResult, runsResult] = await Promise.all([
      invoke<MarketResearchTask[]>('get_market_research_tasks'),
      invoke<MarketResearchRun[]>('get_latest_research_runs', { limit: 20 }),
    ]);
    tasks.value = tasksResult;
    recentRuns.value = runsResult;
  } catch (error) {
    console.error('加载数据失败:', error);
    ElMessage.error('加载数据失败');
  } finally {
    loading.value = false;
  }
}

// 统计数据
const stats = computed(() => {
  const enabledTasks = tasks.value.filter(t => t.is_enabled);
  const completedRuns = recentRuns.value.filter(r => r.status === 'completed');
  const failedRuns = recentRuns.value.filter(r => r.status === 'failed');

  // 按站点统计
  const byMarketplace = new Map<string, number>();
  for (const task of enabledTasks) {
    byMarketplace.set(task.marketplace, (byMarketplace.get(task.marketplace) || 0) + 1);
  }

  return {
    totalTasks: tasks.value.length,
    activeTasks: enabledTasks.length,
    completedRuns: completedRuns.length,
    failedRuns: failedRuns.length,
    byMarketplace: Array.from(byMarketplace.entries()).map(([code, count]) => ({
      code,
      label: getCountryLabel(code),
      flag: getCountryFlag(code),
      count,
    })),
  };
});

// 按站点分组的任务
const tasksByMarketplace = computed(() => {
  const groups = new Map<string, MarketResearchTask[]>();
  for (const task of tasks.value) {
    const list = groups.get(task.marketplace) || [];
    list.push(task);
    groups.set(task.marketplace, list);
  }

  // 转换为数组并按站点排序
  return Array.from(groups.entries())
    .map(([marketplace, taskList]) => ({
      marketplace,
      label: getCountryLabel(marketplace),
      flag: getCountryFlag(marketplace),
      tasks: taskList,
    }))
    .sort((a, b) => a.marketplace.localeCompare(b.marketplace));
});

// 打开任务详情抽屉
async function openTaskDrawer(task: MarketResearchTask) {
  selectedTask.value = task;
  drawerVisible.value = true;
  loadingRuns.value = true;

  try {
    const runs = await invoke<MarketResearchRun[]>('get_research_runs_by_task', {
      taskId: task.id,
      limit: 20,
    });
    taskRuns.value = runs;
    // 默认选中最新的运行记录
    if (runs.length > 0) {
      selectedRunId.value = runs[0].id;
    } else {
      selectedRunId.value = null;
    }
  } catch (error) {
    console.error('加载运行记录失败:', error);
    ElMessage.error('加载运行记录失败');
  } finally {
    loadingRuns.value = false;
  }
}

// 当前选中的运行记录
const selectedRun = computed(() => {
  if (!selectedRunId.value) return null;
  return taskRuns.value.find(r => r.id === selectedRunId.value) || null;
});

// 格式化日期时间
function formatDateTime(dateStr: string | undefined): string {
  if (!dateStr) return '-';
  try {
    // 处理不同格式
    let normalizedStr = dateStr;
    if (dateStr.includes(' ') && !dateStr.includes('T')) {
      normalizedStr = dateStr.replace(' ', 'T') + 'Z';
    }
    const date = new Date(normalizedStr);
    if (isNaN(date.getTime())) return '-';
    return date.toLocaleString('zh-CN', {
      month: 'numeric',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  } catch {
    return '-';
  }
}

// 格式化相对时间
function formatRelativeTime(dateStr: string | undefined): string {
  if (!dateStr) return '从未运行';
  try {
    let normalizedStr = dateStr;
    if (dateStr.includes(' ') && !dateStr.includes('T')) {
      normalizedStr = dateStr.replace(' ', 'T') + 'Z';
    }
    const date = new Date(normalizedStr);
    if (isNaN(date.getTime())) return '从未运行';

    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffMins < 1) return '刚刚';
    if (diffMins < 60) return `${diffMins} 分钟前`;
    if (diffHours < 24) return `${diffHours} 小时前`;
    if (diffDays < 7) return `${diffDays} 天前`;
    return date.toLocaleDateString('zh-CN', { month: 'numeric', day: 'numeric' });
  } catch {
    return '从未运行';
  }
}

// 获取状态标签类型
function getStatusType(status: string | undefined): 'success' | 'warning' | 'danger' | 'info' {
  switch (status) {
    case 'completed': return 'success';
    case 'running': return 'warning';
    case 'failed': return 'danger';
    default: return 'info';
  }
}

// 获取状态文字
function getStatusText(status: string | undefined): string {
  switch (status) {
    case 'completed': return '成功';
    case 'running': return '运行中';
    case 'failed': return '失败';
    default: return '未运行';
  }
}

// 运行任务
function handleRunTask(task: MarketResearchTask) {
  emit('runTask', task);
  drawerVisible.value = false;
}

// 编辑任务
function handleEditTask(task: MarketResearchTask) {
  emit('editTask', task);
}

// Markdown 渲染 - 使用安全的 sanitize 工具
function renderMarkdown(text: string): string {
  return renderSimpleMarkdown(text);
}

// 刷新数据
function refresh() {
  loadData();
}

// 挂载时加载数据
onMounted(() => {
  loadData();
});

// 暴露刷新方法给父组件
defineExpose({
  refresh,
});
</script>

<template>
  <div class="dashboard-container" v-loading="loading">
    <!-- 概览统计区 -->
    <div class="stats-grid">
      <!-- 活跃任务 -->
      <div class="modern-card stat-card">
        <div class="stat-top">
          <div class="icon-circle bg-blue-light">
            <el-icon class="text-blue"><TrendCharts /></el-icon>
          </div>
          <span class="stat-title">活跃任务</span>
        </div>
        <div class="stat-main">
          <span class="stat-number">{{ stats.activeTasks }}</span>
          <span class="stat-total">/ {{ stats.totalTasks }}</span>
        </div>
      </div>

      <!-- 近期成功 -->
      <div class="modern-card stat-card">
        <div class="stat-top">
          <div class="icon-circle bg-green-light">
            <el-icon class="text-green"><CircleCheck /></el-icon>
          </div>
          <span class="stat-title">近期成功</span>
        </div>
        <div class="stat-main">
          <span class="stat-number text-green">{{ stats.completedRuns }}</span>
        </div>
      </div>

      <!-- 近期失败 -->
      <div class="modern-card stat-card">
        <div class="stat-top">
          <div class="icon-circle bg-red-light">
            <el-icon class="text-red"><Warning /></el-icon>
          </div>
          <span class="stat-title">近期失败</span>
        </div>
        <div class="stat-main">
          <span class="stat-number text-red">{{ stats.failedRuns }}</span>
        </div>
      </div>

      <!-- 站点分布 -->
      <div class="modern-card stat-card marketplace-card">
        <div class="stat-top">
          <span class="stat-title">站点分布</span>
        </div>
        <div class="marketplace-list">
          <div
            v-for="item in stats.byMarketplace"
            :key="item.code"
            class="marketplace-item"
          >
            <span class="marketplace-flag" v-html="item.flag"></span>
            <span class="marketplace-count">{{ item.count }}</span>
          </div>
          <div v-if="stats.byMarketplace.length === 0" class="empty-text">
            暂无任务
          </div>
        </div>
      </div>
    </div>

    <!-- 任务卡片网格 (按站点分组) -->
    <div v-if="tasksByMarketplace.length > 0" class="task-groups">
      <div
        v-for="group in tasksByMarketplace"
        :key="group.marketplace"
        class="task-group"
      >
        <div class="group-header">
          <span class="group-flag" v-html="group.flag"></span>
          <span class="group-title">{{ group.label }}</span>
          <span class="group-count">{{ group.tasks.length }} 个任务</span>
        </div>

        <div class="task-grid">
          <div
            v-for="task in group.tasks"
            :key="task.id"
            class="task-card modern-card hover-effect"
            :class="{ disabled: !task.is_enabled }"
            @click="openTaskDrawer(task)"
          >
            <div class="task-header">
              <div class="task-name">{{ task.category_name || task.category_id }}</div>
              <el-tag
                :type="task.is_enabled ? getStatusType(task.last_run_status) : 'info'"
                size="small"
              >
                {{ task.is_enabled ? getStatusText(task.last_run_status) : '已禁用' }}
              </el-tag>
            </div>
            <div class="task-meta">
              <div class="task-category-id">{{ task.category_id }}</div>
              <div class="task-time">
                <el-icon><Clock /></el-icon>
                {{ formatRelativeTime(task.last_run_at) }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else class="empty-state">
      <el-empty description="暂无监控任务">
        <template #default>
          <p class="empty-hint">创建监控任务后，可以在这里查看所有任务的运行状态和历史报告</p>
        </template>
      </el-empty>
    </div>

    <!-- 报告抽屉 -->
    <el-drawer
      v-model="drawerVisible"
      :title="selectedTask?.name || '任务详情'"
      size="60%"
      direction="rtl"
    >
      <template #header>
        <div class="drawer-header">
          <div class="drawer-title">
            <span class="task-flag" v-html="getCountryFlag(selectedTask?.marketplace || '')"></span>
            <span>{{ selectedTask?.name || '任务详情' }}</span>
          </div>
          <div class="drawer-actions">
            <el-button
              :icon="VideoPlay"
              size="small"
              @click="handleRunTask(selectedTask!)"
            >
              立即运行
            </el-button>
            <el-button
              :icon="Edit"
              size="small"
              @click="handleEditTask(selectedTask!)"
            >
              编辑
            </el-button>
          </div>
        </div>
      </template>

      <div class="drawer-content" v-loading="loadingRuns">
        <!-- 历史版本选择 -->
        <div v-if="taskRuns.length > 0" class="history-selector">
          <span class="selector-label">历史版本:</span>
          <el-select
            v-model="selectedRunId"
            placeholder="选择版本"
            style="width: 280px"
          >
            <el-option
              v-for="run in taskRuns"
              :key="run.id"
              :value="run.id"
              :label="`${formatDateTime(run.started_at)} - ${getStatusText(run.status)}`"
            >
              <div class="run-option">
                <el-tag :type="getStatusType(run.status)" size="small">
                  {{ getStatusText(run.status) }}
                </el-tag>
                <span class="run-time">{{ formatDateTime(run.started_at) }}</span>
              </div>
            </el-option>
          </el-select>
          <el-button :icon="Refresh" circle size="small" @click="loadData" title="刷新" />
        </div>

        <!-- 报告内容 -->
        <div v-if="selectedRun" class="report-container">
          <div v-if="selectedRun.status === 'running'" class="running-state">
            <el-icon class="spinning" :size="32"><Loading /></el-icon>
            <p>任务正在运行中...</p>
          </div>

          <div v-else-if="selectedRun.status === 'failed'" class="error-state">
            <el-alert
              type="error"
              :title="selectedRun.error_message || '执行失败'"
              show-icon
              :closable="false"
            />
          </div>

          <div v-else-if="selectedRun.report_content" class="report-content">
            <div class="markdown-content" v-html="renderMarkdown(selectedRun.report_content)"></div>
          </div>

          <div v-else class="empty-report">
            <el-empty description="暂无报告内容" />
          </div>
        </div>

        <!-- 无运行记录 -->
        <div v-else-if="!loadingRuns && taskRuns.length === 0" class="no-runs">
          <el-empty description="暂无运行记录">
            <el-button type="primary" @click="handleRunTask(selectedTask!)">
              立即运行
            </el-button>
          </el-empty>
        </div>
      </div>
    </el-drawer>
  </div>
</template>

<style scoped>
.dashboard-container {
  padding: 20px;
  min-height: 100%;
  box-sizing: border-box;
}

/* Stats Grid */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 20px;
  margin-bottom: 24px;
}

/* Modern Card */
.modern-card {
  background: var(--glass-bg);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border-radius: 16px;
  box-shadow: var(--glass-shadow);
  padding: 20px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  border: 1px solid var(--glass-border);
}

.modern-card.hover-effect:hover {
  transform: translateY(-4px);
  box-shadow: var(--glass-shadow-hover);
  cursor: pointer;
  background: rgba(255, 255, 255, 0.85);
}

html.dark .modern-card.hover-effect:hover {
  background: rgba(30, 41, 59, 0.85);
}

/* Stat Card */
.stat-card {
  display: flex;
  flex-direction: column;
}

.stat-top {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.icon-circle {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
}

.bg-blue-light {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.15) 0%, rgba(59, 130, 246, 0.1) 100%);
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.15);
}
.text-blue { color: #2563EB; }

.bg-green-light {
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.15) 0%, rgba(52, 211, 153, 0.1) 100%);
  box-shadow: 0 4px 12px rgba(16, 185, 129, 0.15);
}
.text-green { color: #10B981; }

.bg-red-light {
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.15) 0%, rgba(248, 113, 113, 0.1) 100%);
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.15);
}
.text-red { color: #EF4444; }

.stat-title {
  color: var(--el-text-color-secondary);
  font-size: 14px;
  font-weight: 500;
}

.stat-main {
  display: flex;
  align-items: baseline;
  gap: 6px;
}

.stat-number {
  font-family: 'Poppins', sans-serif;
  font-size: 28px;
  font-weight: 700;
  color: var(--el-text-color-primary);
  line-height: 1;
}

.stat-total {
  font-size: 14px;
  color: var(--el-text-color-placeholder);
  font-weight: 400;
}

/* Marketplace Card */
.marketplace-card {
  padding: 16px 20px;
}

.marketplace-list {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-top: 8px;
}

.marketplace-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  background: rgba(0, 0, 0, 0.04);
  border-radius: 8px;
}

html.dark .marketplace-item {
  background: rgba(255, 255, 255, 0.08);
}

.marketplace-flag {
  display: inline-flex;
  width: 20px;
  height: 14px;
  border-radius: 2px;
  overflow: hidden;
}

.marketplace-flag :deep(svg) {
  width: 100%;
  height: 100%;
}

.marketplace-count {
  font-family: 'Poppins', sans-serif;
  font-size: 14px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

/* Task Groups */
.task-groups {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.task-group {
  /* No extra styles needed */
}

.group-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.group-flag {
  display: inline-flex;
  width: 24px;
  height: 16px;
  border-radius: 3px;
  overflow: hidden;
}

.group-flag :deep(svg) {
  width: 100%;
  height: 100%;
}

.group-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.group-count {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

/* Task Grid */
.task-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.task-card {
  padding: 16px;
}

.task-card.disabled {
  opacity: 0.6;
}

.task-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 8px;
  margin-bottom: 10px;
}

.task-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  line-height: 1.3;
  flex: 1;
  word-break: break-word;
}

.task-meta {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.task-category-id {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  font-family: monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-time {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--el-text-color-placeholder);
}

/* Empty State */
.empty-state {
  padding: 60px 20px;
  text-align: center;
}

.empty-hint {
  font-size: 13px;
  color: var(--el-text-color-secondary);
  margin-top: 8px;
}

.empty-text {
  font-size: 13px;
  color: var(--el-text-color-placeholder);
}

/* Drawer */
.drawer-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.drawer-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 18px;
  font-weight: 600;
}

.task-flag {
  display: inline-flex;
  width: 24px;
  height: 16px;
  border-radius: 3px;
  overflow: hidden;
}

.task-flag :deep(svg) {
  width: 100%;
  height: 100%;
}

.drawer-actions {
  display: flex;
  gap: 8px;
}

.drawer-content {
  height: 100%;
  display: flex;
  flex-direction: column;
}

/* History Selector */
.history-selector {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
  margin-bottom: 16px;
}

.selector-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-regular);
}

.run-option {
  display: flex;
  align-items: center;
  gap: 10px;
}

.run-time {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

/* Report Container */
.report-container {
  flex: 1;
  overflow: auto;
}

.running-state,
.error-state,
.empty-report,
.no-runs {
  padding: 40px;
  text-align: center;
}

.running-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  color: var(--el-text-color-secondary);
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* Report Content */
.report-content {
  padding: 4px;
}

.markdown-content {
  line-height: 1.6;
  background: #ffffff;
  padding: 20px;
  border-radius: 12px;
  color: #1f2937;
}

.markdown-content :deep(h2) {
  font-size: 18px;
  margin: 16px 0 8px;
  border-bottom: 1px solid #e5e7eb;
  padding-bottom: 4px;
}

.markdown-content :deep(h3) {
  font-size: 16px;
  margin: 12px 0 6px;
}

.markdown-content :deep(h4) {
  font-size: 14px;
  margin: 10px 0 4px;
}

.markdown-content :deep(h5) {
  font-size: 14px;
  font-weight: 600;
  margin: 10px 0 4px;
  color: #374151;
}

.markdown-content :deep(ul) {
  padding-left: 20px;
  margin: 8px 0;
}

.markdown-content :deep(li) {
  margin: 4px 0;
}

.markdown-content :deep(pre) {
  background: #f3f4f6;
  padding: 12px;
  border-radius: 6px;
  overflow-x: auto;
}

.markdown-content :deep(code) {
  font-family: monospace;
  font-size: 13px;
}

/* Responsive */
@media (max-width: 1200px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: 1fr;
  }

  .task-grid {
    grid-template-columns: 1fr;
  }
}
</style>

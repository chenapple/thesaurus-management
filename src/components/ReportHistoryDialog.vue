<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Search, Calendar, Delete, View } from '@element-plus/icons-vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import * as api from '../api';
import type { WeeklyReport } from '../types';

const props = defineProps<{
  modelValue: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'select', report: WeeklyReport): void;
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (v) => emit('update:modelValue', v),
});

const loading = ref(false);
const reports = ref<WeeklyReport[]>([]);
const searchText = ref('');

// 加载周报列表
async function loadReports() {
  loading.value = true;
  try {
    reports.value = await api.listWeeklyReports(50, searchText.value || undefined);
  } catch (error) {
    console.error('加载周报列表失败:', error);
    ElMessage.error('加载周报列表失败');
  } finally {
    loading.value = false;
  }
}

// 对话框打开时加载
watch(visible, (v) => {
  if (v) {
    searchText.value = '';
    loadReports();
  }
});

// 搜索
function handleSearch() {
  loadReports();
}

// 选择周报
function selectReport(report: WeeklyReport) {
  emit('select', report);
}

// 删除周报
async function deleteReport(report: WeeklyReport) {
  try {
    await ElMessageBox.confirm(
      `确定删除 "${report.title}" 吗？此操作将同时删除周报中的所有条目。`,
      '确认删除',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );
    await api.deleteWeeklyReport(report.week_start);
    await loadReports();
    ElMessage.success('已删除');
  } catch (e) {
    if (e !== 'cancel') {
      ElMessage.error('删除失败');
    }
  }
}

// 格式化日期范围
function formatDateRange(report: WeeklyReport): string {
  return `${report.week_start} ~ ${report.week_end}`;
}
</script>

<template>
  <el-dialog
    v-model="visible"
    title="历史周报"
    width="700px"
    :close-on-click-modal="false"
  >
    <!-- 搜索栏 -->
    <div class="search-bar">
      <el-input
        v-model="searchText"
        placeholder="搜索周报标题或内容..."
        :prefix-icon="Search"
        clearable
        @keyup.enter="handleSearch"
        @clear="handleSearch"
      />
      <el-button :icon="Search" @click="handleSearch">搜索</el-button>
    </div>

    <!-- 周报列表 -->
    <div class="report-list" v-loading="loading">
      <el-empty v-if="reports.length === 0" description="暂无历史周报" />
      <div
        v-for="report in reports"
        :key="report.id"
        class="report-item"
        @click="selectReport(report)"
      >
        <div class="report-main">
          <div class="report-title">
            <el-icon><Calendar /></el-icon>
            {{ report.title }}
          </div>
          <div class="report-meta">
            <span class="date-range">{{ formatDateRange(report) }}</span>
          </div>
          <div class="report-summary" v-if="report.summary">
            {{ report.summary.slice(0, 100) }}{{ report.summary.length > 100 ? '...' : '' }}
          </div>
        </div>
        <div class="report-actions">
          <el-button :icon="View" size="small" @click.stop="selectReport(report)">
            查看
          </el-button>
          <el-button :icon="Delete" size="small" type="danger" @click.stop="deleteReport(report)">
            删除
          </el-button>
        </div>
      </div>
    </div>

    <template #footer>
      <el-button @click="visible = false">关闭</el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.search-bar {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
}

.search-bar .el-input {
  flex: 1;
}

.report-list {
  max-height: 400px;
  overflow-y: auto;
}

.report-item {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 16px;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 8px;
  margin-bottom: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.report-item:hover {
  border-color: var(--el-color-primary-light-5);
  background: var(--el-fill-color-light);
}

.report-item:last-child {
  margin-bottom: 0;
}

.report-main {
  flex: 1;
}

.report-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin-bottom: 8px;
}

.report-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.date-range {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.report-summary {
  font-size: 13px;
  color: var(--el-text-color-regular);
  line-height: 1.5;
}

.report-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-left: 16px;
}
</style>

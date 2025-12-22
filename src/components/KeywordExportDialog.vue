<script setup lang="ts">
import { Download } from '@element-plus/icons-vue';

interface ColumnDef {
  key: string;
  label: string;
  default?: boolean;
  required?: boolean;
}

const props = defineProps<{
  visible: boolean;
  scope: 'filtered' | 'all';
  keywordTotal: number;
  hasActiveFilters: boolean;
  columnDefinitions: ColumnDef[];
  columnConfig: Record<string, boolean>;
  loading: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
  (e: 'update:scope', value: 'filtered' | 'all'): void;
  (e: 'export'): void;
  (e: 'editColumns'): void;
}>();

function handleScopeChange(value: 'filtered' | 'all') {
  emit('update:scope', value);
}

function handleExport() {
  emit('export');
}

function handleEditColumns() {
  emit('editColumns');
  emit('update:visible', false);
}

const enabledColumns = () => {
  return props.columnDefinitions.filter(c => props.columnConfig[c.key]);
};
</script>

<template>
  <el-dialog
    :model-value="visible"
    @update:model-value="$emit('update:visible', $event)"
    title="导出关键词数据"
    width="480px"
  >
    <div class="keyword-export-settings">
      <p class="export-desc">选择导出范围（将使用当前列配置）</p>

      <!-- 导出范围选择 -->
      <div class="export-section">
        <h4 class="section-title">导出范围</h4>
        <el-radio-group :model-value="scope" @update:model-value="handleScopeChange" class="export-scope-group">
          <el-radio value="filtered" class="export-scope-item">
            <div class="scope-content">
              <span class="scope-label">导出当前筛选结果</span>
              <span class="scope-count">
                共 {{ keywordTotal }} 条数据
                <el-tag v-if="hasActiveFilters" size="small" type="info" style="margin-left: 8px">已筛选</el-tag>
              </span>
            </div>
          </el-radio>
          <el-radio value="all" class="export-scope-item">
            <div class="scope-content">
              <span class="scope-label">导出全部数据</span>
              <span class="scope-count">忽略当前筛选条件</span>
            </div>
          </el-radio>
        </el-radio-group>
      </div>

      <!-- 列配置预览 -->
      <div class="export-section">
        <h4 class="section-title">
          导出列
          <el-button type="primary" link size="small" @click="handleEditColumns">
            修改列配置
          </el-button>
        </h4>
        <div class="export-columns-preview">
          <el-tag
            v-for="col in enabledColumns()"
            :key="col.key"
            size="small"
            type="info"
          >
            {{ col.label }}
          </el-tag>
        </div>
        <p class="columns-count">
          共 {{ enabledColumns().length }} 列
        </p>
      </div>
    </div>

    <template #footer>
      <el-button @click="$emit('update:visible', false)">取消</el-button>
      <el-button
        type="primary"
        :loading="loading"
        @click="handleExport"
      >
        <el-icon v-if="!loading"><Download /></el-icon>
        {{ loading ? '导出中...' : '导出' }}
      </el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.keyword-export-settings {
  padding: 0 10px;
}

.export-desc {
  color: var(--el-text-color-secondary);
  font-size: 14px;
  margin-bottom: 20px;
}

.export-section {
  margin-bottom: 24px;
}

.section-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.export-scope-group {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 100%;
}

.export-scope-item {
  display: flex;
  align-items: flex-start;
  padding: 12px 16px;
  border: 1px solid var(--el-border-color);
  border-radius: 8px;
  transition: all 0.2s;
  width: 100%;
  margin-right: 0 !important;
  height: auto !important;
}

.export-scope-item :deep(.el-radio__label) {
  flex: 1;
  white-space: normal;
  line-height: 1.5;
  padding-left: 8px;
}

.export-scope-item:hover {
  border-color: var(--el-color-primary-light-5);
}

.export-scope-item.is-checked {
  border-color: var(--el-color-primary);
  background: var(--el-color-primary-light-9);
}

.scope-content {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.scope-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.scope-count {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  display: flex;
  align-items: center;
}

.export-columns-preview {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  max-height: 120px;
  overflow-y: auto;
  padding: 12px;
  background: var(--el-fill-color-light);
  border-radius: 6px;
}

.columns-count {
  margin-top: 8px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  text-align: right;
}
</style>

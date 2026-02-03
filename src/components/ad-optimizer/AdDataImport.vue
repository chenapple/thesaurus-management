<template>
  <div class="ad-data-import">
    <div
      class="drop-zone"
      :class="{ 'is-dragging': isDragging }"
      @dragover.prevent="isDragging = true"
      @dragleave.prevent="isDragging = false"
      @drop.prevent="handleDrop"
      @click="triggerFileInput"
    >
      <input
        ref="fileInputRef"
        type="file"
        accept=".xlsx,.xls,.csv"
        style="display: none"
        @change="handleFileChange"
      />

      <div v-if="!importing" class="drop-content">
        <el-icon class="upload-icon"><Upload /></el-icon>
        <div class="drop-text">
          <p>拖拽亚马逊搜索词报告到此处</p>
          <p class="sub-text">或点击选择文件 (.xlsx, .xls, .csv)</p>
        </div>
      </div>

      <div v-else class="import-progress">
        <el-progress
          :percentage="progress"
          :status="progressStatus"
          :stroke-width="10"
        />
        <p class="progress-text">{{ progressText }}</p>
      </div>
    </div>

    <!-- 导入预览 -->
    <el-dialog v-model="showPreview" title="导入搜索词报告" width="80%">
      <div class="preview-stats">
        <el-alert v-if="parseWarnings.length > 0" type="warning" :closable="false">
          <template #title>
            <div v-for="warning in parseWarnings" :key="warning">{{ warning }}</div>
          </template>
        </el-alert>

        <div class="stats-row">
          <div class="stat-item">
            <span class="stat-label">解析成功</span>
            <span class="stat-value">{{ parsedData.length }} 条</span>
          </div>
          <div class="stat-item" v-if="dateRange.start">
            <span class="stat-label">日期范围</span>
            <span class="stat-value date-range">{{ dateRange.start }} ~ {{ dateRange.end }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">有花费</span>
            <span class="stat-value">{{ parseStats.withSpend }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">有销售</span>
            <span class="stat-value">{{ parseStats.withSales }}</span>
          </div>
        </div>

        <!-- 导入模式选择 -->
        <div class="import-mode-section">
          <div class="mode-label">导入模式：</div>
          <el-radio-group v-model="importMode">
            <el-radio value="replace">
              <span class="mode-title">替换全部</span>
              <span class="mode-desc">清空现有数据，导入新数据</span>
            </el-radio>
            <el-radio value="append">
              <span class="mode-title">追加合并</span>
              <span class="mode-desc">保留现有数据，只添加新数据（已存在的记录不会被覆盖）</span>
            </el-radio>
          </el-radio-group>
        </div>
      </div>

      <el-table :data="previewData" height="400" style="width: 100%">
        <el-table-column prop="customer_search_term" label="搜索词" width="200" />
        <el-table-column prop="campaign_name" label="广告活动" width="150" />
        <el-table-column prop="targeting" label="投放词" width="150" />
        <el-table-column prop="match_type" label="匹配类型" width="80" />
        <el-table-column prop="impressions" label="展示" width="80" />
        <el-table-column prop="clicks" label="点击" width="60" />
        <el-table-column prop="spend" label="花费" width="80">
          <template #default="{ row }">
            ${{ row.spend.toFixed(2) }}
          </template>
        </el-table-column>
        <el-table-column prop="sales" label="销售" width="80">
          <template #default="{ row }">
            ${{ row.sales.toFixed(2) }}
          </template>
        </el-table-column>
        <el-table-column prop="acos" label="ACOS" width="80">
          <template #default="{ row }">
            {{ row.acos.toFixed(1) }}%
          </template>
        </el-table-column>
      </el-table>

      <template #footer>
        <el-button @click="showPreview = false">取消</el-button>
        <el-button type="primary" @click="confirmImport" :loading="importing">
          确认导入 ({{ parsedData.length }} 条)
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { ElMessage } from 'element-plus';
import { Upload } from '@element-plus/icons-vue';
import type { AdSearchTerm } from '../../types';
import { parseAdExcel, validateParseResult, isSupportedFileType } from '../../utils/ad-parser';
import { adImportSearchTerms } from '../../api';

const props = defineProps<{
  projectId?: number;
}>();

const emit = defineEmits<{
  (e: 'imported'): void;
}>();

const fileInputRef = ref<HTMLInputElement | null>(null);
const isDragging = ref(false);
const importing = ref(false);
const progress = ref(0);
const progressStatus = ref<'' | 'success' | 'exception'>('');
const progressText = ref('');

const showPreview = ref(false);
const parsedData = ref<AdSearchTerm[]>([]);
const parseWarnings = ref<string[]>([]);
const parseStats = ref({
  total: 0,
  withSpend: 0,
  withSales: 0,
  withOrders: 0,
});

// 导入模式：replace（替换全部）或 append（追加合并）
const importMode = ref<'replace' | 'append'>('replace');

// 日期范围统计
const dateRange = ref({
  start: '',
  end: '',
});

const previewData = computed(() => parsedData.value.slice(0, 100));

function triggerFileInput() {
  fileInputRef.value?.click();
}

function handleDrop(e: DragEvent) {
  isDragging.value = false;
  const files = e.dataTransfer?.files;
  if (files && files.length > 0) {
    processFile(files[0]);
  }
}

function handleFileChange(e: Event) {
  const target = e.target as HTMLInputElement;
  const files = target.files;
  if (files && files.length > 0) {
    processFile(files[0]);
  }
}

async function processFile(file: File) {
  if (!isSupportedFileType(file.name)) {
    ElMessage.error('不支持的文件类型，请上传 Excel 或 CSV 文件');
    return;
  }

  importing.value = true;
  progress.value = 20;
  progressText.value = '正在解析文件...';

  try {
    // 解析 Excel
    const data = await parseAdExcel(file);
    progress.value = 60;
    progressText.value = '验证数据...';

    // 验证结果
    const validation = validateParseResult(data);

    if (!validation.valid) {
      ElMessage.error(validation.errors.join('; '));
      progressStatus.value = 'exception';
      return;
    }

    // 设置解析结果
    parsedData.value = data;
    parseWarnings.value = validation.warnings;
    parseStats.value = validation.stats;

    // 计算日期范围
    const dates = data
      .map(d => d.report_date)
      .filter((d): d is string => !!d)
      .sort();
    dateRange.value = {
      start: dates[0] || '',
      end: dates[dates.length - 1] || '',
    };

    progress.value = 100;
    progressText.value = '解析完成';
    progressStatus.value = 'success';

    // 显示预览
    showPreview.value = true;
  } catch (error) {
    console.error('解析失败:', error);
    ElMessage.error('文件解析失败: ' + (error as Error).message);
    progressStatus.value = 'exception';
  } finally {
    importing.value = false;
    // 重置文件输入
    if (fileInputRef.value) {
      fileInputRef.value.value = '';
    }
  }
}

async function confirmImport() {
  if (!props.projectId) {
    ElMessage.error('项目 ID 缺失');
    return;
  }

  importing.value = true;
  progressText.value = '正在导入数据...';
  progress.value = 0;

  try {
    const count = await adImportSearchTerms(props.projectId, parsedData.value, importMode.value);
    progress.value = 100;
    progressStatus.value = 'success';
    progressText.value = `成功导入 ${count} 条搜索词`;

    showPreview.value = false;
    emit('imported');
  } catch (error) {
    console.error('导入失败:', error);
    const errorMsg = typeof error === 'string' ? error : (error as Error)?.message || String(error);
    ElMessage.error('导入失败: ' + errorMsg);
    progressStatus.value = 'exception';
  } finally {
    importing.value = false;
  }
}
</script>

<style scoped>
.ad-data-import {
  width: 100%;
}

.drop-zone {
  border: 2px dashed var(--el-border-color);
  border-radius: 8px;
  padding: 60px 20px;
  text-align: center;
  cursor: pointer;
  transition: all 0.2s;
  background: var(--el-fill-color-blank);
}

.drop-zone:hover,
.drop-zone.is-dragging {
  border-color: var(--el-color-primary);
  background: var(--el-color-primary-light-9);
}

.drop-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.upload-icon {
  font-size: 48px;
  color: var(--el-text-color-secondary);
}

.drop-text p {
  margin: 0;
  color: var(--el-text-color-primary);
  font-size: 16px;
}

.drop-text .sub-text {
  color: var(--el-text-color-secondary);
  font-size: 14px;
  margin-top: 8px;
}

.import-progress {
  padding: 20px;
}

.progress-text {
  margin-top: 12px;
  color: var(--el-text-color-secondary);
}

/* 预览 */
.preview-stats {
  margin-bottom: 20px;
}

.stats-row {
  display: flex;
  gap: 24px;
  margin-top: 16px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stat-item .stat-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.stat-item .stat-value {
  font-size: 20px;
  font-weight: 600;
  color: var(--el-color-primary);
}

.stat-item .stat-value.date-range {
  font-size: 14px;
}

/* 导入模式选择 */
.import-mode-section {
  margin-top: 20px;
  padding: 16px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
}

.mode-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 12px;
}

.import-mode-section :deep(.el-radio-group) {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.import-mode-section :deep(.el-radio) {
  display: flex;
  align-items: flex-start;
  height: auto;
  padding: 12px;
  border: 1px solid var(--el-border-color);
  border-radius: 6px;
  margin-right: 0;
  background: var(--el-bg-color);
  transition: all 0.2s;
}

.import-mode-section :deep(.el-radio:hover) {
  border-color: var(--el-color-primary-light-5);
}

.import-mode-section :deep(.el-radio.is-checked) {
  border-color: var(--el-color-primary);
  background: var(--el-color-primary-light-9);
}

.import-mode-section :deep(.el-radio__label) {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding-left: 8px;
}

.mode-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.mode-desc {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
</style>

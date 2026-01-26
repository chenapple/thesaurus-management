<script setup lang="ts">
import { ref, computed } from "vue";
import { ElMessage } from "element-plus";
import { Search, Upload, Plus, RefreshLeft, DocumentCopy } from "@element-plus/icons-vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import * as api from "../api";
import type { KeywordData, Product } from "../types";

const props = defineProps<{
  selectedProduct: Product | null;
  keywordData: KeywordData[];
  keywordTotal: number;
  keywordLoading: boolean;
  keywordPage: number;
  keywordPageSize: number;
  keywordSearch: string;
  keywordFilters: {
    trafficLevel: string[];
    relevanceLevel: string[];
    primaryCategory: string[];
    orderliness: string[];
  };
  columnConfig: Record<string, boolean>;
  hasActiveFilters: boolean;
  amazonDomains: Record<string, string>;
}>();

const emit = defineEmits<{
  (e: 'update:keywordPage', value: number): void;
  (e: 'update:keywordPageSize', value: number): void;
  (e: 'update:keywordSearch', value: string): void;
  (e: 'update:keywordFilters', value: typeof props.keywordFilters): void;
  (e: 'load-data'): void;
  (e: 'filter-change'): void;
  (e: 'reset-filters'): void;
  (e: 'handle-import'): void;
  (e: 'sort-change', payload: { prop: string; order: string | null }): void;
  (e: 'selection-change', rows: KeywordData[]): void;
  (e: 'show-quick-add-monitoring'): void;
}>();

const trafficLevelOptions = ["大词", "中词", "小词"];
const relevanceLevelOptions = ["强相关", "高相关", "中相关", "弱相关"];
const orderlinessOptions = ["有序", "无序"];
const primaryCategoryOptions = ["品类词", "功能词", "场景词", "属性词", "品牌词", "人群词", "受众词", "其他"];

const keywordTableRef = ref<InstanceType<typeof import('element-plus')['ElTable']> | null>(null);
const selectedKeywords = ref<KeywordData[]>([]);

// Phrase tag editing
const editingPhraseTagId = ref<number | null>(null);
const editingPhraseTagValue = ref("");

function startEditPhraseTag(row: KeywordData) {
  editingPhraseTagId.value = row.id;
  editingPhraseTagValue.value = row.phrase_tag || "";
}

async function savePhraseTag(row: KeywordData) {
  if (editingPhraseTagId.value === null) return;
  try {
    await api.updateKeywordField(row.id, 'phrase_tag', editingPhraseTagValue.value);
    row.phrase_tag = editingPhraseTagValue.value;
  } catch (e) {
    ElMessage.error("保存失败: " + e);
  }
  editingPhraseTagId.value = null;
}

function cancelEditPhraseTag() {
  editingPhraseTagId.value = null;
}

// Amazon search
async function openAmazonSearch(keyword: string) {
  if (!props.selectedProduct?.country) return;
  const domain = props.amazonDomains[props.selectedProduct.country];
  if (!domain) return;
  const url = `https://${domain}/s?k=${encodeURIComponent(keyword)}`;
  await openUrl(url);
}

// Copy keyword
async function copyKeyword(keyword: string) {
  try {
    await writeText(keyword);
    ElMessage.success(`已复制: ${keyword}`);
  } catch {
    ElMessage.error("复制失败");
  }
}

// Selection handling
function handleSelectionChange(rows: KeywordData[]) {
  selectedKeywords.value = rows;
  emit('selection-change', rows);
}

function clearSelection() {
  selectedKeywords.value = [];
  keywordTableRef.value?.clearSelection();
}

// Filter handling
function handleFilterChange() {
  emit('filter-change');
}

function resetFilters() {
  emit('reset-filters');
}

// Pagination
function handlePageChange(page: number) {
  emit('update:keywordPage', page);
  emit('load-data');
}

function handleSizeChange(size: number) {
  emit('update:keywordPageSize', size);
  emit('update:keywordPage', 1);
  emit('load-data');
}

// Sort
function handleSortChange(payload: { prop: string; order: string | null }) {
  emit('sort-change', payload);
}

// Computed for filter values
const localSearch = computed({
  get: () => props.keywordSearch,
  set: (val) => emit('update:keywordSearch', val)
});

const localFilters = computed({
  get: () => props.keywordFilters,
  set: (val) => emit('update:keywordFilters', val)
});
</script>

<template>
  <div class="keywords-tab">
    <!-- Filter bar -->
    <div class="keyword-filter-bar">
      <el-input
        v-model="localSearch"
        placeholder="搜索关键词..."
        clearable
        style="width: 200px"
        @clear="handleFilterChange"
        @keyup.enter="handleFilterChange"
      >
        <template #prefix>
          <el-icon><Search /></el-icon>
        </template>
      </el-input>

      <el-select
        v-model="localFilters.trafficLevel"
        multiple
        collapse-tags
        collapse-tags-tooltip
        placeholder="流量级别"
        style="width: 140px"
        @change="handleFilterChange"
      >
        <el-option v-for="opt in trafficLevelOptions" :key="opt" :label="opt" :value="opt" />
      </el-select>

      <el-select
        v-model="localFilters.relevanceLevel"
        multiple
        collapse-tags
        collapse-tags-tooltip
        placeholder="相关性"
        style="width: 140px"
        @change="handleFilterChange"
      >
        <el-option v-for="opt in relevanceLevelOptions" :key="opt" :label="opt" :value="opt" />
      </el-select>

      <el-select
        v-model="localFilters.primaryCategory"
        multiple
        collapse-tags
        collapse-tags-tooltip
        placeholder="一级分类"
        style="width: 140px"
        @change="handleFilterChange"
      >
        <el-option v-for="opt in primaryCategoryOptions" :key="opt" :label="opt" :value="opt" />
      </el-select>

      <el-select
        v-model="localFilters.orderliness"
        multiple
        collapse-tags
        collapse-tags-tooltip
        placeholder="有序性"
        style="width: 120px"
        @change="handleFilterChange"
      >
        <el-option v-for="opt in orderlinessOptions" :key="opt" :label="opt" :value="opt" />
      </el-select>

      <el-button v-if="hasActiveFilters" text type="primary" @click="resetFilters">
        重置筛选
      </el-button>

      <span v-if="hasActiveFilters" class="filter-result-count">
        共 {{ keywordTotal }} 条结果
      </span>

      <!-- Batch operation area -->
      <el-divider direction="vertical" v-if="selectedKeywords.length > 0" />
      <el-button
        v-if="selectedKeywords.length > 0"
        type="success"
        @click="emit('show-quick-add-monitoring')"
      >
        <el-icon><Plus /></el-icon>
        添加到监控 ({{ selectedKeywords.length }})
      </el-button>
      <el-button
        v-if="selectedKeywords.length > 0"
        text
        @click="clearSelection"
      >
        取消选择
      </el-button>
    </div>

    <!-- Keywords table -->
    <div class="keyword-table-container">
      <el-table
        ref="keywordTableRef"
        :data="keywordData"
        v-loading="keywordLoading"
        stripe
        style="width: 100%"
        height="100%"
        @sort-change="handleSortChange"
        @selection-change="handleSelectionChange"
      >
        <template #empty>
          <div class="table-empty-state">
            <div class="empty-icon">
              <el-icon :size="48"><Upload /></el-icon>
            </div>
            <p class="empty-title">{{ hasActiveFilters ? '没有匹配的数据' : '还没有关键词数据' }}</p>
            <p class="empty-desc">{{ hasActiveFilters ? '尝试调整筛选条件' : '导入 Excel 文件开始分析' }}</p>
            <el-button v-if="!hasActiveFilters" type="primary" @click="emit('handle-import')">
              <el-icon><Upload /></el-icon>
              导入 Excel
            </el-button>
            <el-button v-else @click="resetFilters">
              <el-icon><RefreshLeft /></el-icon>
              重置筛选
            </el-button>
          </div>
        </template>
        <el-table-column type="selection" width="40" fixed="left" />
        <el-table-column type="index" label="#" width="50" fixed="left" />

        <!-- Keyword column -->
        <el-table-column prop="keyword" label="关键词" min-width="220" fixed="left" show-overflow-tooltip>
          <template #default="{ row }">
            <div class="keyword-cell">
              <span
                class="keyword-link"
                @click="openAmazonSearch(row.keyword)"
                :title="'在 Amazon 搜索: ' + row.keyword"
              >
                {{ row.keyword }}
              </span>
              <el-icon class="copy-icon" @click.stop="copyKeyword(row.keyword)" title="复制关键词">
                <DocumentCopy />
              </el-icon>
            </div>
          </template>
        </el-table-column>

        <el-table-column v-if="columnConfig.translation" prop="translation" label="翻译" min-width="150" show-overflow-tooltip />

        <!-- Calculated columns -->
        <el-table-column v-if="columnConfig.traffic_level" prop="traffic_level" label="流量级别" width="90" align="center">
          <template #default="{ row }">
            <el-tag v-if="row.traffic_level" :type="row.traffic_level === '大词' ? 'danger' : row.traffic_level === '中词' ? 'warning' : 'info'" size="small">
              {{ row.traffic_level }}
            </el-tag>
            <span v-else class="empty-cell">-</span>
          </template>
        </el-table-column>

        <el-table-column v-if="columnConfig.negative_word" prop="negative_word" label="否词" width="70" align="center">
          <template #default="{ row }">
            {{ row.negative_word || '-' }}
          </template>
        </el-table-column>

        <el-table-column v-if="columnConfig.orderliness" prop="orderliness" label="有序性" width="80" align="center">
          <template #default="{ row }">
            {{ row.orderliness || '-' }}
          </template>
        </el-table-column>

        <el-table-column v-if="columnConfig.phrase_tag" prop="phrase_tag" label="词组标签" min-width="150">
          <template #default="{ row }">
            <el-input
              v-if="editingPhraseTagId === row.id"
              v-model="editingPhraseTagValue"
              size="small"
              @blur="savePhraseTag(row)"
              @keyup.enter="savePhraseTag(row)"
              @keyup.escape="cancelEditPhraseTag"
              autofocus
            />
            <span v-else @dblclick="startEditPhraseTag(row)" class="editable-cell">
              {{ row.phrase_tag || '-' }}
            </span>
          </template>
        </el-table-column>

        <el-table-column v-if="columnConfig.primary_category" prop="primary_category" label="一级分类" width="90" align="center">
          <template #default="{ row }">
            {{ row.primary_category || '-' }}
          </template>
        </el-table-column>

        <el-table-column v-if="columnConfig.secondary_category" prop="secondary_category" label="二级分类" width="90" align="center">
          <template #default="{ row }">
            {{ row.secondary_category || '-' }}
          </template>
        </el-table-column>

        <el-table-column v-if="columnConfig.search_intent" prop="search_intent" label="搜索意图" min-width="120" show-overflow-tooltip>
          <template #default="{ row }">
            {{ row.search_intent || '-' }}
          </template>
        </el-table-column>

        <el-table-column v-if="columnConfig.traffic_share" prop="traffic_share" label="流量占比" width="90" align="right">
          <template #default="{ row }">
            {{ row.traffic_share ? row.traffic_share.toFixed(2) + '%' : '-' }}
          </template>
        </el-table-column>

        <!-- Original data columns -->
        <el-table-column v-if="columnConfig.relevance_score" prop="relevance_score" label="相关性得分" width="100" align="center">
          <template #default="{ row }">
            {{ row.relevance_score || '-' }}
          </template>
        </el-table-column>

        <el-table-column v-if="columnConfig.relevance_level" prop="relevance_level" label="相关性档位" width="100" align="center">
          <template #default="{ row }">
            {{ row.relevance_level || '-' }}
          </template>
        </el-table-column>

        <el-table-column v-if="columnConfig.traffic_total" prop="traffic_total" label="流量总和" width="120" sortable="custom">
          <template #default="{ row }">
            {{ row.traffic_total !== null ? row.traffic_total.toLocaleString() : '-' }}
          </template>
        </el-table-column>

        <el-table-column v-if="columnConfig.avg_keyword_rank" prop="avg_keyword_rank" label="周平均排名" width="130" align="right" sortable="custom">
          <template #default="{ row }">
            {{ row.avg_keyword_rank || '-' }}
          </template>
        </el-table-column>

        <el-table-column v-if="columnConfig.avg_search_volume" prop="avg_search_volume" label="周平均搜索量" width="150" align="right" sortable="custom">
          <template #default="{ row }">
            {{ row.avg_search_volume !== null ? row.avg_search_volume.toLocaleString() : '-' }}
          </template>
        </el-table-column>

        <el-table-column v-if="columnConfig.cpc_bid" prop="cpc_bid" :label="selectedProduct?.cpc_header || 'CPC建议竞价'" width="130" align="center">
          <template #default="{ row }">
            {{ row.cpc_bid ? Number(row.cpc_bid).toFixed(2) : '-' }}
          </template>
        </el-table-column>

        <el-table-column v-if="columnConfig.bid_range" prop="bid_range" :label="selectedProduct?.bid_range_header || '建议竞价范围'" min-width="140" align="center" show-overflow-tooltip>
          <template #default="{ row }">
            {{ row.bid_range || '-' }}
          </template>
        </el-table-column>

        <el-table-column v-if="columnConfig.click_rate" prop="click_rate" label="均点击转化率" width="110">
          <template #default="{ row }">
            {{ row.click_rate ? (Number(row.click_rate) * 100).toFixed(2) + '%' : '-' }}
          </template>
        </el-table-column>

        <el-table-column v-if="columnConfig.conversion_competition" prop="conversion_competition" label="周转化竞争" width="100" align="center">
          <template #default="{ row }">
            {{ row.conversion_competition || '-' }}
          </template>
        </el-table-column>

        <el-table-column v-if="columnConfig.competition_level" prop="competition_level" label="竞争度档位" width="100" align="center">
          <template #default="{ row }">
            {{ row.competition_level || '-' }}
          </template>
        </el-table-column>

        <el-table-column v-if="columnConfig.natural_position_flow" prop="natural_position_flow" label="自然位流动率" width="120" align="center">
          <template #default="{ row }">
            {{ row.natural_position_flow || '-' }}
          </template>
        </el-table-column>

        <el-table-column v-if="columnConfig.top3_click_share" prop="top3_click_share" label="Top3点击份额" min-width="140" align="center" show-overflow-tooltip>
          <template #default="{ row }">
            {{ row.top3_click_share ? (Number(row.top3_click_share) * 100).toFixed(2) + '%' : '-' }}
          </template>
        </el-table-column>

        <el-table-column v-if="columnConfig.avg_conversion_share" prop="avg_conversion_share" label="Top3转化份额" min-width="150" align="center" show-overflow-tooltip>
          <template #default="{ row }">
            {{ row.avg_conversion_share ? (Number(row.avg_conversion_share) * 100).toFixed(2) + '%' : '-' }}
          </template>
        </el-table-column>

        <el-table-column v-if="columnConfig.asin_count" prop="asin_count" label="asin数量" width="100" align="center">
          <template #default="{ row }">
            {{ row.asin_count !== null ? row.asin_count : '-' }}
          </template>
        </el-table-column>
      </el-table>
    </div>

    <!-- Pagination -->
    <div class="pagination">
      <el-pagination
        :current-page="keywordPage"
        :page-size="keywordPageSize"
        :page-sizes="[20, 50, 100, 200]"
        :total="keywordTotal"
        layout="total, sizes, prev, pager, next, jumper"
        @size-change="handleSizeChange"
        @current-change="handlePageChange"
      />
    </div>
  </div>
</template>

<style scoped>
.keywords-tab {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.keyword-filter-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 20px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.filter-result-count {
  margin-left: auto;
  color: var(--el-text-color-secondary);
  font-size: 13px;
}

.keyword-table-container {
  flex: 1;
  overflow: hidden;
  padding: 16px;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.keyword-table-container :deep(.el-table) {
  flex: 1;
}

.keyword-table-container :deep(.el-table__row) {
  height: 48px;
}

.keyword-cell {
  display: flex;
  align-items: center;
  gap: 6px;
}

.keyword-link {
  color: var(--el-color-primary);
  cursor: pointer;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.keyword-link:hover {
  text-decoration: underline;
}

.copy-icon {
  color: var(--el-text-color-secondary);
  cursor: pointer;
  font-size: 14px;
  opacity: 0;
  transition: opacity 0.2s, color 0.2s;
  flex-shrink: 0;
}

.keyword-cell:hover .copy-icon {
  opacity: 1;
}

.copy-icon:hover {
  color: var(--el-color-primary);
}

.empty-cell {
  color: var(--text-muted);
}

.editable-cell {
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 2px;
  display: inline-block;
  min-width: 20px;
  white-space: nowrap;
}

.editable-cell:hover {
  background: var(--el-fill-color-light);
}

.table-empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
}

.table-empty-state .empty-icon {
  color: var(--el-color-info-light-3);
  margin-bottom: 16px;
}

.table-empty-state .empty-title {
  font-size: 15px;
  font-weight: 500;
  color: var(--text-primary);
  margin: 0 0 8px 0;
}

.table-empty-state .empty-desc {
  font-size: 13px;
  color: var(--text-muted);
  margin: 0 0 16px 0;
}

.pagination {
  display: flex;
  justify-content: center;
  padding: 16px;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
  flex-shrink: 0;
}
</style>

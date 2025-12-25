<template>
  <div class="keyword-monitoring-tab">
    <!-- 统计卡片 -->
    <div class="stats-row">
      <el-card shadow="hover" class="stat-card">
        <div class="stat-content">
          <div class="stat-value">{{ stats.total }}</div>
          <div class="stat-label">监控总数</div>
        </div>
      </el-card>
      <el-card shadow="hover" class="stat-card">
        <div class="stat-content">
          <div class="stat-value">{{ stats.active }}</div>
          <div class="stat-label">活跃监控</div>
        </div>
      </el-card>
      <el-card shadow="hover" class="stat-card">
        <div class="stat-content">
          <div class="stat-value">{{ stats.top10_organic }}</div>
          <div class="stat-label">Top 10</div>
        </div>
      </el-card>
      <el-card shadow="hover" class="stat-card">
        <div class="stat-content">
          <div class="stat-value">{{ stats.top30_organic }}</div>
          <div class="stat-label">Top 30</div>
        </div>
      </el-card>
      <el-card shadow="hover" class="stat-card">
        <div class="stat-content">
          <div class="stat-value">{{ stats.with_sponsored }}</div>
          <div class="stat-label">有广告位</div>
        </div>
      </el-card>
    </div>

    <!-- 工具栏 -->
    <div class="toolbar">
      <div class="toolbar-left">
        <el-radio-group v-model="viewMode" size="small" class="view-mode-switch">
          <el-radio-button value="flat">列表</el-radio-button>
          <el-radio-button value="product">按产品</el-radio-button>
          <el-radio-button value="keyword">按关键词</el-radio-button>
        </el-radio-group>
        <el-button type="primary" @click="showAddDialog = true">
          <el-icon><Plus /></el-icon>
          添加监控
        </el-button>
        <el-button
          type="success"
          :loading="checkingAll"
          :disabled="!selectedIds.length && !stats.active"
          @click="handleCheckRankings"
        >
          <el-icon><Refresh /></el-icon>
          {{ selectedIds.length ? `检测选中 (${selectedIds.length})` : '检测全部' }}
        </el-button>
        <el-button
          type="danger"
          :disabled="!selectedIds.length"
          @click="handleBatchDelete"
        >
          <el-icon><Delete /></el-icon>
          删除选中
        </el-button>
      </div>
      <div class="toolbar-right">
        <el-input
          v-model="searchText"
          placeholder="搜索关键词或ASIN"
          clearable
          style="width: 200px"
          @clear="loadData"
          @keyup.enter="loadData"
        >
          <template #prefix>
            <el-icon><Search /></el-icon>
          </template>
        </el-input>
      </div>
    </div>

    <!-- 筛选栏 -->
    <div class="filter-bar">
      <el-select
        v-model="filters.country"
        placeholder="站点"
        clearable
        style="width: 120px"
        @change="loadData"
      >
        <el-option
          v-for="opt in COUNTRY_OPTIONS"
          :key="opt.value"
          :value="opt.value"
        >
          <span class="country-option">
            <span class="country-flag-small" v-html="opt.flag"></span>
            <span>{{ opt.label }}</span>
          </span>
        </el-option>
      </el-select>
      <el-select
        v-model="filters.priority"
        placeholder="优先级"
        clearable
        style="width: 100px"
        @change="loadData"
      >
        <el-option
          v-for="opt in PRIORITY_OPTIONS"
          :key="opt.value"
          :label="opt.label"
          :value="opt.value"
        />
      </el-select>
      <el-select
        v-model="filters.isActive"
        placeholder="状态"
        clearable
        style="width: 100px"
        @change="loadData"
      >
        <el-option label="活跃" :value="true" />
        <el-option label="暂停" :value="false" />
      </el-select>
    </div>

    <!-- 列表视图 - 表格 -->
    <div v-if="viewMode === 'flat'" class="table-container">
      <el-table
        ref="tableRef"
        :data="monitoringList"
        v-loading="loading"
        stripe
        border
        height="100%"
        style="width: 100%"
        row-key="id"
        @selection-change="handleSelectionChange"
        @sort-change="handleSortChange"
      >
        <el-table-column type="selection" width="40" />

        <el-table-column label="状态" prop="is_active" width="70" align="center">
          <template #default="{ row }">
            <el-switch
              :model-value="row.is_active"
              size="small"
              @change="(val: boolean) => handleToggleActive(row.id, val)"
            />
          </template>
        </el-table-column>

        <el-table-column label="图片" width="60">
          <template #default="{ row }">
            <el-image
              v-if="row.image_url"
              :src="row.image_url"
              style="width: 40px; height: 40px"
              fit="contain"
            >
              <template #error>
                <span class="no-image">-</span>
              </template>
            </el-image>
            <span v-else class="no-image">-</span>
          </template>
        </el-table-column>

        <el-table-column label="关键词" prop="keyword" min-width="220">
          <template #default="{ row }">
            <span class="keyword-text">{{ row.keyword }}</span>
          </template>
        </el-table-column>

        <el-table-column label="站点" width="70" align="center">
          <template #default="{ row }">
            <span class="country-tag">
              <span class="country-flag-tiny" v-html="getCountryFlag(row.country)"></span>
              <span>{{ row.country }}</span>
            </span>
          </template>
        </el-table-column>

        <el-table-column label="ASIN" prop="asin" width="120" />

        <el-table-column label="价格" prop="price" width="80" align="right">
          <template #default="{ row }">
            <span>{{ row.price ?? '-' }}</span>
          </template>
        </el-table-column>

        <el-table-column label="评论/星级" width="130">
          <template #default="{ row }">
            <div v-if="row.rating || row.reviews_count" class="rating-cell">
              <div class="star-line">
                <span class="stars-container">
                  <span class="star-filled">{{ '★'.repeat(Math.floor(row.rating || 0)) }}</span><span class="star-empty">{{ '★'.repeat(5 - Math.floor(row.rating || 0)) }}</span>
                </span>
                <span class="rating-num">{{ row.rating?.toFixed(1) }}</span>
              </div>
              <div v-if="row.reviews_count" class="reviews-line">
                {{ formatReviewCount(row.reviews_count) }} 评论
              </div>
            </div>
            <span v-else class="no-rank">-</span>
          </template>
        </el-table-column>

        <el-table-column prop="latest_organic_rank" width="120" align="center">
          <template #header>
            <el-tooltip content="绿色 = 第1页前10名" placement="top">
              <span class="header-with-tip">自然排名</span>
            </el-tooltip>
          </template>
          <template #default="{ row }">
            <span v-if="row.latest_organic_rank" :class="getRankClass(row.latest_organic_page, row.latest_organic_rank)" class="rank-display">
              <span class="rank-page">第{{ row.latest_organic_page || 1 }}页</span>
              <span class="rank-position">第{{ row.latest_organic_rank }}名</span>
            </span>
            <span v-else class="no-rank">前5页无排名</span>
          </template>
        </el-table-column>

        <el-table-column width="120" align="center">
          <template #header>
            <el-tooltip content="最近7天自然排名趋势" placement="top">
              <span class="header-with-tip">趋势</span>
            </el-tooltip>
          </template>
          <template #default="{ row }">
            <Sparkline
              v-if="(organicSparklines[row.id]?.filter(d => d !== null).length ?? 0) > 1"
              :data="organicSparklines[row.id]"
              :width="100"
              color="#67c23a"
              :inverse="true"
              @click="handleShowHistory(row, 'organic')"
            />
            <span v-else class="no-data">-</span>
          </template>
        </el-table-column>

        <el-table-column prop="latest_sponsored_rank" width="120" align="center">
          <template #header>
            <el-tooltip content="广告位排名（付费推广）" placement="top">
              <span class="header-with-tip">广告排名</span>
            </el-tooltip>
          </template>
          <template #default="{ row }">
            <span v-if="row.latest_sponsored_rank" class="rank-sponsored rank-display">
              <span class="rank-page">第{{ row.latest_sponsored_page || 1 }}页</span>
              <span class="rank-position">第{{ row.latest_sponsored_rank }}名</span>
            </span>
            <span v-else class="no-rank">前5页无排名</span>
          </template>
        </el-table-column>

        <el-table-column width="120" align="center">
          <template #header>
            <el-tooltip content="最近7天广告排名趋势" placement="top">
              <span class="header-with-tip">趋势</span>
            </el-tooltip>
          </template>
          <template #default="{ row }">
            <Sparkline
              v-if="(sponsoredSparklines[row.id]?.filter(d => d !== null).length ?? 0) > 1"
              :data="sponsoredSparklines[row.id]"
              :width="100"
              color="#409eff"
              :inverse="true"
              @click="handleShowHistory(row, 'sponsored')"
            />
            <span v-else class="no-data">-</span>
          </template>
        </el-table-column>

        <el-table-column label="优先级" prop="priority" width="80" align="center">
          <template #default="{ row }">
            <el-tag :type="getPriorityType(row.priority)" size="small">
              {{ getPriorityLabel(row.priority) }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column label="最后检测" prop="last_checked" width="140" sortable="custom">
          <template #default="{ row }">
            <span v-if="row.last_checked" class="last-checked">
              {{ formatDateTime(row.last_checked) }}
            </span>
            <span v-else class="no-check">未检测</span>
          </template>
        </el-table-column>

        <el-table-column label="操作" width="100" fixed="right">
          <template #default="{ row }">
            <el-button
              size="small"
              link
              :loading="checkingId === row.id"
              @click="handleCheckSingle(row)"
            >
              检测
            </el-button>
            <el-button
              size="small"
              link
              @click="handleDelete(row.id)"
            >
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <!-- 按产品视图 - 卡片 -->
    <div v-else-if="viewMode === 'product'" class="card-container" v-loading="loading">
      <el-card
        v-for="group in pagedGroupedData"
        :key="group.id"
        class="group-card"
        shadow="hover"
      >
        <template #header>
          <div class="card-header" @click="toggleExpand(group.id)">
            <div class="product-info">
              <el-image
                v-if="(group as GroupRow).image_url"
                :src="(group as GroupRow).image_url!"
                class="product-image"
                fit="contain"
              >
                <template #error>
                  <div class="image-placeholder">-</div>
                </template>
              </el-image>
              <div v-else class="image-placeholder">-</div>
              <span class="asin-text">{{ (group as GroupRow).asin }}</span>
              <span class="country-tag">
                <span class="country-flag-tiny" v-html="getCountryFlag((group as GroupRow).country || '')"></span>
                <span>{{ (group as GroupRow).country }}</span>
              </span>
              <span class="price-text">{{ (group as GroupRow).price ?? '-' }}</span>
              <div v-if="(group as GroupRow).rating" class="rating-inline">
                <span class="stars-container">
                  <span class="star-filled">{{ '★'.repeat(Math.floor((group as GroupRow).rating || 0)) }}</span><span class="star-empty">{{ '★'.repeat(5 - Math.floor((group as GroupRow).rating || 0)) }}</span>
                </span>
                <span class="rating-num">{{ (group as GroupRow).rating?.toFixed(1) }}</span>
                <span v-if="(group as GroupRow).reviews_count" class="reviews-count">({{ formatReviewCount((group as GroupRow).reviews_count!) }})</span>
              </div>
            </div>
            <div class="card-stats">
              <span class="stat-item">{{ (group as GroupRow).children?.length }} 个关键词</span>
              <span class="stat-item">平均排名: {{ calcAvgRank((group as GroupRow).children || []) }}</span>
              <el-button
                size="small"
                type="primary"
                :loading="checkingAll"
                @click.stop="handleCheckGroup(group as GroupRow)"
              >
                批量检测
              </el-button>
              <el-icon class="expand-icon" :class="{ expanded: isExpanded(group.id) }">
                <ArrowDown />
              </el-icon>
            </div>
          </div>
        </template>

        <!-- 子项列表 -->
        <div v-if="isExpanded(group.id) && 'children' in group" class="children-list">
          <!-- 表头 -->
          <div class="child-row child-header">
            <div class="child-cell child-keyword">关键词</div>
            <div class="child-cell child-col-rank">自然排名</div>
            <div class="child-cell child-col-trend">趋势</div>
            <div class="child-cell child-col-rank">广告排名</div>
            <div class="child-cell child-col-trend">趋势</div>
            <div class="child-cell child-col-actions">操作</div>
          </div>
          <!-- 数据行 -->
          <div
            v-for="child in group.children"
            :key="child.id"
            class="child-row"
          >
            <div class="child-cell child-keyword">{{ child.keyword }}</div>
            <div class="child-cell child-col-rank">
              <span v-if="child.latest_organic_rank" :class="getRankClass(child.latest_organic_page, child.latest_organic_rank)" class="rank-display">
                第{{ child.latest_organic_page || 1 }}页{{ child.latest_organic_rank }}名
              </span>
              <span v-else class="no-rank">无排名</span>
            </div>
            <div class="child-cell child-col-trend">
              <Sparkline
                v-if="(organicSparklines[child.id]?.filter(d => d !== null).length ?? 0) > 1"
                :data="organicSparklines[child.id]"
                :width="60"
                color="#67c23a"
                :inverse="true"
                @click="handleShowHistory(child, 'organic')"
              />
              <span v-else class="no-data">-</span>
            </div>
            <div class="child-cell child-col-rank">
              <span v-if="child.latest_sponsored_rank" class="rank-sponsored rank-display">
                第{{ child.latest_sponsored_page || 1 }}页{{ child.latest_sponsored_rank }}名
              </span>
              <span v-else class="no-rank">无排名</span>
            </div>
            <div class="child-cell child-col-trend">
              <Sparkline
                v-if="(sponsoredSparklines[child.id]?.filter(d => d !== null).length ?? 0) > 1"
                :data="sponsoredSparklines[child.id]"
                :width="60"
                color="#409eff"
                :inverse="true"
                @click="handleShowHistory(child, 'sponsored')"
              />
              <span v-else class="no-data">-</span>
            </div>
            <div class="child-cell child-col-actions">
              <el-button size="small" link :loading="checkingId === child.id" @click="handleCheckSingle(child)">检测</el-button>
              <el-button size="small" link type="danger" @click="handleDelete(child.id)">删除</el-button>
            </div>
          </div>
        </div>
      </el-card>
    </div>

    <!-- 按关键词视图 - 卡片 -->
    <div v-else class="card-container" v-loading="loading">
      <el-card
        v-for="group in pagedGroupedData"
        :key="group.id"
        class="group-card"
        shadow="hover"
      >
        <template #header>
          <div class="card-header" @click="toggleExpand(group.id)">
            <div class="keyword-info">
              <span class="keyword-main">{{ (group as GroupRow).keyword }}</span>
              <span class="country-tag">
                <span class="country-flag-tiny" v-html="getCountryFlag((group as GroupRow).country || '')"></span>
                <span>{{ (group as GroupRow).country }}</span>
              </span>
            </div>
            <div class="card-stats">
              <span class="stat-item">{{ (group as GroupRow).children?.length }} 个产品</span>
              <span class="stat-item">最佳排名: {{ calcBestRank((group as GroupRow).children || []) }}</span>
              <el-button
                size="small"
                type="primary"
                :loading="checkingAll"
                @click.stop="handleCheckGroup(group as GroupRow)"
              >
                批量检测
              </el-button>
              <el-icon class="expand-icon" :class="{ expanded: isExpanded(group.id) }">
                <ArrowDown />
              </el-icon>
            </div>
          </div>
        </template>

        <!-- 子项列表 -->
        <div v-if="isExpanded(group.id) && 'children' in group" class="children-list">
          <!-- 表头 -->
          <div class="child-row child-header">
            <div class="child-cell child-col-img">图片</div>
            <div class="child-cell child-col-asin">ASIN</div>
            <div class="child-cell child-col-price">价格</div>
            <div class="child-cell child-col-rating">评分</div>
            <div class="child-cell child-col-rank">自然排名</div>
            <div class="child-cell child-col-trend">趋势</div>
            <div class="child-cell child-col-rank">广告排名</div>
            <div class="child-cell child-col-trend">趋势</div>
            <div class="child-cell child-col-actions">操作</div>
          </div>
          <!-- 数据行 -->
          <div
            v-for="child in group.children"
            :key="child.id"
            class="child-row"
          >
            <div class="child-cell child-col-img">
              <el-image v-if="child.image_url" :src="child.image_url" style="width: 36px; height: 36px" fit="contain">
                <template #error><span class="no-image">-</span></template>
              </el-image>
              <span v-else class="no-image">-</span>
            </div>
            <div class="child-cell child-col-asin">{{ child.asin }}</div>
            <div class="child-cell child-col-price">{{ child.price ?? '-' }}</div>
            <div class="child-cell child-col-rating">
              <span v-if="child.rating" class="rating-compact">
                <span class="star-filled">★</span>{{ child.rating?.toFixed(1) }}
              </span>
              <span v-else>-</span>
            </div>
            <div class="child-cell child-col-rank">
              <span v-if="child.latest_organic_rank" :class="getRankClass(child.latest_organic_page, child.latest_organic_rank)" class="rank-display">
                第{{ child.latest_organic_page || 1 }}页{{ child.latest_organic_rank }}名
              </span>
              <span v-else class="no-rank">无排名</span>
            </div>
            <div class="child-cell child-col-trend">
              <Sparkline
                v-if="(organicSparklines[child.id]?.filter(d => d !== null).length ?? 0) > 1"
                :data="organicSparklines[child.id]"
                :width="60"
                color="#67c23a"
                :inverse="true"
                @click="handleShowHistory(child, 'organic')"
              />
              <span v-else class="no-data">-</span>
            </div>
            <div class="child-cell child-col-rank">
              <span v-if="child.latest_sponsored_rank" class="rank-sponsored rank-display">
                第{{ child.latest_sponsored_page || 1 }}页{{ child.latest_sponsored_rank }}名
              </span>
              <span v-else class="no-rank">无排名</span>
            </div>
            <div class="child-cell child-col-trend">
              <Sparkline
                v-if="(sponsoredSparklines[child.id]?.filter(d => d !== null).length ?? 0) > 1"
                :data="sponsoredSparklines[child.id]"
                :width="60"
                color="#409eff"
                :inverse="true"
                @click="handleShowHistory(child, 'sponsored')"
              />
              <span v-else class="no-data">-</span>
            </div>
            <div class="child-cell child-col-actions">
              <el-button size="small" link :loading="checkingId === child.id" @click="handleCheckSingle(child)">检测</el-button>
              <el-button size="small" link type="danger" @click="handleDelete(child.id)">删除</el-button>
            </div>
          </div>
        </div>
      </el-card>
    </div>

    <!-- 分页 -->
    <div class="pagination-container">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :page-sizes="[20, 50, 100]"
        :total="total"
        layout="total, sizes, prev, pager, next, jumper"
        @size-change="loadData"
        @current-change="loadData"
      />
    </div>

    <!-- 添加对话框 -->
    <AddMonitoringDialog
      v-model="showAddDialog"
      :product-id="productId"
      @success="handleAddSuccess"
    />

    <!-- 历史图表对话框 -->
    <RankingHistoryChart
      v-if="showHistoryDialog"
      v-model="showHistoryDialog"
      :monitoring="selectedMonitoring"
      :display-type="historyType"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Plus, Refresh, Delete, Search, ArrowDown } from '@element-plus/icons-vue';
import {
  getKeywordMonitoringList,
  updateKeywordMonitoring,
  deleteKeywordMonitoring,
  batchDeleteKeywordMonitoring,
  getMonitoringStats,
  getMonitoringSparklines,
  checkSingleRanking,
  checkAllRankings,
} from '../api';
import type { KeywordMonitoring, MonitoringStats } from '../types';
import { COUNTRY_OPTIONS, PRIORITY_OPTIONS, getCountryFlag } from '../types';
import AddMonitoringDialog from './AddMonitoringDialog.vue';
import RankingHistoryChart from './RankingHistoryChart.vue';
import Sparkline from './Sparkline.vue';

const props = defineProps<{
  productId: number;
}>();

// 视图模式: flat(平铺列表), product(按产品分组), keyword(按关键词分组)
type ViewMode = 'flat' | 'product' | 'keyword';
const viewMode = ref<ViewMode>('flat');

// 状态
const loading = ref(false);
const checkingAll = ref(false);
const checkingId = ref<number | null>(null);
const monitoringList = ref<KeywordMonitoring[]>([]);
const allMonitoringList = ref<KeywordMonitoring[]>([]); // 分组模式下存储全部数据
const total = ref(0);
const currentPage = ref(1);
const pageSize = ref(20);
const selectedIds = ref<number[]>([]);
const searchText = ref('');
const organicSparklines = ref<Record<number, (number | null)[]>>({});
const sponsoredSparklines = ref<Record<number, (number | null)[]>>({});

const stats = reactive<MonitoringStats>({
  total: 0,
  active: 0,
  top10_organic: 0,
  top30_organic: 0,
  with_sponsored: 0,
});

const filters = reactive({
  country: '' as string,
  priority: '' as string,
  isActive: undefined as boolean | undefined,
});

const sortBy = ref('');
const sortOrder = ref('');

// 卡片展开状态（默认全部展开，记录已折叠的）
const collapsedGroups = ref<Set<string>>(new Set());

// 切换展开/折叠
function toggleExpand(groupId: string | number) {
  const key = String(groupId);
  if (collapsedGroups.value.has(key)) {
    collapsedGroups.value.delete(key);
  } else {
    collapsedGroups.value.add(key);
  }
}

// 判断是否展开
function isExpanded(groupId: string | number): boolean {
  return !collapsedGroups.value.has(String(groupId));
}

// 计算平均排名
function calcAvgRank(items: KeywordMonitoring[]): string {
  const ranks = items
    .filter(i => i.latest_organic_rank !== null)
    .map(i => i.latest_organic_rank!);
  if (ranks.length === 0) return '-';
  return Math.round(ranks.reduce((a, b) => a + b, 0) / ranks.length).toString();
}

// 计算最佳排名
function calcBestRank(items: KeywordMonitoring[]): string {
  const ranks = items.filter(i => i.latest_organic_rank !== null);
  if (ranks.length === 0) return '-';
  const best = ranks.reduce((a, b) =>
    (a.latest_organic_rank! < b.latest_organic_rank!) ? a : b
  );
  return `第${best.latest_organic_page || 1}页${best.latest_organic_rank}名`;
}

// 检测分组内所有关键词
async function handleCheckGroup(group: GroupRow) {
  checkingAll.value = true;
  try {
    let successCount = 0;
    let errorCount = 0;
    for (const child of group.children) {
      try {
        const result = await checkSingleRanking(child.id, 3);
        if (result.error) {
          errorCount++;
        } else {
          successCount++;
        }
      } catch {
        errorCount++;
      }
    }
    if (errorCount > 0) {
      ElMessage.warning(`检测完成: ${successCount} 成功, ${errorCount} 失败`);
    } else {
      ElMessage.success(`检测完成: 共 ${successCount} 个关键词`);
    }
    loadData();
    loadStats();
  } catch (e) {
    ElMessage.error(`批量检测失败: ${e}`);
  } finally {
    checkingAll.value = false;
  }
}

// 对话框
const showAddDialog = ref(false);
const showHistoryDialog = ref(false);
const selectedMonitoring = ref<KeywordMonitoring | null>(null);
const historyType = ref<'organic' | 'sponsored' | 'all'>('all');

// 分组数据接口
interface GroupRow {
  id: string;  // 分组唯一标识
  isGroup: true;
  // 按产品分组时的字段
  asin?: string;
  country?: string;
  image_url?: string | null;
  price?: string | null;
  rating?: number | null;
  reviews_count?: number | null;
  // 按关键词分组时的字段
  keyword?: string;
  // 子项
  children: KeywordMonitoring[];
}

type TableRow = KeywordMonitoring | GroupRow;

// 分组后的数据
const groupedData = computed<TableRow[]>(() => {
  if (viewMode.value === 'flat') {
    return monitoringList.value;
  }

  const data = allMonitoringList.value;
  if (!data.length) return [];

  if (viewMode.value === 'product') {
    return groupByProduct(data);
  } else {
    return groupByKeyword(data);
  }
});

// 分页后的分组数据
const pagedGroupedData = computed<TableRow[]>(() => {
  if (viewMode.value === 'flat') {
    return monitoringList.value;
  }

  const groups = groupedData.value;
  const start = (currentPage.value - 1) * pageSize.value;
  const end = start + pageSize.value;
  return groups.slice(start, end);
});

// 按产品(ASIN+站点)分组
function groupByProduct(data: KeywordMonitoring[]): GroupRow[] {
  const groups = new Map<string, GroupRow>();

  for (const item of data) {
    const key = `${item.asin}_${item.country}`;
    if (!groups.has(key)) {
      groups.set(key, {
        id: `group_${key}`,
        isGroup: true,
        asin: item.asin,
        country: item.country,
        image_url: item.image_url,
        price: item.price,
        rating: item.rating,
        reviews_count: item.reviews_count,
        children: [],
      });
    }
    groups.get(key)!.children.push(item);
  }

  // 组内按优先级排序
  const priorityOrder = { high: 0, medium: 1, low: 2 };
  for (const group of groups.values()) {
    group.children.sort((a, b) =>
      (priorityOrder[a.priority as keyof typeof priorityOrder] ?? 2) -
      (priorityOrder[b.priority as keyof typeof priorityOrder] ?? 2)
    );
  }

  return Array.from(groups.values());
}

// 按关键词(keyword+站点)分组
function groupByKeyword(data: KeywordMonitoring[]): GroupRow[] {
  const groups = new Map<string, GroupRow>();

  for (const item of data) {
    // 跳过没有关键词的项
    if (!item.keyword) continue;

    const key = `${item.keyword}_${item.country}`;
    if (!groups.has(key)) {
      groups.set(key, {
        id: `group_${key}`,
        isGroup: true,
        keyword: item.keyword,
        country: item.country,
        children: [],
      });
    }
    groups.get(key)!.children.push(item);
  }

  // 组内按自然排名排序(有排名的在前，无排名的在后)
  for (const group of groups.values()) {
    group.children.sort((a, b) => {
      if (a.latest_organic_rank === null && b.latest_organic_rank === null) return 0;
      if (a.latest_organic_rank === null) return 1;
      if (b.latest_organic_rank === null) return -1;
      return a.latest_organic_rank - b.latest_organic_rank;
    });
  }

  return Array.from(groups.values());
}


// 加载数据
async function loadData() {
  loading.value = true;
  try {
    if (viewMode.value === 'flat') {
      // 平铺模式：后端分页
      const [list, count] = await getKeywordMonitoringList({
        productId: props.productId,
        country: filters.country || undefined,
        priority: filters.priority || undefined,
        isActive: filters.isActive,
        search: searchText.value || undefined,
        sortBy: sortBy.value || undefined,
        sortOrder: sortOrder.value || undefined,
        page: currentPage.value,
        pageSize: pageSize.value,
      });
      monitoringList.value = list;
      total.value = count;
    } else {
      // 分组模式：获取全部数据，前端分页
      const [list] = await getKeywordMonitoringList({
        productId: props.productId,
        country: filters.country || undefined,
        priority: filters.priority || undefined,
        isActive: filters.isActive,
        search: searchText.value || undefined,
        sortBy: sortBy.value || undefined,
        sortOrder: sortOrder.value || undefined,
        page: 1,
        pageSize: 10000,  // 获取全部数据
      });
      allMonitoringList.value = list;
      // total 为分组数量
      total.value = groupedData.value.length;
    }

    // 加载 sparkline 数据
    loadSparklines();
  } catch (e) {
    ElMessage.error(`加载数据失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

// 加载迷你图数据
async function loadSparklines() {
  try {
    const sparklines = await getMonitoringSparklines(props.productId, 7);
    organicSparklines.value = Object.fromEntries(
      sparklines.map(s => [s.monitoring_id, s.organic_ranks])
    );
    sponsoredSparklines.value = Object.fromEntries(
      sparklines.map(s => [s.monitoring_id, s.sponsored_ranks])
    );
  } catch (e) {
    console.error('加载迷你图数据失败:', e);
  }
}

// 加载统计
async function loadStats() {
  try {
    const result = await getMonitoringStats(props.productId);
    Object.assign(stats, result);
  } catch (e) {
    console.error('加载统计失败:', e);
  }
}

// 选择变化
function handleSelectionChange(rows: KeywordMonitoring[]) {
  selectedIds.value = rows.map(r => r.id);
}

// 排序变化
function handleSortChange({ prop, order }: { prop: string; order: string | null }) {
  sortBy.value = prop || '';
  sortOrder.value = order === 'ascending' ? 'asc' : order === 'descending' ? 'desc' : '';
  loadData();
}

// 切换活跃状态
async function handleToggleActive(id: number, active: boolean) {
  try {
    await updateKeywordMonitoring(id, undefined, active);
    loadData();
    loadStats();
  } catch (e) {
    ElMessage.error(`更新状态失败: ${e}`);
  }
}

// 检测单个
async function handleCheckSingle(row: KeywordMonitoring) {
  checkingId.value = row.id;
  try {
    const result = await checkSingleRanking(row.id, 3);
    if (result.error) {
      ElMessage.warning(`检测完成，但有错误: ${result.error}`);
    } else {
      ElMessage.success(`检测完成: 自然排名 ${result.organic_rank ?? '-'}, 广告排名 ${result.sponsored_rank ?? '-'}`);
    }
    loadData();
    loadStats();
  } catch (e) {
    ElMessage.error(`检测失败: ${e}`);
  } finally {
    checkingId.value = null;
  }
}

// 检测排名
async function handleCheckRankings() {
  console.log('handleCheckRankings called, productId:', props.productId, 'stats:', stats);

  if (selectedIds.value.length) {
    // TODO: 实现选中检测
    ElMessage.info('暂不支持选中检测，请使用检测全部');
    return;
  }

  checkingAll.value = true;
  try {
    console.log('Calling checkAllRankings...');
    const results = await checkAllRankings(props.productId, 3, 99999);  // 大值 = 无时间限制
    console.log('checkAllRankings results:', results);

    if (results.length === 0) {
      ElMessage.info('没有活跃的监控项');
    } else {
      const successCount = results.filter(([, r]) => !r.error).length;
      const errorCount = results.length - successCount;

      if (errorCount > 0) {
        ElMessage.warning(`检测完成: ${successCount} 成功, ${errorCount} 失败`);
      } else {
        ElMessage.success(`检测完成: 共 ${successCount} 个关键词`);
      }
    }
    loadData();
    loadStats();
  } catch (e) {
    ElMessage.error(`批量检测失败: ${e}`);
  } finally {
    checkingAll.value = false;
  }
}

// 显示历史
function handleShowHistory(row: KeywordMonitoring, type: 'organic' | 'sponsored' | 'all' = 'all') {
  selectedMonitoring.value = row;
  historyType.value = type;
  showHistoryDialog.value = true;
}

// 删除
async function handleDelete(id: number) {
  try {
    await ElMessageBox.confirm('确定要删除这条监控记录吗？', '提示', {
      type: 'warning',
    });
    await deleteKeywordMonitoring(id);
    ElMessage.success('删除成功');
    loadData();
    loadStats();
  } catch (e) {
    if (e !== 'cancel') {
      ElMessage.error(`删除失败: ${e}`);
    }
  }
}

// 批量删除
async function handleBatchDelete() {
  if (!selectedIds.value.length) return;

  try {
    await ElMessageBox.confirm(`确定要删除选中的 ${selectedIds.value.length} 条记录吗？`, '提示', {
      type: 'warning',
    });
    await batchDeleteKeywordMonitoring(selectedIds.value);
    ElMessage.success('删除成功');
    selectedIds.value = [];
    loadData();
    loadStats();
  } catch (e) {
    if (e !== 'cancel') {
      ElMessage.error(`批量删除失败: ${e}`);
    }
  }
}

// 添加成功
function handleAddSuccess() {
  loadData();
  loadStats();
}

// 工具函数
function getRankClass(page: number | null, rank: number | null): string {
  if (rank === null) return 'no-rank';
  // 只有第1页前10名才是 Top 10
  if ((page === null || page === 1) && rank <= 10) return 'rank-top10';
  return '';  // 其他排名使用默认色
}

function getPriorityType(priority: string): 'danger' | 'warning' | 'info' {
  if (priority === 'high') return 'danger';
  if (priority === 'medium') return 'warning';
  return 'info';
}

function getPriorityLabel(priority: string): string {
  const opt = PRIORITY_OPTIONS.find(o => o.value === priority);
  return opt?.label || priority;
}

function formatDateTime(dateStr: string): string {
  // 数据库存储的是 UTC 时间，转换为北京时间 (UTC+8)
  // 添加 'Z' 后缀确保解析为 UTC
  const utcDateStr = dateStr.endsWith('Z') ? dateStr : dateStr.replace(' ', 'T') + 'Z';
  const date = new Date(utcDateStr);
  const beijingTime = new Date(date.getTime() + 8 * 60 * 60 * 1000);
  const month = (beijingTime.getUTCMonth() + 1).toString().padStart(2, '0');
  const day = beijingTime.getUTCDate().toString().padStart(2, '0');
  const hour = beijingTime.getUTCHours().toString().padStart(2, '0');
  const minute = beijingTime.getUTCMinutes().toString().padStart(2, '0');
  return `${month}-${day} ${hour}:${minute}`;
}

function formatReviewCount(count: number): string {
  return count.toLocaleString('fr-FR'); // 使用空格作为千位分隔符
}

// 监听视图模式变化
watch(viewMode, () => {
  currentPage.value = 1;
  collapsedGroups.value.clear();  // 重置折叠状态
  loadData();
});

// 监听产品变化
watch(() => props.productId, () => {
  currentPage.value = 1;
  loadData();
  loadStats();
});

onMounted(() => {
  loadData();
  loadStats();
});
</script>

<style scoped>
.keyword-monitoring-tab {
  padding: 16px;
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.stats-row {
  display: flex;
  gap: 16px;
  margin-bottom: 16px;
  flex-shrink: 0;
}

.stat-card {
  flex: 1;
}

.stat-card :deep(.el-card__body) {
  padding: 12px 16px;
}

.stat-content {
  text-align: center;
}

.stat-value {
  font-size: 24px;
  font-weight: bold;
  color: var(--el-color-primary);
}

.stat-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 4px;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  flex-shrink: 0;
}

.toolbar-left {
  display: flex;
  gap: 8px;
}

.filter-bar {
  display: flex;
  gap: 12px;
  margin-bottom: 12px;
  flex-shrink: 0;
}

.table-container {
  flex: 1;
  overflow: hidden;
  min-height: 0;
}

.keyword-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.no-image {
  color: var(--el-text-color-placeholder);
}

.rank-top10 {
  color: var(--el-color-success);
  font-weight: bold;
}

.rank-sponsored {
  font-weight: 500;
}

.header-with-tip {
  cursor: help;
  border-bottom: 1px dashed var(--el-text-color-placeholder);
}

.no-rank,
.no-data {
  color: var(--el-text-color-placeholder);
  font-size: 12px;
}

.rank-display {
  display: inline-flex;
  gap: 4px;
  white-space: nowrap;
}

.rank-page {
  font-size: 12px;
  opacity: 0.8;
}

.rank-position {
  font-weight: bold;
}

/* 评论星级样式 */
.rating-cell {
  display: flex;
  flex-direction: column;
  gap: 2px;
  line-height: 1.4;
}

.star-line {
  display: flex;
  align-items: center;
  gap: 4px;
}

.stars-container {
  font-size: 13px;
  letter-spacing: -2px;
}

.star-filled {
  color: #f5a623;
}

.star-empty {
  color: #ddd;
}

.rating-num {
  font-size: 12px;
  color: var(--el-text-color-regular);
}

.reviews-line {
  font-size: 11px;
  color: var(--el-text-color-secondary);
}

.last-checked {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.no-check {
  font-size: 12px;
  color: var(--el-text-color-placeholder);
}

.pagination-container {
  display: flex;
  justify-content: flex-end;
  margin-top: 16px;
  flex-shrink: 0;
}

/* 国旗样式 */
.country-option {
  display: flex;
  align-items: center;
  gap: 6px;
}

.country-flag-small {
  display: inline-flex;
  width: 18px;
  height: 12px;
}

.country-flag-small :deep(svg) {
  width: 100%;
  height: 100%;
  border-radius: 2px;
  box-shadow: 0 0 1px rgba(0, 0, 0, 0.2);
}

.country-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 6px;
  background: var(--el-fill-color-light);
  border-radius: 4px;
  font-size: 12px;
}

.country-flag-tiny {
  display: inline-flex;
  width: 14px;
  height: 10px;
}

.country-flag-tiny :deep(svg) {
  width: 100%;
  height: 100%;
  border-radius: 1px;
  box-shadow: 0 0 1px rgba(0, 0, 0, 0.2);
}

/* 视图切换样式 */
.view-mode-switch {
  margin-right: 12px;
}

/* 卡片容器样式 */
.card-container {
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow-y: auto;
  flex: 1;
  padding: 4px;
}

.group-card {
  border-radius: 8px;
  overflow: visible;
}

.group-card :deep(.el-card) {
  overflow: visible;
}

.group-card :deep(.el-card__header) {
  padding: 12px 16px;
  cursor: pointer;
  background: var(--el-fill-color-light);
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.group-card :deep(.el-card__header):hover {
  background: var(--el-fill-color);
}

.group-card :deep(.el-card__body) {
  padding: 0 !important;
  overflow: visible;
  height: auto;
  min-height: 0;
}

/* 当卡片折叠时，隐藏 body */
.group-card.collapsed :deep(.el-card__body) {
  display: none;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.product-info,
.keyword-info {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.product-image {
  width: 44px;
  height: 44px;
  border-radius: 4px;
  border: 1px solid var(--el-border-color-lighter);
  flex-shrink: 0;
}

.image-placeholder {
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--el-fill-color);
  border-radius: 4px;
  color: var(--el-text-color-placeholder);
  flex-shrink: 0;
}

.asin-text {
  font-weight: 600;
  font-size: 14px;
  color: var(--el-text-color-primary);
}

.keyword-main {
  font-weight: 600;
  font-size: 15px;
  color: var(--el-text-color-primary);
}

.price-text {
  font-weight: 500;
  color: var(--el-color-danger);
}

.rating-inline {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
}

.reviews-count {
  color: var(--el-text-color-secondary);
  font-size: 12px;
}

.card-stats {
  display: flex;
  align-items: center;
  gap: 16px;
  flex-shrink: 0;
}

.stat-item {
  color: var(--el-text-color-secondary);
  font-size: 13px;
  white-space: nowrap;
}

.expand-icon {
  font-size: 16px;
  color: var(--el-text-color-secondary);
  transition: transform 0.3s ease;
}

.expand-icon.expanded {
  transform: rotate(180deg);
}

/* 内部表格样式 */
.inner-table {
  border-top: none;
}

.inner-table :deep(.el-table__header-wrapper) {
  background: var(--el-fill-color-lighter);
}

.inner-table :deep(th.el-table__cell) {
  background: var(--el-fill-color-lighter) !important;
  font-size: 12px;
  padding: 8px 0;
}

.inner-table :deep(td.el-table__cell) {
  padding: 8px 0;
}

/* 紧凑评分样式 */
.rating-compact {
  display: flex;
  align-items: center;
  gap: 2px;
}

/* 简单表格样式 */
.inner-table-wrapper {
  overflow-x: auto;
}

.simple-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.simple-table th,
.simple-table td {
  padding: 10px 8px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.simple-table thead {
  background: var(--el-fill-color-lighter);
}

.simple-table th {
  font-weight: 500;
  color: var(--el-text-color-secondary);
  text-align: left;
}

.simple-table tbody tr:hover {
  background: var(--el-fill-color-light);
}

.simple-table tbody tr:last-child td {
  border-bottom: none;
}

/* 子项列表样式 (div-based layout) */
.children-list {
  display: flex;
  flex-direction: column;
  background: var(--el-bg-color);
  flex-shrink: 0;
  overflow: visible;
}

.child-row {
  display: flex;
  align-items: center;
  padding: 10px 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  gap: 12px;
  min-height: 48px;
}

.child-row:last-child {
  border-bottom: none;
}

.child-row:hover {
  background: var(--el-fill-color-light);
}

.child-cell {
  flex-shrink: 0;
  display: flex;
  align-items: center;
}

.child-keyword {
  flex: 1;
  min-width: 200px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.child-actions {
  gap: 4px;
  justify-content: flex-end;
}

/* 列宽度定义 - 按产品视图（关键词列表） */
.child-col-rank {
  width: 120px;
  justify-content: center;
}

.child-col-trend {
  width: 90px;
  justify-content: center;
}

.child-col-actions {
  width: 110px;
  justify-content: flex-end;
}

/* 列宽度定义 - 按关键词视图（产品列表） */
.child-col-img {
  width: 70px;
  justify-content: center;
}

.child-col-asin {
  flex: 1;
  min-width: 120px;
  font-weight: 500;
}

.child-col-price {
  width: 100px;
  justify-content: center;
}

.child-col-rating {
  width: 80px;
  justify-content: center;
}

.child-header {
  background: var(--el-fill-color-lighter);
  font-weight: 500;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  min-height: 36px;
}

.child-header:hover {
  background: var(--el-fill-color-lighter);
}
</style>

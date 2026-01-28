<template>
  <div class="ad-data-charts" :class="{ collapsed: isCollapsed }">
    <div class="charts-header" @click="toggleCollapse">
      <div class="header-left">
        <el-icon class="collapse-icon" :class="{ rotated: !isCollapsed }">
          <ArrowRight />
        </el-icon>
        <span class="header-title">数据可视化</span>
        <span class="header-subtitle">{{ filteredTerms.length }} 条搜索词</span>
        <!-- 国家筛选器 -->
        <div class="country-filter" @click.stop>
          <el-select
            :model-value="selectedCountry"
            @update:model-value="handleCountryChange"
            size="small"
            placeholder="选择国家"
            style="width: 130px"
          >
            <el-option value="all" label="全部国家" />
            <el-option
              v-for="country in availableCountries"
              :key="country.code"
              :value="country.code"
              :label="country.label"
            >
              <span class="country-option">
                <span class="country-flag" v-html="country.flag"></span>
                <span>{{ country.label }}</span>
              </span>
            </el-option>
          </el-select>
        </div>
        <!-- 时间筛选器 -->
        <div class="date-filter" @click.stop>
          <el-dropdown trigger="click" @command="handleDatePreset">
            <el-button size="small" class="date-filter-btn">
              <el-icon><Calendar /></el-icon>
              <span>{{ dateRangeLabel }}</span>
              <el-icon class="arrow-icon"><ArrowDown /></el-icon>
            </el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="all">全部时间</el-dropdown-item>
                <el-dropdown-item command="7">近7天</el-dropdown-item>
                <el-dropdown-item command="15">近15天</el-dropdown-item>
                <el-dropdown-item command="30">近30天</el-dropdown-item>
                <el-dropdown-item divided command="custom">自定义时间...</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </div>
      <div class="header-right" @click.stop>
        <el-button
          v-if="!isCollapsed"
          link
          size="small"
          @click="toggleCollapse"
        >
          收起
        </el-button>
        <el-button
          v-else
          link
          size="small"
          @click="toggleCollapse"
        >
          展开
        </el-button>
      </div>
    </div>

    <Transition name="collapse">
      <div v-show="!isCollapsed" class="charts-body">
        <template v-if="hasData">
          <!-- 时间趋势图 (全宽) -->
          <div class="trend-chart-card">
            <TrendChart
              :data="trendData"
              :currency="currencySymbol"
              :target-acos="targetAcos"
            />
          </div>

          <div class="charts-grid">
            <!-- 效率四象限图 -->
            <div class="chart-card">
              <QuadrantChart
                :data="quadrantData"
                :currency="currencySymbol"
                @select="handleSelect"
              />
            </div>

            <!-- 花费效率散点图 -->
            <div class="chart-card">
              <SpendEfficiencyChart
                :data="spendEfficiencyData"
                :currency="currencySymbol"
                @select="handleSelect"
              />
            </div>

            <!-- 综合仪表盘 -->
            <div class="chart-card">
              <DashboardChart
                :data="dashboardData"
                :currency="currencySymbol"
                @select="handleSelect"
              />
            </div>

            <!-- 匹配类型对比 -->
            <div class="chart-card">
              <MatchTypeChart
                :data="matchTypeData"
                :currency="currencySymbol"
              />
            </div>
          </div>
        </template>
        <template v-else>
          <div class="empty-state">
            <el-empty description="暂无搜索词数据，请先导入数据">
            </el-empty>
          </div>
        </template>
      </div>
    </Transition>

    <!-- 自定义日期范围对话框 -->
    <el-dialog
      v-model="showDateRangeDialog"
      title="选择时间范围"
      width="360px"
      :append-to-body="true"
    >
      <el-date-picker
        v-model="customDateRange"
        type="daterange"
        range-separator="至"
        start-placeholder="开始日期"
        end-placeholder="结束日期"
        format="YYYY-MM-DD"
        value-format="YYYY-MM-DD"
        style="width: 100%"
        :clearable="false"
      />
      <template #footer>
        <el-button @click="showDateRangeDialog = false">取消</el-button>
        <el-button type="primary" @click="applyCustomDateRange">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { ArrowRight, ArrowDown, Calendar } from '@element-plus/icons-vue';
import type { AdSearchTerm } from '../../types';
import { getCountryFlag, getCountryLabel, COUNTRY_CURRENCY_MAP } from '../../types';
import {
  calculateQuadrantData,
  calculateSpendEfficiencyData,
  calculateDashboardData,
  calculateMatchTypeComparison,
  calculateTrendData,
} from '../../utils/ad-chart-utils';
import TrendChart from './charts/TrendChart.vue';
import QuadrantChart from './charts/QuadrantChart.vue';
import SpendEfficiencyChart from './charts/SpendEfficiencyChart.vue';
import DashboardChart from './charts/DashboardChart.vue';
import MatchTypeChart from './charts/MatchTypeChart.vue';

const props = defineProps<{
  terms: AdSearchTerm[];
  targetAcos: number;
  selectedCountry: string;  // 'all' 或国家代码
}>();

const emit = defineEmits<{
  (e: 'select', searchTerm: string): void;
  (e: 'update:selectedCountry', country: string): void;
}>();

// 折叠状态（从 localStorage 读取用户偏好）
const STORAGE_KEY = 'ad-charts-collapsed';
const isCollapsed = ref(loadCollapseState());

function loadCollapseState(): boolean {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    return saved === 'true';
  } catch {
    return false;
  }
}

function saveCollapseState(collapsed: boolean) {
  try {
    localStorage.setItem(STORAGE_KEY, String(collapsed));
  } catch {
    // ignore
  }
}

function toggleCollapse() {
  isCollapsed.value = !isCollapsed.value;
  saveCollapseState(isCollapsed.value);
}

// 获取可用的国家列表
const availableCountries = computed(() => {
  const countrySet = new Set<string>();
  props.terms.forEach(term => {
    if (term.country) {
      countrySet.add(term.country);
    }
  });
  return Array.from(countrySet).map(code => ({
    code,
    label: getCountryLabel(code),
    flag: getCountryFlag(code),
  })).sort((a, b) => a.label.localeCompare(b.label, 'zh'));
});

// 获取当前货币符号
const currencySymbol = computed(() => {
  if (props.selectedCountry === 'all') {
    // 多国家时，尝试获取第一个国家的货币，或使用默认
    const firstCountry = availableCountries.value[0]?.code;
    if (firstCountry) {
      return COUNTRY_CURRENCY_MAP[firstCountry]?.symbol || '$';
    }
    return '$';
  }
  return COUNTRY_CURRENCY_MAP[props.selectedCountry]?.symbol || '$';
});

// 处理国家切换
function handleCountryChange(country: string) {
  emit('update:selectedCountry', country);
}

// ============ 时间筛选器 ============

// 时间筛选状态：'all' | '7' | '15' | '30' | 'custom'
const dateFilterMode = ref<string>('all');
const customDateRange = ref<[string, string] | null>(null);
const showDateRangeDialog = ref(false);

// 获取数据的日期范围
const dataDateRange = computed(() => {
  const dates = props.terms
    .map(t => t.report_date)
    .filter((d): d is string => !!d)
    .sort();
  if (dates.length === 0) return { start: '', end: '' };
  return { start: dates[0], end: dates[dates.length - 1] };
});

// 计算当前选中的日期范围
const selectedDateRange = computed<{ start: string; end: string } | null>(() => {
  if (dateFilterMode.value === 'all') {
    return null; // 全部时间，不过滤
  }

  if (dateFilterMode.value === 'custom' && customDateRange.value) {
    return { start: customDateRange.value[0], end: customDateRange.value[1] };
  }

  // 近N天
  const days = parseInt(dateFilterMode.value);
  if (!isNaN(days) && dataDateRange.value.end) {
    // 以数据中的最新日期为基准，往前推N天
    const endDate = new Date(dataDateRange.value.end);
    const startDate = new Date(endDate);
    startDate.setDate(startDate.getDate() - days + 1);

    const formatDate = (d: Date) => {
      const year = d.getFullYear();
      const month = String(d.getMonth() + 1).padStart(2, '0');
      const day = String(d.getDate()).padStart(2, '0');
      return `${year}-${month}-${day}`;
    };

    return { start: formatDate(startDate), end: formatDate(endDate) };
  }

  return null;
});

// 时间筛选器显示标签
const dateRangeLabel = computed(() => {
  if (dateFilterMode.value === 'all') return '全部时间';
  if (dateFilterMode.value === '7') return '近7天';
  if (dateFilterMode.value === '15') return '近15天';
  if (dateFilterMode.value === '30') return '近30天';
  if (dateFilterMode.value === 'custom' && customDateRange.value) {
    return `${customDateRange.value[0]} ~ ${customDateRange.value[1]}`;
  }
  return '选择时间';
});

// 处理快速选择
function handleDatePreset(command: string) {
  if (command === 'custom') {
    // 打开自定义日期对话框，默认选中数据的日期范围
    if (dataDateRange.value.start && dataDateRange.value.end) {
      customDateRange.value = [dataDateRange.value.start, dataDateRange.value.end];
    }
    showDateRangeDialog.value = true;
  } else {
    dateFilterMode.value = command;
  }
}

// 应用自定义日期范围
function applyCustomDateRange() {
  if (customDateRange.value) {
    dateFilterMode.value = 'custom';
  }
  showDateRangeDialog.value = false;
}

// ============ 数据筛选 ============

// 根据国家筛选
const countryFilteredTerms = computed(() => {
  if (props.selectedCountry === 'all') {
    return props.terms;
  }
  return props.terms.filter(term => term.country === props.selectedCountry);
});

// 根据日期范围进一步筛选
const filteredTerms = computed(() => {
  let terms = countryFilteredTerms.value;

  const range = selectedDateRange.value;
  if (range) {
    terms = terms.filter(term => {
      if (!term.report_date) return false;
      return term.report_date >= range.start && term.report_date <= range.end;
    });
  }

  return terms;
});

// 是否有数据
const hasData = computed(() => filteredTerms.value.length > 0);

// 计算四象限数据（使用筛选后的数据）
const quadrantData = computed(() => {
  return calculateQuadrantData(filteredTerms.value, props.targetAcos);
});

// 计算花费效率数据（使用筛选后的数据）
const spendEfficiencyData = computed(() => {
  return calculateSpendEfficiencyData(filteredTerms.value, props.targetAcos);
});

// 计算仪表盘数据（使用筛选后的数据）
const dashboardData = computed(() => {
  return calculateDashboardData(filteredTerms.value);
});

// 计算匹配类型对比数据（使用筛选后的数据）
const matchTypeData = computed(() => {
  return calculateMatchTypeComparison(filteredTerms.value);
});

// 计算时间趋势数据（使用筛选后的数据）
const trendData = computed(() => {
  return calculateTrendData(filteredTerms.value);
});

// 处理选中搜索词
function handleSelect(searchTerm: string) {
  emit('select', searchTerm);
}

// 监听数据变化，如果没有数据则折叠
watch(() => props.terms.length, (newLen, oldLen) => {
  if (newLen > 0 && oldLen === 0) {
    // 有新数据时展开
    isCollapsed.value = false;
  }
});

onMounted(() => {
  // 如果没有数据，默认折叠
  if (!hasData.value) {
    isCollapsed.value = true;
  }
});
</script>

<style scoped>
.ad-data-charts {
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-light);
  border-radius: 8px;
  margin-bottom: 24px;
  overflow: hidden;
}

.charts-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  cursor: pointer;
  user-select: none;
  transition: background-color 0.2s;
}

.charts-header:hover {
  background: var(--el-fill-color-lighter);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.collapse-icon {
  transition: transform 0.3s;
  color: var(--el-text-color-secondary);
}

.collapse-icon.rotated {
  transform: rotate(90deg);
}

.header-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.header-subtitle {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.country-filter {
  margin-left: 16px;
}

.date-filter {
  margin-left: 8px;
}

.date-filter-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.date-filter-btn .arrow-icon {
  margin-left: 2px;
  font-size: 12px;
}

.country-option {
  display: flex;
  align-items: center;
  gap: 8px;
}

.country-flag {
  display: inline-flex;
  align-items: center;
  width: 20px;
  height: 14px;
}

.country-flag :deep(svg) {
  width: 100%;
  height: 100%;
  border-radius: 2px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.charts-body {
  padding: 0 20px 20px;
}

.trend-chart-card {
  background: var(--el-fill-color-lighter);
  border-radius: 8px;
  padding: 16px;
  height: 400px;
  margin-bottom: 16px;
  display: flex;
  flex-direction: column;
}

.charts-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

.chart-card {
  background: var(--el-fill-color-lighter);
  border-radius: 8px;
  padding: 16px;
  height: 380px;
  display: flex;
  flex-direction: column;
}

.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 200px;
}

/* 折叠动画 */
.collapse-enter-active,
.collapse-leave-active {
  transition: all 0.3s ease;
  overflow: hidden;
}

.collapse-enter-from,
.collapse-leave-to {
  opacity: 0;
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
}

.collapse-enter-to,
.collapse-leave-from {
  opacity: 1;
  max-height: 1000px;
}

/* 响应式布局 */
@media (max-width: 1200px) {
  .charts-grid {
    grid-template-columns: 1fr;
  }

  .chart-card {
    height: 350px;
  }
}
</style>

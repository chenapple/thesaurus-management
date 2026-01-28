<template>
  <div class="dashboard-chart">
    <div class="chart-header">
      <span class="chart-title">综合仪表盘</span>
    </div>
    <div class="dashboard-content">
      <!-- 上排：ACOS 分布 + 花费占比 -->
      <div class="dashboard-row">
        <div class="mini-chart acos-distribution">
          <div class="mini-title">ACOS 分布</div>
          <v-chart class="chart" :option="acosChartOption" autoresize />
        </div>
        <div class="mini-chart spend-share">
          <div class="mini-title">
            花费占比
            <el-radio-group v-model="spendGroupBy" size="small">
              <el-radio-button value="country">按国家</el-radio-button>
              <el-radio-button value="campaign">按活动</el-radio-button>
            </el-radio-group>
          </div>
          <v-chart class="chart" :option="spendChartOption" autoresize />
        </div>
      </div>
      <!-- 下排：Top 排行榜 -->
      <div class="top-ranking">
        <el-tabs v-model="activeTab" class="ranking-tabs">
          <el-tab-pane label="花费 Top10" name="spend">
            <div class="ranking-list">
              <div
                v-for="(item, index) in data.topSpend"
                :key="item.searchTerm"
                class="ranking-item"
                @click="$emit('select', item.searchTerm)"
              >
                <span class="rank-badge" :class="getRankClass(index)">{{ index + 1 }}</span>
                <span class="item-name" :title="item.searchTerm">{{ item.searchTerm }}</span>
                <div class="item-bar-container">
                  <div
                    class="item-bar"
                    :style="{ width: getBarWidth(item.value, maxSpend) + '%' }"
                  ></div>
                </div>
                <span class="item-value">{{ currency }}{{ item.value.toFixed(2) }}</span>
              </div>
              <div v-if="data.topSpend.length === 0" class="empty-tip">暂无数据</div>
            </div>
          </el-tab-pane>
          <el-tab-pane label="销售 Top10" name="sales">
            <div class="ranking-list">
              <div
                v-for="(item, index) in data.topSales"
                :key="item.searchTerm"
                class="ranking-item"
                @click="$emit('select', item.searchTerm)"
              >
                <span class="rank-badge" :class="getRankClass(index)">{{ index + 1 }}</span>
                <span class="item-name" :title="item.searchTerm">{{ item.searchTerm }}</span>
                <div class="item-bar-container">
                  <div
                    class="item-bar sales-bar"
                    :style="{ width: getBarWidth(item.value, maxSales) + '%' }"
                  ></div>
                </div>
                <span class="item-value">{{ currency }}{{ item.value.toFixed(2) }}</span>
              </div>
              <div v-if="data.topSales.length === 0" class="empty-tip">暂无数据</div>
            </div>
          </el-tab-pane>
          <el-tab-pane label="浪费 Top10" name="waste">
            <div class="ranking-list">
              <div
                v-for="(item, index) in data.topWaste"
                :key="item.searchTerm"
                class="ranking-item waste-item"
                @click="$emit('select', item.searchTerm)"
              >
                <span class="rank-badge" :class="getRankClass(index)">{{ index + 1 }}</span>
                <span class="item-name" :title="item.searchTerm">{{ item.searchTerm }}</span>
                <div class="item-bar-container">
                  <div
                    class="item-bar waste-bar"
                    :style="{ width: getBarWidth(item.value, maxWaste) + '%' }"
                  ></div>
                </div>
                <span class="item-value waste-value">{{ currency }}{{ item.value.toFixed(2) }}</span>
              </div>
              <div v-if="data.topWaste.length === 0" class="empty-tip">暂无浪费数据</div>
            </div>
          </el-tab-pane>
        </el-tabs>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import VChart from 'vue-echarts';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { BarChart, PieChart } from 'echarts/charts';
import {
  GridComponent,
  TooltipComponent,
  LegendComponent,
} from 'echarts/components';
import type { ComposeOption } from 'echarts/core';
import type { BarSeriesOption, PieSeriesOption } from 'echarts/charts';
import type {
  GridComponentOption,
  TooltipComponentOption,
  LegendComponentOption,
} from 'echarts/components';
import type { DashboardData } from '../../../utils/ad-chart-utils';

use([
  CanvasRenderer,
  BarChart,
  PieChart,
  GridComponent,
  TooltipComponent,
  LegendComponent,
]);

type EChartsOption = ComposeOption<
  | BarSeriesOption
  | PieSeriesOption
  | GridComponentOption
  | TooltipComponentOption
  | LegendComponentOption
>;

const props = defineProps<{
  data: DashboardData;
  currency: string;
}>();

defineEmits<{
  (e: 'select', searchTerm: string): void;
}>();

const spendGroupBy = ref<'country' | 'campaign'>('country');
const activeTab = ref('spend');

// 计算最大值用于条形图
const maxSpend = computed(() => Math.max(...props.data.topSpend.map(i => i.value), 1));
const maxSales = computed(() => Math.max(...props.data.topSales.map(i => i.value), 1));
const maxWaste = computed(() => Math.max(...props.data.topWaste.map(i => i.value), 1));

// ACOS 分布柱状图配置
const acosChartOption = computed<EChartsOption>(() => ({
  tooltip: {
    trigger: 'axis',
    axisPointer: { type: 'shadow' },
    formatter: (params: any) => {
      const data = params[0];
      return `${data.name}: ${data.value} 个搜索词`;
    },
  },
  grid: {
    left: 8,
    right: 8,
    top: 20,
    bottom: 24,
    containLabel: true,
  },
  xAxis: {
    type: 'category',
    data: props.data.acosDistribution.map(d => d.range),
    axisLabel: {
      fontSize: 10,
      color: 'var(--el-text-color-secondary)',
      rotate: 0,
    },
    axisTick: { show: false },
    axisLine: { show: false },
  },
  yAxis: {
    type: 'value',
    show: false,
  },
  series: [
    {
      type: 'bar',
      data: props.data.acosDistribution.map(d => ({
        value: d.count,
        itemStyle: { color: d.color },
      })),
      barWidth: '60%',
      label: {
        show: true,
        position: 'top',
        fontSize: 10,
        color: 'var(--el-text-color-secondary)',
      },
    },
  ],
}));

// 花费占比饼图配置
const spendChartOption = computed<EChartsOption>(() => {
  const dataSource = spendGroupBy.value === 'country'
    ? props.data.spendByCountry
    : props.data.spendByCampaign;

  return {
    tooltip: {
      trigger: 'item',
      formatter: (params: any) => {
        return `${params.name}: ${props.currency}${params.value.toFixed(2)} (${params.percent}%)`;
      },
    },
    legend: {
      orient: 'vertical',
      right: 0,
      top: 'center',
      itemWidth: 10,
      itemHeight: 10,
      textStyle: {
        fontSize: 10,
        color: 'var(--el-text-color-secondary)',
      },
      formatter: (name: string) => {
        return name.length > 8 ? name.substring(0, 8) + '...' : name;
      },
    },
    series: [
      {
        type: 'pie',
        radius: ['40%', '70%'],
        center: ['35%', '50%'],
        avoidLabelOverlap: false,
        label: { show: false },
        emphasis: {
          label: { show: false },
        },
        data: dataSource.map((item, index) => ({
          name: item.name,
          value: item.value,
          itemStyle: {
            color: [
              '#409eff',
              '#67c23a',
              '#e6a23c',
              '#f56c6c',
              '#909399',
              '#b1a3e6',
            ][index % 6],
          },
        })),
      },
    ],
  };
});

function getRankClass(index: number): string {
  if (index === 0) return 'rank-1';
  if (index === 1) return 'rank-2';
  if (index === 2) return 'rank-3';
  return '';
}

function getBarWidth(value: number, max: number): number {
  return max > 0 ? (value / max) * 100 : 0;
}
</script>

<style scoped>
.dashboard-chart {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.chart-header {
  margin-bottom: 12px;
}

.chart-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.dashboard-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-height: 0;
  overflow: hidden;
}

.dashboard-row {
  display: flex;
  gap: 12px;
  height: 140px;
}

.mini-chart {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--el-fill-color-lighter);
  border-radius: 8px;
  padding: 8px;
}

.mini-title {
  font-size: 12px;
  font-weight: 500;
  color: var(--el-text-color-secondary);
  margin-bottom: 4px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.mini-title :deep(.el-radio-button__inner) {
  padding: 2px 6px;
  font-size: 10px;
}

.mini-chart .chart {
  flex: 1;
  min-height: 0;
}

.top-ranking {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.ranking-tabs {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.ranking-tabs :deep(.el-tabs__header) {
  margin-bottom: 8px;
}

.ranking-tabs :deep(.el-tabs__content) {
  flex: 1;
  overflow: hidden;
}

.ranking-tabs :deep(.el-tab-pane) {
  height: 100%;
}

.ranking-list {
  height: 100%;
  overflow-y: auto;
  padding-right: 4px;
}

.ranking-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 4px;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.ranking-item:hover {
  background: var(--el-fill-color);
}

.rank-badge {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: var(--el-fill-color);
  color: var(--el-text-color-secondary);
  font-size: 11px;
  font-weight: 500;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.rank-badge.rank-1 {
  background: linear-gradient(135deg, #ffd700 0%, #ffb347 100%);
  color: white;
}

.rank-badge.rank-2 {
  background: linear-gradient(135deg, #c0c0c0 0%, #a8a8a8 100%);
  color: white;
}

.rank-badge.rank-3 {
  background: linear-gradient(135deg, #cd7f32 0%, #b87333 100%);
  color: white;
}

.item-name {
  flex: 1;
  font-size: 12px;
  color: var(--el-text-color-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.item-bar-container {
  width: 80px;
  height: 8px;
  background: var(--el-fill-color);
  border-radius: 4px;
  overflow: hidden;
  flex-shrink: 0;
}

.item-bar {
  height: 100%;
  background: linear-gradient(90deg, #409eff 0%, #79bbff 100%);
  border-radius: 4px;
  transition: width 0.3s ease;
}

.item-bar.sales-bar {
  background: linear-gradient(90deg, #67c23a 0%, #95d475 100%);
}

.item-bar.waste-bar {
  background: linear-gradient(90deg, #f56c6c 0%, #fab6b6 100%);
}

.item-value {
  font-size: 12px;
  font-weight: 500;
  color: var(--el-text-color-primary);
  min-width: 60px;
  text-align: right;
}

.waste-item .item-name {
  color: var(--el-color-danger);
}

.waste-value {
  color: var(--el-color-danger) !important;
}

.empty-tip {
  text-align: center;
  color: var(--el-text-color-placeholder);
  font-size: 12px;
  padding: 20px;
}
</style>

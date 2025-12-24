<template>
  <el-dialog
    :model-value="modelValue"
    :title="dialogTitle"
    width="800px"
    @update:model-value="$emit('update:modelValue', $event)"
  >
    <div class="history-container" v-loading="loading">
      <!-- 时间范围选择 -->
      <div class="toolbar">
        <el-radio-group v-model="days" size="small" @change="loadHistory">
          <el-radio-button :value="7">7天</el-radio-button>
          <el-radio-button :value="14">14天</el-radio-button>
          <el-radio-button :value="30">30天</el-radio-button>
        </el-radio-group>
      </div>

      <!-- 图表 -->
      <div class="chart-container" v-if="history.length">
        <v-chart class="chart" :option="chartOption" autoresize />
      </div>
      <el-empty v-else description="暂无排名历史数据" />

      <!-- 数据表格 -->
      <div class="history-table" v-if="history.length">
        <el-table :data="history" size="small" border max-height="200">
          <el-table-column label="日期" prop="check_date" width="100" />
          <el-table-column v-if="props.displayType !== 'sponsored'" label="自然排名" align="center">
            <template #default="{ row }">
              <span v-if="row.organic_rank" :class="getRankClass(row.organic_rank)" class="rank-display">
                <span class="rank-page">第{{ row.organic_page || 1 }}页</span>
                <span class="rank-position">第{{ row.organic_rank }}名</span>
              </span>
              <span v-else class="no-rank">-</span>
            </template>
          </el-table-column>
          <el-table-column v-if="props.displayType !== 'organic'" label="广告排名" align="center">
            <template #default="{ row }">
              <span v-if="row.sponsored_rank" class="rank-sponsored rank-display">
                <span class="rank-page">第{{ row.sponsored_page || 1 }}页</span>
                <span class="rank-position">第{{ row.sponsored_rank }}名</span>
              </span>
              <span v-else class="no-rank">-</span>
            </template>
          </el-table-column>
          <el-table-column label="检测时间" prop="checked_at" width="160">
            <template #default="{ row }">
              {{ formatDateTime(row.checked_at) }}
            </template>
          </el-table-column>
        </el-table>
      </div>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import VChart from 'vue-echarts';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { LineChart } from 'echarts/charts';
import {
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
} from 'echarts/components';
import type { ComposeOption } from 'echarts/core';
import type { LineSeriesOption } from 'echarts/charts';
import type {
  TitleComponentOption,
  TooltipComponentOption,
  LegendComponentOption,
  GridComponentOption,
} from 'echarts/components';
import { getRankingHistory } from '../api';
import type { KeywordMonitoring, RankingHistory } from '../types';

// 注册 ECharts 组件
use([
  CanvasRenderer,
  LineChart,
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
]);

type EChartsOption = ComposeOption<
  | LineSeriesOption
  | TitleComponentOption
  | TooltipComponentOption
  | LegendComponentOption
  | GridComponentOption
>;

const props = withDefaults(defineProps<{
  modelValue: boolean;
  monitoring: KeywordMonitoring | null;
  displayType?: 'organic' | 'sponsored' | 'all';
}>(), {
  displayType: 'all',
});

defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>();

const loading = ref(false);
const days = ref(7);
const history = ref<RankingHistory[]>([]);

// 动态标题
const dialogTitle = computed(() => {
  const keyword = props.monitoring?.keyword || '';
  if (props.displayType === 'organic') {
    return `自然排名历史 - ${keyword}`;
  } else if (props.displayType === 'sponsored') {
    return `广告排名历史 - ${keyword}`;
  }
  return `排名历史 - ${keyword}`;
});

// 加载历史数据
async function loadHistory() {
  if (!props.monitoring) return;

  loading.value = true;
  try {
    history.value = await getRankingHistory(props.monitoring.id, days.value);
  } catch (e) {
    console.error('加载历史数据失败:', e);
    history.value = [];
  } finally {
    loading.value = false;
  }
}

// 格式化日期为短格式 (MM-DD)
function formatDateShort(dateStr: string): string {
  const parts = dateStr.split('-');
  if (parts.length >= 3) {
    return `${parts[1]}-${parts[2]}`;
  }
  return dateStr;
}

// 图表配置
const chartOption = computed<EChartsOption>(() => {
  const dates = history.value.map(h => formatDateShort(h.check_date));
  const organicRanks = history.value.map(h => h.organic_rank);
  const sponsoredRanks = history.value.map(h => h.sponsored_rank);

  // 根据 displayType 过滤图例和系列
  const legendData: string[] = [];
  const series: LineSeriesOption[] = [];

  if (props.displayType !== 'sponsored') {
    legendData.push('自然排名');
    series.push({
      name: '自然排名',
      type: 'line',
      data: organicRanks,
      smooth: true,
      symbol: 'circle',
      symbolSize: 6,
      lineStyle: {
        color: '#67c23a',
        width: 2,
      },
      itemStyle: {
        color: '#67c23a',
      },
      connectNulls: true,
    });
  }

  if (props.displayType !== 'organic') {
    legendData.push('广告排名');
    series.push({
      name: '广告排名',
      type: 'line',
      data: sponsoredRanks,
      smooth: true,
      symbol: 'circle',
      symbolSize: 6,
      lineStyle: {
        color: '#409eff',
        width: 2,
      },
      itemStyle: {
        color: '#409eff',
      },
      connectNulls: true,
    });
  }

  return {
    tooltip: {
      trigger: 'axis',
      formatter: (params: any) => {
        const date = params[0]?.axisValue;
        let html = `<div style="font-weight:bold">${date}</div>`;
        for (const p of params) {
          const value = p.value ?? '-';
          html += `<div>${p.marker} ${p.seriesName}: ${value}</div>`;
        }
        return html;
      },
    },
    legend: {
      data: legendData,
      bottom: 0,
      show: legendData.length > 1, // 只有一个系列时隐藏图例
    },
    grid: {
      left: 50,
      right: 20,
      top: 20,
      bottom: legendData.length > 1 ? 60 : 40,
    },
    xAxis: {
      type: 'category',
      data: dates,
      axisLabel: {
        rotate: 0,
        fontSize: 11,
        interval: 0,
        formatter: (value: string) => value,
      },
      axisTick: {
        alignWithLabel: true,
      },
    },
    yAxis: {
      type: 'value',
      inverse: true, // 排名越小越好，所以倒序
      min: 1,
      axisLabel: {
        formatter: '{value}',
      },
    },
    series,
  };
});

// 工具函数
function getRankClass(rank: number | null): string {
  if (rank === null) return '';
  if (rank <= 10) return 'rank-top10';
  if (rank <= 30) return 'rank-top30';
  return 'rank-low';
}

function formatDateTime(dateStr: string): string {
  // 转换为北京时间 (UTC+8)
  const utcDateStr = dateStr.endsWith('Z') ? dateStr : dateStr.replace(' ', 'T') + 'Z';
  const date = new Date(utcDateStr);
  const beijingTime = new Date(date.getTime() + 8 * 60 * 60 * 1000);
  const month = (beijingTime.getUTCMonth() + 1).toString().padStart(2, '0');
  const day = beijingTime.getUTCDate().toString().padStart(2, '0');
  const hour = beijingTime.getUTCHours().toString().padStart(2, '0');
  const minute = beijingTime.getUTCMinutes().toString().padStart(2, '0');
  return `${month}-${day} ${hour}:${minute}`;
}

// 监听对话框打开
watch(() => props.modelValue, (val) => {
  if (val) {
    loadHistory();
  }
}, { immediate: true });
</script>

<style scoped>
.history-container {
  min-height: 300px;
}

.toolbar {
  margin-bottom: 16px;
}

.chart-container {
  height: 300px;
  margin-bottom: 16px;
}

.chart {
  width: 100%;
  height: 100%;
}

.history-table {
  margin-top: 16px;
}

.rank-top10 {
  color: var(--el-color-success);
  font-weight: bold;
}

.rank-top30 {
  color: var(--el-color-warning);
  font-weight: bold;
}

.rank-low {
  color: var(--el-color-danger);
}

.rank-sponsored {
  color: var(--el-color-primary);
  font-weight: bold;
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

.no-rank {
  color: var(--el-text-color-placeholder);
}
</style>

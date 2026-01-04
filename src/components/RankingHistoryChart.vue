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
        <el-switch
          v-if="props.events?.length"
          v-model="showEventLines"
          size="small"
          active-text="显示事件"
          style="margin-left: 16px;"
        />
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
import { MarkLineComponent } from 'echarts/components';
import type { MarkLineComponentOption } from 'echarts/components';
import { getRankingHistory } from '../api';
import type { KeywordMonitoring, RankingHistory, OptimizationEvent } from '../types';
import { EVENT_MAIN_TYPES, EVENT_SUB_TYPES, type EventMainType } from '../types';

// 注册 ECharts 组件
use([
  CanvasRenderer,
  LineChart,
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
  MarkLineComponent,
]);

// 每页自然位数量（Amazon 固定值）
const ORGANIC_ITEMS_PER_PAGE = 48;

// 计算自然排名的绝对排名
function calculateAbsoluteRank(page: number | null, rank: number | null): number | null {
  if (page === null || rank === null) return null;
  return (page - 1) * ORGANIC_ITEMS_PER_PAGE + rank;
}

type EChartsOption = ComposeOption<
  | LineSeriesOption
  | TitleComponentOption
  | TooltipComponentOption
  | LegendComponentOption
  | GridComponentOption
  | MarkLineComponentOption
>;

const props = withDefaults(defineProps<{
  modelValue: boolean;
  monitoring: KeywordMonitoring | null;
  displayType?: 'organic' | 'sponsored' | 'all';
  events?: OptimizationEvent[];
}>(), {
  displayType: 'all',
  events: () => [],
});

defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>();

const loading = ref(false);
const days = ref(7);
const history = ref<RankingHistory[]>([]);
const showEventLines = ref(true);

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

// 获取子类型标签
function getSubTypeLabel(mainType: string, subType?: string): string {
  if (!subType) return '';
  const subTypes = EVENT_SUB_TYPES[mainType as EventMainType];
  if (!subTypes) return '';
  return subTypes[subType]?.label || '';
}

// 判断事件是否应该显示（基于三级范围匹配）
function shouldShowEvent(event: OptimizationEvent, monitoring: KeywordMonitoring): boolean {
  // 1. 产品级别：target_asin 为空 → 所有监控项都显示
  if (!event.target_asin) return true;

  // 2. ASIN 级别：target_asin 匹配，且无关联关键词 → 该 ASIN 下所有关键词都显示
  if (event.target_asin === monitoring.asin && !event.affected_keywords) return true;

  // 3. 关键词级别：target_asin 匹配 + 关键词匹配 → 只有该组合显示
  if (event.target_asin === monitoring.asin && event.affected_keywords) {
    try {
      const keywords: string[] = JSON.parse(event.affected_keywords);
      return keywords.includes(monitoring.keyword);
    } catch {
      return false;
    }
  }

  return false;
}

// 生成事件标记线数据
function getEventMarkLines() {
  if (!props.events?.length || !props.monitoring) return [];

  const dates = history.value.map(h => h.check_date);

  // 过滤出符合条件的事件
  const filteredEvents = props.events.filter(event =>
    dates.includes(event.event_date) &&
    shouldShowEvent(event, props.monitoring!)
  );

  // 按日期分组，计算每个事件在当天的索引（用于垂直错开）
  const dateIndexMap: Record<string, number> = {};

  return filteredEvents.map(event => {
    const dateKey = event.event_date;
    const indexInDay = dateIndexMap[dateKey] || 0;
    dateIndexMap[dateKey] = indexInDay + 1;

    const typeInfo = EVENT_MAIN_TYPES[event.event_type as EventMainType] || EVENT_MAIN_TYPES.listing;
    const subTypeLabel = getSubTypeLabel(event.event_type, event.event_sub_type);
    const displayLabel = subTypeLabel ? `${typeInfo.label}-${subTypeLabel}` : typeInfo.label;

    // 垂直偏移量：每个事件向下偏移 16px
    const verticalOffset = indexInDay * 16;

    return {
      xAxis: formatDateShort(event.event_date),
      label: {
        show: true,
        formatter: displayLabel,
        position: 'insideStartTop' as const,
        fontSize: 10,
        color: typeInfo.color,
        offset: [0, verticalOffset],  // 垂直错开
      },
      lineStyle: {
        color: typeInfo.color,
        type: 'dashed' as const,
        width: 2,
      },
      // 用于 tooltip
      name: event.title,
      eventType: event.event_type,
      eventSubType: event.event_sub_type,
    };
  });
}

// 图表配置
const chartOption = computed<EChartsOption>(() => {
  const dates = history.value.map(h => formatDateShort(h.check_date));

  // 自然排名：计算绝对排名
  const organicData = history.value.map(h => ({
    page: h.organic_page,
    rank: h.organic_rank,
    absolute: calculateAbsoluteRank(h.organic_page, h.organic_rank),
  }));

  // 广告排名：不计算绝对排名（广告位数量不固定）
  const sponsoredData = history.value.map(h => ({
    page: h.sponsored_page,
    rank: h.sponsored_rank,
    absolute: h.sponsored_rank,  // 直接使用页内排名作为 Y 轴值
  }));

  // 根据 displayType 过滤图例和系列
  const legendData: string[] = [];
  const series: LineSeriesOption[] = [];

  // 获取事件标记线（仅当开关打开时）
  const markLineData = showEventLines.value ? getEventMarkLines() : [];

  // 获取有事件的日期集合（用于高亮数据点）
  const eventDates = new Set(markLineData.map(m => m.xAxis as string));

  // 生成带高亮的数据点
  function createDataWithHighlight(
    dataList: Array<{ page: number | null; rank: number | null; absolute: number | null }>,
    color: string
  ) {
    return dataList.map((item, index) => {
      const date = dates[index];
      const hasEvent = eventDates.has(date);
      if (item.absolute === null) return null;
      return {
        value: item.absolute,  // Y轴使用绝对排名（自然位）或页内排名（广告位）
        page: item.page,       // 保存页码用于 tooltip
        rank: item.rank,       // 保存页内排名用于 tooltip
        symbolSize: hasEvent ? 10 : 6,
        itemStyle: hasEvent ? {
          color: color,
          borderWidth: 3,
          borderColor: '#ff9800',
        } : { color },
      };
    });
  }

  // markLine 配置（带 tooltip）
  const markLineConfig = markLineData.length > 0 ? {
    silent: false,
    symbol: ['none', 'none'],
    data: markLineData,
    tooltip: {
      show: true,
      formatter: (params: any) => {
        const data = params.data;
        const typeInfo = EVENT_MAIN_TYPES[data.eventType as EventMainType] || {};
        const subTypeLabel = getSubTypeLabel(data.eventType, data.eventSubType);
        const typeLabel = subTypeLabel ? `${typeInfo.label}-${subTypeLabel}` : typeInfo.label;
        return `
          <div style="padding: 8px;">
            <div style="font-weight:bold; margin-bottom: 4px;">${data.name}</div>
            <div style="color: ${typeInfo.color};">类型: ${typeLabel}</div>
            <div>日期: ${data.xAxis}</div>
          </div>
        `;
      },
    },
  } : undefined;

  if (props.displayType !== 'sponsored') {
    legendData.push('自然排名');
    series.push({
      name: '自然排名',
      type: 'line',
      data: createDataWithHighlight(organicData, '#67c23a'),
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
      markLine: markLineConfig,
    });
  }

  if (props.displayType !== 'organic') {
    legendData.push('广告排名');
    series.push({
      name: '广告排名',
      type: 'line',
      data: createDataWithHighlight(sponsoredData, '#409eff'),
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
      markLine: (props.displayType === 'sponsored') ? markLineConfig : undefined,
    });
  }

  return {
    tooltip: {
      trigger: 'axis',
      formatter: (params: any) => {
        const date = params[0]?.axisValue;
        let html = `<div style="font-weight:bold">${date}</div>`;
        for (const p of params) {
          if (p.data === null) continue;
          const { page, rank, value } = p.data;
          if (value !== undefined && value !== null) {
            if (p.seriesName === '自然排名') {
              // 自然排名显示绝对排名
              html += `<div>${p.marker} ${p.seriesName}: 第${page}页 第${rank}名 (总第${value}名)</div>`;
            } else {
              // 广告排名只显示页码和页内排名
              html += `<div>${p.marker} ${p.seriesName}: 第${page}页 第${rank}名</div>`;
            }
          } else {
            html += `<div>${p.marker} ${p.seriesName}: -</div>`;
          }
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
  // 数据库存储的是 UTC 时间，添加 'Z' 后缀确保解析为 UTC
  // 然后使用本地时间方法显示（浏览器会自动转换为用户本地时区）
  const utcDateStr = dateStr.endsWith('Z') ? dateStr : dateStr.replace(' ', 'T') + 'Z';
  const date = new Date(utcDateStr);
  const month = (date.getMonth() + 1).toString().padStart(2, '0');
  const day = date.getDate().toString().padStart(2, '0');
  const hour = date.getHours().toString().padStart(2, '0');
  const minute = date.getMinutes().toString().padStart(2, '0');
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
  display: flex;
  align-items: center;
  gap: 16px;
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

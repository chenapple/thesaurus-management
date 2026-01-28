<template>
  <div class="match-type-chart">
    <div class="chart-header">
      <span class="chart-title">匹配类型对比</span>
    </div>
    <div class="chart-body">
      <v-chart class="chart" :option="chartOption" autoresize />
    </div>
    <div class="stats-summary">
      <div
        v-for="stat in data.stats"
        :key="stat.matchType"
        class="stat-item"
      >
        <span class="stat-type">{{ stat.matchType }}</span>
        <span class="stat-count">{{ stat.count }} 条</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import VChart from 'vue-echarts';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { BarChart } from 'echarts/charts';
import {
  GridComponent,
  TooltipComponent,
  LegendComponent,
} from 'echarts/components';
import type { ComposeOption } from 'echarts/core';
import type { BarSeriesOption } from 'echarts/charts';
import type {
  GridComponentOption,
  TooltipComponentOption,
  LegendComponentOption,
} from 'echarts/components';
import type { MatchTypeComparisonData } from '../../../utils/ad-chart-utils';

use([
  CanvasRenderer,
  BarChart,
  GridComponent,
  TooltipComponent,
  LegendComponent,
]);

type EChartsOption = ComposeOption<
  | BarSeriesOption
  | GridComponentOption
  | TooltipComponentOption
  | LegendComponentOption
>;

const props = defineProps<{
  data: MatchTypeComparisonData;
  currency: string;
}>();

const chartOption = computed<EChartsOption>(() => {
  const { stats } = props.data;
  const matchTypes = stats.map(s => s.matchType);

  return {
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'shadow' },
      formatter: (params: any) => {
        const matchType = params[0].axisValue;
        const stat = stats.find(s => s.matchType === matchType);
        if (!stat) return '';

        let html = `<div style="font-weight: bold; margin-bottom: 8px;">${matchType}</div>`;
        params.forEach((p: any) => {
          html += `<div style="display: flex; justify-content: space-between; gap: 16px;">
            <span>${p.marker} ${p.seriesName}:</span>
            <span style="font-weight: 500;">${p.value.toFixed(1)}${p.seriesName.includes('率') || p.seriesName.includes('占比') ? '%' : ''}</span>
          </div>`;
        });
        html += `<div style="margin-top: 8px; padding-top: 8px; border-top: 1px solid rgba(255,255,255,0.2);">
          <div>总花费: ${props.currency}${stat.totalSpend.toFixed(2)}</div>
          <div>总销售: ${props.currency}${stat.totalSales.toFixed(2)}</div>
          <div>搜索词数: ${stat.count}</div>
        </div>`;
        return html;
      },
    },
    legend: {
      data: ['平均 ACOS', '平均转化率', '花费占比', '销售占比'],
      bottom: 0,
      itemWidth: 14,
      itemHeight: 10,
      textStyle: {
        fontSize: 11,
        color: 'var(--el-text-color-secondary)',
      },
    },
    grid: {
      left: 50,
      right: 20,
      top: 30,
      bottom: 60,
    },
    xAxis: {
      type: 'category',
      data: matchTypes,
      axisLabel: {
        color: 'var(--el-text-color-secondary)',
        fontSize: 11,
      },
      axisTick: { show: false },
      axisLine: {
        lineStyle: {
          color: 'var(--el-border-color-lighter)',
        },
      },
    },
    yAxis: {
      type: 'value',
      name: '%',
      nameTextStyle: {
        color: 'var(--el-text-color-secondary)',
        fontSize: 11,
      },
      axisLabel: {
        color: 'var(--el-text-color-secondary)',
        fontSize: 11,
      },
      splitLine: {
        lineStyle: {
          type: 'dashed',
          color: 'var(--el-border-color-lighter)',
        },
      },
    },
    series: [
      {
        name: '平均 ACOS',
        type: 'bar',
        barGap: '10%',
        barWidth: '15%',
        data: stats.map(s => s.avgAcos),
        itemStyle: {
          color: '#f56c6c',
          borderRadius: [4, 4, 0, 0],
        },
      },
      {
        name: '平均转化率',
        type: 'bar',
        barWidth: '15%',
        data: stats.map(s => s.avgConversionRate),
        itemStyle: {
          color: '#67c23a',
          borderRadius: [4, 4, 0, 0],
        },
      },
      {
        name: '花费占比',
        type: 'bar',
        barWidth: '15%',
        data: stats.map(s => s.spendPercent),
        itemStyle: {
          color: '#409eff',
          borderRadius: [4, 4, 0, 0],
        },
      },
      {
        name: '销售占比',
        type: 'bar',
        barWidth: '15%',
        data: stats.map(s => s.salesPercent),
        itemStyle: {
          color: '#e6a23c',
          borderRadius: [4, 4, 0, 0],
        },
      },
    ],
  };
});
</script>

<style scoped>
.match-type-chart {
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

.chart-body {
  flex: 1;
  min-height: 0;
}

.chart {
  width: 100%;
  height: 100%;
}

.stats-summary {
  display: flex;
  justify-content: space-around;
  padding: 8px 0;
  border-top: 1px solid var(--el-border-color-lighter);
  margin-top: 8px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.stat-type {
  font-size: 11px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.stat-count {
  font-size: 10px;
  color: var(--el-text-color-secondary);
}
</style>

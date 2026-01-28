<template>
  <div class="spend-efficiency-chart">
    <div class="chart-header">
      <span class="chart-title">花费效率散点图</span>
      <span class="chart-subtitle">花费 vs 销售额</span>
    </div>
    <div class="chart-body">
      <v-chart class="chart" :option="chartOption" autoresize @click="handleClick" />
    </div>
    <div class="legend-container">
      <div class="legend-item">
        <span class="legend-dot" style="background: #67c23a"></span>
        <span class="legend-label">高效 (ACOS &lt; {{ data.targetAcos }}%)</span>
      </div>
      <div class="legend-item">
        <span class="legend-dot" style="background: #f56c6c"></span>
        <span class="legend-label">低效 (ACOS &gt; {{ data.targetAcos }}%)</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import VChart from 'vue-echarts';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { ScatterChart, LineChart } from 'echarts/charts';
import {
  GridComponent,
  TooltipComponent,
  MarkLineComponent,
} from 'echarts/components';
import type { ComposeOption } from 'echarts/core';
import type { ScatterSeriesOption, LineSeriesOption } from 'echarts/charts';
import type {
  GridComponentOption,
  TooltipComponentOption,
  MarkLineComponentOption,
} from 'echarts/components';
import type { SpendEfficiencyData } from '../../../utils/ad-chart-utils';

use([
  CanvasRenderer,
  ScatterChart,
  LineChart,
  GridComponent,
  TooltipComponent,
  MarkLineComponent,
]);

type EChartsOption = ComposeOption<
  | ScatterSeriesOption
  | LineSeriesOption
  | GridComponentOption
  | TooltipComponentOption
  | MarkLineComponentOption
>;

const props = defineProps<{
  data: SpendEfficiencyData;
  currency: string;
}>();

const emit = defineEmits<{
  (e: 'select', searchTerm: string): void;
}>();

// 计算点大小（根据订单数）
function getSymbolSize(orders: number): number {
  const maxOrders = Math.max(...props.data.points.map(p => p.orders), 1);
  const minSize = 8;
  const maxSize = 30;
  if (maxOrders === 0) return minSize;
  return minSize + (orders / maxOrders) * (maxSize - minSize);
}

const chartOption = computed<EChartsOption>(() => {
  const { points, targetAcos } = props.data;

  // 准备散点数据
  const seriesData = points.map(point => ({
    value: [point.spend, point.sales],
    symbolSize: getSymbolSize(point.orders),
    itemStyle: {
      color: point.isEfficient ? '#67c23a' : '#f56c6c',
      opacity: 0.7,
    },
    name: point.searchTerm,
    data: point,
  }));

  // 计算坐标轴范围
  const maxSpend = Math.max(...points.map(p => p.spend), 100);
  const maxSales = Math.max(...points.map(p => p.sales), 100);
  const axisMax = Math.max(maxSpend, maxSales) * 1.1;

  // 计算盈亏平衡线数据
  // ACOS = spend / sales * 100 = targetAcos
  // sales = spend * 100 / targetAcos
  const breakEvenSlope = 100 / targetAcos;

  return {
    tooltip: {
      trigger: 'item',
      formatter: (params: any) => {
        if (!params.data?.data) return '';
        const point = params.data.data;
        return `
          <div style="padding: 8px;">
            <div style="font-weight: bold; margin-bottom: 8px; max-width: 200px; word-wrap: break-word;">
              ${point.searchTerm}
            </div>
            <div style="display: flex; justify-content: space-between; gap: 16px;">
              <span>花费:</span>
              <span style="font-weight: 500;">${props.currency}${point.spend.toFixed(2)}</span>
            </div>
            <div style="display: flex; justify-content: space-between; gap: 16px;">
              <span>销售额:</span>
              <span style="font-weight: 500;">${props.currency}${point.sales.toFixed(2)}</span>
            </div>
            <div style="display: flex; justify-content: space-between; gap: 16px;">
              <span>ACOS:</span>
              <span style="font-weight: 500; color: ${point.isEfficient ? '#67c23a' : '#f56c6c'};">
                ${point.acos.toFixed(2)}%
              </span>
            </div>
            <div style="display: flex; justify-content: space-between; gap: 16px;">
              <span>订单数:</span>
              <span style="font-weight: 500;">${point.orders}</span>
            </div>
          </div>
        `;
      },
    },
    grid: {
      left: 60,
      right: 30,
      top: 30,
      bottom: 50,
    },
    xAxis: {
      type: 'value',
      name: `花费 (${props.currency})`,
      nameLocation: 'middle',
      nameGap: 30,
      min: 0,
      max: Math.ceil(axisMax),
      splitLine: {
        lineStyle: {
          type: 'dashed',
          color: 'var(--el-border-color-lighter)',
        },
      },
      axisLine: {
        lineStyle: {
          color: 'var(--el-text-color-secondary)',
        },
      },
      axisLabel: {
        color: 'var(--el-text-color-secondary)',
      },
    },
    yAxis: {
      type: 'value',
      name: `销售额 (${props.currency})`,
      nameLocation: 'middle',
      nameGap: 45,
      min: 0,
      max: Math.ceil(axisMax),
      splitLine: {
        lineStyle: {
          type: 'dashed',
          color: 'var(--el-border-color-lighter)',
        },
      },
      axisLine: {
        lineStyle: {
          color: 'var(--el-text-color-secondary)',
        },
      },
      axisLabel: {
        color: 'var(--el-text-color-secondary)',
      },
    },
    series: [
      {
        type: 'scatter',
        data: seriesData,
      },
      // 盈亏平衡线
      {
        type: 'line',
        data: [
          [0, 0],
          [Math.ceil(axisMax), Math.ceil(axisMax) * breakEvenSlope],
        ],
        symbol: 'none',
        lineStyle: {
          color: '#e6a23c',
          type: 'dashed',
          width: 2,
        },
        endLabel: {
          show: true,
          formatter: `ACOS=${targetAcos}%`,
          fontSize: 11,
          color: '#e6a23c',
        },
        z: 1,
      },
    ],
  };
});

function handleClick(params: any) {
  if (params.data?.data?.searchTerm) {
    emit('select', params.data.data.searchTerm);
  }
}
</script>

<style scoped>
.spend-efficiency-chart {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.chart-header {
  display: flex;
  align-items: baseline;
  gap: 8px;
  margin-bottom: 12px;
}

.chart-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.chart-subtitle {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.chart-body {
  flex: 1;
  min-height: 0;
}

.chart {
  width: 100%;
  height: 100%;
}

.legend-container {
  display: flex;
  justify-content: center;
  gap: 16px;
  margin-top: 8px;
  flex-wrap: wrap;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.legend-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.legend-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
</style>

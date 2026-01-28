<template>
  <div class="quadrant-chart">
    <div class="chart-header">
      <span class="chart-title">效率四象限图</span>
      <span class="chart-subtitle">转化率 vs ACOS</span>
    </div>
    <div class="chart-body">
      <v-chart class="chart" :option="chartOption" autoresize @click="handleClick" />
    </div>
    <div class="legend-container">
      <div class="legend-item">
        <span class="legend-dot" style="background: #67c23a"></span>
        <span class="legend-label">高潜力 ({{ data.quadrantCounts.high_potential }})</span>
      </div>
      <div class="legend-item">
        <span class="legend-dot" style="background: #e6a23c"></span>
        <span class="legend-label">待优化 ({{ data.quadrantCounts.needs_optimization }})</span>
      </div>
      <div class="legend-item">
        <span class="legend-dot" style="background: #409eff"></span>
        <span class="legend-label">稳定 ({{ data.quadrantCounts.stable }})</span>
      </div>
      <div class="legend-item">
        <span class="legend-dot" style="background: #f56c6c"></span>
        <span class="legend-label">淘汰 ({{ data.quadrantCounts.eliminate }})</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import VChart from 'vue-echarts';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { ScatterChart } from 'echarts/charts';
import {
  GridComponent,
  TooltipComponent,
  MarkLineComponent,
  MarkAreaComponent,
} from 'echarts/components';
import type { ComposeOption } from 'echarts/core';
import type { ScatterSeriesOption } from 'echarts/charts';
import type {
  GridComponentOption,
  TooltipComponentOption,
  MarkLineComponentOption,
  MarkAreaComponentOption,
} from 'echarts/components';
import type { QuadrantData } from '../../../utils/ad-chart-utils';
import { getQuadrantColor, getQuadrantName } from '../../../utils/ad-chart-utils';

use([
  CanvasRenderer,
  ScatterChart,
  GridComponent,
  TooltipComponent,
  MarkLineComponent,
  MarkAreaComponent,
]);

type EChartsOption = ComposeOption<
  | ScatterSeriesOption
  | GridComponentOption
  | TooltipComponentOption
  | MarkLineComponentOption
  | MarkAreaComponentOption
>;

const props = defineProps<{
  data: QuadrantData;
  currency: string;
}>();

const emit = defineEmits<{
  (e: 'select', searchTerm: string): void;
}>();

// 计算点大小（根据花费）
function getSymbolSize(spend: number): number {
  const maxSpend = Math.max(...props.data.points.map(p => p.spend), 1);
  const minSize = 8;
  const maxSize = 30;
  return minSize + (spend / maxSpend) * (maxSize - minSize);
}

const chartOption = computed<EChartsOption>(() => {
  const { points, avgConversionRate, targetAcos } = props.data;

  // 按象限分组数据
  const seriesData = points.map(point => ({
    value: [point.conversionRate, point.acos],
    symbolSize: getSymbolSize(point.spend),
    itemStyle: {
      color: getQuadrantColor(point.quadrant),
      opacity: 0.7,
    },
    name: point.searchTerm,
    data: point,
  }));

  // 计算坐标轴范围
  const maxConversionRate = Math.max(...points.map(p => p.conversionRate), avgConversionRate * 2, 10);
  const maxAcos = Math.max(...points.map(p => p.acos), targetAcos * 2, 50);

  return {
    tooltip: {
      trigger: 'item',
      formatter: (params: any) => {
        const point = params.data.data;
        return `
          <div style="padding: 8px;">
            <div style="font-weight: bold; margin-bottom: 8px; max-width: 200px; word-wrap: break-word;">
              ${point.searchTerm}
            </div>
            <div style="display: flex; justify-content: space-between; gap: 16px;">
              <span>转化率:</span>
              <span style="font-weight: 500;">${point.conversionRate.toFixed(2)}%</span>
            </div>
            <div style="display: flex; justify-content: space-between; gap: 16px;">
              <span>ACOS:</span>
              <span style="font-weight: 500;">${point.acos.toFixed(2)}%</span>
            </div>
            <div style="display: flex; justify-content: space-between; gap: 16px;">
              <span>花费:</span>
              <span style="font-weight: 500;">${props.currency}${point.spend.toFixed(2)}</span>
            </div>
            <div style="display: flex; justify-content: space-between; gap: 16px;">
              <span>销售额:</span>
              <span style="font-weight: 500;">${props.currency}${point.sales.toFixed(2)}</span>
            </div>
            <div style="margin-top: 8px; padding-top: 8px; border-top: 1px solid rgba(255,255,255,0.2);">
              <span style="color: ${getQuadrantColor(point.quadrant)}; font-weight: 500;">
                ${getQuadrantName(point.quadrant)}
              </span>
            </div>
          </div>
        `;
      },
    },
    grid: {
      left: 50,
      right: 70,
      top: 30,
      bottom: 50,
    },
    xAxis: {
      type: 'value',
      name: '转化率 (%)',
      nameLocation: 'middle',
      nameGap: 30,
      min: 0,
      max: Math.ceil(maxConversionRate * 1.1),
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
      name: 'ACOS (%)',
      nameLocation: 'middle',
      nameGap: 35,
      min: 0,
      max: Math.ceil(maxAcos * 1.1),
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
        markLine: {
          silent: true,
          symbol: 'none',
          lineStyle: {
            color: 'var(--el-color-info)',
            type: 'dashed',
            width: 2,
          },
          label: {
            show: true,
            position: 'end',
            fontSize: 11,
            color: 'var(--el-text-color-secondary)',
          },
          data: [
            {
              xAxis: avgConversionRate,
              label: {
                formatter: `平均转化率\n${avgConversionRate.toFixed(1)}%`,
              },
            },
            {
              yAxis: targetAcos,
              label: {
                formatter: `目标ACOS\n${targetAcos}%`,
              },
            },
          ],
        },
        markArea: {
          silent: true,
          itemStyle: {
            opacity: 0.05,
          },
          data: [
            // 高潜力 - 右下
            [
              { xAxis: avgConversionRate, yAxis: 0, itemStyle: { color: '#67c23a' } },
              { xAxis: Math.ceil(maxConversionRate * 1.1), yAxis: targetAcos },
            ],
            // 待优化 - 右上
            [
              { xAxis: avgConversionRate, yAxis: targetAcos, itemStyle: { color: '#e6a23c' } },
              { xAxis: Math.ceil(maxConversionRate * 1.1), yAxis: Math.ceil(maxAcos * 1.1) },
            ],
            // 稳定 - 左下
            [
              { xAxis: 0, yAxis: 0, itemStyle: { color: '#409eff' } },
              { xAxis: avgConversionRate, yAxis: targetAcos },
            ],
            // 淘汰 - 左上
            [
              { xAxis: 0, yAxis: targetAcos, itemStyle: { color: '#f56c6c' } },
              { xAxis: avgConversionRate, yAxis: Math.ceil(maxAcos * 1.1) },
            ],
          ],
        },
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
.quadrant-chart {
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

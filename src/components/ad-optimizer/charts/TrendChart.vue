<template>
  <div class="trend-chart">
    <div class="chart-header">
      <span class="chart-title">时间趋势图</span>
      <span class="chart-subtitle" v-if="data.dateRange.start">
        {{ data.dateRange.start }} ~ {{ data.dateRange.end }}
        <span v-if="data.comparison?.previousPeriod" class="comparison-hint">
          (环比：{{ data.comparison.previousPeriod.start }} ~ {{ data.comparison.previousPeriod.end }})
        </span>
      </span>
    </div>

    <!-- 汇总指标卡片 -->
    <div class="summary-cards">
      <div class="summary-card">
        <span class="card-label">ACOS</span>
        <span class="card-value" :class="acosStatusClass">
          {{ data.summary.avgAcos.toFixed(2) }}%
        </span>
        <span
          v-if="data.comparison?.acos"
          class="card-change"
          :class="getChangeClass(data.comparison.acos)"
        >
          {{ formatChange(data.comparison.acos) }}
        </span>
      </div>
      <div class="summary-card">
        <span class="card-label">花费</span>
        <span class="card-value">
          {{ currency }}{{ formatNumber(data.summary.totalSpend) }}
        </span>
        <span
          v-if="data.comparison?.spend"
          class="card-change"
          :class="getChangeClass(data.comparison.spend, true)"
        >
          {{ formatChange(data.comparison.spend) }}
        </span>
      </div>
      <div class="summary-card">
        <span class="card-label">订单</span>
        <span class="card-value">{{ data.summary.totalOrders }}</span>
        <span
          v-if="data.comparison?.orders"
          class="card-change"
          :class="getChangeClass(data.comparison.orders)"
        >
          {{ formatChange(data.comparison.orders) }}
        </span>
      </div>
      <div class="summary-card">
        <span class="card-label">销售额</span>
        <span class="card-value">
          {{ currency }}{{ formatNumber(data.summary.totalSales) }}
        </span>
        <span
          v-if="data.comparison?.sales"
          class="card-change"
          :class="getChangeClass(data.comparison.sales)"
        >
          {{ formatChange(data.comparison.sales) }}
        </span>
      </div>
    </div>

    <!-- 折线图 -->
    <div class="chart-body" v-if="data.points.length > 0">
      <v-chart class="chart" :option="chartOption" autoresize />
    </div>
    <div class="empty-state" v-else>
      <span class="empty-text">暂无趋势数据（需要包含日期字段）</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import VChart from 'vue-echarts';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { LineChart } from 'echarts/charts';
import {
  GridComponent,
  TooltipComponent,
  LegendComponent,
} from 'echarts/components';
import type { ComposeOption } from 'echarts/core';
import type { LineSeriesOption } from 'echarts/charts';
import type {
  GridComponentOption,
  TooltipComponentOption,
  LegendComponentOption,
} from 'echarts/components';
import type { TrendData, ComparisonChange } from '../../../utils/ad-chart-utils';

use([
  CanvasRenderer,
  LineChart,
  GridComponent,
  TooltipComponent,
  LegendComponent,
]);

type EChartsOption = ComposeOption<
  | LineSeriesOption
  | GridComponentOption
  | TooltipComponentOption
  | LegendComponentOption
>;

const props = defineProps<{
  data: TrendData;
  currency: string;
  targetAcos?: number;
}>();

// ACOS 状态样式
const acosStatusClass = computed(() => {
  const acos = props.data.summary.avgAcos;
  const target = props.targetAcos || 30;
  if (acos <= target * 0.8) return 'status-good';
  if (acos <= target) return 'status-ok';
  if (acos <= target * 1.5) return 'status-warning';
  return 'status-critical';
});

// 格式化数字
function formatNumber(value: number): string {
  if (value >= 1000) {
    return value.toLocaleString('en-US', { maximumFractionDigits: 2 });
  }
  return value.toFixed(2);
}

// 格式化环比变化
function formatChange(change: ComparisonChange): string {
  if (change.direction === 'same') return '-';
  const arrow = change.direction === 'up' ? '↑' : '↓';
  return `${arrow}${change.value.toFixed(1)}%`;
}

// 获取环比变化的样式类
function getChangeClass(change: ComparisonChange, isNeutral: boolean = false): string {
  if (change.direction === 'same') return 'change-neutral';
  if (isNeutral) return 'change-neutral'; // 花费变化是中性的
  return change.isPositive ? 'change-positive' : 'change-negative';
}

// 图表配色 - 参考亚马逊配色
const COLORS = {
  acos: '#9b59b6',      // 紫色
  spend: '#00bcd4',     // 青色
  orders: '#e91e63',    // 粉色
  sales: '#2196f3',     // 蓝色
};

const chartOption = computed<EChartsOption>(() => {
  const { points } = props.data;

  const dates = points.map(p => {
    // 格式化日期显示（只显示月/日）
    const parts = p.date.split('-');
    return parts.length >= 3 ? `${parts[1]}/${parts[2]}` : p.date;
  });

  const acosData = points.map(p => p.acos);
  const spendData = points.map(p => p.spend);
  const ordersData = points.map(p => p.orders);
  const salesData = points.map(p => p.sales);

  // 计算各轴的最大值
  const maxAcos = Math.max(...acosData, 100);
  const maxMoney = Math.max(...spendData, ...salesData, 100);
  const maxOrders = Math.max(...ordersData, 10);

  return {
    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'cross',
        crossStyle: {
          color: '#999',
        },
      },
      formatter: (params: any) => {
        if (!Array.isArray(params) || params.length === 0) return '';
        const date = points[params[0].dataIndex]?.date || '';
        let html = `<div style="padding: 8px;"><div style="font-weight: bold; margin-bottom: 8px;">${date}</div>`;

        params.forEach((item: any) => {
          const value = item.value;
          let formattedValue = '';
          if (item.seriesName === 'ACOS') {
            formattedValue = `${value.toFixed(2)}%`;
          } else if (item.seriesName === '订单') {
            formattedValue = String(value);
          } else {
            formattedValue = `${props.currency}${value.toFixed(2)}`;
          }
          html += `
            <div style="display: flex; justify-content: space-between; gap: 16px; align-items: center;">
              <span style="display: flex; align-items: center; gap: 4px;">
                <span style="display: inline-block; width: 10px; height: 10px; border-radius: 50%; background: ${item.color};"></span>
                ${item.seriesName}:
              </span>
              <span style="font-weight: 500;">${formattedValue}</span>
            </div>
          `;
        });
        html += '</div>';
        return html;
      },
    },
    legend: {
      data: ['ACOS', '花费', '订单', '销售额'],
      bottom: 0,
      icon: 'circle',
      itemWidth: 10,
      itemHeight: 10,
      itemGap: 20,
      textStyle: {
        color: 'var(--el-text-color-secondary)',
        fontSize: 12,
      },
      selectedMode: true,
    },
    grid: {
      left: 100,
      right: 60,
      top: 35,
      bottom: 65,
    },
    xAxis: {
      type: 'category',
      data: dates,
      axisLine: {
        lineStyle: {
          color: 'var(--el-border-color)',
        },
      },
      axisLabel: {
        color: 'var(--el-text-color-secondary)',
        fontSize: 11,
        rotate: dates.length > 15 ? 45 : 0,
      },
      axisTick: {
        show: false,
      },
    },
    yAxis: [
      // 左轴1: 订单数
      {
        type: 'value',
        name: '订单',
        position: 'left',
        offset: 0,
        min: 0,
        max: Math.ceil(maxOrders * 1.2),
        axisLine: {
          show: true,
          lineStyle: {
            color: COLORS.orders,
          },
        },
        axisLabel: {
          color: COLORS.orders,
          fontSize: 11,
        },
        splitLine: {
          show: false,
        },
        nameTextStyle: {
          color: COLORS.orders,
          fontSize: 11,
        },
      },
      // 左轴2: ACOS (%)
      {
        type: 'value',
        name: 'ACOS%',
        position: 'left',
        offset: 45,
        min: 0,
        max: Math.ceil(maxAcos * 1.2),
        axisLine: {
          show: true,
          lineStyle: {
            color: COLORS.acos,
          },
        },
        axisLabel: {
          color: COLORS.acos,
          fontSize: 11,
          formatter: '{value}%',
        },
        splitLine: {
          show: false,
        },
        nameTextStyle: {
          color: COLORS.acos,
          fontSize: 11,
        },
      },
      // 右轴: 花费/销售额 (货币)
      {
        type: 'value',
        name: `金额(${props.currency})`,
        position: 'right',
        min: 0,
        max: Math.ceil(maxMoney * 1.2),
        axisLine: {
          show: true,
          lineStyle: {
            color: COLORS.spend,
          },
        },
        axisLabel: {
          color: 'var(--el-text-color-secondary)',
          fontSize: 11,
          formatter: (value: number) => {
            if (value >= 1000) return `${(value / 1000).toFixed(1)}k`;
            return String(value);
          },
        },
        splitLine: {
          lineStyle: {
            type: 'dashed',
            color: 'var(--el-border-color-lighter)',
          },
        },
        nameTextStyle: {
          color: 'var(--el-text-color-secondary)',
          fontSize: 11,
        },
      },
    ],
    series: [
      {
        name: 'ACOS',
        type: 'line',
        yAxisIndex: 1,
        data: acosData,
        symbol: 'circle',
        symbolSize: 6,
        lineStyle: {
          color: COLORS.acos,
          width: 2,
        },
        itemStyle: {
          color: COLORS.acos,
        },
        smooth: true,
      },
      {
        name: '花费',
        type: 'line',
        yAxisIndex: 2,
        data: spendData,
        symbol: 'circle',
        symbolSize: 6,
        lineStyle: {
          color: COLORS.spend,
          width: 2,
        },
        itemStyle: {
          color: COLORS.spend,
        },
        smooth: true,
      },
      {
        name: '订单',
        type: 'line',
        yAxisIndex: 0,
        data: ordersData,
        symbol: 'circle',
        symbolSize: 6,
        lineStyle: {
          color: COLORS.orders,
          width: 2,
        },
        itemStyle: {
          color: COLORS.orders,
        },
        smooth: true,
      },
      {
        name: '销售额',
        type: 'line',
        yAxisIndex: 2,
        data: salesData,
        symbol: 'circle',
        symbolSize: 6,
        lineStyle: {
          color: COLORS.sales,
          width: 2,
        },
        itemStyle: {
          color: COLORS.sales,
        },
        smooth: true,
      },
    ],
  };
});
</script>

<style scoped>
.trend-chart {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.chart-header {
  display: flex;
  align-items: baseline;
  gap: 8px;
  margin-bottom: 8px;
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

.summary-cards {
  display: flex;
  gap: 16px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.summary-card {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 8px 12px;
  background: var(--el-fill-color);
  border-radius: 6px;
  min-width: 80px;
}

.card-label {
  font-size: 11px;
  color: var(--el-text-color-secondary);
}

.card-value {
  font-size: 16px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.card-value.status-good {
  color: #67c23a;
}

.card-value.status-ok {
  color: #409eff;
}

.card-value.status-warning {
  color: #e6a23c;
}

.card-value.status-critical {
  color: #f56c6c;
}

.card-change {
  font-size: 12px;
  font-weight: 500;
  margin-top: 2px;
}

.card-change.change-positive {
  color: #67c23a;
}

.card-change.change-negative {
  color: #f56c6c;
}

.card-change.change-neutral {
  color: var(--el-text-color-secondary);
}

.comparison-hint {
  font-size: 12px;
  color: var(--el-text-color-placeholder);
  margin-left: 8px;
}

.chart-body {
  flex: 1;
  min-height: 0;
}

.chart {
  width: 100%;
  height: 100%;
}

.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-text {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}
</style>

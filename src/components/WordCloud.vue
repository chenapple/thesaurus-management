<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import * as echarts from 'echarts';
import 'echarts-wordcloud';

interface Root {
  id: number;
  word: string;
  translation: string | null;
  contains_count: number;
  percentage: number;
  categories: number[];
}

interface Category {
  id: number;
  name: string;
}

const props = defineProps<{
  roots: Root[];
  categories: Category[];
  loading?: boolean;
}>();

const emit = defineEmits<{
  (e: 'wordClick', word: string): void;
}>();

const chartRef = ref<HTMLDivElement>();
let chartInstance: echarts.ECharts | null = null;

// 分类颜色映射
const categoryColors: Record<string, string> = {
  '品类词': '#409EFF',
  '品牌': '#67C23A',
  '颜色': '#E6A23C',
  '形状': '#F56C6C',
  '功能': '#909399',
  '适用人群': '#9B59B6',
  '材质': '#1ABC9C',
  '尺寸': '#3498DB',
  '使用场景': '#E74C3C',
  '情绪价值': '#F39C12',
  '使用地点': '#2ECC71',
  '节假日': '#E91E63',
  '适配': '#00BCD4',
  '其他': '#95A5A6',
};

// 根据分类获取颜色
function getColorByCategory(categories: number[]): string {
  if (categories.length === 0) return '#666666';
  const catId = categories[0];
  const cat = props.categories.find(c => c.id === catId);
  if (cat && categoryColors[cat.name]) {
    return categoryColors[cat.name];
  }
  return '#409EFF';
}

// 计算词云数据
const wordCloudData = computed(() => {
  if (!props.roots || props.roots.length === 0) return [];

  // 找到最大和最小的 contains_count
  const maxCount = Math.max(...props.roots.map(r => r.contains_count));
  const minCount = Math.min(...props.roots.map(r => r.contains_count));

  return props.roots.map(root => {
    // 根据 contains_count 计算字体大小 (12-80)
    const normalizedSize = maxCount === minCount
      ? 0.5
      : (root.contains_count - minCount) / (maxCount - minCount);
    const fontSize = 12 + normalizedSize * 68;

    return {
      name: root.word,
      value: root.contains_count,
      textStyle: {
        color: getColorByCategory(root.categories),
        fontSize: fontSize,
      },
      translation: root.translation,
      percentage: root.percentage,
    };
  });
});

// 初始化图表
function initChart() {
  if (!chartRef.value) return;

  if (chartInstance) {
    chartInstance.dispose();
  }

  chartInstance = echarts.init(chartRef.value);
  updateChart();

  // 点击事件
  chartInstance.on('click', (params: any) => {
    if (params.name) {
      emit('wordClick', params.name);
    }
  });

  // 窗口大小变化时重绘
  window.addEventListener('resize', handleResize);
}

function handleResize() {
  chartInstance?.resize();
}

function updateChart() {
  if (!chartInstance) return;

  const option = {
    tooltip: {
      show: true,
      formatter: (params: any) => {
        const data = params.data;
        return `<div style="padding: 8px;">
          <div style="font-weight: bold; font-size: 14px; margin-bottom: 4px;">${data.name}</div>
          ${data.translation ? `<div style="color: #666;">翻译: ${data.translation}</div>` : ''}
          <div style="color: #666;">包含词数: ${data.value}</div>
          <div style="color: #666;">占比: ${data.percentage.toFixed(2)}%</div>
        </div>`;
      },
    },
    series: [{
      type: 'wordCloud',
      shape: 'circle',
      left: 'center',
      top: 'center',
      width: '90%',
      height: '90%',
      sizeRange: [12, 80],
      rotationRange: [-45, 45],
      rotationStep: 15,
      gridSize: 8,
      drawOutOfBound: false,
      shrinkToFit: true,
      layoutAnimation: true,
      textStyle: {
        fontFamily: 'sans-serif',
        fontWeight: 'bold',
      },
      emphasis: {
        focus: 'self',
        textStyle: {
          textShadowBlur: 10,
          textShadowColor: 'rgba(0, 0, 0, 0.3)',
        },
      },
      data: wordCloudData.value,
    }],
  };

  chartInstance.setOption(option);
}

// 导出图片
function exportImage() {
  if (!chartInstance) return;

  const dataUrl = chartInstance.getDataURL({
    type: 'png',
    pixelRatio: 2,
    backgroundColor: '#fff',
  });

  const link = document.createElement('a');
  link.download = `词云_${new Date().toLocaleDateString()}.png`;
  link.href = dataUrl;
  link.click();
}

// 监听数据变化
watch(() => props.roots, () => {
  if (chartInstance) {
    updateChart();
  }
}, { deep: true });

onMounted(() => {
  initChart();
});

// 暴露方法给父组件
defineExpose({
  exportImage,
  resize: handleResize,
});
</script>

<template>
  <div class="word-cloud-container" v-loading="loading" element-loading-text="加载中...">
    <div v-if="!loading && roots.length === 0" class="empty-state">
      <el-empty description="暂无词根数据" />
    </div>
    <div v-else-if="!loading" ref="chartRef" class="chart"></div>

    <!-- 图例 -->
    <div class="legend" v-if="roots.length > 0">
      <div class="legend-title">分类图例</div>
      <div class="legend-items">
        <div
          v-for="(color, name) in categoryColors"
          :key="name"
          class="legend-item"
        >
          <span class="legend-color" :style="{ backgroundColor: color }"></span>
          <span class="legend-name">{{ name }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.word-cloud-container {
  position: relative;
  width: 100%;
  height: 100%;
  min-height: 500px;
  display: flex;
  flex-direction: column;
}

.chart {
  flex: 1;
  width: 100%;
  min-height: 450px;
}

.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.legend {
  padding: 12px 16px;
  border-top: 1px solid var(--el-border-color-lighter);
  background: var(--el-bg-color);
}

.legend-title {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 8px;
}

.legend-items {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.legend-color {
  width: 12px;
  height: 12px;
  border-radius: 2px;
}

.legend-name {
  font-size: 12px;
  color: var(--el-text-color-regular);
}

/* 深色模式适配 */
:global(.dark) .loading-overlay {
  background: rgba(30, 30, 30, 0.8);
}
</style>

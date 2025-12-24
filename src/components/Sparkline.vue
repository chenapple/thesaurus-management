<template>
  <svg
    :width="width"
    :height="height"
    class="sparkline"
    @click="$emit('click')"
  >
    <polyline
      v-if="points"
      :points="points"
      fill="none"
      :stroke="color"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  data: (number | null)[];
  width?: number;
  height?: number;
  color?: string;
  inverse?: boolean;  // 排名越小越好，需要反转Y轴
}>(), {
  width: 60,
  height: 20,
  color: '#67c23a',
  inverse: true,
});

defineEmits<{
  (e: 'click'): void;
}>();

const points = computed(() => {
  // 过滤有效数据
  const validData = props.data.filter(d => d !== null) as number[];
  if (validData.length < 2) return '';

  const min = Math.min(...validData);
  const max = Math.max(...validData);
  const range = max - min || 1;

  const padding = 2;
  const w = props.width - padding * 2;
  const h = props.height - padding * 2;

  // 生成有效数据点的坐标
  const validPoints: string[] = [];
  let validIndex = 0;

  for (let i = 0; i < props.data.length; i++) {
    const v = props.data[i];
    if (v !== null) {
      const x = padding + (validIndex / (validData.length - 1)) * w;
      let y = padding + ((v - min) / range) * h;
      // 排名小的在上面（反转Y轴）
      if (props.inverse) {
        y = props.height - y;
      }
      validPoints.push(`${x.toFixed(1)},${y.toFixed(1)}`);
      validIndex++;
    }
  }

  return validPoints.join(' ');
});
</script>

<style scoped>
.sparkline {
  display: inline-block;
  vertical-align: middle;
  cursor: pointer;
}

.sparkline:hover polyline {
  stroke-width: 2;
}
</style>

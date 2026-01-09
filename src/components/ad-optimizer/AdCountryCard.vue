<template>
  <div class="ad-country-card" :class="cardClass" @click="$emit('analyze')">
    <div class="card-header">
      <div class="country-info">
        <div class="flag-wrapper" v-html="getCountryFlag(stats.country)"></div>
        <div class="country-text">
          <span class="country-code">{{ stats.country }}</span>
          <span class="country-name">{{ getCountryLabel(stats.country) }}</span>
        </div>
      </div>
      <div class="health-badge" :class="acosStatus.class">
        {{ acosStatus.label }}
      </div>
    </div>

    <div class="metrics-grid">
      <div class="metric-item">
        <div class="metric-label">
            <el-icon><Coin /></el-icon> 花费
        </div>
        <div class="metric-value">{{ formatCurrency(stats.total_spend, stats.country) }}</div>
      </div>
      <div class="metric-item">
        <div class="metric-label">
            <el-icon><Trophy /></el-icon> 销售额
        </div>
        <div class="metric-value">{{ formatCurrency(stats.total_sales, stats.country) }}</div>
      </div>
    </div>

    <div class="acos-section">
      <div class="acos-header">
        <span class="label">ACOS</span>
        <span class="value" :class="acosStatus.textClass">{{ stats.avg_acos.toFixed(1) }}%</span>
      </div>
      <!-- ACOS 进度条 -->
      <div class="acos-bar-bg">
        <div 
            class="acos-bar-fill" 
            :class="acosStatus.barClass"
            :style="{ width: Math.min((stats.avg_acos / (targetAcos * 2)) * 100, 100) + '%' }"
        ></div>
        <!-- 目标线 -->
        <div class="target-line" style="left: 50%" title="目标 ACOS"></div>
      </div>
      <div class="acos-footer">
        <span class="target-label">目标: {{ targetAcos }}%</span>
        <span class="volume-label">{{ stats.term_count }} 个关键词</span>
      </div>
    </div>

    <div class="card-footer">
        <el-button class="analyze-btn" type="primary" plain size="small" style="width: 100%">
            查看详情 <el-icon class="el-icon--right"><Right /></el-icon>
        </el-button>
        <!-- 气泡装饰 -->
        <div class="bg-blob"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Coin, Trophy, Right } from '@element-plus/icons-vue';
import { getCountryFlag, getCountryLabel, COUNTRY_CURRENCY_MAP } from '../../types';

const props = defineProps<{
  stats: any;
  targetAcos: number;
}>();

defineEmits(['analyze']);

const acosStatus = computed(() => {
  const acos = props.stats.avg_acos;
  const target = props.targetAcos;

  if (acos <= target) {
    return { label: '优秀', class: 'badge-success', textClass: 'text-success', barClass: 'bg-success' };
  } else if (acos <= target * 1.2) {
    return { label: '良好', class: 'badge-warning', textClass: 'text-warning', barClass: 'bg-warning' };
  } else {
    return { label: '需优化', class: 'badge-danger', textClass: 'text-danger', barClass: 'bg-danger' };
  }
});

const cardClass = computed(() => {
  return acosStatus.value.class.replace('badge-', 'border-');
});

function formatCurrency(amount: number, country: string) {
  const symbol = COUNTRY_CURRENCY_MAP[country]?.symbol || '';
  return `${symbol}${amount.toFixed(2)}`;
}
</script>

<style scoped>
.ad-country-card {
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 12px;
  padding: 20px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  min-height: 220px;
}

.ad-country-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 24px rgba(0, 0, 0, 0.08);
}

/* 状态边框 (Hover时加强) */
.border-success:hover { border-color: var(--el-color-success-light-5); }
.border-warning:hover { border-color: var(--el-color-warning-light-5); }
.border-danger:hover { border-color: var(--el-color-danger-light-5); }

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;
  position: relative;
  z-index: 2;
}

.country-info {
  display: flex;
  gap: 12px;
  align-items: center;
}

.flag-wrapper {
  width: 48px;
  height: 32px;
  border-radius: 6px;
  overflow: hidden;
  box-shadow: 0 2px 6px rgba(0,0,0,0.1);
  display: flex;
  align-items: center;
  justify-content: center;
}

.flag-wrapper :deep(svg) {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.country-text {
  display: flex;
  flex-direction: column;
}

.country-code {
  font-size: 16px;
  font-weight: 700;
  color: var(--el-text-color-primary);
  line-height: 1;
  margin-bottom: 4px;
}

.country-name {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.health-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 10px;
  font-weight: 600;
  text-transform: uppercase;
}
.badge-success { background: var(--el-color-success-light-9); color: var(--el-color-success); }
.badge-warning { background: var(--el-color-warning-light-9); color: var(--el-color-warning); }
.badge-danger { background: var(--el-color-danger-light-9); color: var(--el-color-danger); }

/* Metrics */
.metrics-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  margin-bottom: 24px;
  position: relative;
  z-index: 2;
}

.metric-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.metric-label {
    font-size: 12px;
    color: var(--el-text-color-secondary);
    display: flex;
    align-items: center;
    gap: 4px;
}

.metric-value {
    font-size: 18px;
    font-weight: 600;
    color: var(--el-text-color-primary);
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}

/* ACOS Section */
.acos-section {
    margin-bottom: 20px;
    position: relative;
    z-index: 2;
}

.acos-header {
    display: flex;
    justify-content: space-between;
    font-size: 13px;
    margin-bottom: 8px;
}

.acos-header .label { color: var(--el-text-color-secondary); font-weight: 500; }
.acos-bar-bg {
    height: 8px;
    background: var(--el-fill-color);
    border-radius: 4px;
    position: relative;
    margin-bottom: 8px;
    overflow: visible; /* Show target line */
}

.acos-bar-fill {
    height: 100%;
    border-radius: 4px;
    transition: width 0.5s ease;
}
.bg-success { background: var(--el-color-success); }
.bg-warning { background: var(--el-color-warning); }
.bg-danger { background: linear-gradient(90deg, #ff7875, #ff4d4f); }

.target-line {
    position: absolute;
    top: -2px;
    bottom: -2px;
    width: 2px;
    background: var(--el-text-color-primary);
    opacity: 0.3;
    z-index: 10;
}

.text-success { color: var(--el-color-success); font-weight: 600; }
.text-warning { color: var(--el-color-warning); font-weight: 600; }
.text-danger { color: var(--el-color-danger); font-weight: 600; }

.acos-footer {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    color: var(--el-text-color-placeholder);
}

.card-footer {
    margin-top: auto;
    position: relative;
    z-index: 2;
}

/* Background blob decoration */
.bg-blob {
    position: absolute;
    bottom: -40px;
    right: -40px;
    width: 120px;
    height: 120px;
    background: radial-gradient(circle, var(--el-fill-color-darker) 0%, transparent 70%);
    opacity: 0.5;
    border-radius: 50%;
    z-index: 1;
    pointer-events: none;
}
</style>

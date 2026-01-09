<template>
  <div class="ad-country-detail">
    <div class="detail-header">
      <div class="title-section">
        <div class="flag-icon" v-if="countryCode !== 'GLOBAL'" v-html="getCountryFlag(countryCode)"></div>
        <div class="flag-icon global-icon" v-else>
          <el-icon><Compass /></el-icon>
        </div>
        <div class="country-name-group">
            <h3 class="country-name">{{ countryName }}</h3>
            <span class="view-label">{{ countryCode === 'GLOBAL' ? 'Global Overview' : 'Country Analysis' }}</span>
        </div>
      </div>
      <div class="header-actions">
           <!-- 预留操作按钮 -->
      </div>
    </div>

    <!-- 核心指标大卡片 -->
    <div class="kpi-grid">
      <div class="kpi-card spend-card">
        <div class="kpi-icon"><el-icon><Coin /></el-icon></div>
        <div class="kpi-content">
          <div class="kpi-label">Total Spend</div>
          <div class="kpi-value">{{ formatCurrency(data.total_spend) }}</div>
        </div>
      </div>
      
      <div class="kpi-card sales-card">
        <div class="kpi-icon"><el-icon><Trophy /></el-icon></div>
        <div class="kpi-content">
          <div class="kpi-label">Total Sales</div>
          <div class="kpi-value">{{ formatCurrency(data.total_sales) }}</div>
        </div>
      </div>

      <div class="kpi-card acos-card" :class="getAcosStatus(data.avg_acos)">
        <div class="kpi-icon"><el-icon><TrendCharts /></el-icon></div>
        <div class="kpi-content">
          <div class="kpi-label">Average ACOS</div>
          <div class="kpi-value">{{ data.avg_acos.toFixed(1) }}%</div>
          <div class="kpi-target">Target: &lt;{{ targetAcos }}%</div>
        </div>
      </div>
    </div>

    <!-- 如果是全球视图，显示国家列表排行榜 -->
    <div v-if="countryCode === 'GLOBAL'" class="country-ranking-section">
        <h4 class="section-title">Market Performance</h4>
        <div class="ranking-list">
            <div 
                v-for="country in allCountryStats" 
                :key="country.country" 
                class="ranking-item"
                @click="$emit('select-country', country.country)"
            >
                <div class="ranking-info">
                     <span class="mini-flag" v-html="getCountryFlag(country.country)"></span>
                     <span class="mini-name">{{ getCountryLabel(country.country) }}</span>
                </div>
                <div class="ranking-bar-wrapper">
                    <!-- 简单的进度条展示 Spend 占比 -->
                    <div class="ranking-bar-bg">
                        <div class="ranking-bar-fill" :style="{ width: (country.total_spend / data.total_spend * 100) + '%' }"></div>
                    </div>
                </div>
                <div class="ranking-metrics">
                    <span class="ranking-spend">{{ formatCurrency(country.total_spend, country.country) }}</span>
                    <span class="ranking-acos" :class="{ 'warning': country.avg_acos > targetAcos }">{{ country.avg_acos.toFixed(1) }}%</span>
                </div>
            </div>
        </div>
    </div>

    <!-- 如果是单国家视图，显示简单的数据分布（如果有）或其他信息 -->
    <div v-else class="single-country-insight">
       <div class="insight-card">
           <div class="insight-header">
               <span>Search Terms Volume</span>
           </div>
           <div class="insight-value">{{ data.term_count || 0 }} <span class="unit">Keywords</span></div>
           <div class="insight-desc">Analyzed keywords in this market</div>
       </div>
       
       <!-- 这里可以放一个 ECharts 仪表盘 (Placeholder) -->
       <div class="insight-card tech-card">
           <div class="tech-bg"></div>
           <div class="tech-content">
               <div class="tech-label">Performance Score</div>
               <div class="tech-score">{{ calculateScore(data.avg_acos, targetAcos) }}</div>
               <div class="tech-status">{{ getScoreLabel(data.avg_acos, targetAcos) }}</div>
           </div>
       </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Coin, Trophy, TrendCharts, Compass } from '@element-plus/icons-vue';
import { getCountryFlag, getCountryLabel, COUNTRY_CURRENCY_MAP } from '../../types';

const props = defineProps<{
  countryCode: string; // 'GLOBAL' or country code
  data: any; // Stats object
  targetAcos: number;
  allCountryStats?: any[]; // Only needed for Global view
}>();

defineEmits(['select-country']);

const countryName = computed(() => {
  return props.countryCode === 'GLOBAL' ? 'Global Market' : getCountryLabel(props.countryCode);
});

function formatCurrency(amount: number, countryCode: string = '') {
  // 如果是 Global，默认用第一个国家的货币符号或者通用符号，这里简单处理
  // 实际应用中 Global 的货币汇总是个难题（需要汇率），这里假设已经转换或者仅仅展示数字?
  // 按照原逻辑，Global view 下的总计可能只是数字加和（不准确），暂时使用当前选中国家的货币符号
  const code = countryCode || (props.countryCode === 'GLOBAL' ? (props.allCountryStats?.[0]?.country || 'US') : props.countryCode);
  const symbol = COUNTRY_CURRENCY_MAP[code]?.symbol || '$';
  return `${symbol}${amount.toFixed(2)}`;
}

function getAcosStatus(acos: number) {
  if (acos > props.targetAcos) return 'status-danger';
  if (acos > props.targetAcos * 0.8) return 'status-warning';
  return 'status-success';
}

function calculateScore(acos: number, target: number) {
    // 简单的打分逻辑：目标/实际 * 80
    if (acos === 0) return 0;
    let score = Math.round((target / acos) * 85);
    if (score > 100) score = 99; // 留点余地
    if (score < 40) score = 40; // 保底
    return score;
}

function getScoreLabel(acos: number, target: number) {
    if (acos <= target) return 'Excellent';
    if (acos <= target * 1.2) return 'Good';
    return 'Optimization Needed';
}
</script>

<style scoped>
.ad-country-detail {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--el-bg-color);
  border-radius: 12px;
  border: 1px solid var(--el-border-color-lighter);
  padding: 24px;
}

.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 32px;
}

.title-section {
  display: flex;
  align-items: center;
  gap: 16px;
}

.flag-icon {
    width: 48px;
    height: 32px;
    border-radius: 4px;
    overflow: hidden;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
    display: flex;
    align-items: center;
    justify-content: center;
}

.flag-icon :deep(svg) {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.global-icon {
    background: var(--el-color-primary-light-9);
    color: var(--el-color-primary);
    font-size: 24px;
}

.country-name {
    margin: 0;
    font-size: 20px;
    font-weight: 700;
    color: var(--el-text-color-primary);
    line-height: 1.2;
}

.view-label {
    font-size: 12px;
    color: var(--el-text-color-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

/* KPI Cards */
.kpi-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 16px;
    margin-bottom: 32px;
}

.kpi-card {
    background: var(--el-bg-color-page);
    border-radius: 12px;
    padding: 20px;
    display: flex;
    align-items: center;
    gap: 16px;
    border: 1px solid transparent;
    transition: all 0.3s ease;
}

.kpi-card:hover {
    background: var(--el-bg-color);
    border-color: var(--el-border-color-lighter);
    box-shadow: 0 4px 12px rgba(0,0,0,0.05);
}

.kpi-icon {
    width: 48px;
    height: 48px;
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 24px;
}

.spend-card .kpi-icon { background: #e6f7ff; color: #1890ff; }
.sales-card .kpi-icon { background: #f6ffed; color: #52c41a; }
.acos-card .kpi-icon { background: #fff7e6; color: #fa8c16; }

.kpi-content {
    flex: 1;
}

.kpi-label {
    font-size: 13px;
    color: var(--el-text-color-secondary);
    margin-bottom: 4px;
}

.kpi-value {
    font-size: 24px;
    font-weight: 700;
    color: var(--el-text-color-primary);
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
}

/* ACOS Status Colors */
.status-success .kpi-value { color: var(--el-color-success); }
.status-warning .kpi-value { color: var(--el-color-warning); }
.status-danger .kpi-value { color: var(--el-color-danger); }

.kpi-target {
    font-size: 11px;
    color: var(--el-text-color-placeholder);
    margin-top: 2px;
}

/* Ranking List */
.section-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--el-text-color-primary);
    margin-bottom: 16px;
    text-transform: uppercase;
}

.ranking-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.ranking-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    background: var(--el-bg-color-page);
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.2s;
}

.ranking-item:hover {
    background: var(--el-fill-color);
}

.ranking-info {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100px;
}

.mini-flag :deep(svg) {
    width: 20px;
    height: 14px;
    border-radius: 2px;
    vertical-align: middle;
}

.mini-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--el-text-color-regular);
}

.ranking-bar-wrapper {
    flex: 1;
}

.ranking-bar-bg {
    height: 6px;
    background: var(--el-border-color-lighter);
    border-radius: 3px;
    overflow: hidden;
}

.ranking-bar-fill {
    height: 100%;
    background: var(--el-color-primary);
    border-radius: 3px;
}

.ranking-metrics {
    text-align: right;
    width: 120px;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
}

.ranking-spend {
    font-size: 13px;
    font-weight: 600;
    color: var(--el-text-color-primary);
}

.ranking-acos {
    font-size: 11px;
    color: var(--el-text-color-secondary);
}

.ranking-acos.warning {
    color: var(--el-color-warning);
}

/* Single Country Insight */
.single-country-insight {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    margin-top: auto; /* Push to bottom if space allows */
}

.insight-card {
    background: var(--el-bg-color-page);
    border-radius: 12px;
    padding: 16px;
    text-align: center;
}

.insight-header {
    font-size: 12px;
    color: var(--el-text-color-secondary);
    margin-bottom: 8px;
}

.insight-value {
    font-size: 24px;
    font-weight: 700;
    color: var(--el-text-color-primary);
}

.insight-value .unit {
    font-size: 12px;
    font-weight: normal;
    color: var(--el-text-color-secondary);
}

.insight-desc {
    font-size: 11px;
    color: var(--el-text-color-placeholder);
    margin-top: 4px;
}

/* Tech Score Card */
.tech-card {
    background: linear-gradient(135deg, var(--el-color-primary-dark-2) 0%, var(--el-color-primary) 100%);
    color: white;
    position: relative;
    overflow: hidden;
}

.tech-label {
    opacity: 0.8;
    font-size: 12px;
}

.tech-score {
    font-size: 36px;
    font-weight: 800;
    margin: 4px 0;
}

.tech-status {
    background: rgba(255,255,255,0.2);
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 11px;
    display: inline-block;
}
</style>

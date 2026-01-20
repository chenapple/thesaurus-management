<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue';
import { FullScreen, Refresh, Close } from '@element-plus/icons-vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import * as echarts from 'echarts';
import type { MonitoringStats, TrafficLevelStats, OptimizationEvent, Product, SchedulerSettings, SchedulerStatus } from '../types';

// Props
const props = defineProps<{
  selectedProduct: Product | null;
  stats: { keywordCount: number; rootCount: number };
  monitoringStats: MonitoringStats;
  trafficStats: TrafficLevelStats;
  kbStats: { documentCount: number; conversationCount: number };
  recentEvents: OptimizationEvent[];
  topRisers: Array<{ keyword: string; change: number }>;
  topFallers: Array<{ keyword: string; change: number }>;
  schedulerSettings: SchedulerSettings | null;
  schedulerStatus: SchedulerStatus | null;
  isInWindow: boolean;
  countdownHours: string;
  countdownMinutes: string;
  countdownSeconds: string;
}>();

const emit = defineEmits<{
  (e: 'exit'): void;
  (e: 'refresh'): void;
}>();

// 全屏状态
const isFullscreen = ref(false);

// 当前时间
const currentTime = ref('');
const currentDate = ref('');
let timeTimer: ReturnType<typeof setInterval> | null = null;

// 数字动画
const animatedKeywordCount = ref(0);
const animatedRootCount = ref(0);
const animatedMonitoringCount = ref(0);
const animatedTop10 = ref(0);

// ECharts 实例
const rankingChartRef = ref<HTMLElement | null>(null);
const trafficChartRef = ref<HTMLElement | null>(null);
let rankingChart: echarts.ECharts | null = null;
let trafficChart: echarts.ECharts | null = null;

// 更新时间
function updateTime() {
  const now = new Date();
  currentTime.value = now.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  currentDate.value = now.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit', weekday: 'long' });
}

// 数字滚动动画
function animateNumber(target: number, current: ReturnType<typeof ref<number>>, duration: number = 1500) {
  const startValue = current.value ?? 0;
  const startTime = performance.now();

  function update(time: number) {
    const elapsed = time - startTime;
    const progress = Math.min(elapsed / duration, 1);
    const easeProgress = progress === 1 ? 1 : 1 - Math.pow(2, -10 * progress);
    current.value = Math.round(startValue + (target - startValue) * easeProgress);

    if (progress < 1) {
      requestAnimationFrame(update);
    }
  }

  requestAnimationFrame(update);
}

// 初始化排名分布图表
function initRankingChart() {
  if (!rankingChartRef.value) return;

  const rect = rankingChartRef.value.getBoundingClientRect();
  if (rect.width === 0 || rect.height === 0) {
    setTimeout(initRankingChart, 100);
    return;
  }

  rankingChart = echarts.init(rankingChartRef.value);

  const option: echarts.EChartsOption = {
    backgroundColor: 'transparent',
    tooltip: {
      trigger: 'item',
      backgroundColor: 'rgba(0, 20, 40, 0.9)',
      borderColor: '#00d4ff',
      borderWidth: 1,
      textStyle: { color: '#fff' }
    },
    series: [{
      type: 'gauge',
      startAngle: 200,
      endAngle: -20,
      min: 0,
      max: 100,
      splitNumber: 5,
      radius: '90%',
      progress: {
        show: true,
        width: 20,
        itemStyle: {
          color: {
            type: 'linear',
            x: 0, y: 0, x2: 1, y2: 0,
            colorStops: [
              { offset: 0, color: '#00d4ff' },
              { offset: 1, color: '#00ff88' }
            ]
          }
        }
      },
      pointer: { show: false },
      axisLine: {
        lineStyle: {
          width: 20,
          color: [[1, 'rgba(0, 212, 255, 0.15)']]
        }
      },
      axisTick: { show: false },
      splitLine: { show: false },
      axisLabel: { show: false },
      anchor: { show: false },
      title: {
        show: true,
        offsetCenter: [0, '30%'],
        fontSize: 14,
        color: 'rgba(255, 255, 255, 0.6)'
      },
      detail: {
        valueAnimation: true,
        fontSize: 42,
        fontFamily: 'Orbitron, monospace',
        fontWeight: 700,
        offsetCenter: [0, '-5%'],
        formatter: '{value}%',
        color: '#00ff88'
      },
      data: [{
        value: props.monitoringStats.active > 0
          ? Math.round((props.monitoringStats.top10_organic / props.monitoringStats.active) * 100)
          : 0,
        name: 'Top 10 占比'
      }]
    }]
  };

  rankingChart.setOption(option);
}

// 初始化流量分布图表
function initTrafficChart() {
  if (!trafficChartRef.value) return;

  const rect = trafficChartRef.value.getBoundingClientRect();
  if (rect.width === 0 || rect.height === 0) {
    setTimeout(initTrafficChart, 100);
    return;
  }

  trafficChart = echarts.init(trafficChartRef.value);

  const option: echarts.EChartsOption = {
    backgroundColor: 'transparent',
    tooltip: {
      trigger: 'item',
      backgroundColor: 'rgba(0, 20, 40, 0.9)',
      borderColor: '#00d4ff',
      borderWidth: 1,
      textStyle: { color: '#fff' },
      formatter: '{b}: {c} ({d}%)'
    },
    series: [{
      type: 'pie',
      radius: ['50%', '75%'],
      center: ['50%', '50%'],
      avoidLabelOverlap: false,
      itemStyle: {
        borderRadius: 6,
        borderColor: 'rgba(0, 20, 40, 0.8)',
        borderWidth: 3
      },
      label: {
        show: true,
        position: 'outside',
        color: 'rgba(255, 255, 255, 0.8)',
        fontSize: 12,
        formatter: '{b}\n{d}%'
      },
      labelLine: {
        show: true,
        lineStyle: { color: 'rgba(255, 255, 255, 0.3)' }
      },
      emphasis: {
        label: { fontSize: 14, fontWeight: 'bold' },
        itemStyle: {
          shadowBlur: 20,
          shadowColor: 'rgba(0, 212, 255, 0.5)'
        }
      },
      data: [
        {
          value: props.trafficStats.big_count,
          name: '大词',
          itemStyle: {
            color: {
              type: 'linear',
              x: 0, y: 0, x2: 1, y2: 1,
              colorStops: [
                { offset: 0, color: '#ff6b6b' },
                { offset: 1, color: '#ee5a5a' }
              ]
            }
          }
        },
        {
          value: props.trafficStats.medium_count,
          name: '中词',
          itemStyle: {
            color: {
              type: 'linear',
              x: 0, y: 0, x2: 1, y2: 1,
              colorStops: [
                { offset: 0, color: '#ffd93d' },
                { offset: 1, color: '#f5c800' }
              ]
            }
          }
        },
        {
          value: props.trafficStats.small_count,
          name: '小词',
          itemStyle: {
            color: {
              type: 'linear',
              x: 0, y: 0, x2: 1, y2: 1,
              colorStops: [
                { offset: 0, color: '#6bcfff' },
                { offset: 1, color: '#4db8ff' }
              ]
            }
          }
        }
      ]
    }]
  };

  trafficChart.setOption(option);
}

// 切换全屏
async function toggleFullscreen() {
  try {
    const appWindow = getCurrentWindow();
    const currentFullscreen = await appWindow.isFullscreen();

    if (currentFullscreen) {
      await appWindow.setFullscreen(false);
      isFullscreen.value = false;
    } else {
      await appWindow.setFullscreen(true);
      isFullscreen.value = true;
    }
  } catch (e) {
    // 降级到浏览器全屏
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen();
      isFullscreen.value = true;
    } else {
      document.exitFullscreen();
      isFullscreen.value = false;
    }
  }
}

// 更新图表
function updateCharts() {
  if (rankingChart) {
    const value = props.monitoringStats.active > 0
      ? Math.round((props.monitoringStats.top10_organic / props.monitoringStats.active) * 100)
      : 0;
    rankingChart.setOption({
      series: [{ data: [{ value, name: 'Top 10 占比' }] }]
    });
  }

  if (trafficChart) {
    trafficChart.setOption({
      series: [{
        data: [
          { value: props.trafficStats.big_count, name: '大词' },
          { value: props.trafficStats.medium_count, name: '中词' },
          { value: props.trafficStats.small_count, name: '小词' }
        ]
      }]
    });
  }
}

// 处理窗口大小变化
function handleResize() {
  rankingChart?.resize();
  trafficChart?.resize();
}

// 监听数据变化
watch(() => props.stats.keywordCount, (val) => animateNumber(val, animatedKeywordCount));
watch(() => props.stats.rootCount, (val) => animateNumber(val, animatedRootCount));
watch(() => props.monitoringStats.active, (val) => animateNumber(val, animatedMonitoringCount));
watch(() => props.monitoringStats.top10_organic, (val) => animateNumber(val, animatedTop10));

watch([() => props.monitoringStats, () => props.trafficStats], () => {
  updateCharts();
}, { deep: true });

onMounted(() => {
  updateTime();
  timeTimer = setInterval(updateTime, 1000);

  // 初始动画
  animateNumber(props.stats.keywordCount, animatedKeywordCount);
  animateNumber(props.stats.rootCount, animatedRootCount);
  animateNumber(props.monitoringStats.active, animatedMonitoringCount);
  animateNumber(props.monitoringStats.top10_organic, animatedTop10);

  // 延迟初始化图表
  nextTick(() => {
    setTimeout(() => {
      initRankingChart();
      initTrafficChart();
    }, 200);
  });

  window.addEventListener('resize', handleResize);
});

onUnmounted(() => {
  if (timeTimer) clearInterval(timeTimer);
  window.removeEventListener('resize', handleResize);
  rankingChart?.dispose();
  trafficChart?.dispose();
});
</script>

<template>
  <div class="bigscreen-overlay" style="position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: 99999; background: #0a0e17; overflow: hidden;">
    <div class="bigscreen-container" style="width: 100%; height: 100%; display: flex; flex-direction: column; overflow: hidden; background: #0a0e17;">
      <!-- 背景效果 -->
      <div class="bg-grid"></div>
      <div class="bg-gradient"></div>
      <div class="scan-line"></div>

      <!-- 顶部标题栏 -->
      <header class="screen-header">
        <div class="header-left">
          <div class="logo-area">
            <div class="logo-glow"></div>
            <span class="logo-text">THESAURUS</span>
          </div>
          <div class="product-name" v-if="selectedProduct">
            <span class="label">当前产品</span>
            <span class="value">{{ selectedProduct.name }}</span>
          </div>
        </div>

        <div class="header-center">
          <h1 class="main-title">
            <span class="title-text">数据监控中心</span>
            <span class="title-sub">DATA MONITORING CENTER</span>
          </h1>
        </div>

        <div class="header-right">
          <div class="datetime">
            <div class="time">{{ currentTime }}</div>
            <div class="date">{{ currentDate }}</div>
          </div>
          <div class="header-actions">
            <button class="action-btn" @click="emit('refresh')" title="刷新数据">
              <el-icon><Refresh /></el-icon>
            </button>
            <button class="action-btn" @click="toggleFullscreen" title="全屏">
              <el-icon><FullScreen /></el-icon>
            </button>
            <button class="action-btn exit-btn" @click="emit('exit')" title="退出大屏">
              <el-icon><Close /></el-icon>
              <span>退出</span>
            </button>
          </div>
        </div>
      </header>

      <!-- 主内容区 -->
      <main class="screen-main">
        <!-- 左侧面板 -->
        <aside class="panel panel-left">
          <!-- 关键词统计 -->
          <div class="data-card">
            <div class="card-header">
              <span class="card-title">关键词数据</span>
              <span class="card-badge">KEYWORDS</span>
            </div>
            <div class="card-body stats-row">
              <div class="stat-item">
                <div class="stat-value glow-cyan">{{ animatedKeywordCount.toLocaleString() }}</div>
                <div class="stat-label">总关键词</div>
              </div>
              <div class="stat-divider"></div>
              <div class="stat-item">
                <div class="stat-value glow-purple">{{ animatedRootCount.toLocaleString() }}</div>
                <div class="stat-label">词根数量</div>
              </div>
            </div>
          </div>

          <!-- 排名上升榜 -->
          <div class="data-card flex-grow">
            <div class="card-header">
              <span class="card-title">排名上升 TOP5</span>
              <span class="card-badge rise">RISING</span>
            </div>
            <div class="card-body">
              <div class="rank-list">
                <div v-for="(item, index) in topRisers.slice(0, 5)" :key="index" class="rank-item rise">
                  <span class="rank-index">{{ index + 1 }}</span>
                  <span class="rank-keyword">{{ item.keyword }}</span>
                  <span class="rank-change">+{{ item.change }}</span>
                </div>
                <div v-if="topRisers.length === 0" class="empty-hint">暂无数据</div>
              </div>
            </div>
          </div>

          <!-- 排名下降榜 -->
          <div class="data-card flex-grow">
            <div class="card-header">
              <span class="card-title">排名下降 TOP5</span>
              <span class="card-badge fall">FALLING</span>
            </div>
            <div class="card-body">
              <div class="rank-list">
                <div v-for="(item, index) in topFallers.slice(0, 5)" :key="index" class="rank-item fall">
                  <span class="rank-index">{{ index + 1 }}</span>
                  <span class="rank-keyword">{{ item.keyword }}</span>
                  <span class="rank-change">{{ item.change }}</span>
                </div>
                <div v-if="topFallers.length === 0" class="empty-hint">暂无数据</div>
              </div>
            </div>
          </div>
        </aside>

        <!-- 中央面板 -->
        <section class="panel panel-center">
          <!-- 核心指标 -->
          <div class="core-metrics">
            <div class="metric-card">
              <div class="metric-icon">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
                </svg>
              </div>
              <div class="metric-info">
                <div class="metric-value">{{ animatedMonitoringCount }}</div>
                <div class="metric-label">监控中</div>
              </div>
            </div>

            <div class="metric-card highlight">
              <div class="metric-icon">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                </svg>
              </div>
              <div class="metric-info">
                <div class="metric-value">{{ animatedTop10 }}</div>
                <div class="metric-label">Top 10</div>
              </div>
            </div>

            <div class="metric-card">
              <div class="metric-icon">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="3" y="3" width="18" height="18" rx="2"/>
                  <path d="M3 9h18M9 21V9"/>
                </svg>
              </div>
              <div class="metric-info">
                <div class="metric-value">{{ monitoringStats.top30_organic }}</div>
                <div class="metric-label">Top 30</div>
              </div>
            </div>

            <div class="metric-card">
              <div class="metric-icon">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10"/>
                  <path d="M12 6v6l4 2"/>
                </svg>
              </div>
              <div class="metric-info">
                <div class="metric-value">{{ kbStats.documentCount }}</div>
                <div class="metric-label">知识库</div>
              </div>
            </div>
          </div>

          <!-- 图表区域 -->
          <div class="charts-area">
            <div class="chart-card">
              <div class="chart-header">
                <span class="chart-title">Top 10 占比</span>
              </div>
              <div ref="rankingChartRef" class="chart-container"></div>
            </div>

            <div class="chart-card">
              <div class="chart-header">
                <span class="chart-title">流量级别分布</span>
              </div>
              <div ref="trafficChartRef" class="chart-container"></div>
            </div>
          </div>
        </section>

        <!-- 右侧面板 -->
        <aside class="panel panel-right">
          <!-- 自动检测状态 -->
          <div class="data-card scheduler-card">
            <div class="card-header">
              <span class="card-title">自动检测</span>
              <span class="card-badge" :class="{ active: schedulerSettings?.enabled }">
                {{ schedulerSettings?.enabled ? 'ACTIVE' : 'INACTIVE' }}
              </span>
            </div>
            <div class="card-body countdown-area">
              <template v-if="schedulerSettings?.enabled">
                <template v-if="!isInWindow">
                  <div class="countdown-display">
                    <div class="countdown-unit">
                      <span class="unit-value">{{ countdownHours }}</span>
                      <span class="unit-label">时</span>
                    </div>
                    <span class="countdown-sep">:</span>
                    <div class="countdown-unit">
                      <span class="unit-value">{{ countdownMinutes }}</span>
                      <span class="unit-label">分</span>
                    </div>
                    <span class="countdown-sep">:</span>
                    <div class="countdown-unit">
                      <span class="unit-value">{{ countdownSeconds }}</span>
                      <span class="unit-label">秒</span>
                    </div>
                  </div>
                  <div class="countdown-label">距离下次扫描</div>
                </template>
                <template v-else>
                  <div class="scanning-status">
                    <div class="scanning-ring"></div>
                    <div class="scanning-ring delay-1"></div>
                    <div class="scanning-ring delay-2"></div>
                    <span class="scanning-text">扫描中...</span>
                  </div>
                </template>
              </template>
              <div v-else class="scheduler-off">
                <span>未启用</span>
              </div>
            </div>
          </div>

          <!-- 优化事件 -->
          <div class="data-card flex-grow">
            <div class="card-header">
              <span class="card-title">优化日志</span>
              <span class="card-badge">EVENTS</span>
            </div>
            <div class="card-body">
              <div class="event-list">
                <div v-for="event in recentEvents.slice(0, 6)" :key="event.id" class="event-item">
                  <div class="event-dot"></div>
                  <div class="event-content">
                    <div class="event-title">{{ event.title }}</div>
                    <div class="event-meta">
                      <span class="event-date">{{ new Date(event.event_date).toLocaleDateString('zh-CN', { month: '2-digit', day: '2-digit' }) }}</span>
                      <span class="event-type">{{ event.event_type === 'listing' ? 'Listing' : '广告' }}</span>
                    </div>
                  </div>
                </div>
                <div v-if="recentEvents.length === 0" class="empty-hint">暂无记录</div>
              </div>
            </div>
          </div>

          <!-- 广告位占领 -->
          <div class="data-card">
            <div class="card-header">
              <span class="card-title">广告位占领</span>
              <span class="card-badge">ADS</span>
            </div>
            <div class="card-body">
              <div class="ads-stat">
                <div class="ads-value glow-orange">{{ monitoringStats.with_sponsored }}</div>
                <div class="ads-progress">
                  <div class="progress-track">
                    <div
                      class="progress-fill"
                      :style="{ width: monitoringStats.active > 0 ? (monitoringStats.with_sponsored / monitoringStats.active * 100) + '%' : '0%' }"
                    ></div>
                  </div>
                  <span class="progress-label">
                    {{ monitoringStats.active > 0 ? Math.round(monitoringStats.with_sponsored / monitoringStats.active * 100) : 0 }}%
                  </span>
                </div>
              </div>
            </div>
          </div>
        </aside>
      </main>

      <!-- 底部状态栏 -->
      <footer class="screen-footer">
        <div class="footer-left">
          <span class="status-dot active"></span>
          <span>系统运行正常</span>
        </div>
        <div class="footer-center">
          <span>THESAURUS MANAGEMENT SYSTEM v0.9.7</span>
        </div>
        <div class="footer-right">
          <span>数据更新于 {{ currentTime }}</span>
        </div>
      </footer>
    </div>
  </div>
</template>

<style scoped>
@import url('https://fonts.googleapis.com/css2?family=Orbitron:wght@400;500;600;700;800;900&family=Rajdhani:wght@300;400;500;600;700&display=swap');

/* 全屏覆盖层 - 关键修复：使用独立的覆盖层结构 */
.bigscreen-overlay {
  position: fixed !important;
  top: 0 !important;
  left: 0 !important;
  right: 0 !important;
  bottom: 0 !important;
  width: 100vw !important;
  height: 100vh !important;
  z-index: 99999 !important;
  background: #0a0e17 !important;
  overflow: hidden !important;
}

.bigscreen-container {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  color: #fff;
  font-family: 'Rajdhani', 'PingFang SC', sans-serif;
  overflow: hidden;
  position: relative;
  background: #0a0e17;
}

/* 背景效果 */
.bg-grid {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(rgba(0, 212, 255, 0.03) 1px, transparent 1px),
    linear-gradient(90deg, rgba(0, 212, 255, 0.03) 1px, transparent 1px);
  background-size: 50px 50px;
  pointer-events: none;
}

.bg-gradient {
  position: absolute;
  inset: 0;
  background:
    radial-gradient(ellipse at 20% 20%, rgba(0, 100, 150, 0.15) 0%, transparent 50%),
    radial-gradient(ellipse at 80% 80%, rgba(100, 0, 150, 0.1) 0%, transparent 50%),
    radial-gradient(ellipse at 50% 50%, rgba(0, 50, 100, 0.1) 0%, transparent 70%);
  pointer-events: none;
}

.scan-line {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, transparent, rgba(0, 212, 255, 0.5), transparent);
  animation: scanLine 4s linear infinite;
  pointer-events: none;
}

@keyframes scanLine {
  0% { top: 0; opacity: 0; }
  10% { opacity: 1; }
  90% { opacity: 1; }
  100% { top: 100%; opacity: 0; }
}

/* 头部 */
.screen-header {
  position: relative;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 32px;
  background: linear-gradient(180deg, rgba(0, 20, 40, 0.9) 0%, rgba(0, 20, 40, 0.5) 100%);
  border-bottom: 1px solid rgba(0, 212, 255, 0.2);
  z-index: 10;
  flex-shrink: 0;
}

.header-left, .header-right {
  display: flex;
  align-items: center;
  gap: 24px;
  flex: 1;
}

.header-right {
  justify-content: flex-end;
}

.header-center {
  flex: 2;
  text-align: center;
}

.logo-area {
  position: relative;
  display: flex;
  align-items: center;
}

.logo-glow {
  position: absolute;
  width: 40px;
  height: 40px;
  background: radial-gradient(circle, rgba(0, 212, 255, 0.4) 0%, transparent 70%);
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { transform: scale(1); opacity: 0.5; }
  50% { transform: scale(1.2); opacity: 0.8; }
}

.logo-text {
  font-family: 'Orbitron', monospace;
  font-size: 18px;
  font-weight: 700;
  letter-spacing: 3px;
  background: linear-gradient(135deg, #00d4ff 0%, #00ff88 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.product-name {
  display: flex;
  flex-direction: column;
  padding-left: 24px;
  border-left: 1px solid rgba(0, 212, 255, 0.3);
}

.product-name .label {
  font-size: 10px;
  color: rgba(255, 255, 255, 0.4);
  text-transform: uppercase;
  letter-spacing: 1px;
}

.product-name .value {
  font-size: 14px;
  color: #00d4ff;
  font-weight: 600;
}

.main-title {
  margin: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.title-text {
  font-family: 'Rajdhani', sans-serif;
  font-size: 28px;
  font-weight: 700;
  letter-spacing: 8px;
  background: linear-gradient(135deg, #fff 0%, #00d4ff 50%, #00ff88 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.title-sub {
  font-family: 'Orbitron', monospace;
  font-size: 10px;
  letter-spacing: 4px;
  color: rgba(255, 255, 255, 0.4);
  margin-top: 4px;
}

.datetime {
  text-align: right;
}

.datetime .time {
  font-family: 'Orbitron', monospace;
  font-size: 24px;
  font-weight: 600;
  color: #00d4ff;
  text-shadow: 0 0 10px rgba(0, 212, 255, 0.5);
}

.datetime .date {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
}

.header-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  min-width: 36px;
  height: 36px;
  padding: 0 12px;
  background: rgba(0, 212, 255, 0.1);
  border: 1px solid rgba(0, 212, 255, 0.3);
  border-radius: 6px;
  color: #00d4ff;
  cursor: pointer;
  transition: all 0.3s ease;
  font-family: 'Rajdhani', sans-serif;
  font-size: 14px;
  font-weight: 600;
}

.action-btn:hover {
  background: rgba(0, 212, 255, 0.2);
  border-color: #00d4ff;
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.3);
}

.action-btn.exit-btn {
  background: rgba(255, 100, 100, 0.1);
  border-color: rgba(255, 100, 100, 0.3);
  color: #ff6b6b;
}

.action-btn.exit-btn:hover {
  background: rgba(255, 100, 100, 0.2);
  border-color: #ff6b6b;
  box-shadow: 0 0 15px rgba(255, 100, 100, 0.3);
}

/* 主内容 */
.screen-main {
  flex: 1;
  display: flex;
  gap: 20px;
  padding: 20px 32px;
  overflow: hidden;
  position: relative;
  z-index: 1;
  min-height: 0;
}

.panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-height: 0;
}

.panel-left, .panel-right {
  width: 320px;
  flex-shrink: 0;
}

.panel-center {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 20px;
  min-width: 0;
}

/* 数据卡片 */
.data-card {
  position: relative;
  background: linear-gradient(135deg, rgba(0, 30, 60, 0.8) 0%, rgba(0, 20, 40, 0.9) 100%);
  border: 1px solid rgba(0, 212, 255, 0.2);
  border-radius: 8px;
  overflow: hidden;
}

.data-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, transparent, #00d4ff, transparent);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid rgba(0, 212, 255, 0.1);
}

.card-title {
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
  letter-spacing: 1px;
}

.card-badge {
  font-family: 'Orbitron', monospace;
  font-size: 10px;
  padding: 2px 8px;
  background: rgba(0, 212, 255, 0.15);
  border: 1px solid rgba(0, 212, 255, 0.3);
  border-radius: 4px;
  color: #00d4ff;
  letter-spacing: 1px;
}

.card-badge.rise {
  background: rgba(0, 255, 136, 0.15);
  border-color: rgba(0, 255, 136, 0.3);
  color: #00ff88;
}

.card-badge.fall {
  background: rgba(255, 107, 107, 0.15);
  border-color: rgba(255, 107, 107, 0.3);
  color: #ff6b6b;
}

.card-badge.active {
  background: rgba(0, 255, 136, 0.15);
  border-color: rgba(0, 255, 136, 0.3);
  color: #00ff88;
  animation: badgePulse 2s ease-in-out infinite;
}

@keyframes badgePulse {
  0%, 100% { box-shadow: 0 0 5px rgba(0, 255, 136, 0.3); }
  50% { box-shadow: 0 0 15px rgba(0, 255, 136, 0.5); }
}

.card-body {
  padding: 16px;
}

.flex-grow {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.flex-grow .card-body {
  flex: 1;
  overflow: auto;
}

/* 统计行 */
.stats-row {
  display: flex;
  align-items: center;
  justify-content: space-around;
}

.stat-item {
  text-align: center;
}

.stat-value {
  font-family: 'Orbitron', monospace;
  font-size: 36px;
  font-weight: 700;
  line-height: 1;
}

.stat-label {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  margin-top: 8px;
}

.stat-divider {
  width: 1px;
  height: 50px;
  background: linear-gradient(to bottom, transparent, rgba(0, 212, 255, 0.3), transparent);
}

.glow-cyan {
  color: #00d4ff;
  text-shadow: 0 0 20px rgba(0, 212, 255, 0.5);
}

.glow-purple {
  color: #a855f7;
  text-shadow: 0 0 20px rgba(168, 85, 247, 0.5);
}

.glow-orange {
  color: #ff9500;
  text-shadow: 0 0 20px rgba(255, 149, 0, 0.5);
}

/* 排名列表 */
.rank-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.rank-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 6px;
  border-left: 3px solid transparent;
  transition: all 0.3s ease;
}

.rank-item.rise {
  border-left-color: #00ff88;
}

.rank-item.fall {
  border-left-color: #ff6b6b;
}

.rank-item:hover {
  background: rgba(0, 212, 255, 0.1);
  transform: translateX(4px);
}

.rank-index {
  font-family: 'Orbitron', monospace;
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.4);
  width: 20px;
}

.rank-keyword {
  flex: 1;
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: rgba(255, 255, 255, 0.8);
}

.rank-change {
  font-family: 'Orbitron', monospace;
  font-size: 14px;
  font-weight: 600;
}

.rank-item.rise .rank-change {
  color: #00ff88;
}

.rank-item.fall .rank-change {
  color: #ff6b6b;
}

.empty-hint {
  text-align: center;
  color: rgba(255, 255, 255, 0.3);
  padding: 24px;
  font-size: 13px;
}

/* 核心指标 */
.core-metrics {
  display: flex;
  gap: 16px;
  flex-shrink: 0;
}

.metric-card {
  flex: 1;
  position: relative;
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px 24px;
  background: linear-gradient(135deg, rgba(0, 30, 60, 0.8) 0%, rgba(0, 20, 40, 0.9) 100%);
  border: 1px solid rgba(0, 212, 255, 0.2);
  border-radius: 12px;
  overflow: hidden;
}

.metric-card.highlight {
  border-color: rgba(0, 255, 136, 0.4);
  background: linear-gradient(135deg, rgba(0, 50, 40, 0.8) 0%, rgba(0, 30, 30, 0.9) 100%);
}

.metric-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 212, 255, 0.1);
  border-radius: 12px;
  color: #00d4ff;
}

.metric-card.highlight .metric-icon {
  background: rgba(0, 255, 136, 0.1);
  color: #00ff88;
}

.metric-icon svg {
  width: 24px;
  height: 24px;
}

.metric-info {
  flex: 1;
}

.metric-value {
  font-family: 'Orbitron', monospace;
  font-size: 32px;
  font-weight: 700;
  color: #00d4ff;
  line-height: 1;
  text-shadow: 0 0 20px rgba(0, 212, 255, 0.4);
}

.metric-card.highlight .metric-value {
  color: #00ff88;
  text-shadow: 0 0 20px rgba(0, 255, 136, 0.4);
}

.metric-label {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.5);
  margin-top: 4px;
}

/* 图表区域 */
.charts-area {
  flex: 1;
  display: flex;
  gap: 20px;
  min-height: 0;
}

.chart-card {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, rgba(0, 30, 60, 0.8) 0%, rgba(0, 20, 40, 0.9) 100%);
  border: 1px solid rgba(0, 212, 255, 0.2);
  border-radius: 12px;
  overflow: hidden;
  min-height: 0;
}

.chart-header {
  padding: 12px 16px;
  border-bottom: 1px solid rgba(0, 212, 255, 0.1);
  flex-shrink: 0;
}

.chart-title {
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
  letter-spacing: 1px;
}

.chart-container {
  flex: 1;
  min-height: 200px;
}

/* 倒计时 */
.countdown-area {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 120px;
}

.countdown-display {
  display: flex;
  align-items: center;
  gap: 4px;
}

.countdown-unit {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.unit-value {
  font-family: 'Orbitron', monospace;
  font-size: 36px;
  font-weight: 700;
  color: #00d4ff;
  text-shadow: 0 0 20px rgba(0, 212, 255, 0.5);
  line-height: 1;
}

.unit-label {
  font-size: 10px;
  color: rgba(255, 255, 255, 0.4);
  margin-top: 4px;
}

.countdown-sep {
  font-family: 'Orbitron', monospace;
  font-size: 28px;
  color: rgba(0, 212, 255, 0.5);
  animation: blink 1s step-end infinite;
  margin-bottom: 16px;
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

.countdown-label {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  margin-top: 12px;
}

.scanning-status {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100px;
}

.scanning-ring {
  position: absolute;
  width: 60px;
  height: 60px;
  border: 2px solid rgba(0, 255, 136, 0.5);
  border-radius: 50%;
  animation: scanRing 2s ease-out infinite;
}

.scanning-ring.delay-1 {
  animation-delay: 0.6s;
}

.scanning-ring.delay-2 {
  animation-delay: 1.2s;
}

@keyframes scanRing {
  0% { transform: scale(0.5); opacity: 1; }
  100% { transform: scale(2); opacity: 0; }
}

.scanning-text {
  font-size: 14px;
  color: #00ff88;
  font-weight: 600;
  z-index: 1;
}

.scheduler-off {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.3);
}

/* 事件列表 */
.event-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.event-item {
  display: flex;
  gap: 12px;
  padding: 10px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 6px;
  transition: all 0.3s ease;
}

.event-item:hover {
  background: rgba(0, 212, 255, 0.1);
}

.event-dot {
  width: 8px;
  height: 8px;
  background: #00d4ff;
  border-radius: 50%;
  margin-top: 6px;
  flex-shrink: 0;
  box-shadow: 0 0 8px rgba(0, 212, 255, 0.5);
}

.event-content {
  flex: 1;
  min-width: 0;
}

.event-title {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.9);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.event-meta {
  display: flex;
  gap: 8px;
  margin-top: 4px;
}

.event-date {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.4);
}

.event-type {
  font-size: 10px;
  padding: 1px 6px;
  background: rgba(0, 212, 255, 0.15);
  border-radius: 3px;
  color: #00d4ff;
}

/* 广告统计 */
.ads-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.ads-value {
  font-family: 'Orbitron', monospace;
  font-size: 48px;
  font-weight: 700;
}

.ads-progress {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 12px;
}

.progress-track {
  flex: 1;
  height: 8px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #ff9500, #ffcc00);
  border-radius: 4px;
  transition: width 1s ease;
  box-shadow: 0 0 10px rgba(255, 149, 0, 0.5);
}

.progress-label {
  font-family: 'Orbitron', monospace;
  font-size: 14px;
  color: #ff9500;
  min-width: 40px;
  text-align: right;
}

/* 底部 */
.screen-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 32px;
  background: linear-gradient(0deg, rgba(0, 20, 40, 0.9) 0%, rgba(0, 20, 40, 0.5) 100%);
  border-top: 1px solid rgba(0, 212, 255, 0.2);
  font-size: 12px;
  color: rgba(255, 255, 255, 0.4);
  position: relative;
  z-index: 10;
  flex-shrink: 0;
}

.footer-left, .footer-center, .footer-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #666;
}

.status-dot.active {
  background: #00ff88;
  box-shadow: 0 0 8px rgba(0, 255, 136, 0.5);
  animation: statusPulse 2s ease-in-out infinite;
}

@keyframes statusPulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

/* 滚动条 */
::-webkit-scrollbar {
  width: 4px;
}

::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 2px;
}

::-webkit-scrollbar-thumb {
  background: rgba(0, 212, 255, 0.3);
  border-radius: 2px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 212, 255, 0.5);
}
</style>

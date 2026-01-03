<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { TrendCharts, Document, Monitor, Folder, Top, Bottom } from '@element-plus/icons-vue';
import * as api from '../api';
import type { MonitoringStats, TrafficLevelStats, OptimizationEvent, Product, SchedulerSettings, SchedulerStatus } from '../types';

// Props
const props = defineProps<{
  selectedProduct: Product | null;
}>();

// ViewMode 类型
type ViewMode = 'dashboard' | 'keywords' | 'roots' | 'wordcloud' | 'monitoring' | 'knowledge';

// Emits
const emit = defineEmits<{
  (e: 'switchView', view: ViewMode): void;
}>();

// 加载状态
const loading = ref(false);

// 统计数据
const stats = ref({
  keywordCount: 0,
  rootCount: 0,
});

// 监控统计
const monitoringStats = ref<MonitoringStats>({
  total: 0,
  active: 0,
  top10_organic: 0,
  top30_organic: 0,
  with_sponsored: 0,
});

// 流量级别统计
const trafficStats = ref<TrafficLevelStats>({
  big_count: 0,
  medium_count: 0,
  small_count: 0,
});

// 知识库统计
const kbStats = ref({
  documentCount: 0,
  conversationCount: 0,
});

// 最近优化事件
const recentEvents = ref<OptimizationEvent[]>([]);

// 排名变化榜
interface RankChange {
  keyword: string;
  monitoringId: number;
  change: number;  // 正数表示上升，负数表示下降
  oldRank: number | null;
  newRank: number | null;
}
const topRisers = ref<RankChange[]>([]);
const topFallers = ref<RankChange[]>([]);

// 定时任务状态
const schedulerStatus = ref<SchedulerStatus | null>(null);
const schedulerSettings = ref<SchedulerSettings | null>(null);
const countdownText = ref('');
let countdownTimer: ReturnType<typeof setInterval> | null = null;

// 分离的倒计时数据（用于大字体显示）
const countdownHours = ref('00');
const countdownMinutes = ref('00');
const countdownSeconds = ref('00');
const nextWindowLabel = ref('');
const isInWindow = ref(false);

// 加载所有数据
async function loadDashboardData() {
  if (!props.selectedProduct) return;

  loading.value = true;

  try {
    // 并行加载所有数据
    const [
      statsResult,
      monitoringResult,
      trafficResult,
      eventsResult,
      documentsResult,
      conversationsResult,
    ] = await Promise.all([
      api.getStats(props.selectedProduct.id).catch(() => [0, 0] as [number, number]),
      api.getMonitoringStats(props.selectedProduct.id).catch(() => ({
        total: 0, active: 0, top10_organic: 0, top30_organic: 0, with_sponsored: 0
      })),
      api.getTrafficLevelStats(props.selectedProduct.id).catch(() => ({
        big_count: 0, medium_count: 0, small_count: 0
      })),
      api.getOptimizationEvents(props.selectedProduct.id).catch(() => []),
      api.kbGetDocuments().catch(() => []),
      api.kbGetConversations().catch(() => []),
    ]);

    stats.value = {
      keywordCount: statsResult[0],
      rootCount: statsResult[1],
    };

    monitoringStats.value = monitoringResult;
    trafficStats.value = trafficResult;
    recentEvents.value = eventsResult.slice(0, 5); // 只取最近5条

    kbStats.value = {
      documentCount: documentsResult.length,
      conversationCount: conversationsResult.length,
    };

    // 加载排名变化榜数据
    await loadRankingChanges();

    // 加载调度器状态
    await loadSchedulerStatus();
  } catch (e) {
    console.error('加载仪表板数据失败:', e);
  } finally {
    loading.value = false;
  }
}

// 加载排名变化榜数据
async function loadRankingChanges() {
  if (!props.selectedProduct) return;

  try {
    // 并行获取迷你图数据和监控列表
    const [sparklines, [monitoringList]] = await Promise.all([
      api.getMonitoringSparklines(props.selectedProduct.id, 7),
      api.getKeywordMonitoringList({
        productId: props.selectedProduct.id,
        page: 1,
        pageSize: 10000,
      }),
    ]);

    // 创建 monitoring_id -> keyword 的映射
    const keywordMap = new Map<number, string>();
    for (const item of monitoringList) {
      keywordMap.set(item.id, item.keyword);
    }

    // 计算每个监控项的排名变化
    const changes: RankChange[] = [];
    for (const sparkline of sparklines) {
      const ranks = sparkline.organic_ranks;
      if (!ranks || ranks.length === 0) continue;

      // 找到第一个非空值和最后一个非空值
      let firstRank: number | null = null;
      let lastRank: number | null = null;

      for (let i = 0; i < ranks.length; i++) {
        if (ranks[i] !== null) {
          if (firstRank === null) firstRank = ranks[i];
          lastRank = ranks[i];
        }
      }

      // 如果有有效数据，计算变化
      if (firstRank !== null && lastRank !== null) {
        const change = firstRank - lastRank; // 正数表示排名上升（数字变小）
        if (change !== 0) {
          changes.push({
            keyword: keywordMap.get(sparkline.monitoring_id) || `#${sparkline.monitoring_id}`,
            monitoringId: sparkline.monitoring_id,
            change,
            oldRank: firstRank,
            newRank: lastRank,
          });
        }
      }
    }

    // 排序：上升最多的（change 最大）和下降最多的（change 最小）
    const sorted = [...changes].sort((a, b) => b.change - a.change);
    topRisers.value = sorted.filter(c => c.change > 0).slice(0, 5);
    topFallers.value = sorted.filter(c => c.change < 0).slice(0, 5);
  } catch (e) {
    console.error('加载排名变化数据失败:', e);
  }
}

// 加载调度器状态
async function loadSchedulerStatus() {
  try {
    const [status, settings] = await Promise.all([
      api.getSchedulerStatus(),
      api.getSchedulerSettings(),
    ]);
    schedulerStatus.value = status;
    schedulerSettings.value = settings;
    updateCountdown();
  } catch (e) {
    console.error('加载调度器状态失败:', e);
  }
}

// 计算并更新倒计时
function updateCountdown() {
  if (!schedulerSettings.value || !schedulerSettings.value.enabled) {
    countdownText.value = '';
    isInWindow.value = false;
    return;
  }

  const now = new Date();
  const hour = now.getHours();
  const settings = schedulerSettings.value;

  // 判断当前在哪个时段
  let nextWindowStart: Date;
  let windowLabel: string;

  if (hour < settings.morning_start) {
    // 早间窗口之前
    nextWindowStart = new Date(now);
    nextWindowStart.setHours(settings.morning_start, 0, 0, 0);
    windowLabel = `${settings.morning_start}:00-${settings.morning_end}:00`;
    isInWindow.value = false;
  } else if (hour >= settings.morning_start && hour < settings.morning_end) {
    // 在早间窗口内
    countdownText.value = '检测窗口进行中';
    isInWindow.value = true;
    return;
  } else if (hour < settings.evening_start) {
    // 早间和晚间之间
    nextWindowStart = new Date(now);
    nextWindowStart.setHours(settings.evening_start, 0, 0, 0);
    windowLabel = `${settings.evening_start}:00-${settings.evening_end}:00`;
    isInWindow.value = false;
  } else if (hour >= settings.evening_start && hour < settings.evening_end) {
    // 在晚间窗口内
    countdownText.value = '检测窗口进行中';
    isInWindow.value = true;
    return;
  } else {
    // 晚间窗口之后，计算到明天早间
    nextWindowStart = new Date(now);
    nextWindowStart.setDate(nextWindowStart.getDate() + 1);
    nextWindowStart.setHours(settings.morning_start, 0, 0, 0);
    windowLabel = `明天 ${settings.morning_start}:00-${settings.morning_end}:00`;
    isInWindow.value = false;
  }

  // 计算剩余时间
  const diff = nextWindowStart.getTime() - now.getTime();
  const hours = Math.floor(diff / (1000 * 60 * 60));
  const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));
  const seconds = Math.floor((diff % (1000 * 60)) / 1000);

  // 更新分离的倒计时数据
  countdownHours.value = hours.toString().padStart(2, '0');
  countdownMinutes.value = minutes.toString().padStart(2, '0');
  countdownSeconds.value = seconds.toString().padStart(2, '0');
  nextWindowLabel.value = windowLabel;

  if (hours > 0) {
    countdownText.value = `${windowLabel} (${hours}小时${minutes}分${seconds}秒后)`;
  } else if (minutes > 0) {
    countdownText.value = `${windowLabel} (${minutes}分${seconds}秒后)`;
  } else {
    countdownText.value = `${windowLabel} (${seconds}秒后)`;
  }
}

// 启动倒计时定时器
function startCountdownTimer() {
  if (countdownTimer) clearInterval(countdownTimer);
  countdownTimer = setInterval(updateCountdown, 1000); // 每秒更新
}

// 计算流量级别总数
function getTotalTraffic() {
  return trafficStats.value.big_count + trafficStats.value.medium_count + trafficStats.value.small_count;
}

// 计算百分比
function getPercentage(count: number, total: number): string {
  if (total === 0) return '0';
  return ((count / total) * 100).toFixed(0);
}

// 格式化日期
function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  return `${(date.getMonth() + 1).toString().padStart(2, '0')}-${date.getDate().toString().padStart(2, '0')}`;
}

// 获取事件类型标签
function getEventTypeLabel(type: string): string {
  const labels: Record<string, string> = {
    'listing': 'Listing优化',
    'advertising': '广告优化',
    'pricing': '价格调整',
    'inventory': '库存管理',
    'other': '其他',
  };
  return labels[type] || type;
}

// 解析受影响关键词
function parseAffectedKeywords(json?: string): string[] {
  if (!json) return [];
  try {
    return JSON.parse(json);
  } catch {
    return [];
  }
}

// 监听产品变化
watch(() => props.selectedProduct, () => {
  loadDashboardData();
}, { immediate: true });

onMounted(() => {
  loadDashboardData();
  startCountdownTimer();
});

onUnmounted(() => {
  if (countdownTimer) {
    clearInterval(countdownTimer);
    countdownTimer = null;
  }
});

// 格式化时间
function formatDateTime(dateStr: string | null): string {
  if (!dateStr) return '-';
  const date = new Date(dateStr);
  return `${date.getMonth() + 1}/${date.getDate()} ${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}`;
}
</script>

<template>
  <div class="dashboard-container" v-loading="loading">
    <!-- 未选择产品时的提示 -->
    <div v-if="!selectedProduct" class="no-product-state">
      <el-empty description="请先选择或创建一个产品">
        <el-button type="primary" @click="emit('switchView', 'keywords')">
          进入关键词管理
        </el-button>
      </el-empty>
    </div>

    <!-- 有产品时的内容 -->
    <div v-else class="dashboard-content">
    <!-- 欢迎区域 -->
    <div class="welcome-section">
      <h2>数据概览</h2>
      <span class="product-name" v-if="selectedProduct">
        {{ selectedProduct.name }}
      </span>
    </div>

    <!-- 指标卡片 -->
    <div class="stat-cards">
      <div class="stat-card" @click="emit('switchView', 'keywords')">
        <div class="stat-icon keywords">
          <el-icon :size="24"><Document /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ stats.keywordCount.toLocaleString() }}</div>
          <div class="stat-label">关键词</div>
        </div>
      </div>

      <div class="stat-card" @click="emit('switchView', 'roots')">
        <div class="stat-icon roots">
          <el-icon :size="24"><TrendCharts /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ stats.rootCount.toLocaleString() }}</div>
          <div class="stat-label">词根</div>
        </div>
      </div>

      <div class="stat-card" @click="emit('switchView', 'monitoring')">
        <div class="stat-icon monitoring">
          <el-icon :size="24"><Monitor /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">
            {{ monitoringStats.active }} / {{ monitoringStats.total }}
          </div>
          <div class="stat-label">排名监控 (活跃/总数)</div>
        </div>
      </div>

      <div class="stat-card" @click="emit('switchView', 'knowledge')">
        <div class="stat-icon knowledge">
          <el-icon :size="24"><Folder /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ kbStats.documentCount }}</div>
          <div class="stat-label">知识库文档</div>
          <div class="stat-sub">{{ kbStats.conversationCount }} 个对话</div>
        </div>
      </div>
    </div>

    <!-- 排名变化概览 -->
    <div class="section-title">排名监控概览</div>
    <div class="ranking-overview">
      <div class="ranking-card top10">
        <div class="ranking-value">{{ monitoringStats.top10_organic }}</div>
        <div class="ranking-label">Top 10</div>
        <div class="ranking-desc">有机排名前10</div>
      </div>
      <div class="ranking-card top30">
        <div class="ranking-value">{{ monitoringStats.top30_organic }}</div>
        <div class="ranking-label">Top 30</div>
        <div class="ranking-desc">有机排名前30</div>
      </div>
      <div class="ranking-card sponsored">
        <div class="ranking-value">{{ monitoringStats.with_sponsored }}</div>
        <div class="ranking-label">广告位</div>
        <div class="ranking-desc">有广告排名</div>
      </div>
    </div>

    <!-- 中间部分：排名变化榜 + 定时任务 -->
    <div class="middle-section">
      <!-- 排名变化榜 -->
      <div class="ranking-changes-section">
        <div class="section-title">排名变化榜 (近7天)</div>
        <div class="ranking-changes-content" v-if="topRisers.length > 0 || topFallers.length > 0">
          <div class="changes-column risers">
            <div class="column-header">
              <el-icon color="#67c23a"><Top /></el-icon>
              上升 TOP 5
            </div>
            <div class="changes-list">
              <div
                v-for="item in topRisers"
                :key="item.monitoringId"
                class="change-item"
              >
                <span class="keyword-name">{{ item.keyword }}</span>
                <span class="change-value positive">+{{ item.change }}</span>
              </div>
              <div v-if="topRisers.length === 0" class="no-data">暂无上升</div>
            </div>
          </div>
          <div class="changes-column fallers">
            <div class="column-header">
              <el-icon color="#f56c6c"><Bottom /></el-icon>
              下降 TOP 5
            </div>
            <div class="changes-list">
              <div
                v-for="item in topFallers"
                :key="item.monitoringId"
                class="change-item"
              >
                <span class="keyword-name">{{ item.keyword }}</span>
                <span class="change-value negative">{{ item.change }}</span>
              </div>
              <div v-if="topFallers.length === 0" class="no-data">暂无下降</div>
            </div>
          </div>
        </div>
        <div class="empty-state" v-else>
          <p>暂无排名变化数据</p>
          <el-button size="small" @click="emit('switchView', 'monitoring')">
            添加监控
          </el-button>
        </div>
      </div>

      <!-- 定时任务状态 -->
      <div class="scheduler-section">
        <div class="section-title">定时检测</div>
        <div class="scheduler-content" v-if="schedulerSettings">
          <div class="scheduler-status">
            <span class="status-label">状态</span>
            <span class="status-value" :class="{ active: schedulerSettings.enabled && schedulerStatus?.is_running }">
              <span class="status-dot"></span>
              {{ schedulerSettings.enabled ? (schedulerStatus?.is_running ? '运行中' : '已启用') : '已停止' }}
            </span>
          </div>
          <div class="scheduler-info" v-if="schedulerStatus?.last_check_time">
            <span class="info-label">上次检测</span>
            <span class="info-value">{{ formatDateTime(schedulerStatus.last_check_time) }}</span>
          </div>
          <div class="scheduler-windows" v-if="schedulerSettings.enabled">
            <span class="windows-label">检测窗口</span>
            <span class="windows-value">
              {{ schedulerSettings.morning_start }}:00-{{ schedulerSettings.morning_end }}:00,
              {{ schedulerSettings.evening_start }}:00-{{ schedulerSettings.evening_end }}:00
            </span>
          </div>
          <!-- 大倒计时显示 -->
          <div class="countdown-highlight" v-if="countdownText && schedulerSettings.enabled && !isInWindow">
            <div class="countdown-label">下次检测窗口</div>
            <div class="countdown-window">{{ nextWindowLabel }}</div>
            <div class="countdown-timer">
              <span class="countdown-num">{{ countdownHours }}</span>
              <span class="countdown-unit">时</span>
              <span class="countdown-num">{{ countdownMinutes }}</span>
              <span class="countdown-unit">分</span>
              <span class="countdown-num">{{ countdownSeconds }}</span>
              <span class="countdown-unit">秒</span>
            </div>
          </div>
          <!-- 检测窗口进行中状态 -->
          <div class="countdown-highlight in-progress" v-else-if="isInWindow && schedulerSettings.enabled">
            <div class="countdown-label">检测窗口进行中</div>
            <div class="in-progress-indicator">
              <span class="pulse-dot"></span>
              <span class="progress-text">正在执行检测任务</span>
            </div>
          </div>
        </div>
        <div class="empty-state" v-else>
          <p>加载中...</p>
        </div>
      </div>
    </div>

    <!-- 下半部分：两栏布局 -->
    <div class="bottom-section">
      <!-- 流量级别分布 -->
      <div class="traffic-section">
        <div class="section-title">流量级别分布</div>
        <div class="traffic-chart" v-if="getTotalTraffic() > 0">
          <div class="traffic-item">
            <div class="traffic-bar">
              <div
                class="traffic-fill big"
                :style="{ width: getPercentage(trafficStats.big_count, getTotalTraffic()) + '%' }"
              ></div>
            </div>
            <div class="traffic-info">
              <span class="traffic-label">大词</span>
              <span class="traffic-value">{{ trafficStats.big_count }} ({{ getPercentage(trafficStats.big_count, getTotalTraffic()) }}%)</span>
            </div>
          </div>
          <div class="traffic-item">
            <div class="traffic-bar">
              <div
                class="traffic-fill medium"
                :style="{ width: getPercentage(trafficStats.medium_count, getTotalTraffic()) + '%' }"
              ></div>
            </div>
            <div class="traffic-info">
              <span class="traffic-label">中词</span>
              <span class="traffic-value">{{ trafficStats.medium_count }} ({{ getPercentage(trafficStats.medium_count, getTotalTraffic()) }}%)</span>
            </div>
          </div>
          <div class="traffic-item">
            <div class="traffic-bar">
              <div
                class="traffic-fill small"
                :style="{ width: getPercentage(trafficStats.small_count, getTotalTraffic()) + '%' }"
              ></div>
            </div>
            <div class="traffic-info">
              <span class="traffic-label">小词</span>
              <span class="traffic-value">{{ trafficStats.small_count }} ({{ getPercentage(trafficStats.small_count, getTotalTraffic()) }}%)</span>
            </div>
          </div>
        </div>
        <div class="empty-state" v-else>
          <p>暂无流量数据</p>
          <el-button size="small" @click="emit('switchView', 'keywords')">
            导入关键词
          </el-button>
        </div>
      </div>

      <!-- 最近优化事件 -->
      <div class="events-section">
        <div class="section-title">最近优化事件</div>
        <div class="events-list" v-if="recentEvents.length > 0">
          <div
            class="event-item"
            v-for="event in recentEvents"
            :key="event.id"
          >
            <div class="event-date">{{ formatDate(event.event_date) }}</div>
            <div class="event-content">
              <div class="event-title">{{ event.title }}</div>
              <div class="event-meta">
                <el-tag size="small" type="info">{{ getEventTypeLabel(event.event_type) }}</el-tag>
                <span
                  v-if="parseAffectedKeywords(event.affected_keywords).length > 0"
                  class="affected-count"
                >
                  影响 {{ parseAffectedKeywords(event.affected_keywords).length }} 个关键词
                </span>
              </div>
            </div>
          </div>
        </div>
        <div class="empty-state" v-else>
          <p>暂无优化事件</p>
          <el-button size="small" @click="emit('switchView', 'monitoring')">
            记录事件
          </el-button>
        </div>
      </div>
    </div>
    </div>
  </div>
</template>

<style scoped>
.dashboard-container {
  padding: 24px;
}

/* 未选择产品状态 */
.no-product-state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 400px;
}

/* 欢迎区域 */
.welcome-section {
  display: flex;
  align-items: baseline;
  gap: 12px;
  margin-bottom: 24px;
}

.welcome-section h2 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.product-name {
  font-size: 14px;
  color: var(--el-text-color-secondary);
  background: var(--el-fill-color-light);
  padding: 4px 12px;
  border-radius: 4px;
}

/* 指标卡片 */
.stat-cards {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 32px;
}

.stat-card {
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 8px;
  padding: 20px;
  display: flex;
  align-items: center;
  gap: 16px;
  cursor: pointer;
  transition: all 0.2s;
}

.stat-card:hover {
  border-color: var(--el-color-primary);
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.stat-icon.keywords {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.stat-icon.roots {
  background: linear-gradient(135deg, #11998e 0%, #38ef7d 100%);
}

.stat-icon.monitoring {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
}

.stat-icon.knowledge {
  background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
}

.stat-content {
  flex: 1;
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  line-height: 1.2;
}

.stat-label {
  font-size: 13px;
  color: var(--el-text-color-secondary);
  margin-top: 4px;
}

.stat-sub {
  font-size: 12px;
  color: var(--el-text-color-placeholder);
  margin-top: 2px;
}

/* 区域标题 */
.section-title {
  font-size: 16px;
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 16px;
}

/* 排名概览 */
.ranking-overview {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-bottom: 32px;
}

.ranking-card {
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 8px;
  padding: 20px;
  text-align: center;
}

.ranking-value {
  font-size: 32px;
  font-weight: 600;
  margin-bottom: 4px;
}

.ranking-card.top10 .ranking-value {
  color: #67c23a;
}

.ranking-card.top30 .ranking-value {
  color: #e6a23c;
}

.ranking-card.sponsored .ranking-value {
  color: #409eff;
}

.ranking-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.ranking-desc {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 4px;
}

/* 中间部分两栏 */
.middle-section {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: 24px;
  margin-bottom: 24px;
}

/* 排名变化榜 */
.ranking-changes-section,
.scheduler-section {
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 8px;
  padding: 20px;
}

.ranking-changes-content {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.changes-column {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.column-header {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 500;
  color: var(--el-text-color-regular);
  padding-bottom: 8px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.changes-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.change-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: var(--el-fill-color-lighter);
  border-radius: 4px;
}

.keyword-name {
  font-size: 13px;
  color: var(--el-text-color-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 150px;
}

.change-value {
  font-size: 13px;
  font-weight: 600;
}

.change-value.positive {
  color: #67c23a;
}

.change-value.negative {
  color: #f56c6c;
}

.no-data {
  font-size: 13px;
  color: var(--el-text-color-placeholder);
  text-align: center;
  padding: 20px;
}

/* 定时任务状态 */
.scheduler-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.scheduler-status,
.scheduler-info,
.scheduler-windows {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 13px;
}

.status-label,
.info-label,
.windows-label {
  color: var(--el-text-color-secondary);
}

.status-value {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--el-text-color-regular);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #909399;
}

.status-value.active .status-dot {
  background: #67c23a;
  box-shadow: 0 0 8px rgba(103, 194, 58, 0.6);
}

.info-value {
  color: var(--el-text-color-primary);
}

.info-value.countdown {
  color: var(--el-color-primary);
  font-weight: 500;
}

.windows-value {
  font-size: 12px;
  color: var(--el-text-color-placeholder);
}

/* 倒计时突出显示 */
.countdown-highlight {
  margin-top: 16px;
  padding: 16px;
  background: var(--el-color-primary-light-9);
  border-radius: 8px;
  text-align: center;
}

.countdown-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 4px;
}

.countdown-window {
  font-size: 14px;
  color: var(--el-color-primary);
  margin-bottom: 12px;
}

.countdown-timer {
  display: flex;
  justify-content: center;
  align-items: baseline;
  gap: 4px;
}

.countdown-num {
  font-size: 28px;
  font-weight: 600;
  color: var(--el-color-primary);
  font-variant-numeric: tabular-nums;
  min-width: 36px;
}

.countdown-unit {
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

.countdown-highlight.in-progress {
  background: var(--el-color-success-light-9);
}

.in-progress-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin-top: 12px;
}

.pulse-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--el-color-success);
  animation: pulse-animation 1.5s infinite;
}

@keyframes pulse-animation {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.5;
    transform: scale(1.2);
  }
}

.progress-text {
  font-size: 14px;
  color: var(--el-color-success);
  font-weight: 500;
}

/* 下半部分两栏 */
.bottom-section {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
}

/* 流量分布 */
.traffic-section,
.events-section {
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 8px;
  padding: 20px;
}

.traffic-chart {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.traffic-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.traffic-bar {
  height: 8px;
  background: var(--el-fill-color-light);
  border-radius: 4px;
  overflow: hidden;
}

.traffic-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.3s ease;
}

.traffic-fill.big {
  background: linear-gradient(90deg, #f56c6c 0%, #e6a23c 100%);
}

.traffic-fill.medium {
  background: linear-gradient(90deg, #e6a23c 0%, #67c23a 100%);
}

.traffic-fill.small {
  background: linear-gradient(90deg, #909399 0%, #c0c4cc 100%);
}

.traffic-info {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
}

.traffic-label {
  color: var(--el-text-color-regular);
}

.traffic-value {
  color: var(--el-text-color-secondary);
}

/* 优化事件 */
.events-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.event-item {
  display: flex;
  gap: 12px;
  padding: 12px;
  background: var(--el-fill-color-lighter);
  border-radius: 6px;
}

.event-date {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  white-space: nowrap;
  padding-top: 2px;
}

.event-content {
  flex: 1;
  min-width: 0;
}

.event-title {
  font-size: 14px;
  color: var(--el-text-color-primary);
  margin-bottom: 6px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.event-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.affected-count {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

/* 空状态 */
.empty-state {
  text-align: center;
  padding: 32px 20px;
  color: var(--el-text-color-secondary);
}

.empty-state p {
  margin: 0 0 12px;
}

/* 响应式 */
@media (max-width: 1100px) {
  .middle-section {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 900px) {
  .stat-cards {
    grid-template-columns: repeat(2, 1fr);
  }

  .bottom-section {
    grid-template-columns: 1fr;
  }

  .ranking-changes-content {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 600px) {
  .stat-cards {
    grid-template-columns: 1fr;
  }

  .ranking-overview {
    grid-template-columns: 1fr;
  }
}
</style>

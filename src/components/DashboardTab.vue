<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { ElMessage } from 'element-plus';
import { TrendCharts, Document, Monitor, Folder, Top, Bottom, Timer, FullScreen, Calendar, ArrowLeft, ArrowRight, Loading } from '@element-plus/icons-vue';
import BigScreenView from './BigScreenView.vue';
import * as api from '../api';
import VChart from 'vue-echarts';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { LineChart } from 'echarts/charts';
import { GridComponent, TooltipComponent, MarkPointComponent } from 'echarts/components';

// 注册 ECharts 组件
use([CanvasRenderer, LineChart, GridComponent, TooltipComponent, MarkPointComponent]);
import { chatStream, checkApiKeyConfigured } from '../ai-service';

// 视图模式
const viewMode = ref<'normal' | 'bigscreen'>('normal');

// 切换到大屏模式
function switchToBigScreen() {
  viewMode.value = 'bigscreen';
}
import type { MonitoringStats, TrafficLevelStats, OptimizationEvent, Product, SchedulerSettings, SchedulerStatus } from '../types';
import { EXCHANGE_RATE_CURRENCIES } from '../types';

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
let exchangeRateRefreshTimer: ReturnType<typeof setInterval> | null = null;

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

// 监听产品变化
watch(() => props.selectedProduct, () => {
  loadDashboardData();
}, { immediate: true });

// 监听汇率设置变更事件
function handleExchangeRateSettingsChanged() {
  loadCurrencyPreference();
  // 重置轮播索引并重启轮播
  currentRateIndex.value = 0;
  startRateCarousel();
}

onMounted(() => {
  loadDashboardData();
  startCountdownTimer();
  // 加载市场时钟偏好并启动
  loadMarketClockPreference();
  startClock();
  // 启动汇率轮播
  startRateCarousel();
  // 监听汇率设置变更
  window.addEventListener('exchange-rate-settings-changed', handleExchangeRateSettingsChanged);
  // 检查节日提醒
  checkHolidayReminder();
});

onUnmounted(() => {
  if (countdownTimer) {
    clearInterval(countdownTimer);
    countdownTimer = null;
  }
  // 停止市场时钟
  stopClock();
  // 停止汇率轮播
  stopRateCarousel();
  // 停止汇率自动刷新
  stopExchangeRateRefreshTimer();
  // 移除事件监听
  window.removeEventListener('exchange-rate-settings-changed', handleExchangeRateSettingsChanged);
});

// 格式化时间
function formatDateTime(dateStr: string | null): string {
  if (!dateStr) return '-';
  // 支持多种格式：RFC3339 (含时区) 或数据库格式 (UTC)
  let normalizedStr = dateStr;
  // 如果是数据库格式 "YYYY-MM-DD HH:MM:SS"，转换为 ISO 格式
  if (dateStr.includes(' ') && !dateStr.includes('T')) {
    normalizedStr = dateStr.replace(' ', 'T') + 'Z';
  }
  const date = new Date(normalizedStr);
  if (isNaN(date.getTime())) return '-';
  return `${date.getMonth() + 1}/${date.getDate()} ${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}`;
}

// ==================== 市场时钟相关 ====================
const marketClocks = [
  { code: 'US', name: '美国', timezone: 'America/New_York', flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="20" fill="#B22234"/><rect y="1.54" width="30" height="1.54" fill="white"/><rect y="4.62" width="30" height="1.54" fill="white"/><rect y="7.69" width="30" height="1.54" fill="white"/><rect y="10.77" width="30" height="1.54" fill="white"/><rect y="13.85" width="30" height="1.54" fill="white"/><rect y="16.92" width="30" height="1.54" fill="white"/><rect width="12" height="10.77" fill="#3C3B6E"/></svg>` },
  { code: 'UK', name: '英国', timezone: 'Europe/London', flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="20" fill="#012169"/><path d="M0,0 L30,20 M30,0 L0,20" stroke="white" stroke-width="4"/><path d="M0,0 L30,20 M30,0 L0,20" stroke="#C8102E" stroke-width="2.5"/><path d="M15,0 V20 M0,10 H30" stroke="white" stroke-width="6"/><path d="M15,0 V20 M0,10 H30" stroke="#C8102E" stroke-width="3.5"/></svg>` },
  { code: 'DE', name: '德国', timezone: 'Europe/Berlin', flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="6.67" fill="#000"/><rect y="6.67" width="30" height="6.67" fill="#DD0000"/><rect y="13.33" width="30" height="6.67" fill="#FFCE00"/></svg>` },
  { code: 'FR', name: '法国', timezone: 'Europe/Paris', flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="10" height="20" fill="#002395"/><rect x="10" width="10" height="20" fill="white"/><rect x="20" width="10" height="20" fill="#ED2939"/></svg>` },
  { code: 'JP', name: '日本', timezone: 'Asia/Tokyo', flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="20" fill="white"/><circle cx="15" cy="10" r="6" fill="#BC002D"/></svg>` },
  { code: 'CN', name: '中国', timezone: 'Asia/Shanghai', flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="20" fill="#DE2910"/><g fill="#FFDE00"><polygon points="5,4 6,7 3,5 7,5 4,7"/></g></svg>` },
];

const currentMarketIndex = ref(0);
const currentTime = ref('');
let clockTimer: ReturnType<typeof setInterval> | null = null;

// 当前选中的市场
const currentMarket = computed(() => marketClocks[currentMarketIndex.value]);

// 更新时钟显示
function updateClock() {
  const market = currentMarket.value;
  const now = new Date();
  const timeStr = now.toLocaleTimeString('zh-CN', {
    timeZone: market.timezone,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  });
  currentTime.value = timeStr;
}

// 滚轮切换市场
function handleClockWheel(event: WheelEvent) {
  event.preventDefault();
  if (event.deltaY > 0) {
    // 向下滚动，下一个市场
    currentMarketIndex.value = (currentMarketIndex.value + 1) % marketClocks.length;
  } else {
    // 向上滚动，上一个市场
    currentMarketIndex.value = (currentMarketIndex.value - 1 + marketClocks.length) % marketClocks.length;
  }
  // 立即更新时钟
  updateClock();
  // 保存偏好
  localStorage.setItem('market_clock_index', currentMarketIndex.value.toString());
}

// 加载市场时钟偏好
function loadMarketClockPreference() {
  const saved = localStorage.getItem('market_clock_index');
  if (saved) {
    const index = parseInt(saved, 10);
    if (!isNaN(index) && index >= 0 && index < marketClocks.length) {
      currentMarketIndex.value = index;
    }
  }
}

// 启动时钟
function startClock() {
  updateClock();
  clockTimer = setInterval(updateClock, 1000);
}

// 停止时钟
function stopClock() {
  if (clockTimer) {
    clearInterval(clockTimer);
    clockTimer = null;
  }
}

// ==================== 电商日历相关 ====================
const showCalendarDialog = ref(false);
const calendarYear = ref(new Date().getFullYear());
const calendarMonth = ref(new Date().getMonth()); // 0-11

// 节日类型
type HolidayType = 'promo' | 'western' | 'chinese' | 'japan' | 'universal';

interface Holiday {
  name: string;
  type: HolidayType;
  markets?: string[]; // 适用市场
}

// 节日数据 - 格式: "MM-DD" 或 "MM-DD-YYYY" 用于特定年份
const holidayData: Record<string, Holiday[]> = {
  // ==================== 一月 ====================
  '01-01': [{ name: '元旦', type: 'universal' }, { name: '年货节开始', type: 'promo', markets: ['CN'] }],
  '01-02': [{ name: '日本新年假期', type: 'japan', markets: ['JP'] }],
  '01-03': [{ name: '日本新年假期', type: 'japan', markets: ['JP'] }],
  '01-06': [{ name: '主显节', type: 'western', markets: ['ES', 'IT'] }],
  '01-13': [{ name: '成人日 (日本)', type: 'japan', markets: ['JP'] }],
  '01-15': [{ name: 'MLK Day (美)', type: 'western', markets: ['US'] }],
  '01-26': [{ name: '澳大利亚国庆日', type: 'western', markets: ['AU'] }, { name: '共和国日 (印度)', type: 'western', markets: ['IN'] }],
  // ==================== 二月 ====================
  '02-01': [{ name: '年货节结束', type: 'promo', markets: ['CN'] }],
  '02-03': [{ name: '节分 (日本)', type: 'japan', markets: ['JP'] }],
  '02-11': [{ name: '建国纪念日 (日本)', type: 'japan', markets: ['JP'] }],
  '02-14': [{ name: '情人节', type: 'universal' }],
  '02-17': [{ name: '总统日 (美)', type: 'western', markets: ['US'] }],
  '02-23': [{ name: '天皇诞辰 (日本)', type: 'japan', markets: ['JP'] }],
  // ==================== 三月 ====================
  '03-03': [{ name: '女儿节 (日本)', type: 'japan', markets: ['JP'] }],
  '03-08': [{ name: '38女王节', type: 'promo', markets: ['CN'] }, { name: '妇女节', type: 'universal' }],
  '03-14': [{ name: '白色情人节', type: 'japan', markets: ['JP', 'KR'] }],
  '03-17': [{ name: '圣帕特里克节', type: 'western', markets: ['US', 'UK', 'IE'] }],
  '03-20': [{ name: '春分日 (日本)', type: 'japan', markets: ['JP'] }],
  '03-21': [{ name: '春季大促开始', type: 'promo' }],
  // ==================== 四月 ====================
  '04-01': [{ name: '愚人节', type: 'western' }],
  '04-22': [{ name: '地球日', type: 'universal' }],
  '04-23': [{ name: '圣乔治日 (英)', type: 'western', markets: ['UK'] }],
  '04-27': [{ name: '国王节 (荷兰)', type: 'western', markets: ['NL'] }],
  '04-29': [{ name: '昭和日 (日本)', type: 'japan', markets: ['JP'] }],
  // ==================== 五月 ====================
  '05-01': [{ name: '劳动节', type: 'universal' }],
  '05-03': [{ name: '宪法纪念日 (日本)', type: 'japan', markets: ['JP'] }],
  '05-04': [{ name: '绿之日 (日本)', type: 'japan', markets: ['JP'] }],
  '05-05': [{ name: '儿童节 (日本)', type: 'japan', markets: ['JP'] }, { name: '五五大促', type: 'promo', markets: ['CN'] }],
  '05-06': [{ name: '黄金周结束 (日本)', type: 'japan', markets: ['JP'] }],
  '05-09': [{ name: '欧洲日', type: 'western', markets: ['EU'] }],
  '05-26': [{ name: 'Memorial Day (美)', type: 'western', markets: ['US'] }],
  // ==================== 六月 ====================
  '06-01': [{ name: '儿童节', type: 'chinese', markets: ['CN'] }, { name: '618预热开始', type: 'promo', markets: ['CN'] }],
  '06-14': [{ name: '美国国旗日', type: 'western', markets: ['US'] }],
  '06-16': [{ name: '618预售开始', type: 'promo', markets: ['CN'] }],
  '06-18': [{ name: '618 大促', type: 'promo', markets: ['CN'] }],
  '06-20': [{ name: '618返场', type: 'promo', markets: ['CN'] }],
  '06-21': [{ name: '夏至', type: 'universal' }],
  // ==================== 七月 ====================
  '07-01': [{ name: '加拿大国庆日', type: 'western', markets: ['CA'] }, { name: '暑期大促开始', type: 'promo' }],
  '07-04': [{ name: '美国独立日', type: 'western', markets: ['US'] }],
  '07-07': [{ name: '七夕 (日本)', type: 'japan', markets: ['JP'] }],
  '07-14': [{ name: '法国国庆日', type: 'western', markets: ['FR'] }],
  '07-15': [{ name: 'Prime Day', type: 'promo', markets: ['US', 'UK', 'DE', 'JP', 'CA', 'FR', 'IT', 'ES'] }],
  '07-16': [{ name: 'Prime Day', type: 'promo', markets: ['US', 'UK', 'DE', 'JP', 'CA', 'FR', 'IT', 'ES'] }],
  '07-17': [{ name: '海之日 (日本)', type: 'japan', markets: ['JP'] }],
  // ==================== 八月 ====================
  '08-01': [{ name: '瑞士国庆日', type: 'western', markets: ['CH'] }],
  '08-11': [{ name: '山之日 (日本)', type: 'japan', markets: ['JP'] }],
  '08-13': [{ name: '盂兰盆节开始 (日本)', type: 'japan', markets: ['JP'] }],
  '08-15': [{ name: '盂兰盆节 (日本)', type: 'japan', markets: ['JP'] }, { name: '印度独立日', type: 'western', markets: ['IN'] }],
  '08-16': [{ name: '盂兰盆节结束 (日本)', type: 'japan', markets: ['JP'] }],
  // ==================== 九月 ====================
  '09-01': [{ name: '返校季', type: 'promo', markets: ['US', 'UK', 'DE'] }],
  '09-02': [{ name: 'Labor Day (美)', type: 'western', markets: ['US'] }],
  '09-09': [{ name: '99大促', type: 'promo', markets: ['CN'] }],
  '09-15': [{ name: '敬老日 (日本)', type: 'japan', markets: ['JP'] }],
  '09-22': [{ name: '秋分日 (日本)', type: 'japan', markets: ['JP'] }],
  // ==================== 十月 ====================
  '10-01': [{ name: '国庆节', type: 'chinese', markets: ['CN'] }, { name: '国庆大促', type: 'promo', markets: ['CN'] }],
  '10-03': [{ name: '德国统一日', type: 'western', markets: ['DE'] }],
  '10-08': [{ name: '秋季Prime Day', type: 'promo', markets: ['US', 'UK', 'DE', 'JP'] }],
  '10-09': [{ name: '秋季Prime Day', type: 'promo', markets: ['US', 'UK', 'DE', 'JP'] }, { name: '感恩节 (加)', type: 'western', markets: ['CA'] }],
  '10-12': [{ name: '哥伦布日 (美)', type: 'western', markets: ['US'] }, { name: '西班牙国庆日', type: 'western', markets: ['ES'] }],
  '10-14': [{ name: '体育日 (日本)', type: 'japan', markets: ['JP'] }],
  '10-20': [{ name: '双十一预热开始', type: 'promo', markets: ['CN'] }],
  '10-31': [{ name: '万圣节', type: 'western', markets: ['US', 'UK', 'CA', 'AU'] }],
  // ==================== 十一月 ====================
  '11-01': [{ name: '双十一预售', type: 'promo', markets: ['CN'] }, { name: '万圣节翌日', type: 'western', markets: ['MX'] }],
  '11-03': [{ name: '文化日 (日本)', type: 'japan', markets: ['JP'] }],
  '11-11': [{ name: '双十一', type: 'promo', markets: ['CN'] }, { name: '光棍节', type: 'chinese', markets: ['CN'] }, { name: '退伍军人节 (美)', type: 'western', markets: ['US'] }],
  '11-15': [{ name: '七五三 (日本)', type: 'japan', markets: ['JP'] }],
  '11-23': [{ name: '勤劳感谢日 (日本)', type: 'japan', markets: ['JP'] }],
  // ==================== 十二月 ====================
  '12-02': [{ name: '捐赠星期二', type: 'promo', markets: ['US'] }],
  '12-05': [{ name: '圣诞购物季开始', type: 'promo' }],
  '12-12': [{ name: '双十二', type: 'promo', markets: ['CN'] }, { name: '绿色星期一', type: 'promo', markets: ['US'] }],
  '12-14': [{ name: '免运日 (美)', type: 'promo', markets: ['US'] }],
  '12-21': [{ name: '冬至', type: 'universal' }, { name: '超级星期六', type: 'promo', markets: ['US'] }],
  '12-24': [{ name: '平安夜', type: 'western' }],
  '12-25': [{ name: '圣诞节', type: 'western' }],
  '12-26': [{ name: 'Boxing Day', type: 'western', markets: ['UK', 'CA', 'AU'] }, { name: '节后大促开始', type: 'promo' }],
  '12-31': [{ name: '除夕', type: 'universal' }, { name: '年末清仓', type: 'promo' }],
};

// 特定年份节日（农历节日、动态节日等）
const yearlyHolidays: Record<number, Record<string, Holiday[]>> = {
  2025: {
    '01-29': [{ name: '春节', type: 'chinese', markets: ['CN'] }],
    '02-12': [{ name: '元宵节', type: 'chinese', markets: ['CN'] }],
    '04-04': [{ name: '清明节', type: 'chinese', markets: ['CN'] }],
    '04-20': [{ name: '复活节', type: 'western' }],
    '05-05': [{ name: '端午节', type: 'chinese', markets: ['CN'] }],
    '05-11': [{ name: '母亲节', type: 'universal' }],
    '06-15': [{ name: '父亲节', type: 'universal' }],
    '08-10': [{ name: '七夕节', type: 'chinese', markets: ['CN'] }],
    '10-06': [{ name: '中秋节', type: 'chinese', markets: ['CN'] }],
    '10-29': [{ name: '重阳节', type: 'chinese', markets: ['CN'] }],
    '11-27': [{ name: '感恩节 (美)', type: 'western', markets: ['US'] }],
    '11-28': [{ name: '黑色星期五', type: 'promo' }],
    '11-29': [{ name: '小企业星期六', type: 'promo', markets: ['US'] }],
    '12-01': [{ name: '网络星期一', type: 'promo' }],
  },
  2026: {
    '02-17': [{ name: '春节', type: 'chinese', markets: ['CN'] }],
    '03-03': [{ name: '元宵节', type: 'chinese', markets: ['CN'] }],
    '04-05': [{ name: '清明节', type: 'chinese', markets: ['CN'] }, { name: '复活节', type: 'western' }],
    '05-10': [{ name: '母亲节', type: 'universal' }],
    '05-31': [{ name: '端午节', type: 'chinese', markets: ['CN'] }],
    '06-21': [{ name: '父亲节', type: 'universal' }],
    '08-19': [{ name: '七夕节', type: 'chinese', markets: ['CN'] }],
    '10-06': [{ name: '中秋节', type: 'chinese', markets: ['CN'] }],
    '10-18': [{ name: '重阳节', type: 'chinese', markets: ['CN'] }],
    '11-26': [{ name: '感恩节 (美)', type: 'western', markets: ['US'] }],
    '11-27': [{ name: '黑色星期五', type: 'promo' }],
    '11-28': [{ name: '小企业星期六', type: 'promo', markets: ['US'] }],
    '11-30': [{ name: '网络星期一', type: 'promo' }],
  },
  2027: {
    '02-06': [{ name: '春节', type: 'chinese', markets: ['CN'] }],
    '02-20': [{ name: '元宵节', type: 'chinese', markets: ['CN'] }],
    '03-28': [{ name: '复活节', type: 'western' }],
    '04-05': [{ name: '清明节', type: 'chinese', markets: ['CN'] }],
    '05-09': [{ name: '母亲节', type: 'universal' }],
    '06-19': [{ name: '端午节', type: 'chinese', markets: ['CN'] }],
    '06-20': [{ name: '父亲节', type: 'universal' }],
    '08-08': [{ name: '七夕节', type: 'chinese', markets: ['CN'] }],
    '09-25': [{ name: '中秋节', type: 'chinese', markets: ['CN'] }],
    '10-07': [{ name: '重阳节', type: 'chinese', markets: ['CN'] }],
    '11-25': [{ name: '感恩节 (美)', type: 'western', markets: ['US'] }],
    '11-26': [{ name: '黑色星期五', type: 'promo' }],
    '11-27': [{ name: '小企业星期六', type: 'promo', markets: ['US'] }],
    '11-29': [{ name: '网络星期一', type: 'promo' }],
  },
  2028: {
    '01-26': [{ name: '春节', type: 'chinese', markets: ['CN'] }],
    '02-09': [{ name: '元宵节', type: 'chinese', markets: ['CN'] }],
    '04-04': [{ name: '清明节', type: 'chinese', markets: ['CN'] }],
    '04-16': [{ name: '复活节', type: 'western' }],
    '05-14': [{ name: '母亲节', type: 'universal' }],
    '06-06': [{ name: '端午节', type: 'chinese', markets: ['CN'] }],
    '06-18': [{ name: '父亲节', type: 'universal' }],
    '08-26': [{ name: '七夕节', type: 'chinese', markets: ['CN'] }],
    '10-13': [{ name: '中秋节', type: 'chinese', markets: ['CN'] }],
    '10-25': [{ name: '重阳节', type: 'chinese', markets: ['CN'] }],
    '11-23': [{ name: '感恩节 (美)', type: 'western', markets: ['US'] }],
    '11-24': [{ name: '黑色星期五', type: 'promo' }],
    '11-25': [{ name: '小企业星期六', type: 'promo', markets: ['US'] }],
    '11-27': [{ name: '网络星期一', type: 'promo' }],
  },
};

// 获取某月的天数
function getDaysInMonth(year: number, month: number): number {
  return new Date(year, month + 1, 0).getDate();
}

// 获取某月第一天是星期几 (0=周日)
function getFirstDayOfMonth(year: number, month: number): number {
  return new Date(year, month, 1).getDay();
}

// 获取某天的节日
function getHolidaysForDate(year: number, month: number, day: number): Holiday[] {
  const mmdd = `${String(month + 1).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
  const holidays: Holiday[] = [];

  // 查找固定节日
  if (holidayData[mmdd]) {
    holidays.push(...holidayData[mmdd]);
  }

  // 查找特定年份节日（农历节日、动态节日等）
  const yearHolidays = yearlyHolidays[year];
  if (yearHolidays && yearHolidays[mmdd]) {
    holidays.push(...yearHolidays[mmdd]);
  }

  return holidays;
}

// 生成日历数据
const calendarDays = computed(() => {
  const year = calendarYear.value;
  const month = calendarMonth.value;
  const daysInMonth = getDaysInMonth(year, month);
  const firstDay = getFirstDayOfMonth(year, month);

  const days: Array<{ day: number; holidays: Holiday[]; isToday: boolean } | null> = [];

  // 填充月初空白
  for (let i = 0; i < firstDay; i++) {
    days.push(null);
  }

  // 填充日期
  const today = new Date();
  for (let d = 1; d <= daysInMonth; d++) {
    const isToday = year === today.getFullYear() && month === today.getMonth() && d === today.getDate();
    days.push({
      day: d,
      holidays: getHolidaysForDate(year, month, d),
      isToday,
    });
  }

  return days;
});

// 月份名称
const monthNames = ['一月', '二月', '三月', '四月', '五月', '六月', '七月', '八月', '九月', '十月', '十一月', '十二月'];

// 切换月份
function prevMonth() {
  if (calendarMonth.value === 0) {
    calendarMonth.value = 11;
    calendarYear.value--;
  } else {
    calendarMonth.value--;
  }
}

function nextMonth() {
  if (calendarMonth.value === 11) {
    calendarMonth.value = 0;
    calendarYear.value++;
  } else {
    calendarMonth.value++;
  }
}

// 回到今天
function goToToday() {
  const today = new Date();
  calendarYear.value = today.getFullYear();
  calendarMonth.value = today.getMonth();
}

// 节日类型颜色
function getHolidayTypeColor(type: HolidayType): string {
  switch (type) {
    case 'promo': return '#f56c6c';
    case 'western': return '#409eff';
    case 'chinese': return '#e6a23c';
    case 'japan': return '#f472b6';
    case 'universal': return '#67c23a';
    default: return '#909399';
  }
}

// 节日类型标签
function getHolidayTypeLabel(type: HolidayType): string {
  switch (type) {
    case 'promo': return '🛒 大促';
    case 'western': return '🎄 西方';
    case 'chinese': return '🏮 中国';
    case 'japan': return '🎌 日本';
    case 'universal': return '🌍 通用';
    default: return '';
  }
}

// ==================== 节日提醒相关 ====================
interface UpcomingHoliday {
  name: string;
  type: HolidayType;
  date: Date;
  daysLeft: number;
  markets?: string[];
}

const showHolidayReminder = ref(false);
const upcomingHolidays = ref<UpcomingHoliday[]>([]);
const currentHolidayIndex = ref(0);
const holidayAiSuggestions = ref<string[]>([]);
const holidayAiLoading = ref(false);
const selectedSuggestions = ref<number[]>([]);
const addingToNotes = ref(false);
// 缓存每个节日的 AI 建议，key 为节日名称
const holidaySuggestionsCache = ref<Map<string, string[]>>(new Map());

// 通用建议模板（AI 失败时兜底）
const fallbackSuggestions: Record<HolidayType, string[]> = {
  promo: ['检查热销品库存，确保备货充足', '设置促销折扣和优惠券', '提前调整广告预算', '优化产品详情页和 A+ 内容'],
  western: ['准备节日主题包装和营销素材', '推出礼品组合和套装', '提前备货应对物流高峰', '更新产品图片增加节日元素'],
  chinese: ['关注海外华人市场需求', '调整发货时效预期', '准备节日元素营销内容', '检查供应链和库存情况'],
  japan: ['设置日本站专属促销', '准备本地化营销内容', '关注日本物流时效', '了解当地节日消费习惯'],
  universal: ['检查各站点库存情况', '关注市场动态和竞品', '准备促销活动方案', '优化广告投放策略'],
};

// 获取当前显示的节日
const currentHoliday = computed(() => {
  if (upcomingHolidays.value.length === 0) return null;
  return upcomingHolidays.value[currentHolidayIndex.value];
});

// 查找 14 天内的所有节日
function findUpcomingHolidays(): UpcomingHoliday[] {
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const result: UpcomingHoliday[] = [];
  const dismissedHolidays = JSON.parse(localStorage.getItem('holiday_reminder_dismissed') || '[]');

  // 检查未来 14 天
  for (let i = 1; i <= 14; i++) {
    const checkDate = new Date(today);
    checkDate.setDate(today.getDate() + i);

    const holidays = getHolidaysForDate(
      checkDate.getFullYear(),
      checkDate.getMonth(),
      checkDate.getDate()
    );

    for (const h of holidays) {
      // 跳过用户已屏蔽的节日
      if (dismissedHolidays.includes(h.name)) continue;

      result.push({
        name: h.name,
        type: h.type,
        date: checkDate,
        daysLeft: i,
        markets: h.markets,
      });
    }
  }

  // 按天数排序
  result.sort((a, b) => a.daysLeft - b.daysLeft);
  return result;
}

// 生成 AI 建议（带缓存）
async function generateHolidaySuggestions(holiday: UpcomingHoliday) {
  // 检查缓存，如果已有建议则直接使用
  const cached = holidaySuggestionsCache.value.get(holiday.name);
  if (cached) {
    holidayAiSuggestions.value = cached;
    return;
  }

  holidayAiLoading.value = true;
  holidayAiSuggestions.value = [];

  // 检查 API Key 并确定可用的 provider
  let provider: 'deepseek' | 'openai' | 'qwen' | null = null;
  if (await checkApiKeyConfigured('deepseek')) {
    provider = 'deepseek';
  } else if (await checkApiKeyConfigured('qwen')) {
    provider = 'qwen';
  } else if (await checkApiKeyConfigured('openai')) {
    provider = 'openai';
  }

  if (!provider) {
    const suggestions = fallbackSuggestions[holiday.type] || fallbackSuggestions.universal;
    holidayAiSuggestions.value = suggestions;
    holidaySuggestionsCache.value.set(holiday.name, suggestions);
    holidayAiLoading.value = false;
    return;
  }

  const prompt = `你是电商运营专家。${holiday.name}还有${holiday.daysLeft}天到来。
节日类型：${getHolidayTypeLabel(holiday.type)}
${holiday.markets ? `相关市场：${holiday.markets.join(', ')}` : ''}

请给出 4-5 条简洁的准备建议，帮助亚马逊卖家为这个节日做好准备。
要求：每条建议不超过 20 字，直接输出建议列表，每条一行，不要编号。`;

  try {
    let fullResponse = '';
    for await (const chunk of chatStream(
      [{ role: 'user', content: prompt }],
      { provider, maxTokens: 300 }
    )) {
      fullResponse += chunk.content;
    }

    // 解析建议
    const suggestions = fullResponse
      .split('\n')
      .map(s => s.trim())
      .filter(s => s.length > 0 && s.length <= 50);

    if (suggestions.length > 0) {
      holidayAiSuggestions.value = suggestions.slice(0, 5);
      holidaySuggestionsCache.value.set(holiday.name, suggestions.slice(0, 5));
    } else {
      const fallback = fallbackSuggestions[holiday.type] || fallbackSuggestions.universal;
      holidayAiSuggestions.value = fallback;
      holidaySuggestionsCache.value.set(holiday.name, fallback);
    }
  } catch (e) {
    console.error('生成节日建议失败:', e);
    const fallback = fallbackSuggestions[holiday.type] || fallbackSuggestions.universal;
    holidayAiSuggestions.value = fallback;
    holidaySuggestionsCache.value.set(holiday.name, fallback);
  } finally {
    holidayAiLoading.value = false;
  }
}

// 切换到上一个节日
function prevHoliday() {
  if (upcomingHolidays.value.length <= 1) return;
  currentHolidayIndex.value = (currentHolidayIndex.value - 1 + upcomingHolidays.value.length) % upcomingHolidays.value.length;
  if (currentHoliday.value) {
    generateHolidaySuggestions(currentHoliday.value);
  }
}

// 切换到下一个节日
function nextHoliday() {
  if (upcomingHolidays.value.length <= 1) return;
  currentHolidayIndex.value = (currentHolidayIndex.value + 1) % upcomingHolidays.value.length;
  if (currentHoliday.value) {
    generateHolidaySuggestions(currentHoliday.value);
  }
}

// 关闭提醒（记录今日已提醒）
function closeHolidayReminder() {
  showHolidayReminder.value = false;
  selectedSuggestions.value = [];
  holidaySuggestionsCache.value.clear();
  localStorage.setItem('holiday_reminder_last_date', new Date().toISOString().split('T')[0]);
}

// 切换建议选中状态
function toggleSuggestion(idx: number) {
  const index = selectedSuggestions.value.indexOf(idx);
  if (index > -1) {
    selectedSuggestions.value.splice(index, 1);
  } else {
    selectedSuggestions.value.push(idx);
  }
}

// 将选中的建议加入备忘录
async function addSuggestionsToNotes() {
  if (selectedSuggestions.value.length === 0 || !currentHoliday.value) return;

  addingToNotes.value = true;
  try {
    // 按索引排序，保持原顺序
    const sortedIndices = [...selectedSuggestions.value].sort((a, b) => a - b);

    for (const idx of sortedIndices) {
      const suggestion = holidayAiSuggestions.value[idx];
      if (suggestion) {
        // 添加节日标签前缀
        const content = `【${currentHoliday.value.name}】${suggestion}`;
        await api.addQuickNote(content);
      }
    }

    ElMessage.success(`已添加 ${selectedSuggestions.value.length} 条建议到备忘录`);
    selectedSuggestions.value = [];
  } catch (e) {
    console.error('添加备忘录失败:', e);
    ElMessage.error('添加失败，请重试');
  } finally {
    addingToNotes.value = false;
  }
}

// 本节日不再提醒
function dismissCurrentHoliday() {
  if (!currentHoliday.value) return;

  const dismissed = JSON.parse(localStorage.getItem('holiday_reminder_dismissed') || '[]');
  if (!dismissed.includes(currentHoliday.value.name)) {
    dismissed.push(currentHoliday.value.name);
    localStorage.setItem('holiday_reminder_dismissed', JSON.stringify(dismissed));
  }

  // 如果还有其他节日，切换到下一个
  upcomingHolidays.value = upcomingHolidays.value.filter(h => h.name !== currentHoliday.value?.name);
  if (upcomingHolidays.value.length > 0) {
    currentHolidayIndex.value = 0;
    generateHolidaySuggestions(upcomingHolidays.value[0]);
  } else {
    closeHolidayReminder();
  }
}

// 检查并显示节日提醒
async function checkHolidayReminder() {
  // 检查今天是否已提醒
  const lastDate = localStorage.getItem('holiday_reminder_last_date');
  const today = new Date().toISOString().split('T')[0];
  if (lastDate === today) return;

  // 查找即将到来的节日
  const holidays = findUpcomingHolidays();
  if (holidays.length === 0) return;

  upcomingHolidays.value = holidays;
  currentHolidayIndex.value = 0;
  showHolidayReminder.value = true;

  // 生成第一个节日的建议
  await generateHolidaySuggestions(holidays[0]);
}

// ==================== 汇率相关 ====================
const exchangeRates = ref<Map<string, number>>(new Map());
const previousExchangeRates = ref<Map<string, number>>(new Map()); // 上次汇率用于比较涨跌
const exchangeRatesLoading = ref(false);
const exchangeRatesUpdatedAt = ref<string | null>(null);

// 用户选择的显示货币（默认前3个）
const selectedCurrencies = ref<string[]>(['USD', 'EUR', 'GBP']);

// 获取汇率涨跌方向: 1=上涨, -1=下跌, 0=持平, null=无数据
function getRateDirection(currency: string): number | null {
  const current = exchangeRates.value.get(currency);
  const previous = previousExchangeRates.value.get(currency);
  if (!current || !previous) return null;
  if (current > previous) return 1;
  if (current < previous) return -1;
  return 0;
}

// 汇率轮播相关
const currentRateIndex = ref(0);
const rateSlideDirection = ref<'up' | 'down'>('up'); // 滑动方向
let rateCarouselTimer: ReturnType<typeof setInterval> | null = null;
const RATE_CAROUSEL_INTERVAL = 3000; // 3秒切换一次
const isRateHovered = ref(false); // 鼠标是否悬停在汇率区域

// 获取当前显示的货币
const currentDisplayCurrency = computed(() => {
  const currencies = displayCurrencies.value;
  if (currencies.length === 0) return null;
  return currencies[currentRateIndex.value % currencies.length];
});

// 开始汇率自动轮播
function startRateCarousel() {
  stopRateCarousel();
  if (displayCurrencies.value.length > 1 && !isRateHovered.value) {
    rateCarouselTimer = setInterval(() => {
      rateSlideDirection.value = 'up'; // 自动轮播向上滑动
      currentRateIndex.value = (currentRateIndex.value + 1) % displayCurrencies.value.length;
    }, RATE_CAROUSEL_INTERVAL);
  }
}

// 停止汇率自动轮播
function stopRateCarousel() {
  if (rateCarouselTimer) {
    clearInterval(rateCarouselTimer);
    rateCarouselTimer = null;
  }
}

// 鼠标悬停汇率区域时暂停轮播
function onRateMouseEnter() {
  isRateHovered.value = true;
  stopRateCarousel();
}

// 鼠标离开汇率区域时恢复轮播
function onRateMouseLeave() {
  isRateHovered.value = false;
  startRateCarousel();
}

// 汇率滚轮切换
function handleRateWheel(event: WheelEvent) {
  event.preventDefault();
  const len = displayCurrencies.value.length;
  if (len <= 1) return;

  // 用户交互时暂停自动轮播
  stopRateCarousel();

  if (event.deltaY > 0) {
    rateSlideDirection.value = 'up'; // 向下滚动 = 下一个 = 向上滑出
    currentRateIndex.value = (currentRateIndex.value + 1) % len;
  } else {
    rateSlideDirection.value = 'down'; // 向上滚动 = 上一个 = 向下滑出
    currentRateIndex.value = (currentRateIndex.value - 1 + len) % len;
  }

  // 3秒后恢复自动轮播
  setTimeout(() => {
    startRateCarousel();
  }, 3000);
}

// 加载用户汇率偏好
function loadCurrencyPreference() {
  try {
    const saved = localStorage.getItem('exchange_rate_currencies');
    if (saved) {
      const parsed = JSON.parse(saved);
      if (Array.isArray(parsed) && parsed.length > 0 && parsed.length <= 5) {
        selectedCurrencies.value = parsed;
      }
    }
  } catch (e) {
    console.error('加载汇率偏好失败:', e);
  }
}

// 获取选中的货币配置
const displayCurrencies = computed(() => {
  return EXCHANGE_RATE_CURRENCIES.filter(c => selectedCurrencies.value.includes(c.code));
});

// 从 localStorage 加载上次汇率（用于比较涨跌）
function loadPreviousRatesFromStorage() {
  try {
    const saved = localStorage.getItem('previous_exchange_rates');
    if (saved) {
      const parsed = JSON.parse(saved) as [string, number][];
      previousExchangeRates.value = new Map(parsed);
    }
  } catch (e) {
    console.error('加载上次汇率失败:', e);
  }
}

// 将当前汇率保存到 localStorage（作为下次比较的基准）
function saveCurrentRatesToStorage() {
  try {
    if (exchangeRates.value.size > 0) {
      const data = Array.from(exchangeRates.value.entries());
      localStorage.setItem('previous_exchange_rates', JSON.stringify(data));
    }
  } catch (e) {
    console.error('保存汇率失败:', e);
  }
}

// 加载缓存的汇率
async function loadCachedRates() {
  try {
    // 先从 localStorage 加载上次汇率
    loadPreviousRatesFromStorage();

    const cached = await api.getExchangeRates();
    if (cached.length > 0) {
      cached.forEach(item => {
        exchangeRates.value.set(item.currency, item.rate);
      });
      exchangeRatesUpdatedAt.value = cached[0]?.updated_at || null;

      // 如果没有上次汇率数据，用当前汇率初始化（首次使用）
      if (previousExchangeRates.value.size === 0) {
        previousExchangeRates.value = new Map(exchangeRates.value);
        saveCurrentRatesToStorage();
      }
    }
  } catch (e) {
    console.error('加载缓存汇率失败:', e);
  }
}

// 从网络获取最新汇率（通过后端代理）
async function fetchExchangeRates() {
  exchangeRatesLoading.value = true;
  try {
    // 获取需要的货币代码列表
    const currencies = EXCHANGE_RATE_CURRENCIES.map(c => c.code);

    // 在获取新汇率前，将当前汇率保存为"上次汇率"
    if (exchangeRates.value.size > 0) {
      previousExchangeRates.value = new Map(exchangeRates.value);
      saveCurrentRatesToStorage();
    }

    // 调用后端 API 获取汇率
    const result = await api.fetchExchangeRates(currencies);

    // 更新本地状态
    if (result && result.length > 0) {
      result.forEach(item => {
        exchangeRates.value.set(item.currency, item.rate);
      });
      exchangeRatesUpdatedAt.value = result[0]?.updated_at || new Date().toISOString();
    }
  } catch (e) {
    console.error('获取汇率失败:', e);
  } finally {
    exchangeRatesLoading.value = false;
  }
}

// 格式化汇率显示
function formatRate(currency: string): string {
  const rate = exchangeRates.value.get(currency);
  if (!rate) return '-';

  const config = EXCHANGE_RATE_CURRENCIES.find(c => c.code === currency);
  const multiplier = config?.multiplier || 1;

  if (multiplier > 1) {
    return (rate * multiplier).toFixed(2);
  }
  return rate.toFixed(2);
}

// 启动汇率自动刷新定时器（每小时刷新一次）
function startExchangeRateRefreshTimer() {
  // 清除已存在的定时器
  if (exchangeRateRefreshTimer) {
    clearInterval(exchangeRateRefreshTimer);
  }
  // 每小时刷新一次汇率
  exchangeRateRefreshTimer = setInterval(() => {
    console.log('自动刷新汇率...');
    fetchExchangeRates();
  }, 60 * 60 * 1000); // 1小时 = 3600000毫秒
}

// 停止汇率自动刷新定时器
function stopExchangeRateRefreshTimer() {
  if (exchangeRateRefreshTimer) {
    clearInterval(exchangeRateRefreshTimer);
    exchangeRateRefreshTimer = null;
  }
}

// ==================== 汇率趋势图 ====================

import type { ExchangeRateHistory } from '../types';

const rateHistoryLoading = ref(false);
const rateHistoryData = ref<ExchangeRateHistory[]>([]);
const rateHistoryCache = ref<Map<string, ExchangeRateHistory[]>>(new Map());

// 加载汇率历史数据
async function loadRateHistory(currency: string) {
  // 检查缓存
  if (rateHistoryCache.value.has(currency)) {
    rateHistoryData.value = rateHistoryCache.value.get(currency) || [];
    return;
  }

  rateHistoryLoading.value = true;
  try {
    const history = await api.getExchangeRateHistory(currency, 30);
    rateHistoryData.value = history;
    // 缓存数据
    rateHistoryCache.value.set(currency, history);
  } catch (e) {
    console.error('加载汇率历史失败:', e);
    rateHistoryData.value = [];
  } finally {
    rateHistoryLoading.value = false;
  }
}

// 计算历史数据最高最低值
const rateHistoryMax = computed(() => {
  if (rateHistoryData.value.length === 0) return 0;
  return Math.max(...rateHistoryData.value.map(d => d.rate));
});

const rateHistoryMin = computed(() => {
  if (rateHistoryData.value.length === 0) return 0;
  return Math.min(...rateHistoryData.value.map(d => d.rate));
});

// 趋势图配置
const rateChartOption = computed(() => {
  if (rateHistoryData.value.length === 0) return {};

  const dates = rateHistoryData.value.map(d => d.date.slice(5)); // 只显示 MM-DD
  const rates = rateHistoryData.value.map(d => d.rate);
  const maxRate = rateHistoryMax.value;
  const minRate = rateHistoryMin.value;
  const maxIdx = rates.indexOf(maxRate);
  const minIdx = rates.indexOf(minRate);

  return {
    grid: {
      left: 45,
      right: 15,
      top: 20,
      bottom: 25,
    },
    tooltip: {
      trigger: 'axis',
      formatter: (params: any) => {
        const data = params[0];
        return `${data.axisValue}<br/>汇率: ${data.value.toFixed(4)}`;
      }
    },
    xAxis: {
      type: 'category',
      data: dates,
      axisLabel: {
        fontSize: 10,
        interval: Math.floor(dates.length / 5) - 1,
      },
      axisLine: { lineStyle: { color: '#ddd' } },
    },
    yAxis: {
      type: 'value',
      scale: true,
      axisLabel: {
        fontSize: 10,
        formatter: (v: number) => v.toFixed(2),
      },
      splitLine: { lineStyle: { color: '#f0f0f0' } },
    },
    series: [{
      type: 'line',
      data: rates,
      smooth: true,
      symbol: 'circle',
      symbolSize: 4,
      lineStyle: { width: 2, color: '#409eff' },
      areaStyle: {
        color: {
          type: 'linear',
          x: 0, y: 0, x2: 0, y2: 1,
          colorStops: [
            { offset: 0, color: 'rgba(64,158,255,0.3)' },
            { offset: 1, color: 'rgba(64,158,255,0.05)' }
          ]
        }
      },
      markPoint: {
        symbol: 'circle',
        symbolSize: 8,
        data: [
          {
            name: '最高',
            coord: [maxIdx, maxRate],
            itemStyle: { color: '#f56c6c' },
          },
          {
            name: '最低',
            coord: [minIdx, minRate],
            itemStyle: { color: '#67c23a' },
          }
        ]
      }
    }]
  };
});

// 初始化时加载用户汇率偏好
loadCurrencyPreference();

// 初始化时加载汇率
loadCachedRates().then(() => {
  // 如果缓存为空或超过1小时，自动刷新
  if (exchangeRates.value.size === 0) {
    fetchExchangeRates();
  } else if (exchangeRatesUpdatedAt.value) {
    const lastUpdate = new Date(exchangeRatesUpdatedAt.value);
    const hourAgo = new Date(Date.now() - 60 * 60 * 1000);
    if (lastUpdate < hourAgo) {
      fetchExchangeRates();
    }
  }
  // 启动每小时自动刷新定时器
  startExchangeRateRefreshTimer();
});
</script>

<template>
  <div class="dashboard-container" v-loading="loading">
    <!-- 大屏视图 -->
    <BigScreenView
      v-if="viewMode === 'bigscreen'"
      :selectedProduct="selectedProduct"
      :stats="stats"
      :monitoringStats="monitoringStats"
      :trafficStats="trafficStats"
      :kbStats="kbStats"
      :recentEvents="recentEvents"
      :topRisers="topRisers"
      :topFallers="topFallers"
      :schedulerSettings="schedulerSettings"
      :schedulerStatus="schedulerStatus"
      :isInWindow="isInWindow"
      :countdownHours="countdownHours"
      :countdownMinutes="countdownMinutes"
      :countdownSeconds="countdownSeconds"
      @exit="viewMode = 'normal'"
      @refresh="loadDashboardData"
    />

    <!-- 未选择产品时的提示 -->
    <div v-if="!selectedProduct" class="no-product-state">
      <el-empty description="请先选择或创建一个产品">
        <el-button type="primary" @click="emit('switchView', 'keywords')">
          进入关键词管理
        </el-button>
      </el-empty>
    </div>

    <!-- 有产品时的内容 -->
    <div v-else class="dashboard-content-new">
      <!-- 顶部头部 -->
      <div class="dashboard-header">
        <div class="header-left">
          <h2>数据概览</h2>
          <span class="product-badge" v-if="selectedProduct">{{ selectedProduct.name }}</span>
        </div>
        <div class="header-right">
          <!-- 电商日历 -->
          <el-button
            class="calendar-btn"
            :icon="Calendar"
            @click="showCalendarDialog = true"
            title="电商日历"
          >
            日历
          </el-button>
          <span class="header-divider"></span>
          <!-- 市场时钟 -->
          <div
            class="market-clock"
            @wheel="handleClockWheel"
            :title="`${currentMarket.name}时间 (滚轮切换市场)`"
          >
            <span class="clock-flag" v-html="currentMarket.flag"></span>
            <span class="clock-time">{{ currentTime }}</span>
          </div>
          <span class="header-divider"></span>
          <!-- 汇率显示（轮播）带趋势图 -->
          <div
            class="exchange-rates-wrapper"
            v-if="currentDisplayCurrency"
            @mouseenter="onRateMouseEnter"
            @mouseleave="onRateMouseLeave"
          >
            <el-popover
              placement="bottom"
              :width="320"
              trigger="hover"
              :show-after="300"
              @before-enter="loadRateHistory(currentDisplayCurrency.code)"
            >
              <template #reference>
                <div
                  class="exchange-rates"
                  @wheel="handleRateWheel"
                  :title="`1 ${currentDisplayCurrency.code} = ${formatRate(currentDisplayCurrency.code)} CNY${currentDisplayCurrency.multiplier ? ` (×${currentDisplayCurrency.multiplier})` : ''} (滚轮切换货币)`"
                >
                  <transition :name="rateSlideDirection === 'up' ? 'rate-slide-up' : 'rate-slide-down'" mode="out-in">
                    <span class="rate-item" :key="currentDisplayCurrency.code">
                      <span class="rate-flag" v-html="currentDisplayCurrency.flag"></span>
                      <span class="rate-code">{{ currentDisplayCurrency.code }}</span>
                      <span class="rate-value">{{ formatRate(currentDisplayCurrency.code) }}</span>
                      <span
                        v-if="getRateDirection(currentDisplayCurrency.code) !== null"
                        class="rate-direction"
                        :class="{
                          up: getRateDirection(currentDisplayCurrency.code) === 1,
                          down: getRateDirection(currentDisplayCurrency.code) === -1,
                          equal: getRateDirection(currentDisplayCurrency.code) === 0
                        }"
                      >
                        {{ getRateDirection(currentDisplayCurrency.code) === 1 ? '↑' : getRateDirection(currentDisplayCurrency.code) === -1 ? '↓' : '—' }}
                      </span>
                    </span>
                  </transition>
                </div>
              </template>
              <!-- 趋势图内容 -->
              <div class="rate-trend-popover">
                <div class="trend-header">
                  <span class="trend-title">{{ currentDisplayCurrency.code }}/CNY 汇率趋势 (30天)</span>
                </div>
                <div v-if="rateHistoryLoading" class="trend-loading">
                  <el-icon class="is-loading"><Loading /></el-icon>
                  <span>加载中...</span>
                </div>
                <div v-else-if="rateHistoryData.length < 2" class="trend-no-data">
                  <span>暂无历史数据，需要持续记录几天后才能显示趋势</span>
                </div>
                <div v-else class="trend-chart-container">
                  <v-chart class="trend-chart" :option="rateChartOption" autoresize />
                  <div class="trend-stats">
                    <span class="stat-item">
                      <span class="label">最高:</span>
                      <span class="value high">{{ rateHistoryMax.toFixed(4) }}</span>
                    </span>
                    <span class="stat-item">
                      <span class="label">最低:</span>
                      <span class="value low">{{ rateHistoryMin.toFixed(4) }}</span>
                    </span>
                    <span class="stat-item">
                      <span class="label">当前:</span>
                      <span class="value">{{ formatRate(currentDisplayCurrency.code) }}</span>
                    </span>
                  </div>
                </div>
              </div>
            </el-popover>
          </div>
          <!-- 大屏模式切换 -->
          <el-button
            class="bigscreen-btn"
            type="primary"
            :icon="FullScreen"
            @click="switchToBigScreen"
          >
            智慧大屏
          </el-button>
        </div>
      </div>

      <!-- 第一排：关键指标卡片 -->
      <div class="stats-grid">
        <div class="modern-card stat-card hover-effect" @click="emit('switchView', 'keywords')">
          <div class="stat-top">
            <div class="icon-circle bg-blue-light">
              <el-icon class="text-blue"><Document /></el-icon>
            </div>
            <span class="stat-title">总关键词</span>
          </div>
          <div class="stat-main">
            <span class="stat-number">{{ stats.keywordCount.toLocaleString() }}</span>
          </div>
        </div>

        <div class="modern-card stat-card hover-effect" @click="emit('switchView', 'roots')">
          <div class="stat-top">
            <div class="icon-circle bg-purple-light">
              <el-icon class="text-purple"><TrendCharts /></el-icon>
            </div>
            <span class="stat-title">词根数量</span>
          </div>
          <div class="stat-main">
            <span class="stat-number">{{ stats.rootCount.toLocaleString() }}</span>
          </div>
        </div>

        <div class="modern-card stat-card hover-effect" @click="emit('switchView', 'monitoring')">
          <div class="stat-top">
            <div class="icon-circle bg-green-light">
              <el-icon class="text-green"><Monitor /></el-icon>
            </div>
            <span class="stat-title">监控中</span>
          </div>
          <div class="stat-main">
            <span class="stat-number">{{ monitoringStats.active }} <span class="stat-total">/ {{ monitoringStats.total }}</span></span>
          </div>
        </div>

        <div class="modern-card stat-card hover-effect" @click="emit('switchView', 'knowledge')">
          <div class="stat-top">
            <div class="icon-circle bg-orange-light">
              <el-icon class="text-orange"><Folder /></el-icon>
            </div>
            <span class="stat-title">知识库</span>
          </div>
          <div class="stat-main">
            <span class="stat-number">{{ kbStats.documentCount }}</span>
            <span class="stat-sub-text">{{ kbStats.conversationCount }} 个对话</span>
          </div>
        </div>
      </div>

      <!-- 第二排：排名概览 (大卡片) -->
      <div class="modern-card ranking-overview-card">
        <div class="card-header">
          <h3>排名分布概览</h3>
        </div>
        <div class="ranking-bars-container">
          <!-- Top 10 -->
          <div class="ranking-bar-group">
            <div class="bar-info">
              <span class="bar-label">Top 10 排名</span>
              <span class="bar-value text-green">{{ monitoringStats.top10_organic }}</span>
            </div>
            <div class="progress-bg">
              <div class="progress-fill fill-green" :style="{ width: getPercentage(monitoringStats.top10_organic, monitoringStats.active) + '%' }"></div>
            </div>
          </div>
          
          <!-- Top 30 -->
          <div class="ranking-bar-group">
            <div class="bar-info">
              <span class="bar-label">Top 30 排名</span>
              <span class="bar-value text-blue">{{ monitoringStats.top30_organic }}</span>
            </div>
            <div class="progress-bg">
              <div class="progress-fill fill-blue" :style="{ width: getPercentage(monitoringStats.top30_organic, monitoringStats.active) + '%' }"></div>
            </div>
          </div>

          <!-- Advertising -->
          <div class="ranking-bar-group">
            <div class="bar-info">
              <span class="bar-label">广告位占领</span>
              <span class="bar-value text-indigo">{{ monitoringStats.with_sponsored }}</span>
            </div>
            <div class="progress-bg">
              <div class="progress-fill fill-indigo" :style="{ width: getPercentage(monitoringStats.with_sponsored, monitoringStats.active) + '%' }"></div>
            </div>
          </div>
        </div>
      </div>

      <!-- 第三排：两栏布局 (排名变化 + 定时器) -->
      <div class="grid-section">
        <!-- 左侧：排名变化 -->
        <div class="modern-card movers-card">
          <div class="card-header border-bottom">
            <h3>近期排名波动 (7天)</h3>
          </div>
          <div class="movers-content">
            <div class="movers-column">
              <div class="column-title text-green"><el-icon><Top /></el-icon> 上升 Top 5</div>
              <div class="movers-list">
                <div v-for="item in topRisers" :key="item.monitoringId" class="mover-item">
                  <span class="mover-name" :title="item.keyword">{{ item.keyword }}</span>
                  <span class="mover-badge badge-green">+{{ item.change }}</span>
                </div>
                <div v-if="topRisers.length === 0" class="empty-text">暂无上升</div>
              </div>
            </div>
            <div class="divider-vertical"></div>
            <div class="movers-column">
              <div class="column-title text-red"><el-icon><Bottom /></el-icon> 下降 Top 5</div>
              <div class="movers-list">
                <div v-for="item in topFallers" :key="item.monitoringId" class="mover-item">
                  <span class="mover-name" :title="item.keyword">{{ item.keyword }}</span>
                  <span class="mover-badge badge-red">{{ item.change }}</span>
                </div>
                <div v-if="topFallers.length === 0" class="empty-text">暂无下降</div>
              </div>
            </div>
          </div>
        </div>

        <!-- 右侧：Next Scan 定时器 -->
        <div class="modern-card scheduler-card">
          <div class="card-header">
            <h3>自动检测</h3>
            <div class="status-indicator" v-if="schedulerSettings">
               <span class="status-dot" :class="{ 'is-active': schedulerSettings.enabled }"></span>
               {{ schedulerSettings.enabled ? '已开启' : '已关闭' }}
            </div>
          </div>
          
          <div class="timer-display-area">
             <template v-if="schedulerSettings && schedulerSettings.enabled">
                <div class="digital-clock" v-if="!isInWindow">
                   <div class="time-unit">{{ countdownHours }}</div>
                   <div class="colon">:</div>
                   <div class="time-unit">{{ countdownMinutes }}</div>
                   <div class="colon">:</div>
                   <div class="time-unit">{{ countdownSeconds }}</div>
                </div>
                <div class="scanning-animation" v-else>
                   <div class="pulse-ring"></div>
                   <div class="scanning-text">正在扫描窗口期...</div>
                </div>
                <div class="next-window-label" v-if="!isInWindow">
                  距离下次扫描 ({{ nextWindowLabel }})
                </div>
             </template>
             <div v-else class="scheduler-disabled">
                <el-icon :size="40" class="text-gray"><Timer /></el-icon>
                <p>自动检测未开启</p>
             </div>
          </div>
          
           <div class="scheduler-footer" v-if="schedulerStatus?.last_check_time">
            上次扫描: {{ formatDateTime(schedulerStatus.last_check_time) }}
          </div>
        </div>
      </div>

      <!-- 第四排：流量与事件 -->
      <div class="grid-section">
         <!-- 流量分布 -->
         <div class="modern-card traffic-card">
           <div class="card-header">
             <h3>流量级别分布</h3>
           </div>
           <div class="traffic-content">
             <div class="traffic-bars-visual">
                <div class="traffic-segment big" :style="{ flex: trafficStats.big_count || 1 }" v-if="getTotalTraffic() > 0"></div>
                <div class="traffic-segment medium" :style="{ flex: trafficStats.medium_count || 1 }" v-if="getTotalTraffic() > 0"></div>
                <div class="traffic-segment small" :style="{ flex: trafficStats.small_count || 1 }" v-if="getTotalTraffic() > 0"></div>
                <div class="traffic-placeholder" v-if="getTotalTraffic() === 0">暂无数据</div>
             </div>
             <div class="traffic-legend">
               <div class="legend-item">
                 <span class="dot bg-red"></span>
                 <span class="legend-name">大词</span>
                 <span class="legend-val">{{ trafficStats.big_count }}</span>
               </div>
               <div class="legend-item">
                 <span class="dot bg-orange"></span>
                 <span class="legend-name">中词</span>
                 <span class="legend-val">{{ trafficStats.medium_count }}</span>
               </div>
               <div class="legend-item">
                 <span class="dot bg-gray"></span>
                 <span class="legend-name">小词</span>
                 <span class="legend-val">{{ trafficStats.small_count }}</span>
               </div>
             </div>
           </div>
         </div>

         <!-- 最近事件 -->
         <div class="modern-card events-card">
           <div class="card-header border-bottom">
             <h3>优化日志</h3>
             <el-button link type="primary" size="small" @click="emit('switchView', 'monitoring')">查看全部</el-button>
           </div>
           <div class="events-timeline">
             <div v-for="(event, index) in recentEvents" :key="event.id" class="timeline-item">
               <div class="timeline-line" v-if="index !== recentEvents.length - 1"></div>
               <div class="timeline-dot"></div>
               <div class="timeline-content">
                 <div class="timeline-time">{{ formatDate(event.event_date) }}</div>
                 <div class="timeline-title">{{ event.title }}</div>
                 <div class="timeline-tag">{{ getEventTypeLabel(event.event_type) }}</div>
               </div>
             </div>
             <div v-if="recentEvents.length === 0" class="empty-text">暂无记录</div>
           </div>
         </div>
      </div>

    </div>

    <!-- 电商日历弹窗 -->
    <el-dialog
      v-model="showCalendarDialog"
      title="电商日历"
      width="520px"
      class="calendar-dialog"
      destroy-on-close
    >
      <div class="calendar-container">
        <!-- 月份导航 -->
        <div class="calendar-nav">
          <el-button :icon="ArrowLeft" text @click="prevMonth" />
          <div class="calendar-title">
            <span class="calendar-year">{{ calendarYear }}</span>
            <span class="calendar-month">{{ monthNames[calendarMonth] }}</span>
          </div>
          <el-button :icon="ArrowRight" text @click="nextMonth" />
          <el-button size="small" @click="goToToday" style="margin-left: 12px;">今天</el-button>
        </div>

        <!-- 星期标题 -->
        <div class="calendar-weekdays">
          <span v-for="day in ['日', '一', '二', '三', '四', '五', '六']" :key="day">{{ day }}</span>
        </div>

        <!-- 日历网格 -->
        <div class="calendar-grid">
          <div
            v-for="(cell, index) in calendarDays"
            :key="index"
            class="calendar-cell"
            :class="{
              'is-empty': !cell,
              'is-today': cell?.isToday,
              'has-holiday': cell?.holidays?.length && cell.holidays.length > 0
            }"
          >
            <template v-if="cell">
              <span class="cell-day">{{ cell.day }}</span>
              <div class="cell-holidays" v-if="cell.holidays && cell.holidays.length > 0">
                <el-tooltip
                  v-for="(holiday, hIndex) in cell.holidays"
                  :key="hIndex"
                  :content="`${getHolidayTypeLabel(holiday.type)} ${holiday.name}${holiday.markets ? ' (' + holiday.markets.join(', ') + ')' : ''}`"
                  placement="top"
                >
                  <span
                    class="holiday-dot"
                    :style="{ background: getHolidayTypeColor(holiday.type) }"
                  ></span>
                </el-tooltip>
              </div>
            </template>
          </div>
        </div>

        <!-- 图例 -->
        <div class="calendar-legend">
          <span class="legend-item">
            <span class="legend-dot" style="background: #f56c6c;"></span>
            大促
          </span>
          <span class="legend-item">
            <span class="legend-dot" style="background: #409eff;"></span>
            西方
          </span>
          <span class="legend-item">
            <span class="legend-dot" style="background: #e6a23c;"></span>
            中国
          </span>
          <span class="legend-item">
            <span class="legend-dot" style="background: #f472b6;"></span>
            日本
          </span>
          <span class="legend-item">
            <span class="legend-dot" style="background: #67c23a;"></span>
            通用
          </span>
        </div>
      </div>
    </el-dialog>

    <!-- 节日提醒弹窗 -->
    <el-dialog
      v-model="showHolidayReminder"
      title=""
      width="420px"
      :show-close="false"
      :close-on-click-modal="false"
      class="holiday-reminder-dialog"
    >
      <div class="holiday-reminder" v-if="currentHoliday">
        <!-- 节日信息头部 -->
        <div class="reminder-header">
          <el-button
            v-if="upcomingHolidays.length > 1"
            class="nav-btn"
            :icon="ArrowLeft"
            circle
            size="small"
            @click="prevHoliday"
          />
          <div class="holiday-info">
            <span class="holiday-icon">{{ getHolidayTypeLabel(currentHoliday.type).split(' ')[0] }}</span>
            <span class="holiday-name">{{ currentHoliday.name }}</span>
            <span class="holiday-countdown">还剩 {{ currentHoliday.daysLeft }} 天</span>
          </div>
          <el-button
            v-if="upcomingHolidays.length > 1"
            class="nav-btn"
            :icon="ArrowRight"
            circle
            size="small"
            @click="nextHoliday"
          />
        </div>

        <!-- 节日数量指示器 -->
        <div class="holiday-indicator" v-if="upcomingHolidays.length > 1">
          ({{ currentHolidayIndex + 1 }}/{{ upcomingHolidays.length }})
        </div>

        <!-- AI 建议 -->
        <div class="reminder-suggestions">
          <div class="suggestions-title">
            <el-icon v-if="holidayAiLoading" class="is-loading"><Timer /></el-icon>
            <span>{{ holidayAiLoading ? 'AI 正在生成建议...' : '准备建议' }}</span>
            <span v-if="!holidayAiLoading && holidayAiSuggestions.length > 0" class="suggestions-hint">
              (勾选后可加入备忘录)
            </span>
          </div>
          <ul class="suggestions-list" v-if="!holidayAiLoading && holidayAiSuggestions.length > 0">
            <li
              v-for="(suggestion, idx) in holidayAiSuggestions"
              :key="idx"
              class="suggestion-item"
              :class="{ selected: selectedSuggestions.includes(idx) }"
              @click="toggleSuggestion(idx)"
            >
              <el-checkbox
                :model-value="selectedSuggestions.includes(idx)"
                @click.stop
                @change="toggleSuggestion(idx)"
              />
              <span class="suggestion-text">{{ suggestion }}</span>
            </li>
          </ul>
          <div class="suggestions-loading" v-if="holidayAiLoading">
            <el-skeleton :rows="4" animated />
          </div>
        </div>

        <!-- 按钮 -->
        <div class="reminder-actions">
          <el-button @click="dismissCurrentHoliday">本节日不再提醒</el-button>
          <el-button
            v-if="selectedSuggestions.length > 0"
            type="success"
            :loading="addingToNotes"
            @click="addSuggestionsToNotes"
          >
            加入备忘录 ({{ selectedSuggestions.length }})
          </el-button>
          <el-button type="primary" @click="closeHolidayReminder">知道了</el-button>
        </div>
      </div>
    </el-dialog>
  </div>
</template>

<style scoped>
.dashboard-container {
  padding: 24px;
  background: var(--gradient-bg);
  min-height: 100%;
  box-sizing: border-box;
  position: relative;
}

/* 添加微妙的噪点纹理增加质感 */
.dashboard-container::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.8' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noise)'/%3E%3C/svg%3E");
  opacity: 0.02;
  pointer-events: none;
  z-index: 0;
}

.dashboard-content-new {
  position: relative;
  z-index: 1;
}

/* Header */
.dashboard-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 28px;
  padding-bottom: 20px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.header-left h2 {
  font-family: 'Poppins', sans-serif;
  font-size: 28px;
  font-weight: 700;
  color: var(--el-text-color-primary);
  margin: 0;
  background: var(--gradient-primary);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

/* 大屏模式按钮 */
.bigscreen-btn {
  background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%);
  border: 1px solid rgba(0, 212, 255, 0.3);
  color: #00d4ff;
  font-weight: 600;
}

.bigscreen-btn:hover {
  background: linear-gradient(135deg, #1e293b 0%, #334155 100%);
  border-color: #00d4ff;
  box-shadow: 0 0 20px rgba(0, 212, 255, 0.3);
}

.product-badge {
  display: inline-flex;
  align-items: center;
  background: var(--glass-bg);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  color: var(--el-color-primary);
  padding: 6px 16px;
  border-radius: 20px;
  font-size: 13px;
  font-weight: 600;
  border: 1px solid var(--glass-border);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
}

.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

/* 市场时钟 */
.market-clock {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  background: var(--glass-bg);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  border: 1px solid var(--glass-border);
  border-radius: 20px;
  cursor: pointer;
  user-select: none;
  transition: all 0.2s;
}

.market-clock:hover {
  border-color: var(--el-color-primary-light-5);
  background: var(--el-color-primary-light-9);
}


.clock-flag {
  width: 20px;
  height: 14px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 2px;
  overflow: hidden;
}

.clock-flag :deep(svg) {
  width: 100%;
  height: 100%;
}

.clock-time {
  font-family: 'SF Mono', 'Monaco', 'Consolas', monospace;
  font-size: 14px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  min-width: 70px;
  text-align: center;
}

.header-divider {
  width: 1px;
  height: 20px;
  background: var(--el-border-color);
  opacity: 0.6;
}

/* 日历按钮 */
.calendar-btn {
  background: var(--glass-bg);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  border: 1px solid var(--glass-border);
}

.calendar-btn:hover {
  border-color: var(--el-color-primary-light-5);
  background: var(--el-color-primary-light-9);
}

/* 电商日历弹窗 */
.calendar-container {
  padding: 8px;
}

.calendar-nav {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 16px;
}

.calendar-title {
  min-width: 140px;
  text-align: center;
  font-weight: 600;
}

.calendar-year {
  color: var(--el-text-color-secondary);
  font-size: 14px;
  margin-right: 8px;
}

.calendar-month {
  color: var(--el-text-color-primary);
  font-size: 18px;
}

.calendar-weekdays {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  text-align: center;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  padding-bottom: 8px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  margin-bottom: 8px;
}

.calendar-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 4px;
}

.calendar-cell {
  aspect-ratio: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  cursor: default;
  position: relative;
  min-height: 48px;
}

.calendar-cell.is-empty {
  background: transparent;
}

.calendar-cell:not(.is-empty):hover {
  background: var(--el-fill-color-light);
}

.calendar-cell.is-today {
  background: var(--el-color-primary-light-9);
}

.calendar-cell.is-today .cell-day {
  color: var(--el-color-primary);
  font-weight: 700;
}

.calendar-cell.has-holiday {
  background: var(--el-fill-color-lighter);
}

.cell-day {
  font-size: 14px;
  color: var(--el-text-color-primary);
}

.cell-holidays {
  display: flex;
  gap: 3px;
  margin-top: 4px;
}

.holiday-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  cursor: pointer;
}

.calendar-legend {
  display: flex;
  justify-content: center;
  gap: 16px;
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px solid var(--el-border-color-lighter);
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.legend-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

/* 汇率显示 */
.exchange-rates-wrapper {
  display: inline-flex;
}

.exchange-rates {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 6px 12px;
  background: var(--glass-bg);
  border-radius: 20px;
  border: 1px solid var(--glass-border);
  overflow: hidden;
  min-width: 120px; /* 避免切换时宽度跳动 */
}

.rate-item {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  cursor: default;
}

.rate-flag {
  display: inline-flex;
  align-items: center;
  width: 18px;
  height: 12px;
  border-radius: 2px;
  overflow: hidden;
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.1);
}

.rate-flag :deep(svg) {
  width: 100%;
  height: 100%;
}

.rate-code {
  color: var(--el-text-color-secondary);
  font-weight: 500;
}

.rate-value {
  font-family: 'Poppins', sans-serif;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.rate-direction {
  margin-left: 2px;
  font-size: 12px;
  font-weight: 600;
}

.rate-direction.up {
  color: #67c23a;
}

.rate-direction.down {
  color: #f56c6c;
}

.rate-direction.equal {
  color: var(--el-text-color-secondary);
}

/* 汇率轮播滑动动画 - 向上滑动 */
.rate-slide-up-enter-active,
.rate-slide-up-leave-active {
  transition: all 0.3s ease-out;
}

.rate-slide-up-enter-from {
  opacity: 0;
  transform: translateY(12px);
}

.rate-slide-up-leave-to {
  opacity: 0;
  transform: translateY(-12px);
}

/* 汇率轮播滑动动画 - 向下滑动 */
.rate-slide-down-enter-active,
.rate-slide-down-leave-active {
  transition: all 0.3s ease-out;
}

.rate-slide-down-enter-from {
  opacity: 0;
  transform: translateY(-12px);
}

.rate-slide-down-leave-to {
  opacity: 0;
  transform: translateY(12px);
}

/* 汇率趋势图弹出框 */
.rate-trend-popover {
  .trend-header {
    margin-bottom: 12px;
    padding-bottom: 8px;
    border-bottom: 1px solid #f0f0f0;

    .trend-title {
      font-size: 14px;
      font-weight: 500;
      color: #333;
    }
  }

  .trend-loading,
  .trend-no-data {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 30px 0;
    color: #999;
    font-size: 13px;
  }

  .trend-chart-container {
    .trend-chart {
      width: 100%;
      height: 160px;
    }

    .trend-stats {
      display: flex;
      justify-content: space-between;
      margin-top: 10px;
      padding-top: 10px;
      border-top: 1px solid #f0f0f0;

      .stat-item {
        display: flex;
        align-items: center;
        gap: 4px;
        font-size: 12px;

        .label {
          color: #999;
        }

        .value {
          font-weight: 500;
          color: #333;

          &.high {
            color: #f56c6c;
          }

          &.low {
            color: #67c23a;
          }
        }
      }
    }
  }
}

/* Card Utility - Glassmorphism */
.modern-card {
  background: var(--glass-bg);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border-radius: 20px;
  box-shadow: var(--glass-shadow);
  padding: 24px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  border: 1px solid var(--glass-border);
}

.modern-card.hover-effect:hover {
  transform: translateY(-4px);
  box-shadow: var(--glass-shadow-hover);
  cursor: pointer;
  background: rgba(255, 255, 255, 0.85);
}

html.dark .modern-card.hover-effect:hover {
  background: rgba(30, 41, 59, 0.85);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.card-header h3 {
  margin: 0;
  font-family: 'Poppins', sans-serif;
  font-size: 16px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.border-bottom {
  border-bottom: 1px solid rgba(0, 0, 0, 0.06);
  padding-bottom: 16px;
  margin-bottom: 16px;
}

html.dark .border-bottom {
  border-bottom-color: rgba(255, 255, 255, 0.08);
}

/* Stats Grid */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 24px;
  margin-bottom: 24px;
}

.stat-top {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}

.icon-circle {
  width: 52px;
  height: 52px;
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 22px;
  transition: transform 0.2s ease;
}

.stat-card:hover .icon-circle {
  transform: scale(1.05);
}

.bg-blue-light {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.15) 0%, rgba(59, 130, 246, 0.1) 100%);
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.15);
}
.text-blue { color: #2563EB; }

.bg-purple-light {
  background: linear-gradient(135deg, rgba(139, 92, 246, 0.15) 0%, rgba(167, 139, 250, 0.1) 100%);
  box-shadow: 0 4px 12px rgba(139, 92, 246, 0.15);
}
.text-purple { color: #8B5CF6; }

.bg-green-light {
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.15) 0%, rgba(52, 211, 153, 0.1) 100%);
  box-shadow: 0 4px 12px rgba(16, 185, 129, 0.15);
}
.text-green { color: #10B981; }

.bg-orange-light {
  background: linear-gradient(135deg, rgba(249, 115, 22, 0.15) 0%, rgba(251, 146, 60, 0.1) 100%);
  box-shadow: 0 4px 12px rgba(249, 115, 22, 0.15);
}
.text-orange { color: #F97316; }

.stat-title {
  color: var(--el-text-color-secondary);
  font-size: 14px;
  font-weight: 500;
  letter-spacing: 0.3px;
}

.stat-main {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.stat-number {
  font-family: 'Poppins', sans-serif;
  font-size: 32px;
  font-weight: 700;
  color: var(--el-text-color-primary);
  line-height: 1;
  letter-spacing: -0.5px;
}

.stat-total {
  font-size: 14px;
  color: var(--el-text-color-placeholder);
  font-weight: 400;
}

.stat-sub-text {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

/* Ranking Overview (Big Card) */
.ranking-overview-card {
  margin-bottom: 24px;
  padding-bottom: 32px;
}

.ranking-bars-container {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 48px;
}

.ranking-bar-group {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.bar-info {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
}

.bar-label {
  font-size: 14px;
  color: var(--el-text-color-regular);
  font-weight: 500;
}

.bar-value {
  font-family: 'Poppins', sans-serif;
  font-size: 26px;
  font-weight: 700;
  letter-spacing: -0.5px;
}

.text-indigo { color: #7C3AED; }

.fill-indigo {
  background: linear-gradient(90deg, #A78BFA 0%, #7C3AED 100%);
}
.fill-green {
  background: linear-gradient(90deg, #34D399 0%, #10B981 100%);
}
.fill-blue {
  background: linear-gradient(90deg, #60A5FA 0%, #3B82F6 100%);
}

.progress-bg {
  height: 10px;
  background: rgba(0, 0, 0, 0.06);
  border-radius: 5px;
  overflow: hidden;
}

html.dark .progress-bg {
  background: rgba(255, 255, 255, 0.1);
}

.progress-fill {
  height: 100%;
  border-radius: 5px;
  transition: width 0.6s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
}

/* 进度条光泽效果 */
.progress-fill::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 50%;
  background: linear-gradient(to bottom, rgba(255, 255, 255, 0.3), transparent);
  border-radius: 5px 5px 0 0;
}

/* Grid Section (2 columns) */
.grid-section {
  display: grid;
  grid-template-columns: 2fr 1.5fr;
  gap: 24px;
  margin-bottom: 24px;
}

/* Top Movers */
.movers-content {
  display: grid;
  grid-template-columns: 1fr 1px 1fr;
  gap: 24px;
}

.divider-vertical {
  background: linear-gradient(to bottom, transparent, rgba(0,0,0,0.08), transparent);
  height: 100%;
}

html.dark .divider-vertical {
  background: linear-gradient(to bottom, transparent, rgba(255,255,255,0.1), transparent);
}

.column-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 16px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.text-red { color: var(--el-color-danger); }

.movers-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.mover-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  background: rgba(0, 0, 0, 0.03);
  border-radius: 10px;
  transition: all 0.2s ease;
  cursor: default;
}

html.dark .mover-item {
  background: rgba(255, 255, 255, 0.05);
}

.mover-item:hover {
  background: rgba(0, 0, 0, 0.06);
  transform: translateX(4px);
}

html.dark .mover-item:hover {
  background: rgba(255, 255, 255, 0.08);
}

.mover-name {
  font-size: 13px;
  color: var(--el-text-color-primary);
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 150px;
}

.mover-badge {
  font-family: 'Poppins', sans-serif;
  font-size: 12px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 12px;
}

.badge-green {
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.15) 0%, rgba(52, 211, 153, 0.1) 100%);
  color: #10B981;
}
.badge-red {
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.15) 0%, rgba(248, 113, 113, 0.1) 100%);
  color: #EF4444;
}

/* Scheduler / Digital Clock */
.status-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.status-dot {
  width: 8px;
  height: 8px;
  background: var(--el-border-color);
  border-radius: 50%;
}
.status-dot.is-active {
  background: var(--el-color-success);
  box-shadow: 0 0 0 2px var(--el-color-success-light-7);
}

.timer-display-area {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 24px 0;
  min-height: 180px;
}

.digital-clock {
  display: flex;
  align-items: baseline;
  font-family: 'Poppins', -apple-system, BlinkMacSystemFont, sans-serif;
  font-variant-numeric: tabular-nums;
  margin-bottom: 12px;
}

.time-unit {
  font-size: 60px;
  font-weight: 800;
  letter-spacing: -2px;
  line-height: 1;
  display: inline-block;
  background: var(--gradient-primary);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.colon {
  font-size: 48px;
  font-weight: 300;
  color: var(--el-text-color-placeholder);
  margin: 0 6px;
  transform: translateY(-6px);
  animation: blink 1s step-end infinite;
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

.next-window-label {
  font-size: 13px;
  color: var(--el-text-color-secondary);
  margin-top: 8px;
  font-weight: 500;
}

.scanning-animation {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.pulse-ring {
  width: 70px;
  height: 70px;
  border-radius: 50%;
  background: linear-gradient(135deg, #10B981 0%, #34D399 100%);
  opacity: 0.3;
  animation: pulse-ring 2s cubic-bezier(0.4, 0, 0.2, 1) infinite;
  margin-bottom: 16px;
}

.scanning-text {
  color: #10B981;
  font-weight: 600;
  font-size: 14px;
  letter-spacing: 0.5px;
}

@keyframes pulse-ring {
  0% { transform: scale(0.8); opacity: 0.4; }
  50% { opacity: 0.2; }
  100% { transform: scale(1.6); opacity: 0; }
}

.scheduler-disabled {
  display: flex;
  flex-direction: column;
  align-items: center;
  color: var(--el-text-color-placeholder);
  gap: 12px;
}

.scheduler-footer {
  text-align: center;
  font-size: 12px;
  color: var(--el-text-color-placeholder);
}

/* Traffic */
.traffic-bars-visual {
  display: flex;
  height: 28px;
  border-radius: 14px;
  overflow: hidden;
  margin-bottom: 24px;
  background: rgba(0, 0, 0, 0.04);
}

html.dark .traffic-bars-visual {
  background: rgba(255, 255, 255, 0.06);
}

.traffic-segment {
  height: 100%;
  transition: flex 0.4s ease;
}

.traffic-segment.big {
  background: linear-gradient(135deg, #F87171 0%, #EF4444 100%);
}
.traffic-segment.medium {
  background: linear-gradient(135deg, #FBBF24 0%, #F59E0B 100%);
}
.traffic-segment.small {
  background: linear-gradient(135deg, #9CA3AF 0%, #6B7280 100%);
}

.traffic-legend {
  display: flex;
  justify-content: center;
  gap: 36px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
}

.dot {
  width: 10px;
  height: 10px;
  border-radius: 3px;
}

.bg-red {
  background: linear-gradient(135deg, #F87171 0%, #EF4444 100%);
}
.bg-orange {
  background: linear-gradient(135deg, #FBBF24 0%, #F59E0B 100%);
}
.bg-gray {
  background: linear-gradient(135deg, #9CA3AF 0%, #6B7280 100%);
}

.legend-val {
  font-family: 'Poppins', sans-serif;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

/* Events Timeline */
.events-timeline {
  padding: 0 12px;
}

.timeline-item {
  position: relative;
  display: flex;
  padding-bottom: 24px;
}

.timeline-line {
  position: absolute;
  left: 5px;
  top: 14px;
  bottom: 0;
  width: 2px;
  background: linear-gradient(to bottom, var(--el-color-primary-light-7), transparent);
}

.timeline-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--gradient-primary);
  border: none;
  z-index: 1;
  margin-right: 16px;
  margin-top: 4px;
  flex-shrink: 0;
  box-shadow: 0 0 0 4px rgba(37, 99, 235, 0.15);
}

.timeline-content {
  flex: 1;
}

.timeline-time {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 2px;
}

.timeline-title {
  font-size: 14px;
  color: var(--el-text-color-primary);
  margin-bottom: 4px;
}

.timeline-tag {
  display: inline-block;
  font-size: 11px;
  font-weight: 500;
  color: #2563EB;
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.1) 0%, rgba(59, 130, 246, 0.05) 100%);
  padding: 3px 10px;
  border-radius: 6px;
}

.empty-text {
  color: var(--el-text-color-secondary);
  text-align: center;
  padding: 24px;
  font-size: 13px;
}

/* Dark Theme Overrides */
html.dark .dashboard-header {
  border-bottom-color: rgba(255, 255, 255, 0.06);
}

html.dark .header-left h2 {
  background: linear-gradient(135deg, #60A5FA 0%, #93C5FD 100%);
  -webkit-background-clip: text;
  background-clip: text;
}

html.dark .time-unit {
  background: linear-gradient(135deg, #60A5FA 0%, #93C5FD 100%);
  -webkit-background-clip: text;
  background-clip: text;
}

html.dark .timeline-dot {
  box-shadow: 0 0 0 4px rgba(96, 165, 250, 0.2);
}

html.dark .timeline-tag {
  color: #60A5FA;
  background: linear-gradient(135deg, rgba(96, 165, 250, 0.15) 0%, rgba(147, 197, 253, 0.08) 100%);
}

/* 节日提醒弹窗 */
.holiday-reminder-dialog :deep(.el-dialog__header) {
  display: none;
}

.holiday-reminder-dialog :deep(.el-dialog__body) {
  padding: 0;
}

.holiday-reminder {
  padding: 24px;
}

.reminder-header {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  margin-bottom: 8px;
}

.nav-btn {
  flex-shrink: 0;
}

.holiday-info {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.holiday-icon {
  font-size: 36px;
}

.holiday-name {
  font-size: 20px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.holiday-countdown {
  font-size: 16px;
  color: var(--el-color-primary);
  font-weight: 500;
}

.holiday-indicator {
  text-align: center;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 16px;
}

.reminder-suggestions {
  background: var(--el-fill-color-light);
  border-radius: 12px;
  padding: 16px;
  margin-bottom: 20px;
}

.suggestions-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 12px;
}

.suggestions-hint {
  font-size: 12px;
  color: var(--el-text-color-placeholder);
  margin-left: 8px;
}

.suggestions-list {
  margin: 0;
  padding: 0;
  list-style: none;
}

.suggestion-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px 12px;
  margin: 4px 0;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  background: var(--el-fill-color-light);
}

.suggestion-item:hover {
  background: var(--el-fill-color);
}

.suggestion-item.selected {
  background: var(--el-color-success-light-9);
  border: 1px solid var(--el-color-success-light-5);
}

.suggestion-item .el-checkbox {
  flex-shrink: 0;
  margin-top: 2px;
}

.suggestion-text {
  font-size: 14px;
  color: var(--el-text-color-regular);
  line-height: 1.6;
  flex: 1;
}

.suggestions-loading {
  padding: 8px 0;
}

.reminder-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  flex-wrap: wrap;
}

/* Responsive */
@media (max-width: 1200px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }
  .ranking-bars-container {
    grid-template-columns: 1fr;
    gap: 24px;
  }
}

@media (max-width: 900px) {
  .grid-section {
    grid-template-columns: 1fr;
  }
  .movers-content {
    grid-template-columns: 1fr;
  }
  .divider-vertical {
    display: none;
  }
}
</style>

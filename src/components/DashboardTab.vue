<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { TrendCharts, Document, Monitor, Folder, Top, Bottom, Timer, FullScreen, Calendar, ArrowLeft, ArrowRight } from '@element-plus/icons-vue';
import BigScreenView from './BigScreenView.vue';
import * as api from '../api';

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
}

onMounted(() => {
  loadDashboardData();
  startCountdownTimer();
  // 加载市场时钟偏好并启动
  loadMarketClockPreference();
  startClock();
  // 监听汇率设置变更
  window.addEventListener('exchange-rate-settings-changed', handleExchangeRateSettingsChanged);
});

onUnmounted(() => {
  if (countdownTimer) {
    clearInterval(countdownTimer);
    countdownTimer = null;
  }
  // 停止市场时钟
  stopClock();
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
  // 一月
  '01-01': [{ name: '元旦', type: 'universal' }],
  '01-15': [{ name: 'MLK Day (美)', type: 'western', markets: ['US'] }],
  '01-26': [{ name: '澳大利亚国庆日', type: 'western', markets: ['AU'] }],
  // 二月
  '02-14': [{ name: '情人节', type: 'universal' }],
  // 三月
  '03-08': [{ name: '妇女节', type: 'universal' }],
  '03-17': [{ name: '圣帕特里克节', type: 'western', markets: ['US', 'UK'] }],
  // 四月
  '04-01': [{ name: '愚人节', type: 'western' }],
  // 五月 - 母亲节是5月第二个周日，这里用固定日期近似
  '05-01': [{ name: '劳动节', type: 'universal' }],
  '05-05': [{ name: '儿童节 (日本)', type: 'japan', markets: ['JP'] }],
  '05-11': [{ name: '母亲节', type: 'universal' }],
  '05-31': [{ name: 'Memorial Day (美)', type: 'western', markets: ['US'] }],
  // 六月
  '06-01': [{ name: '儿童节', type: 'chinese', markets: ['CN'] }],
  '06-15': [{ name: '父亲节', type: 'universal' }],
  '06-18': [{ name: '618 大促', type: 'promo', markets: ['CN'] }],
  // 七月
  '07-04': [{ name: '美国独立日', type: 'western', markets: ['US'] }],
  '07-15': [{ name: 'Prime Day (预估)', type: 'promo', markets: ['US', 'UK', 'DE', 'JP'] }],
  '07-16': [{ name: 'Prime Day (预估)', type: 'promo', markets: ['US', 'UK', 'DE', 'JP'] }],
  // 八月
  '08-15': [{ name: '盂兰盆节', type: 'japan', markets: ['JP'] }],
  // 九月
  '09-01': [{ name: '返校季开始', type: 'promo', markets: ['US'] }],
  '09-04': [{ name: 'Labor Day (美)', type: 'western', markets: ['US'] }],
  // 十月
  '10-01': [{ name: '国庆节', type: 'chinese', markets: ['CN'] }],
  '10-09': [{ name: '感恩节 (加)', type: 'western', markets: ['CA'] }],
  '10-31': [{ name: '万圣节', type: 'western', markets: ['US', 'UK'] }],
  // 十一月
  '11-11': [{ name: '双十一', type: 'promo', markets: ['CN'] }, { name: '光棍节', type: 'chinese', markets: ['CN'] }],
  '11-23': [{ name: '感恩节 (美)', type: 'western', markets: ['US'] }],
  '11-24': [{ name: '黑色星期五', type: 'promo' }],
  '11-27': [{ name: '网络星期一', type: 'promo' }],
  // 十二月
  '12-12': [{ name: '双十二', type: 'promo', markets: ['CN'] }],
  '12-24': [{ name: '平安夜', type: 'western' }],
  '12-25': [{ name: '圣诞节', type: 'western' }],
  '12-26': [{ name: 'Boxing Day', type: 'western', markets: ['UK', 'CA', 'AU'] }],
  '12-31': [{ name: '除夕', type: 'universal' }],
};

// 2026年特定节日（农历节日等）
const holiday2026: Record<string, Holiday[]> = {
  '02-17': [{ name: '春节', type: 'chinese', markets: ['CN'] }],
  '04-05': [{ name: '清明节', type: 'chinese', markets: ['CN'] }],
  '04-05-2026': [{ name: '复活节', type: 'western' }],
  '05-31': [{ name: '端午节', type: 'chinese', markets: ['CN'] }],
  '10-06': [{ name: '中秋节', type: 'chinese', markets: ['CN'] }],
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

  // 查找特定年份节日
  if (year === 2026 && holiday2026[mmdd]) {
    holidays.push(...holiday2026[mmdd]);
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

// ==================== 汇率相关 ====================
const exchangeRates = ref<Map<string, number>>(new Map());
const exchangeRatesLoading = ref(false);
const exchangeRatesUpdatedAt = ref<string | null>(null);

// 用户选择的显示货币（默认前3个）
const selectedCurrencies = ref<string[]>(['USD', 'EUR', 'GBP']);

// 加载用户汇率偏好
function loadCurrencyPreference() {
  try {
    const saved = localStorage.getItem('exchange_rate_currencies');
    if (saved) {
      const parsed = JSON.parse(saved);
      if (Array.isArray(parsed) && parsed.length > 0 && parsed.length <= 3) {
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

// 加载缓存的汇率
async function loadCachedRates() {
  try {
    const cached = await api.getExchangeRates();
    if (cached.length > 0) {
      cached.forEach(item => {
        exchangeRates.value.set(item.currency, item.rate);
      });
      exchangeRatesUpdatedAt.value = cached[0]?.updated_at || null;
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
          <!-- 汇率显示 -->
          <div class="exchange-rates" v-if="exchangeRates.size > 0">
            <span
              v-for="currency in displayCurrencies"
              :key="currency.code"
              class="rate-item"
              :title="`1 ${currency.code} = ${formatRate(currency.code)} CNY${currency.multiplier ? ` (×${currency.multiplier})` : ''}`"
            >
              <span class="rate-flag" v-html="currency.flag"></span>
              <span class="rate-code">{{ currency.code }}</span>
              <span class="rate-value">{{ formatRate(currency.code) }}</span>
            </span>
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
              'has-holiday': cell?.holidays.length > 0
            }"
          >
            <template v-if="cell">
              <span class="cell-day">{{ cell.day }}</span>
              <div class="cell-holidays" v-if="cell.holidays.length > 0">
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
.exchange-rates {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 6px 12px;
  background: var(--glass-bg);
  border-radius: 20px;
  border: 1px solid var(--glass-border);
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

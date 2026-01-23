<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, defineAsyncComponent } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { open, save } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { readFile, writeFile } from "@tauri-apps/plugin-fs";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { getVersion } from "@tauri-apps/api/app";
import * as XLSX from "xlsx";
import * as api from "./api";
import { batchAnalyzeWords, batchAnalyzeKeywordCategories } from "./deepseek";
import type { BackupInfo, Category, KeywordData, Product, Root, WorkflowStatus } from "./types";
import { EXCHANGE_RATE_CURRENCIES } from "./types";
import type { UnlistenFn } from "@tauri-apps/api/event";

// 懒加载组件 - 只在需要时才加载
const WordCloud = defineAsyncComponent(() => import("./components/WordCloud.vue"));
const TrafficSettingsDialog = defineAsyncComponent(() => import("./components/TrafficSettingsDialog.vue"));
const KeywordExportDialog = defineAsyncComponent(() => import("./components/KeywordExportDialog.vue"));
const BackupDialog = defineAsyncComponent(() => import("./components/BackupDialog.vue"));
const ColumnConfigDialog = defineAsyncComponent(() => import("./components/ColumnConfigDialog.vue"));
const ProductDialog = defineAsyncComponent(() => import("./components/ProductDialog.vue"));
const ShortcutsDialog = defineAsyncComponent(() => import("./components/ShortcutsDialog.vue"));
const ApiKeyDialog = defineAsyncComponent(() => import("./components/ApiKeyDialog.vue"));
const KeywordMonitoringTab = defineAsyncComponent(() => import("./components/KeywordMonitoringTab.vue"));
const SettingsDialog = defineAsyncComponent(() => import("./components/SettingsDialog.vue"));
const QuickAddMonitoringDialog = defineAsyncComponent(() => import("./components/QuickAddMonitoringDialog.vue"));
const KnowledgeBaseTab = defineAsyncComponent(() => import("./components/KnowledgeBaseTab.vue"));
const SetupWizardDialog = defineAsyncComponent(() => import("./components/SetupWizardDialog.vue"));
const DashboardTab = defineAsyncComponent(() => import("./components/DashboardTab.vue"));
const SmartCopyTab = defineAsyncComponent(() => import("./components/SmartCopyTab.vue"));
const AdOptimizerTab = defineAsyncComponent(() => import("./components/AdOptimizerTab.vue"));
const AgentTab = defineAsyncComponent(() => import("./components/AgentTab.vue"));
const GlobalNotification = defineAsyncComponent(() => import("./components/GlobalNotification.vue"));
const QuickNotes = defineAsyncComponent(() => import("./components/QuickNotes.vue"));

// ==================== 产品相关状态 ====================
const products = ref<Product[]>([]);
const selectedProduct = ref<Product | null>(null);
const showProductDialog = ref(false);
const productForm = ref({ id: 0, name: "", country: "" });
const isEditingProduct = ref(false);

// 国家选项（使用SVG国旗）
const countryOptions = [
  { code: "US", name: "美国", flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="20" fill="#B22234"/><rect y="1.54" width="30" height="1.54" fill="white"/><rect y="4.62" width="30" height="1.54" fill="white"/><rect y="7.69" width="30" height="1.54" fill="white"/><rect y="10.77" width="30" height="1.54" fill="white"/><rect y="13.85" width="30" height="1.54" fill="white"/><rect y="16.92" width="30" height="1.54" fill="white"/><rect width="12" height="10.77" fill="#3C3B6E"/></svg>` },
  { code: "UK", name: "英国", flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="20" fill="#012169"/><path d="M0,0 L30,20 M30,0 L0,20" stroke="white" stroke-width="4"/><path d="M0,0 L30,20 M30,0 L0,20" stroke="#C8102E" stroke-width="2.5"/><path d="M15,0 V20 M0,10 H30" stroke="white" stroke-width="6"/><path d="M15,0 V20 M0,10 H30" stroke="#C8102E" stroke-width="3.5"/></svg>` },
  { code: "DE", name: "德国", flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="6.67" fill="#000"/><rect y="6.67" width="30" height="6.67" fill="#DD0000"/><rect y="13.33" width="30" height="6.67" fill="#FFCE00"/></svg>` },
  { code: "FR", name: "法国", flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="10" height="20" fill="#002395"/><rect x="10" width="10" height="20" fill="white"/><rect x="20" width="10" height="20" fill="#ED2939"/></svg>` },
  { code: "IT", name: "意大利", flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="10" height="20" fill="#009246"/><rect x="10" width="10" height="20" fill="white"/><rect x="20" width="10" height="20" fill="#CE2B37"/></svg>` },
  { code: "ES", name: "西班牙", flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="5" fill="#AA151B"/><rect y="5" width="30" height="10" fill="#F1BF00"/><rect y="15" width="30" height="5" fill="#AA151B"/></svg>` },
  { code: "JP", name: "日本", flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="20" fill="white"/><circle cx="15" cy="10" r="6" fill="#BC002D"/></svg>` },
];

// Amazon 域名映射
const amazonDomains: Record<string, string> = {
  US: "www.amazon.com",
  UK: "www.amazon.co.uk",
  DE: "www.amazon.de",
  FR: "www.amazon.fr",
  IT: "www.amazon.it",
  ES: "www.amazon.es",
  JP: "www.amazon.co.jp",
};

// 打开 Amazon 搜索
async function openAmazonSearch(keyword: string) {
  if (!selectedProduct.value?.country) return;

  const domain = amazonDomains[selectedProduct.value.country];
  if (!domain) return;

  const url = `https://${domain}/s?k=${encodeURIComponent(keyword)}`;
  await openUrl(url);
}

// 复制关键词到剪贴板
async function copyKeyword(keyword: string) {
  try {
    await writeText(keyword);
    ElMessage.success(`已复制: ${keyword}`);
  } catch (e) {
    ElMessage.error("复制失败");
  }
}

// 获取国家名称
function getCountryName(code: string | null): string {
  if (!code) return "";
  const country = countryOptions.find(c => c.code === code);
  return country?.name || code;
}

// 获取国家国旗SVG
function getCountryFlag(code: string | null): string {
  if (!code) return "";
  const country = countryOptions.find(c => c.code === code);
  return country?.flag || "";
}

// ==================== 词根相关状态 ====================
const categories = ref<Category[]>([]);
const roots = ref<Root[]>([]);
const total = ref(0);
const loading = ref(false);
const importing = ref(false);
const isDragging = ref(false);
const analyzing = ref(false);
const analysisProgress = ref({ current: 0, total: 0 });
const exporting = ref(false);
const analysisAbortController = ref<AbortController | null>(null);

// 关键词导出相关状态
const showKeywordExportDialog = ref(false);
const keywordExportScope = ref<'filtered' | 'all'>('filtered');
const keywordExporting = ref(false);

// 备份管理相关状态
const showBackupDialog = ref(false);
const backups = ref<BackupInfo[]>([]);
const restoring = ref(false);

// 关键词分类相关状态
const classifying = ref(false);
const classifyProgress = ref({ current: 0, total: 0 });
const classifyAbortController = ref<AbortController | null>(null);

// 词组打标状态
const phraseTagging = ref(false);

// 词组标签编辑
const editingPhraseTagId = ref<number | null>(null);
const editingPhraseTagValue = ref("");

// 流程状态
const workflowStatus = ref<WorkflowStatus>({
  has_data: false,
  has_traffic_level: false,
  has_category: false,
  has_phrase_tag: false,
  has_orderliness: false,
});

// 筛选和分页
const searchText = ref("");
const selectedCategories = ref<number[]>([]);
const currentPage = ref(1);
const pageSize = ref(50);
const sortBy = ref("contains_count");
const sortOrder = ref("desc");

// 统计
const stats = ref({ keywordCount: 0, rootCount: 0 });
const categoryCounts = ref<Map<number, number>>(new Map());

// 编辑翻译
const editingId = ref<number | null>(null);
const editingTranslation = ref("");

// 分类下拉
const categoryDropdownVisible = ref<{ [key: number]: boolean }>({});

// 主题
const isDarkMode = ref(false);

// 侧边栏宽度
const sidebarWidth = ref(240);
const isResizing = ref(false);
const MIN_SIDEBAR_WIDTH = 180;
const MAX_SIDEBAR_WIDTH = 400;

// 快捷键帮助弹窗
const showShortcutsDialog = ref(false);

// API Key 设置弹窗
const showApiKeyDialog = ref(false);

// 汇率显示设置弹窗
const showExchangeRateSettings = ref(false);
const selectedCurrencies = ref<string[]>(['USD', 'EUR', 'GBP']);

// 加载汇率显示设置
function loadExchangeRateSettings() {
  const saved = localStorage.getItem('exchange_rate_currencies');
  if (saved) {
    try {
      const parsed = JSON.parse(saved);
      if (Array.isArray(parsed) && parsed.length > 0) {
        selectedCurrencies.value = parsed;
      }
    } catch (e) {
      console.error('解析汇率设置失败:', e);
    }
  }
}

// 保存汇率显示设置
function saveExchangeRateSettings() {
  if (selectedCurrencies.value.length !== 3) {
    ElMessage.warning('请选择 3 个货币');
    return;
  }
  localStorage.setItem('exchange_rate_currencies', JSON.stringify(selectedCurrencies.value));
  // 派发自定义事件通知其他组件
  window.dispatchEvent(new CustomEvent('exchange-rate-settings-changed'));
  showExchangeRateSettings.value = false;
  ElMessage.success('汇率显示设置已保存');
}

// 切换货币选择
function toggleCurrencySelection(code: string) {
  const index = selectedCurrencies.value.indexOf(code);
  if (index > -1) {
    // 如果已选中，取消选择
    if (selectedCurrencies.value.length > 1) {
      selectedCurrencies.value.splice(index, 1);
    } else {
      ElMessage.warning('至少需要选择 1 个货币');
    }
  } else {
    // 如果未选中，添加选择
    if (selectedCurrencies.value.length < 3) {
      selectedCurrencies.value.push(code);
    } else {
      ElMessage.warning('最多只能选择 3 个货币');
    }
  }
}

// 帮助弹窗
const showHelpDialog = ref(false);
const activeHelpTab = ref('dashboard');
const helpSearchQuery = ref('');

// 帮助内容定义（用于搜索）
const helpSections = [
  { id: 'dashboard', title: '数据概览', icon: 'DataAnalysis', keywords: ['首页', '数据', '统计', '关键词', '监控', '排名变化', '待办'] },
  { id: 'keywords', title: '词库管理', icon: 'Collection', keywords: ['关键词', '导入', '分类', '词根', '流量', '搜索量', 'AI', '西柚', '卖家精灵'] },
  { id: 'monitoring', title: '排名监控', icon: 'TrendCharts', keywords: ['排名', '监控', '追踪', '变化', '调度', '定时', '通知'] },
  { id: 'smartcopy', title: '智能文案', icon: 'EditPen', keywords: ['文案', '标题', '五点', 'listing', 'AI', '竞品', '分析', '新品', '老品'] },
  { id: 'ads', title: '智能广告', icon: 'Promotion', keywords: ['广告', 'ACOS', 'CPC', '否词', '优化', '预算', '投放'] },
  { id: 'knowledge', title: '知识库', icon: 'FolderOpened', keywords: ['知识', '文档', 'AI问答', '向量', '搜索', 'RAG', '分类', '双向链接', '知识图谱', 'Callout', '大纲', '保存笔记', 'Obsidian'] },
];

// 搜索过滤后的菜单项
const filteredHelpSections = computed(() => {
  const query = helpSearchQuery.value.toLowerCase().trim();
  if (!query) return helpSections;
  return helpSections.filter(section =>
    section.title.toLowerCase().includes(query) ||
    section.keywords.some(k => k.toLowerCase().includes(query))
  );
});

// 是否显示搜索结果
const isSearchingHelp = computed(() => helpSearchQuery.value.trim().length > 0);

// 打开帮助并跳转到指定模块
function openHelp(tab?: string) {
  if (tab) {
    activeHelpTab.value = tab;
  }
  helpSearchQuery.value = ''; // 打开时清空搜索
  showHelpDialog.value = true;
}

// 首次启动配置向导
const showSetupWizard = ref(false);

// API Key 配置状态（用于功能状态提示）
const apiKeyStatus = ref({
  deepseek: false,
  qwen: false,
});

// 检查 API Key 配置状态
async function checkApiKeyStatus() {
  try {
    apiKeyStatus.value.deepseek = await api.hasApiKey('deepseek');
    apiKeyStatus.value.qwen = await api.hasApiKey('qwen');
  } catch (e) {
    console.error('检查 API Key 状态失败:', e);
  }
}

// 监控设置弹窗
const showSettingsDialog = ref(false);
const settingsInitialTab = ref<'monitoring' | 'auto' | 'logs'>('monitoring');

// 打开监控设置指定标签页
function openSettingsTab(tab: 'monitoring' | 'auto' | 'logs') {
  settingsInitialTab.value = tab;
  showSettingsDialog.value = true;
}

// 应用版本
const appVersion = ref("");

// 自动更新相关状态
const showUpdateDialog = ref(false);
const updateVersion = ref("");
const updateDownloading = ref(false);
const updateProgress = ref(0);
const updateTotal = ref(0);

// 视图模式: 'keywords' | 'roots' | 'wordcloud' | 'monitoring' | 'smartcopy' | 'knowledge' | 'ads'
const viewMode = ref<'dashboard' | 'keywords' | 'roots' | 'wordcloud' | 'monitoring' | 'smartcopy' | 'knowledge' | 'ads' | 'agent'>('dashboard');
// 是否启用智能体功能（通过环境变量控制，开发环境显示，生产环境隐藏）
const enableAgent = import.meta.env.VITE_ENABLE_AGENT === 'true';
const wordCloudRef = ref<InstanceType<typeof WordCloud> | null>(null);
const allRootsForCloud = ref<Root[]>([]);
const loadingCloud = ref(false);

// ==================== 流量设置相关状态 ====================
const showTrafficDialog = ref(false);

// ==================== 关键词数据相关状态 ====================
const keywordData = ref<KeywordData[]>([]);
const keywordTotal = ref(0);
const keywordLoading = ref(false);
const keywordPage = ref(1);
const keywordPageSize = ref(50);
const keywordSearch = ref("");
const keywordSortBy = ref("id");
const keywordSortOrder = ref("asc");

// 关键词多选
const selectedKeywords = ref<KeywordData[]>([]);
const keywordTableRef = ref<InstanceType<typeof import('element-plus')['ElTable']> | null>(null);

// 关键词筛选
const keywordFilters = ref({
  trafficLevel: [] as string[],      // 流量级别：大词/中词/小词
  relevanceLevel: [] as string[],    // 相关性：强相关/高相关/中相关/弱相关
  primaryCategory: [] as string[],   // 一级分类
  orderliness: [] as string[],       // 有序性：有序/无序
});

const trafficLevelOptions = ["大词", "中词", "小词"];
const relevanceLevelOptions = ["强相关", "高相关", "中相关", "弱相关"];
const orderlinessOptions = ["有序", "无序"];
const primaryCategoryOptions = ["品类词", "功能词", "场景词", "属性词", "品牌词", "人群词", "受众词", "其他"];

// 重置筛选
function resetKeywordFilters() {
  keywordFilters.value = {
    trafficLevel: [],
    relevanceLevel: [],
    primaryCategory: [],
    orderliness: [],
  };
  keywordSearch.value = "";
  keywordPage.value = 1;
  loadKeywordData();
}

// 筛选变化时重新加载
function handleFilterChange() {
  keywordPage.value = 1;
  loadKeywordData();
}

// 是否有活跃筛选
const hasActiveFilters = computed(() => {
  return !!(keywordSearch.value ||
    keywordFilters.value.trafficLevel.length > 0 ||
    keywordFilters.value.relevanceLevel.length > 0 ||
    keywordFilters.value.primaryCategory.length > 0 ||
    keywordFilters.value.orderliness.length > 0);
});

// 关键词多选处理
function handleKeywordSelectionChange(rows: KeywordData[]) {
  selectedKeywords.value = rows;
}

function clearKeywordSelection() {
  selectedKeywords.value = [];
  keywordTableRef.value?.clearSelection();
}

// 快速添加监控对话框
const showQuickAddMonitoringDialog = ref(false);

function handleQuickAddMonitoringSuccess() {
  clearKeywordSelection();
  ElMessage.success('添加成功，可在「排名监控」标签页查看');
}

// 列配置
const showColumnConfig = ref(false);
const columnDefinitions = [
  { key: "keyword", label: "关键词", required: true },
  { key: "translation", label: "翻译", default: true },
  { key: "traffic_level", label: "流量级别", default: true },
  { key: "negative_word", label: "否词", default: false },
  { key: "orderliness", label: "有序性", default: true },
  { key: "phrase_tag", label: "词组标签", default: true },
  { key: "primary_category", label: "一级分类", default: true },
  { key: "secondary_category", label: "二级分类", default: true },
  { key: "search_intent", label: "搜索意图", default: true },
  { key: "traffic_share", label: "流量占比", default: true },
  { key: "relevance_score", label: "相关性得分", default: false },
  { key: "relevance_level", label: "相关性档位", default: true },
  { key: "traffic_total", label: "流量总和", default: true },
  { key: "avg_keyword_rank", label: "周平均排名", default: true },
  { key: "avg_search_volume", label: "周平均搜索量", default: true },
  { key: "cpc_bid", label: "CPC建议竞价", default: false },
  { key: "bid_range", label: "建议竞价范围", default: false },
  { key: "click_rate", label: "点击转化率", default: false },
  { key: "conversion_competition", label: "周转化竞争", default: false },
  { key: "competition_level", label: "竞争度档位", default: false },
  { key: "natural_position_flow", label: "自然位流动率", default: false },
  { key: "top3_click_share", label: "Top3点击份额", default: false },
  { key: "avg_conversion_share", label: "Top3转化份额", default: false },
  { key: "asin_count", label: "ASIN数量", default: false },
];

// 从 localStorage 加载列配置，如果没有则使用默认值（默认全选）
const getDefaultColumnConfig = () => {
  const config: Record<string, boolean> = {};
  columnDefinitions.forEach(col => {
    config[col.key] = true; // 默认全选
  });
  return config;
};

// 全选状态计算属性
const isAllColumnsSelected = computed(() => {
  return columnDefinitions.filter(col => !col.required).every(col => columnConfig.value[col.key]);
});

// 全选/取消全选
const toggleSelectAllColumns = (val: boolean) => {
  columnDefinitions.forEach(col => {
    if (!col.required) {
      columnConfig.value[col.key] = val;
    }
  });
  saveColumnConfig();
};

const loadColumnConfig = () => {
  const saved = localStorage.getItem("keywordColumnConfig");
  if (saved) {
    try {
      return JSON.parse(saved);
    } catch {
      return getDefaultColumnConfig();
    }
  }
  return getDefaultColumnConfig();
};

const columnConfig = ref<Record<string, boolean>>(loadColumnConfig());

const saveColumnConfig = () => {
  localStorage.setItem("keywordColumnConfig", JSON.stringify(columnConfig.value));
};

// 一级分类和二级分类
const primaryCategories = computed(() =>
  categories.value.filter((c) =>
    ["品类词", "品牌", "颜色", "形状", "功能"].includes(c.name)
  )
);

const secondaryCategories = computed(() =>
  categories.value.filter(
    (c) => !["品类词", "品牌", "颜色", "形状", "功能"].includes(c.name)
  )
);

// ==================== 产品管理 ====================

async function loadProducts() {
  try {
    products.value = await api.getProducts();
    // 如果有产品且没有选中的产品，选中第一个
    if (products.value.length > 0 && !selectedProduct.value) {
      selectedProduct.value = products.value[0];
    }
  } catch (e) {
    ElMessage.error("加载产品失败: " + e);
  }
}

function selectProduct(product: Product) {
  selectedProduct.value = product;
  currentPage.value = 1;
  keywordPage.value = 1;
  allRootsForCloud.value = []; // 清空词云数据
  keywordData.value = []; // 清空关键词数据
  loadStats();
  loadWorkflowStatus();

  // 根据当前视图加载数据
  if (viewMode.value === 'keywords') {
    loadKeywordData();
  } else if (viewMode.value === 'roots') {
    loadRoots();
  } else if (viewMode.value === 'wordcloud') {
    loadAllRootsForCloud();
  }
}

function openAddProductDialog() {
  productForm.value = { id: 0, name: "", country: "" };
  isEditingProduct.value = false;
  showProductDialog.value = true;
}

function openEditProductDialog(product: Product) {
  productForm.value = {
    id: product.id,
    name: product.name,
    country: product.country || "",
  };
  isEditingProduct.value = true;
  showProductDialog.value = true;
}

async function saveProduct() {
  if (!productForm.value.name.trim()) {
    ElMessage.warning("请输入产品名称");
    return;
  }

  try {
    if (isEditingProduct.value) {
      await api.updateProduct(
        productForm.value.id,
        productForm.value.name,
        productForm.value.country || undefined
      );
      ElMessage.success("产品已更新");
    } else {
      const newId = await api.createProduct(
        productForm.value.name,
        productForm.value.country || undefined
      );
      ElMessage.success("产品已创建");
      // 选中新创建的产品
      await loadProducts();
      const newProduct = products.value.find((p) => p.id === newId);
      if (newProduct) {
        selectedProduct.value = newProduct;
      }
    }
    showProductDialog.value = false;
    await loadProducts();
  } catch (e) {
    ElMessage.error("保存失败: " + e);
  }
}

async function deleteProduct(product: Product) {
  try {
    await ElMessageBox.confirm(
      `确定要删除产品"${product.name}"吗？该产品下的所有关键词和词根数据都会被删除！`,
      "警告",
      {
        confirmButtonText: "确定删除",
        cancelButtonText: "取消",
        type: "warning",
      }
    );

    await api.deleteProduct(product.id);
    ElMessage.success("产品已删除");

    // 如果删除的是当前选中的产品，重新选择
    if (selectedProduct.value?.id === product.id) {
      selectedProduct.value = null;
    }
    await loadProducts();

    // 重新加载数据
    if (selectedProduct.value) {
      await loadRoots();
      await loadStats();
    } else {
      roots.value = [];
      total.value = 0;
      stats.value = { keywordCount: 0, rootCount: 0 };
    }
  } catch (e) {
    if (e !== "cancel") {
      ElMessage.error("删除失败: " + e);
    }
  }
}

// ==================== 分类和词根 ====================

async function loadCategories() {
  try {
    categories.value = await api.getCategories();
  } catch (e) {
    ElMessage.error("加载分类失败: " + e);
  }
}

async function loadRoots() {
  if (!selectedProduct.value) {
    roots.value = [];
    total.value = 0;
    return;
  }

  loading.value = true;
  try {
    const [data, count] = await api.getRoots({
      productId: selectedProduct.value.id,
      search: searchText.value || undefined,
      categoryIds: selectedCategories.value.length
        ? selectedCategories.value
        : undefined,
      sortBy: sortBy.value,
      sortOrder: sortOrder.value,
      page: currentPage.value,
      pageSize: pageSize.value,
    });
    roots.value = data;
    total.value = count;
  } catch (e) {
    ElMessage.error("加载词根失败: " + e);
  } finally {
    loading.value = false;
  }
}

// 加载所有词根用于词云显示
async function loadAllRootsForCloud() {
  if (!selectedProduct.value) {
    allRootsForCloud.value = [];
    return;
  }

  loadingCloud.value = true;
  try {
    const [data] = await api.getRoots({
      productId: selectedProduct.value.id,
      sortBy: 'contains_count',
      sortOrder: 'desc',
      page: 1,
      pageSize: 500, // 加载前500个词根用于词云
    });
    allRootsForCloud.value = data;
  } catch (e) {
    console.error("加载词云数据失败:", e);
  } finally {
    loadingCloud.value = false;
  }
}

// 切换视图模式
function switchViewMode(mode: 'dashboard' | 'keywords' | 'roots' | 'wordcloud' | 'monitoring' | 'smartcopy' | 'knowledge' | 'ads' | 'agent') {
  viewMode.value = mode;
  if (mode === 'wordcloud' && allRootsForCloud.value.length === 0) {
    loadAllRootsForCloud();
  } else if (mode === 'keywords' && keywordData.value.length === 0) {
    loadKeywordData();
  } else if (mode === 'roots' && roots.value.length === 0) {
    loadRoots();
  }
  // dashboard, monitoring 和 knowledge 视图由组件自行加载数据
}

// 处理市场调研通知点击详情
function handleNotificationDetails(_notification: { taskId: number; taskName: string; runId: number }) {
  // 切换到智能体视图（仅在启用智能体功能时）
  if (enableAgent) {
    switchViewMode('agent');
  }
}

// 词云点击处理
function handleWordCloudClick(word: string) {
  searchText.value = word;
  viewMode.value = 'roots';
  handleSearch();
}

// ==================== 关键词数据管理 ====================

async function loadKeywordData() {
  if (!selectedProduct.value) {
    keywordData.value = [];
    keywordTotal.value = 0;
    return;
  }

  keywordLoading.value = true;
  try {
    const [data, count] = await api.getKeywordData({
      productId: selectedProduct.value.id,
      search: keywordSearch.value || undefined,
      trafficLevels: keywordFilters.value.trafficLevel,
      relevanceLevels: keywordFilters.value.relevanceLevel,
      primaryCategories: keywordFilters.value.primaryCategory,
      orderlinessValues: keywordFilters.value.orderliness,
      sortBy: keywordSortBy.value,
      sortOrder: keywordSortOrder.value,
      page: keywordPage.value,
      pageSize: keywordPageSize.value,
    });
    keywordData.value = data;
    keywordTotal.value = count;
  } catch (e) {
    ElMessage.error("加载关键词数据失败: " + e);
  } finally {
    keywordLoading.value = false;
  }
}

function handleKeywordPageChange(page: number) {
  keywordPage.value = page;
  loadKeywordData();
}

function handleKeywordSizeChange(size: number) {
  keywordPageSize.value = size;
  keywordPage.value = 1;
  loadKeywordData();
}

function handleKeywordSortChange({ prop, order }: { prop: string; order: string | null }) {
  if (order) {
    keywordSortBy.value = prop;
    keywordSortOrder.value = order === "ascending" ? "asc" : "desc";
  } else {
    // 取消排序时恢复默认
    keywordSortBy.value = "id";
    keywordSortOrder.value = "asc";
  }
  keywordPage.value = 1;
  loadKeywordData();
}

async function loadStats() {
  if (!selectedProduct.value) {
    stats.value = { keywordCount: 0, rootCount: 0 };
    categoryCounts.value = new Map();
    return;
  }

  try {
    const [keywordCount, rootCount] = await api.getStats(selectedProduct.value.id);
    stats.value = { keywordCount, rootCount };

    // 加载分类统计
    const counts = await api.getCategoryCounts(selectedProduct.value.id);
    categoryCounts.value = new Map(counts);
  } catch (e) {
    console.error("加载统计失败:", e);
  }
}

// ==================== 流程状态 ====================

async function loadWorkflowStatus() {
  if (!selectedProduct.value) {
    workflowStatus.value = {
      has_data: false,
      has_traffic_level: false,
      has_category: false,
      has_phrase_tag: false,
      has_orderliness: false,
    };
    return;
  }

  try {
    workflowStatus.value = await api.getWorkflowStatus(selectedProduct.value.id);
  } catch (e) {
    console.error("加载流程状态失败:", e);
  }
}

// 获取工作流状态文字
function getWorkflowStatusText(): { text: string; type: 'warning' | 'info' | 'success' } {
  const s = workflowStatus.value;
  if (!s.has_data) return { text: '待导入', type: 'warning' };
  if (!s.has_traffic_level) return { text: '待设流量', type: 'info' };
  if (!s.has_category) return { text: '待分类', type: 'info' };
  if (!s.has_phrase_tag) return { text: '待打标', type: 'info' };
  if (!s.has_orderliness) return { text: '待排序', type: 'info' };
  return { text: '已完成 ✓', type: 'success' };
}

// ==================== 流量设置功能 ====================

function openTrafficDialog() {
  if (!selectedProduct.value) {
    ElMessage.warning("请先选择一个产品");
    return;
  }
  showTrafficDialog.value = true;
}

async function onTrafficApplied(bigThreshold: number, mediumThreshold: number) {
  if (!selectedProduct.value) return;

  // 更新本地产品数据
  selectedProduct.value.big_word_threshold = bigThreshold;
  selectedProduct.value.medium_word_threshold = mediumThreshold;

  // 更新产品列表
  const idx = products.value.findIndex(p => p.id === selectedProduct.value!.id);
  if (idx >= 0) {
    products.value[idx].big_word_threshold = bigThreshold;
    products.value[idx].medium_word_threshold = mediumThreshold;
  }

  // 重新加载数据
  await loadKeywordData();
}

// ==================== 导入功能 ====================

async function processExcelBuffer(buffer: ArrayBuffer) {
  if (!selectedProduct.value) {
    ElMessage.warning("请先选择或创建一个产品");
    return;
  }

  const workbook = XLSX.read(buffer, { type: "array" });
  const sheetName = workbook.SheetNames[0];
  const sheet = workbook.Sheets[sheetName];
  const data = XLSX.utils.sheet_to_json<{ [key: string]: any }>(sheet, { header: 1 });

  if (data.length < 2) {
    ElMessage.warning("Excel中没有数据");
    return;
  }

  // 获取表头（第一行）
  const headers = data[0] as string[];
  const rows = data.slice(1) as any[][];

  // 解析关键词和完整数据
  const keywords: string[] = [];
  const keywordDataList: KeywordData[] = [];

  // 找出P列之后的ASIN列
  const asinColumns: { index: number; name: string }[] = [];
  for (let i = 16; i < headers.length; i++) {
    if (headers[i]) {
      asinColumns.push({ index: i, name: String(headers[i]) });
    }
  }

  for (const row of rows) {
    const keyword = row[0] ? String(row[0]).trim() : "";
    if (!keyword) continue;

    keywords.push(keyword);

    // 收集ASIN数据
    const asinData: { [key: string]: any } = {};
    for (const col of asinColumns) {
      if (row[col.index] !== undefined && row[col.index] !== null) {
        asinData[col.name] = row[col.index];
      }
    }

    // 列映射 (A-P):
    // A(0): 关键词, B(1): 翻译, C(2): 相关性得分, D(3): 相关性档位
    // E(4): 流量总和, F(5): 周平均关键词排名, G(6): 周平均搜索量
    // H(7): CPC建议竞价, I(8): 建议竞价范围, J(9): 点击转化率/周
    // K(10): 周转化竞争, L(11): 竞争度档位, M(12): 自然位流动率%
    // N(13): Top3周平均点击份额, O(14): 周平均转化份额, P(15): asin数量
    const kwData: KeywordData = {
      id: 0,
      product_id: selectedProduct.value.id,
      keyword: keyword,                                                    // A
      translation: row[1] ? String(row[1]) : null,                        // B
      relevance_score: row[2] !== undefined ? String(row[2]) : null,      // C: 相关性得分
      relevance_level: row[3] ? String(row[3]) : null,                    // D: 相关性档位
      traffic_total: row[4] !== undefined ? Number(row[4]) : null,        // E: 流量总和
      avg_keyword_rank: row[5] ? String(row[5]) : null,                   // F: 周平均关键词排名
      avg_search_volume: row[6] !== undefined ? Number(row[6]) : null,    // G: 周平均搜索量
      cpc_bid: row[7] ? String(row[7]) : null,                            // H: CPC建议竞价
      bid_range: row[8] ? String(row[8]) : null,                          // I: 建议竞价范围
      click_rate: row[9] ? String(row[9]) : null,                         // J: 点击转化率/周
      conversion_competition: row[10] ? String(row[10]) : null,           // K: 周转化竞争
      competition_level: row[11] ? String(row[11]) : null,                // L: 竞争度档位
      natural_position_flow: row[12] ? String(row[12]) : null,            // M: 自然位流动率%
      top3_click_share: row[13] ? String(row[13]) : null,                 // N: Top3周平均点击份额
      avg_conversion_share: row[14] ? String(row[14]) : null,             // O: 周平均转化份额
      asin_count: row[15] !== undefined ? Number(row[15]) : null,         // P: asin数量
      // 新增计算列（暂为空）
      traffic_level: null,
      negative_word: null,
      orderliness: null,
      phrase_tag: null,
      primary_category: null,
      secondary_category: null,
      search_intent: null,
      traffic_share: null,
      asin_data: Object.keys(asinData).length > 0 ? JSON.stringify(asinData) : null,
    };
    keywordDataList.push(kwData);
  }

  if (keywords.length === 0) {
    ElMessage.warning("Excel中没有找到关键词");
    return;
  }

  // 保存Excel表头（CPC和竞价范围列包含货币符号）
  const cpcHeader = headers[7] ? String(headers[7]) : null;
  const bidRangeHeader = headers[8] ? String(headers[8]) : null;
  if (cpcHeader || bidRangeHeader) {
    await api.updateProductHeaders(selectedProduct.value.id, cpcHeader || undefined, bidRangeHeader || undefined);
    // 更新本地产品数据
    selectedProduct.value.cpc_header = cpcHeader;
    selectedProduct.value.bid_range_header = bidRangeHeader;
    // 同步更新产品列表
    const idx = products.value.findIndex(p => p.id === selectedProduct.value!.id);
    if (idx >= 0) {
      products.value[idx].cpc_header = cpcHeader;
      products.value[idx].bid_range_header = bidRangeHeader;
    }
  }

  // 检查是否有现有数据，如果有则自动备份
  const hasExistingData = stats.value.keywordCount > 0;
  if (hasExistingData) {
    try {
      await ElMessageBox.confirm(
        '导入新数据将覆盖现有数据。系统将自动创建备份，您可以随时回滚到当前版本。',
        '导入确认',
        {
          confirmButtonText: '确认导入',
          cancelButtonText: '取消',
          type: 'warning',
        }
      );

      // 创建自动备份
      const timestamp = new Date().toLocaleString('zh-CN');
      await api.createBackup(
        selectedProduct.value.id,
        `导入前自动备份 - ${timestamp}`
      );
      ElMessage.success('已创建备份');
    } catch (e) {
      if (e === 'cancel') {
        ElMessage.info('已取消导入');
        return;
      }
      throw e;
    }
  }

  // 导入完整关键词数据
  await api.importKeywordData(selectedProduct.value.id, keywordDataList);

  // 同时导入关键词用于词根分析
  await api.importKeywords(selectedProduct.value.id, keywords);

  ElMessage.success(`成功导入 ${keywords.length} 个关键词到"${selectedProduct.value.name}"`);

  // 自动计算流量级别（使用产品保存的阈值或默认值）
  const bigThreshold = selectedProduct.value.big_word_threshold || 20000;
  const mediumThreshold = selectedProduct.value.medium_word_threshold || 100000;
  await api.calculateTrafficLevels(selectedProduct.value.id, bigThreshold, mediumThreshold);

  // 自动计算流量占比
  await api.calculateTrafficShare(selectedProduct.value.id);

  // 刷新数据
  await loadKeywordData();
  await loadRoots();
  await loadStats();
  await loadWorkflowStatus();
}

async function handleImport() {
  if (!selectedProduct.value) {
    ElMessage.warning("请先选择或创建一个产品");
    return;
  }

  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Excel", extensions: ["xlsx", "xls"] }],
    });

    if (!selected) return;

    importing.value = true;
    const fileData = await readFile(selected);
    const buffer = fileData.buffer;
    await processExcelBuffer(buffer);
  } catch (e) {
    ElMessage.error("导入失败: " + e);
  } finally {
    importing.value = false;
  }
}

// ==================== 备份管理 ====================

async function loadBackups() {
  if (!selectedProduct.value) {
    backups.value = [];
    return;
  }
  try {
    backups.value = await api.getBackups(selectedProduct.value.id);
  } catch (e) {
    console.error('Failed to load backups:', e);
  }
}

async function openBackupDialog() {
  if (!selectedProduct.value) {
    ElMessage.warning("请先选择一个产品");
    return;
  }
  await loadBackups();
  showBackupDialog.value = true;
}

async function handleRestoreBackup(backup: BackupInfo) {
  const backupName = backup.backup_name || new Date(backup.created_at).toLocaleString('zh-CN');
  try {
    await ElMessageBox.confirm(
      `确定要回滚到"${backupName}"吗？当前数据将被覆盖！`,
      '确认回滚',
      {
        confirmButtonText: '确认回滚',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );

    restoring.value = true;
    await api.restoreBackup(backup.id);
    ElMessage.success('数据已成功回滚');

    // 刷新所有数据
    await loadKeywordData();
    await loadRoots();
    await loadStats();
    await loadWorkflowStatus();
    showBackupDialog.value = false;
  } catch (e) {
    if (e !== 'cancel') {
      ElMessage.error('回滚失败: ' + e);
    }
  } finally {
    restoring.value = false;
  }
}

async function handleDeleteBackup(backup: BackupInfo) {
  try {
    await ElMessageBox.confirm(
      '确定要删除此备份吗？删除后无法恢复！',
      '确认删除',
      {
        confirmButtonText: '确认删除',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );

    await api.deleteBackup(backup.id);
    ElMessage.success('备份已删除');
    await loadBackups();
  } catch (e) {
    if (e !== 'cancel') {
      ElMessage.error('删除失败: ' + e);
    }
  }
}

// 拖拽导入
let unlistenDragDrop: UnlistenFn | null = null;

async function setupDragDrop() {
  const webview = getCurrentWebview();
  unlistenDragDrop = await webview.onDragDropEvent(async (event) => {
    // 全局 Excel 拖拽导入只在关键词/词根视图起作用
    if (!['keywords', 'roots'].includes(viewMode.value)) {
      isDragging.value = false;
      return;
    }

    if (event.payload.type === "over") {
      isDragging.value = true;
    } else if (event.payload.type === "leave") {
      isDragging.value = false;
    } else if (event.payload.type === "drop") {
      isDragging.value = false;

      if (!selectedProduct.value) {
        ElMessage.warning("请先选择或创建一个产品");
        return;
      }

      const paths = event.payload.paths;
      if (paths.length === 0) return;

      const filePath = paths[0];
      const validExtensions = [".xlsx", ".xls"];
      const isValidFile = validExtensions.some((ext) =>
        filePath.toLowerCase().endsWith(ext)
      );

      if (!isValidFile) {
        ElMessage.warning("请拖入Excel文件（.xlsx或.xls）");
        return;
      }

      try {
        importing.value = true;
        const fileData = await readFile(filePath);
        const buffer = fileData.buffer;
        await processExcelBuffer(buffer);
      } catch (e) {
        ElMessage.error("导入失败: " + e);
      } finally {
        importing.value = false;
      }
    }
  });
}

// ==================== 其他功能 ====================

async function handleClearData() {
  if (!selectedProduct.value) {
    ElMessage.warning("请先选择一个产品");
    return;
  }

  try {
    await ElMessageBox.confirm(
      `确定要重置"${selectedProduct.value.name}"的词库吗？所有关键词和词根数据都会被删除，此操作不可恢复！`,
      "重置词库",
      {
        confirmButtonText: "确定重置",
        cancelButtonText: "取消",
        type: "warning",
      }
    );

    await api.clearProductData(selectedProduct.value.id);
    ElMessage.success("词库已重置");
    await loadKeywordData();
    await loadRoots();
    await loadStats();
  } catch (e) {
    if (e !== "cancel") {
      ElMessage.error("重置失败: " + e);
    }
  }
}

async function handleExport() {
  if (!selectedProduct.value) {
    ElMessage.warning("请先选择一个产品");
    return;
  }

  if (stats.value.rootCount === 0) {
    ElMessage.warning("当前产品没有词根数据可导出");
    return;
  }

  try {
    // 生成默认文件名
    const defaultFileName = `${selectedProduct.value.name}_词库分析_${new Date().toISOString().slice(0, 10)}.xlsx`;

    // 弹出保存对话框让用户选择位置
    const filePath = await save({
      defaultPath: defaultFileName,
      filters: [{ name: "Excel文件", extensions: ["xlsx"] }],
    });

    // 用户取消了保存
    if (!filePath) {
      return;
    }

    exporting.value = true;

    // 获取所有词根数据（使用大的pageSize获取全部）
    const [allRoots] = await api.getRoots({
      productId: selectedProduct.value.id,
      page: 1,
      pageSize: 100000, // 获取全部数据
    });

    // 准备Excel数据
    const exportData = allRoots.map((root) => ({
      词根: root.word,
      中文翻译: root.translation || "",
      词根长度: root.word.length,
      包含词数: root.contains_count,
      词根占比: root.percentage.toFixed(2) + "%",
      分类: root.categories.map((id) => getCategoryName(id)).join(", "),
    }));

    // 创建工作簿
    const worksheet = XLSX.utils.json_to_sheet(exportData);
    const workbook = XLSX.utils.book_new();
    XLSX.utils.book_append_sheet(workbook, worksheet, "词根分析");

    // 设置列宽
    worksheet["!cols"] = [
      { wch: 20 }, // 词根
      { wch: 25 }, // 中文翻译
      { wch: 10 }, // 词根长度
      { wch: 10 }, // 包含词数
      { wch: 10 }, // 词根占比
      { wch: 30 }, // 分类
    ];

    // 生成二进制数据
    const excelBuffer = XLSX.write(workbook, { bookType: "xlsx", type: "array" });

    // 使用 Tauri API 写入文件
    await writeFile(filePath, new Uint8Array(excelBuffer));

    ElMessage.success(`成功导出 ${allRoots.length} 条词根数据`);
  } catch (e) {
    ElMessage.error("导出失败: " + e);
  } finally {
    exporting.value = false;
  }
}

// 打开关键词导出弹窗
function handleKeywordExport() {
  if (!selectedProduct.value) {
    ElMessage.warning("请先选择一个产品");
    return;
  }

  if (keywordTotal.value === 0) {
    ElMessage.warning("当前产品没有关键词数据可导出");
    return;
  }

  // 默认选择筛选结果（如果有筛选条件）或全部数据
  keywordExportScope.value = hasActiveFilters.value ? 'filtered' : 'all';
  showKeywordExportDialog.value = true;
}

// 执行关键词导出
async function executeKeywordExport() {
  if (!selectedProduct.value) return;

  try {
    // 生成默认文件名（过滤特殊字符）
    const safeName = selectedProduct.value.name.replace(/[<>:"/\\|?*]/g, '_');
    const scopeSuffix = keywordExportScope.value === 'filtered' ? '_筛选结果' : '';
    const defaultFileName = `${safeName}_关键词数据${scopeSuffix}_${new Date().toISOString().slice(0, 10)}.xlsx`;

    // 弹出保存对话框
    const filePath = await save({
      defaultPath: defaultFileName,
      filters: [{ name: "Excel文件", extensions: ["xlsx"] }],
    });

    if (!filePath) return;

    keywordExporting.value = true;
    showKeywordExportDialog.value = false;

    // 根据导出范围获取数据
    let exportData: KeywordData[];

    if (keywordExportScope.value === 'all') {
      // 导出全部数据，不带筛选条件
      const [allData] = await api.getKeywordData({
        productId: selectedProduct.value.id,
        page: 1,
        pageSize: 1000000,
      });
      exportData = allData;
    } else {
      // 导出当前筛选结果
      const [filteredData] = await api.getKeywordData({
        productId: selectedProduct.value.id,
        search: keywordSearch.value || undefined,
        trafficLevels: keywordFilters.value.trafficLevel.length > 0
          ? keywordFilters.value.trafficLevel : undefined,
        relevanceLevels: keywordFilters.value.relevanceLevel.length > 0
          ? keywordFilters.value.relevanceLevel : undefined,
        primaryCategories: keywordFilters.value.primaryCategory.length > 0
          ? keywordFilters.value.primaryCategory : undefined,
        orderlinessValues: keywordFilters.value.orderliness.length > 0
          ? keywordFilters.value.orderliness : undefined,
        sortBy: keywordSortBy.value,
        sortOrder: keywordSortOrder.value,
        page: 1,
        pageSize: 1000000,
      });
      exportData = filteredData;
    }

    if (exportData.length === 0) {
      ElMessage.warning("没有可导出的数据");
      keywordExporting.value = false;
      return;
    }

    // 获取启用的列
    const enabledColumns = columnDefinitions.filter(col => columnConfig.value[col.key]);

    // 格式化数据
    const formattedData = exportData.map(item => {
      const row: Record<string, string | number | null> = {};

      enabledColumns.forEach(col => {
        const value = item[col.key as keyof KeywordData];
        row[col.label] = formatExportCellValue(col.key, value);
      });

      return row;
    });

    // 创建工作簿
    const worksheet = XLSX.utils.json_to_sheet(formattedData);
    const workbook = XLSX.utils.book_new();
    XLSX.utils.book_append_sheet(workbook, worksheet, "关键词数据");

    // 设置列宽
    worksheet["!cols"] = enabledColumns.map(col => ({
      wch: getExportColumnWidth(col.key),
    }));

    // 生成二进制数据并写入文件
    const excelBuffer = XLSX.write(workbook, { bookType: "xlsx", type: "array" });
    await writeFile(filePath, new Uint8Array(excelBuffer));

    ElMessage.success(`成功导出 ${exportData.length} 条关键词数据`);
  } catch (e) {
    ElMessage.error("导出失败: " + e);
  } finally {
    keywordExporting.value = false;
  }
}

// 格式化导出单元格值
function formatExportCellValue(key: string, value: unknown): string | number | null {
  if (value === null || value === undefined) {
    return null;
  }

  switch (key) {
    // 流量占比：数据库存的已经是百分比值，直接加 %
    case 'traffic_share':
      return typeof value === 'number' ? value.toFixed(2) + '%' : String(value);
    // 这些列数据库存的是小数，需要乘 100 转换为百分比
    case 'click_rate':
    case 'top3_click_share':
    case 'avg_conversion_share':
      return typeof value === 'number' || !isNaN(Number(value))
        ? (Number(value) * 100).toFixed(2) + '%'
        : String(value);
    // 数值列保持原样
    case 'traffic_total':
    case 'avg_search_volume':
    case 'asin_count':
      return typeof value === 'number' ? value : String(value);
    default:
      return String(value);
  }
}

// 获取导出列宽
function getExportColumnWidth(key: string): number {
  const widthMap: Record<string, number> = {
    keyword: 30,
    translation: 25,
    traffic_level: 10,
    negative_word: 10,
    orderliness: 10,
    phrase_tag: 20,
    primary_category: 12,
    secondary_category: 12,
    search_intent: 15,
    traffic_share: 12,
    relevance_score: 12,
    relevance_level: 12,
    traffic_total: 12,
    avg_keyword_rank: 15,
    avg_search_volume: 15,
    cpc_bid: 15,
    bid_range: 20,
    click_rate: 15,
    conversion_competition: 15,
    competition_level: 12,
    natural_position_flow: 15,
    top3_click_share: 15,
    avg_conversion_share: 15,
    asin_count: 12,
  };
  return widthMap[key] || 15;
}

async function handleAIAnalysis() {
  if (!selectedProduct.value) {
    ElMessage.warning("请先选择一个产品");
    return;
  }

  try {
    const untranslatedWords = await api.getUntranslatedRoots(selectedProduct.value.id);

    if (untranslatedWords.length === 0) {
      ElMessage.info("所有词根已完成翻译和分类");
      return;
    }

    await ElMessageBox.confirm(
      `发现 ${untranslatedWords.length} 个未分析的词根，是否使用AI进行智能翻译和分类？`,
      "智能分析",
      {
        confirmButtonText: "开始分析",
        cancelButtonText: "取消",
        type: "info",
      }
    );

    analyzing.value = true;
    analysisProgress.value = { current: 0, total: untranslatedWords.length };
    analysisAbortController.value = new AbortController();

    let savedCount = 0;
    const productId = selectedProduct.value.id;

    await batchAnalyzeWords(untranslatedWords, {
      batchSize: 30,
      concurrency: 3,
      signal: analysisAbortController.value.signal,
      onProgress: (current, total) => {
        analysisProgress.value = { current, total };
      },
      onBatchComplete: async (batchResults) => {
        // 每批完成后立即保存到数据库
        const updates: [string, string, string[]][] = batchResults.map((r) => [
          r.word,
          r.translation,
          r.categories,
        ]);
        await api.batchUpdateRootAnalysis(productId, updates);
        savedCount += batchResults.length;
      },
    });

    ElMessage.success(`成功分析 ${savedCount} 个词根`);
    await loadRoots();
  } catch (e) {
    if (e !== "cancel") {
      // 检查是否是用户主动取消
      if (e instanceof DOMException && e.name === "AbortError") {
        ElMessage.info("分析已停止，已完成的部分已保存");
        await loadRoots();
      } else {
        ElMessage.error("分析失败: " + e);
      }
    }
  } finally {
    analyzing.value = false;
    analysisProgress.value = { current: 0, total: 0 };
    analysisAbortController.value = null;
  }
}

function cancelAnalysis() {
  if (analysisAbortController.value) {
    analysisAbortController.value.abort();
  }
}

// 关键词AI分类
async function handleKeywordClassify() {
  if (!selectedProduct.value) {
    ElMessage.warning("请先选择一个产品");
    return;
  }

  try {
    const uncategorizedKeywords = await api.getUncategorizedKeywords(selectedProduct.value.id);

    if (uncategorizedKeywords.length === 0) {
      ElMessage.info("所有关键词已完成分类");
      return;
    }

    await ElMessageBox.confirm(
      `发现 ${uncategorizedKeywords.length} 个未分类的关键词，是否使用AI进行智能分类？`,
      "AI分类",
      {
        confirmButtonText: "开始分类",
        cancelButtonText: "取消",
        type: "info",
      }
    );

    classifying.value = true;
    classifyProgress.value = { current: 0, total: uncategorizedKeywords.length };
    classifyAbortController.value = new AbortController();

    let savedCount = 0;
    const productId = selectedProduct.value.id;

    // 准备关键词列表
    const keywordsForAnalysis = uncategorizedKeywords.map((k) => ({
      keyword: k.keyword,
      translation: k.translation,
    }));

    await batchAnalyzeKeywordCategories(keywordsForAnalysis, {
      batchSize: 30,
      concurrency: 3,
      signal: classifyAbortController.value.signal,
      onProgress: (current, total) => {
        classifyProgress.value = { current, total };
      },
      onBatchComplete: async (batchResults) => {
        // 每批完成后立即保存到数据库
        const updates: [string, string, string, string][] = batchResults.map((r) => [
          r.keyword,
          r.primary_category,
          r.secondary_category,
          r.search_intent,
        ]);
        await api.batchUpdateKeywordCategories(productId, updates);
        savedCount += batchResults.length;
      },
    });

    ElMessage.success(`成功分类 ${savedCount} 个关键词`);
    await loadKeywordData();
  } catch (e) {
    if (e !== "cancel") {
      if (e instanceof DOMException && e.name === "AbortError") {
        ElMessage.info("分类已停止，已完成的部分已保存");
        await loadKeywordData();
      } else {
        ElMessage.error("分类失败: " + e);
      }
    }
  } finally {
    classifying.value = false;
    classifyProgress.value = { current: 0, total: 0 };
    classifyAbortController.value = null;
    await loadWorkflowStatus();
  }
}

function cancelClassify() {
  if (classifyAbortController.value) {
    classifyAbortController.value.abort();
  }
}

// 词组打标
async function handlePhraseTagging() {
  if (!selectedProduct.value) {
    ElMessage.warning("请先选择一个产品");
    return;
  }

  try {
    await ElMessageBox.confirm(
      `<div style="line-height: 1.8;">
<div style="font-weight: 500; margin-bottom: 8px;">打标规则：</div>
<div>1. <b>候选词组</b>：流量级别为「大词/中词」且相关性为「强相关/高相关」且单词数≤5</div>
<div>2. <b>打标顺序</b>：先打长词组（3-5词），再打短词组（1-2词）</div>
<div>3. <b>匹配方式</b>：关键词包含候选词组即打标</div>
<div>4. <b>不覆盖</b>：已有标签的关键词不会被覆盖</div>
</div>`,
      "词组打标",
      {
        confirmButtonText: "开始打标",
        cancelButtonText: "取消",
        type: "info",
        dangerouslyUseHTMLString: true,
      }
    );

    phraseTagging.value = true;
    await api.calculatePhraseTags(selectedProduct.value.id);
    // 打标完成后自动计算有序性
    await api.calculateOrderliness(selectedProduct.value.id);
    ElMessage.success("词组打标完成，有序性已计算");
    await loadKeywordData();
  } catch (e) {
    if (e !== "cancel") {
      ElMessage.error("打标失败: " + e);
    }
  } finally {
    phraseTagging.value = false;
    await loadWorkflowStatus();
  }
}

// ==================== 词组标签编辑 ====================

function startEditPhraseTag(row: KeywordData) {
  editingPhraseTagId.value = row.id;
  editingPhraseTagValue.value = row.phrase_tag || "";
}

async function savePhraseTag(row: KeywordData) {
  if (editingPhraseTagId.value !== row.id) return;

  try {
    await api.updateKeywordField(row.id, "phrase_tag", editingPhraseTagValue.value);
    row.phrase_tag = editingPhraseTagValue.value || null;
  } catch (e) {
    ElMessage.error("保存失败");
  }

  editingPhraseTagId.value = null;
}

function cancelEditPhraseTag() {
  editingPhraseTagId.value = null;
}

function handleSearch() {
  currentPage.value = 1;
  loadRoots();
}

function toggleCategory(id: number) {
  const index = selectedCategories.value.indexOf(id);
  if (index > -1) {
    selectedCategories.value.splice(index, 1);
  } else {
    selectedCategories.value.push(id);
  }
  currentPage.value = 1;
  loadRoots();
}

function handlePageChange(page: number) {
  currentPage.value = page;
  loadRoots();
}

function handleSizeChange(size: number) {
  pageSize.value = size;
  currentPage.value = 1;
  loadRoots();
}

function startEdit(root: Root) {
  editingId.value = root.id;
  editingTranslation.value = root.translation || "";
}

async function saveTranslation(root: Root) {
  try {
    await api.updateRootTranslation(root.id, editingTranslation.value);
    root.translation = editingTranslation.value;
    editingId.value = null;
    ElMessage.success("翻译已保存");
  } catch (e) {
    ElMessage.error("保存失败: " + e);
  }
}

function cancelEdit() {
  editingId.value = null;
}

async function toggleRootCategory(root: Root, categoryId: number) {
  try {
    const hasCategory = root.categories.includes(categoryId);
    if (hasCategory) {
      await api.removeRootCategory(root.id, categoryId);
      root.categories = root.categories.filter((id) => id !== categoryId);
    } else {
      await api.addRootCategory(root.id, categoryId);
      root.categories.push(categoryId);
    }
  } catch (e) {
    ElMessage.error("操作失败: " + e);
  }
}

function getCategoryName(id: number): string {
  const cat = categories.value.find((c) => c.id === id);
  return cat?.name || "";
}

function getCategoryCount(categoryId: number): number {
  return categoryCounts.value.get(categoryId) || 0;
}

// ==================== 主题切换 ====================

function initTheme() {
  const saved = localStorage.getItem("theme");
  isDarkMode.value = saved === "dark";
  applyTheme();
}

function applyTheme() {
  document.documentElement.classList.toggle("dark", isDarkMode.value);
}

function toggleTheme() {
  isDarkMode.value = !isDarkMode.value;
  localStorage.setItem("theme", isDarkMode.value ? "dark" : "light");
  applyTheme();
}

// ==================== 侧边栏拖动调整 ====================

function initSidebarWidth() {
  const saved = localStorage.getItem("sidebarWidth");
  if (saved) {
    const width = parseInt(saved, 10);
    if (width >= MIN_SIDEBAR_WIDTH && width <= MAX_SIDEBAR_WIDTH) {
      sidebarWidth.value = width;
    }
  }
}

function startResize(e: MouseEvent) {
  isResizing.value = true;
  document.addEventListener("mousemove", handleResize);
  document.addEventListener("mouseup", stopResize);
  document.body.style.cursor = "col-resize";
  document.body.style.userSelect = "none";
  e.preventDefault();
}

function handleResize(e: MouseEvent) {
  if (!isResizing.value) return;
  const newWidth = Math.min(Math.max(e.clientX, MIN_SIDEBAR_WIDTH), MAX_SIDEBAR_WIDTH);
  sidebarWidth.value = newWidth;
}

function stopResize() {
  isResizing.value = false;
  document.removeEventListener("mousemove", handleResize);
  document.removeEventListener("mouseup", stopResize);
  document.body.style.cursor = "";
  document.body.style.userSelect = "";
  localStorage.setItem("sidebarWidth", sidebarWidth.value.toString());
}

// ==================== 键盘快捷键 ====================

function handleKeyboard(e: KeyboardEvent) {
  // 检测 Ctrl (Windows) 或 Cmd (Mac)
  const isMod = e.ctrlKey || e.metaKey;

  // 如果正在输入框中，只响应 Escape
  const isInputting =
    document.activeElement?.tagName === "INPUT" ||
    document.activeElement?.tagName === "TEXTAREA";

  if (isInputting && e.key !== "Escape") {
    return;
  }

  // Ctrl/Cmd + N: 创建新产品
  if (isMod && e.key === "n") {
    e.preventDefault();
    openAddProductDialog();
    return;
  }

  // Ctrl/Cmd + I: 导入 Excel
  if (isMod && e.key === "i") {
    e.preventDefault();
    if (selectedProduct.value) {
      handleImport();
    }
    return;
  }

  // Ctrl/Cmd + E: 导出 Excel
  if (isMod && e.key === "e") {
    e.preventDefault();
    if (selectedProduct.value) {
      if (viewMode.value === 'keywords') {
        handleKeywordExport();
      } else {
        handleExport();
      }
    }
    return;
  }

  // Ctrl/Cmd + F: 聚焦搜索框
  if (isMod && e.key === "f") {
    e.preventDefault();
    const searchInput = document.querySelector(
      ".header-right .el-input__inner"
    ) as HTMLInputElement;
    searchInput?.focus();
    return;
  }

  // Ctrl/Cmd + Enter: 根据视图触发 AI 操作
  if (isMod && e.key === "Enter") {
    e.preventDefault();
    if (selectedProduct.value) {
      if (viewMode.value === 'keywords' && !classifying.value) {
        handleKeywordClassify();
      } else if (viewMode.value === 'roots' && !analyzing.value) {
        handleAIAnalysis();
      }
    }
    return;
  }

  // Ctrl/Cmd + D: 切换深色模式
  if (isMod && e.key === "d") {
    e.preventDefault();
    toggleTheme();
    return;
  }

  // Ctrl/Cmd + H: 显示帮助中心
  if (isMod && e.key === "h") {
    e.preventDefault();
    openHelp(viewMode.value);
    return;
  }

  // ? 或 Ctrl/Cmd + /: 显示快捷键帮助
  if (e.key === "?" || (isMod && e.key === "/")) {
    e.preventDefault();
    showShortcutsDialog.value = true;
    return;
  }

  // Escape: 关闭对话框 / 取消编辑
  if (e.key === "Escape") {
    if (showShortcutsDialog.value) {
      showShortcutsDialog.value = false;
    } else if (editingId.value !== null) {
      cancelEdit();
    }
    return;
  }

  // 上下箭头: 在产品列表中导航
  if (e.key === "ArrowUp" || e.key === "ArrowDown") {
    if (products.value.length === 0) return;

    const currentIndex = selectedProduct.value
      ? products.value.findIndex((p) => p.id === selectedProduct.value?.id)
      : -1;

    let newIndex: number;
    if (e.key === "ArrowUp") {
      newIndex = currentIndex > 0 ? currentIndex - 1 : products.value.length - 1;
    } else {
      newIndex = currentIndex < products.value.length - 1 ? currentIndex + 1 : 0;
    }

    selectProduct(products.value[newIndex]);
    return;
  }
}

// ==================== 自动更新 ====================

async function checkForUpdates() {
  try {
    const update = await check();
    if (update) {
      let userConfirmed = false;
      try {
        await ElMessageBox.confirm(
          `发现新版本 ${update.version}，是否立即更新？`,
          "版本更新",
          {
            confirmButtonText: "立即更新",
            cancelButtonText: "稍后提醒",
            type: "info",
          }
        );
        userConfirmed = true;
      } catch {
        // 用户点击取消
        return;
      }

      if (userConfirmed) {
        // 显示下载进度弹窗
        updateVersion.value = update.version;
        updateProgress.value = 0;
        updateTotal.value = 0;
        updateDownloading.value = true;
        showUpdateDialog.value = true;

        try {
          let downloaded = 0;

          await update.downloadAndInstall((progress) => {
            if (progress.event === "Started" && progress.data) {
              updateTotal.value = progress.data.contentLength || 0;
            } else if (progress.event === "Progress" && progress.data) {
              downloaded += progress.data.chunkLength || 0;
              if (updateTotal.value > 0) {
                updateProgress.value = Math.round((downloaded / updateTotal.value) * 100);
              }
            } else if (progress.event === "Finished") {
              updateProgress.value = 100;
            }
          });

          updateDownloading.value = false;

          await ElMessageBox.alert("更新已下载完成，点击确定重启应用", "更新完成", {
            confirmButtonText: "重启应用",
          });

          showUpdateDialog.value = false;
          await relaunch();
        } catch (downloadError) {
          console.error("下载更新失败:", downloadError);
          showUpdateDialog.value = false;
          updateDownloading.value = false;

          await ElMessageBox.alert(
            `下载更新失败: ${downloadError instanceof Error ? downloadError.message : "未知错误"}\n\n请稍后重试或手动下载更新。`,
            "更新失败",
            {
              confirmButtonText: "确定",
              type: "error",
            }
          );
        }
      }
    }
  } catch (e) {
    // 检查更新失败时静默处理，不影响用户使用
    console.log("检查更新失败:", e);
    showUpdateDialog.value = false;
    updateDownloading.value = false;
  }
}

// ==================== 首次启动配置向导 ====================

async function checkSetupWizard() {
  try {
    // 检查向导是否已完成
    const completed = await api.hasApiKey('__setup_wizard_completed');
    if (completed) return;

    // 检查必要的 API Key 是否已配置
    const hasDeepseek = await api.hasApiKey('deepseek');
    const hasQwen = await api.hasApiKey('qwen');

    // 如果都没配置，显示向导
    if (!hasDeepseek && !hasQwen) {
      showSetupWizard.value = true;
    }
  } catch (e) {
    console.error('检查向导状态失败:', e);
  }
}

// ==================== 初始化 ====================

onMounted(async () => {
  // 初始化主题
  initTheme();

  // 加载汇率显示设置
  loadExchangeRateSettings();

  // 检查是否需要显示首次配置向导
  await checkSetupWizard();

  // 检查 API Key 配置状态（用于功能状态提示）
  await checkApiKeyStatus();

  // 初始化侧边栏宽度
  initSidebarWidth();

  // 获取应用版本
  appVersion.value = await getVersion();

  // 注册键盘快捷键
  window.addEventListener("keydown", handleKeyboard);

  await loadProducts();
  await loadCategories();
  if (selectedProduct.value) {
    await loadStats();
    await loadWorkflowStatus();
    // 根据默认视图模式加载数据
    if (viewMode.value === 'keywords') {
      await loadKeywordData();
    } else if (viewMode.value === 'roots') {
      await loadRoots();
    }
  }
  await setupDragDrop();

  // 启动后检查更新
  checkForUpdates();
});

onUnmounted(() => {
  // 移除键盘监听
  window.removeEventListener("keydown", handleKeyboard);

  if (unlistenDragDrop) {
    unlistenDragDrop();
  }
});
</script>

<template>
  <div class="app-container">
    <!-- 拖拽遮罩 -->
    <div v-if="isDragging" class="drop-overlay">
      <div class="drop-content">
        <el-icon class="drop-icon"><Upload /></el-icon>
        <p>释放以导入Excel文件</p>
      </div>
    </div>

    <!-- 顶部导航栏 - 视图切换（始终可见，固定在最上方） -->
    <nav class="top-nav">
      <el-button-group class="view-toggle">
        <el-button
          :type="viewMode === 'dashboard' ? 'primary' : 'default'"
          @click="switchViewMode('dashboard')"
        >
          <el-icon><DataLine /></el-icon>
          概览
        </el-button>
        <el-button
          :type="viewMode === 'keywords' ? 'primary' : 'default'"
          @click="switchViewMode('keywords')"
        >
          <el-icon><Document /></el-icon>
          关键词
        </el-button>
        <el-button
          :type="viewMode === 'roots' ? 'primary' : 'default'"
          @click="switchViewMode('roots')"
        >
          <el-icon><Grid /></el-icon>
          词根
        </el-button>
        <el-button
          :type="viewMode === 'wordcloud' ? 'primary' : 'default'"
          @click="switchViewMode('wordcloud')"
        >
          <el-icon><PieChart /></el-icon>
          词云
        </el-button>
        <el-button
          :type="viewMode === 'monitoring' ? 'primary' : 'default'"
          @click="switchViewMode('monitoring')"
        >
          <el-icon><TrendCharts /></el-icon>
          排名监控
        </el-button>
        <el-button
          :type="viewMode === 'smartcopy' ? 'primary' : 'default'"
          @click="switchViewMode('smartcopy')"
        >
          <el-icon><EditPen /></el-icon>
          智能文案
        </el-button>
        <el-button
          :type="viewMode === 'ads' ? 'primary' : 'default'"
          @click="switchViewMode('ads')"
        >
          <el-icon><Promotion /></el-icon>
          智能广告
        </el-button>
        <el-button
          :type="viewMode === 'knowledge' ? 'primary' : 'default'"
          @click="switchViewMode('knowledge')"
        >
          <el-icon><ChatDotRound /></el-icon>
          知识库
        </el-button>
        <el-button
          v-if="enableAgent"
          :type="viewMode === 'agent' ? 'primary' : 'default'"
          @click="switchViewMode('agent')"
        >
          <el-icon><Cpu /></el-icon>
          智能体
        </el-button>
      </el-button-group>
      <el-dropdown trigger="click" class="global-settings-dropdown">
        <el-button>
          <el-icon><Setting /></el-icon>
          设置
          <el-icon class="el-icon--right"><ArrowDown /></el-icon>
        </el-button>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item @click="showApiKeyDialog = true">API Key</el-dropdown-item>
            <el-dropdown-item @click="showShortcutsDialog = true">快捷键</el-dropdown-item>
            <el-dropdown-item @click="showExchangeRateSettings = true">汇率显示</el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
      <el-button class="nav-help-btn" @click="showHelpDialog = true">
        <el-icon><QuestionFilled /></el-icon>
        帮助
      </el-button>
    </nav>

    <!-- 主体区域 -->
    <div class="app-body">
      <!-- 侧边栏 - 产品列表（仪表板和知识库视图时隐藏） -->
    <aside v-if="viewMode !== 'knowledge' && viewMode !== 'dashboard' && viewMode !== 'smartcopy' && viewMode !== 'ads' && viewMode !== 'agent'" class="sidebar" :style="{ width: sidebarWidth + 'px' }">
      <div class="sidebar-header">
        <span class="sidebar-title">产品列表</span>
        <el-button type="primary" size="small" circle @click="openAddProductDialog">
          <el-icon><Plus /></el-icon>
        </el-button>
      </div>
      <div class="product-list">
        <div
          v-for="product in products"
          :key="product.id"
          class="product-item"
          :class="{ active: selectedProduct?.id === product.id }"
          @click="selectProduct(product)"
        >
          <div class="product-info">
            <div class="product-name">{{ product.name }}</div>
            <div class="product-meta" v-if="product.country">
              <span class="country-flag-small" v-html="getCountryFlag(product.country)"></span>
              <span>{{ getCountryName(product.country) }}</span>
            </div>
            <!-- 工作流状态标签 -->
            <div class="product-status" v-if="selectedProduct?.id === product.id">
              <el-tag size="small" :type="getWorkflowStatusText().type">
                {{ getWorkflowStatusText().text }}
              </el-tag>
            </div>
          </div>
          <el-dropdown trigger="click" @click.stop>
            <el-button size="small" text class="product-action">
              <el-icon><MoreFilled /></el-icon>
            </el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item @click="openEditProductDialog(product)">
                  <el-icon><Edit /></el-icon> 编辑
                </el-dropdown-item>
                <el-dropdown-item @click="deleteProduct(product)" divided>
                  <el-icon color="#f56c6c"><Delete /></el-icon>
                  <span style="color: #f56c6c">删除</span>
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
        <div v-if="products.length === 0" class="empty-state sidebar-empty">
          <div class="empty-icon">
            <el-icon :size="48"><Box /></el-icon>
          </div>
          <p class="empty-title">还没有产品</p>
          <p class="empty-desc">创建第一个产品开始分析</p>
          <el-button type="primary" @click="openAddProductDialog">
            <el-icon><Plus /></el-icon>
            新建产品
          </el-button>
        </div>
      </div>
      <div class="sidebar-footer">
        <el-button text @click="toggleTheme" class="theme-toggle">
          <el-icon><Sunny v-if="isDarkMode" /><Moon v-else /></el-icon>
          <span>{{ isDarkMode ? '浅色模式' : '深色模式' }}</span>
        </el-button>
      </div>
    </aside>

    <!-- 拖动调整手柄（仪表板、知识库、智能文案视图时隐藏） -->
    <div
      v-if="viewMode !== 'knowledge' && viewMode !== 'dashboard' && viewMode !== 'smartcopy' && viewMode !== 'ads' && viewMode !== 'agent'"
      class="resize-handle"
      :class="{ resizing: isResizing }"
      @mousedown="startResize"
    ></div>

    <!-- 主内容区 -->
    <main class="main-content">
      <!-- 顶部工具栏（仪表板、知识库、智能文案视图时隐藏） -->
      <header v-if="viewMode !== 'knowledge' && viewMode !== 'dashboard' && viewMode !== 'smartcopy' && viewMode !== 'ads' && viewMode !== 'agent'" class="header">
        <div class="header-left">
          <h1 class="title">{{ selectedProduct?.name || '请选择产品' }}</h1>
          <div class="header-stats" v-if="selectedProduct">
            <span>关键词: {{ stats.keywordCount }}</span>
            <span>词根: {{ stats.rootCount }}</span>
          </div>
        </div>

        <!-- 一级分类标签 - 仅词根视图显示 -->
        <div class="category-tabs" v-if="selectedProduct && viewMode === 'roots'">
          <el-tag
            v-for="cat in primaryCategories"
            :key="cat.id"
            :type="selectedCategories.includes(cat.id) ? '' : 'info'"
            :effect="selectedCategories.includes(cat.id) ? 'dark' : 'plain'"
            class="category-tag"
            @click="toggleCategory(cat.id)"
          >
            {{ cat.name }}({{ getCategoryCount(cat.id) }})
          </el-tag>

          <el-dropdown trigger="click">
            <el-tag type="info" effect="plain" class="category-tag more-tag">
              更多
            </el-tag>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item
                  v-for="cat in secondaryCategories"
                  :key="cat.id"
                  @click="toggleCategory(cat.id)"
                >
                  <el-icon v-if="selectedCategories.includes(cat.id)"><Check /></el-icon>
                  {{ cat.name }}({{ getCategoryCount(cat.id) }})
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>

        <!-- 搜索框 - 仅词根视图显示 -->
        <div class="header-right" v-if="selectedProduct && viewMode === 'roots'">
          <el-input
            v-model="searchText"
            placeholder="搜索词根"
            clearable
            style="width: 200px"
            @keyup.enter="handleSearch"
            @clear="handleSearch"
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
        </div>

        <!-- 操作按钮区域 -->
        <div class="header-actions" v-if="selectedProduct">
          <!-- 工作流操作 - 关键词视图 -->
          <template v-if="viewMode === 'keywords'">
            <el-button size="small" @click="openTrafficDialog">
              流量设置
            </el-button>
            <el-tooltip
              v-if="!classifying"
              :disabled="apiKeyStatus.deepseek"
              content="请先在设置中配置 DeepSeek API Key"
              placement="bottom"
            >
              <el-button
                size="small"
                :disabled="!apiKeyStatus.deepseek"
                @click="handleKeywordClassify"
              >
                AI分类
              </el-button>
            </el-tooltip>
            <el-button
              v-else
              size="small"
              type="danger"
              @click="cancelClassify"
            >
              <el-icon><Close /></el-icon>
              停止 ({{ classifyProgress.current }}/{{ classifyProgress.total }})
            </el-button>
            <el-button
              size="small"
              :loading="phraseTagging"
              @click="handlePhraseTagging"
            >
              {{ phraseTagging ? '打标中...' : '词组打标' }}
            </el-button>
          </template>

          <!-- 工作流操作 - 词根视图 -->
          <template v-if="viewMode === 'roots'">
            <el-tooltip
              v-if="!analyzing"
              :disabled="apiKeyStatus.deepseek"
              content="请先在设置中配置 DeepSeek API Key"
              placement="bottom"
            >
              <el-button
                size="small"
                type="success"
                :disabled="!apiKeyStatus.deepseek"
                @click="handleAIAnalysis"
              >
                <el-icon><MagicStick /></el-icon>
                智能分析
              </el-button>
            </el-tooltip>
            <el-button
              v-else
              size="small"
              type="danger"
              @click="cancelAnalysis"
            >
              <el-icon><Close /></el-icon>
              停止 ({{ analysisProgress.current }}/{{ analysisProgress.total }})
            </el-button>
          </template>

          <!-- 排名监控视图 - 独立按钮 -->
          <template v-if="viewMode === 'monitoring'">
            <el-divider direction="vertical" />
            <el-button size="small" @click="openSettingsTab('monitoring')">
              <el-icon><Setting /></el-icon>
              监控设置
            </el-button>
            <el-button size="small" @click="openSettingsTab('auto')">
              <el-icon><Timer /></el-icon>
              自动检测
            </el-button>
            <el-button size="small" @click="openSettingsTab('logs')">
              <el-icon><Document /></el-icon>
              任务记录
            </el-button>
          </template>

          <!-- 其他视图 - 数据管理下拉菜单 -->
          <template v-else>
            <el-divider direction="vertical" />
            <el-dropdown trigger="click">
              <el-button size="small">
                <el-icon><FolderOpened /></el-icon>
                数据
                <el-icon class="el-icon--right"><ArrowDown /></el-icon>
              </el-button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item @click="handleImport" :disabled="importing">
                    <el-icon><Upload /></el-icon> 导入Excel
                  </el-dropdown-item>
                  <el-dropdown-item
                    v-if="viewMode === 'keywords'"
                    @click="handleKeywordExport"
                    :disabled="keywordExporting"
                  >
                    <el-icon><Download /></el-icon> 导出关键词
                  </el-dropdown-item>
                  <el-dropdown-item
                    v-else
                    @click="handleExport"
                    :disabled="exporting"
                  >
                    <el-icon><Download /></el-icon> 导出词根
                  </el-dropdown-item>
                  <el-dropdown-item divided @click="openBackupDialog">
                    <el-icon><FolderOpened /></el-icon> 备份管理
                  </el-dropdown-item>
                  <el-dropdown-item divided @click="handleClearData">
                    <el-icon color="#f56c6c"><Delete /></el-icon>
                    <span style="color: #f56c6c">重置词库</span>
                  </el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </template>

          <el-button v-if="viewMode === 'keywords'" size="small" @click="showColumnConfig = true">
            <el-icon><Setting /></el-icon>
            列配置
          </el-button>
        </div>
      </header>

      <!-- 关键词筛选栏 -->
      <div class="keyword-filter-bar" v-if="selectedProduct && viewMode === 'keywords'">
        <el-input
          v-model="keywordSearch"
          placeholder="搜索关键词..."
          clearable
          style="width: 200px"
          @clear="handleFilterChange"
          @keyup.enter="handleFilterChange"
        >
          <template #prefix>
            <el-icon><Search /></el-icon>
          </template>
        </el-input>

        <el-select
          v-model="keywordFilters.trafficLevel"
          multiple
          collapse-tags
          collapse-tags-tooltip
          placeholder="流量级别"
          style="width: 140px"
          @change="handleFilterChange"
        >
          <el-option v-for="opt in trafficLevelOptions" :key="opt" :label="opt" :value="opt" />
        </el-select>

        <el-select
          v-model="keywordFilters.relevanceLevel"
          multiple
          collapse-tags
          collapse-tags-tooltip
          placeholder="相关性"
          style="width: 140px"
          @change="handleFilterChange"
        >
          <el-option v-for="opt in relevanceLevelOptions" :key="opt" :label="opt" :value="opt" />
        </el-select>

        <el-select
          v-model="keywordFilters.primaryCategory"
          multiple
          collapse-tags
          collapse-tags-tooltip
          placeholder="一级分类"
          style="width: 140px"
          @change="handleFilterChange"
        >
          <el-option v-for="opt in primaryCategoryOptions" :key="opt" :label="opt" :value="opt" />
        </el-select>

        <el-select
          v-model="keywordFilters.orderliness"
          multiple
          collapse-tags
          collapse-tags-tooltip
          placeholder="有序性"
          style="width: 120px"
          @change="handleFilterChange"
        >
          <el-option v-for="opt in orderlinessOptions" :key="opt" :label="opt" :value="opt" />
        </el-select>

        <el-button v-if="hasActiveFilters" text type="primary" @click="resetKeywordFilters">
          重置筛选
        </el-button>

        <span v-if="hasActiveFilters" class="filter-result-count">
          共 {{ keywordTotal }} 条结果
        </span>

        <!-- 批量操作区域 -->
        <el-divider direction="vertical" v-if="selectedKeywords.length > 0" />
        <el-button
          v-if="selectedKeywords.length > 0"
          type="success"
          @click="showQuickAddMonitoringDialog = true"
        >
          <el-icon><Plus /></el-icon>
          添加到监控 ({{ selectedKeywords.length }})
        </el-button>
        <el-button
          v-if="selectedKeywords.length > 0"
          text
          @click="clearKeywordSelection"
        >
          取消选择
        </el-button>
      </div>

      <!-- 关键词表格 -->
      <div class="keyword-table-container" v-if="selectedProduct && viewMode === 'keywords'">
        <el-table
          ref="keywordTableRef"
          :data="keywordData"
          v-loading="keywordLoading"
          stripe
          style="width: 100%"
          height="100%"
          @sort-change="handleKeywordSortChange"
          @selection-change="handleKeywordSelectionChange"
        >
          <template #empty>
            <div class="table-empty-state">
              <div class="empty-icon">
                <el-icon :size="48"><Document /></el-icon>
              </div>
              <p class="empty-title">{{ hasActiveFilters ? '没有匹配的数据' : '还没有关键词数据' }}</p>
              <p class="empty-desc">{{ hasActiveFilters ? '尝试调整筛选条件' : '导入 Excel 文件开始分析' }}</p>
              <el-button v-if="!hasActiveFilters" type="primary" @click="handleImport">
                <el-icon><Upload /></el-icon>
                导入 Excel
              </el-button>
              <el-button v-else @click="resetKeywordFilters">
                <el-icon><RefreshLeft /></el-icon>
                重置筛选
              </el-button>
            </div>
          </template>
          <el-table-column type="selection" width="40" fixed="left" />
          <el-table-column type="index" label="#" width="50" fixed="left" />

          <!-- 原始Excel列 -->
          <el-table-column prop="keyword" label="关键词" min-width="220" fixed="left" show-overflow-tooltip>
            <template #default="{ row }">
              <div class="keyword-cell">
                <span
                  class="keyword-link"
                  @click="openAmazonSearch(row.keyword)"
                  :title="'在 Amazon 搜索: ' + row.keyword"
                >
                  {{ row.keyword }}
                </span>
                <el-icon class="copy-icon" @click.stop="copyKeyword(row.keyword)" title="复制关键词">
                  <DocumentCopy />
                </el-icon>
              </div>
            </template>
          </el-table-column>
          <el-table-column v-if="columnConfig.translation" prop="translation" label="翻译" min-width="150" show-overflow-tooltip />

          <!-- 新增计算列 -->
          <el-table-column v-if="columnConfig.traffic_level" prop="traffic_level" label="流量级别" width="90" align="center">
            <template #default="{ row }">
              <el-tag v-if="row.traffic_level" :type="row.traffic_level === '大词' ? 'danger' : row.traffic_level === '中词' ? 'warning' : 'info'" size="small">
                {{ row.traffic_level }}
              </el-tag>
              <span v-else class="empty-cell">-</span>
            </template>
          </el-table-column>
          <el-table-column v-if="columnConfig.negative_word" prop="negative_word" label="否词" width="70" align="center">
            <template #default="{ row }">
              {{ row.negative_word || '-' }}
            </template>
          </el-table-column>
          <el-table-column v-if="columnConfig.orderliness" prop="orderliness" label="有序性" width="80" align="center">
            <template #default="{ row }">
              {{ row.orderliness || '-' }}
            </template>
          </el-table-column>
          <el-table-column v-if="columnConfig.phrase_tag" prop="phrase_tag" label="词组标签" min-width="150">
            <template #default="{ row }">
              <el-input
                v-if="editingPhraseTagId === row.id"
                v-model="editingPhraseTagValue"
                size="small"
                @blur="savePhraseTag(row)"
                @keyup.enter="savePhraseTag(row)"
                @keyup.escape="cancelEditPhraseTag"
                autofocus
              />
              <span v-else @dblclick="startEditPhraseTag(row)" class="editable-cell">
                {{ row.phrase_tag || '-' }}
              </span>
            </template>
          </el-table-column>
          <el-table-column v-if="columnConfig.primary_category" prop="primary_category" label="一级分类" width="90" align="center">
            <template #default="{ row }">
              {{ row.primary_category || '-' }}
            </template>
          </el-table-column>
          <el-table-column v-if="columnConfig.secondary_category" prop="secondary_category" label="二级分类" width="90" align="center">
            <template #default="{ row }">
              {{ row.secondary_category || '-' }}
            </template>
          </el-table-column>
          <el-table-column v-if="columnConfig.search_intent" prop="search_intent" label="搜索意图" min-width="120" show-overflow-tooltip>
            <template #default="{ row }">
              {{ row.search_intent || '-' }}
            </template>
          </el-table-column>
          <el-table-column v-if="columnConfig.traffic_share" prop="traffic_share" label="流量占比" width="90" align="right">
            <template #default="{ row }">
              {{ row.traffic_share ? row.traffic_share.toFixed(2) + '%' : '-' }}
            </template>
          </el-table-column>

          <!-- 原始数据列 (C-P) -->
          <el-table-column v-if="columnConfig.relevance_score" prop="relevance_score" label="相关性得分" width="100" align="center">
            <template #default="{ row }">
              {{ row.relevance_score || '-' }}
            </template>
          </el-table-column>
          <el-table-column v-if="columnConfig.relevance_level" prop="relevance_level" label="相关性档位" width="100" align="center">
            <template #default="{ row }">
              {{ row.relevance_level || '-' }}
            </template>
          </el-table-column>
          <el-table-column v-if="columnConfig.traffic_total" prop="traffic_total" label="流量总和" width="120" sortable="custom">
            <template #default="{ row }">
              {{ row.traffic_total !== null ? row.traffic_total.toLocaleString() : '-' }}
            </template>
          </el-table-column>
          <el-table-column v-if="columnConfig.avg_keyword_rank" prop="avg_keyword_rank" label="周平均排名" width="130" align="right" sortable="custom">
            <template #default="{ row }">
              {{ row.avg_keyword_rank || '-' }}
            </template>
          </el-table-column>
          <el-table-column v-if="columnConfig.avg_search_volume" prop="avg_search_volume" label="周平均搜索量" width="150" align="right" sortable="custom">
            <template #default="{ row }">
              {{ row.avg_search_volume !== null ? row.avg_search_volume.toLocaleString() : '-' }}
            </template>
          </el-table-column>
          <el-table-column v-if="columnConfig.cpc_bid" prop="cpc_bid" :label="selectedProduct?.cpc_header || 'CPC建议竞价'" width="130" align="center">
            <template #default="{ row }">
              {{ row.cpc_bid ? Number(row.cpc_bid).toFixed(2) : '-' }}
            </template>
          </el-table-column>
          <el-table-column v-if="columnConfig.bid_range" prop="bid_range" :label="selectedProduct?.bid_range_header || '建议竞价范围'" min-width="140" align="center" show-overflow-tooltip>
            <template #default="{ row }">
              {{ row.bid_range || '-' }}
            </template>
          </el-table-column>
          <el-table-column v-if="columnConfig.click_rate" prop="click_rate" label="均点击转化率" width="110">
            <template #default="{ row }">
              {{ row.click_rate ? (Number(row.click_rate) * 100).toFixed(2) + '%' : '-' }}
            </template>
          </el-table-column>
          <el-table-column v-if="columnConfig.conversion_competition" prop="conversion_competition" label="周转化竞争" width="100" align="center">
            <template #default="{ row }">
              {{ row.conversion_competition || '-' }}
            </template>
          </el-table-column>
          <el-table-column v-if="columnConfig.competition_level" prop="competition_level" label="竞争度档位" width="100" align="center">
            <template #default="{ row }">
              {{ row.competition_level || '-' }}
            </template>
          </el-table-column>
          <el-table-column v-if="columnConfig.natural_position_flow" prop="natural_position_flow" label="自然位流动率" width="120" align="center">
            <template #default="{ row }">
              {{ row.natural_position_flow || '-' }}
            </template>
          </el-table-column>
          <el-table-column v-if="columnConfig.top3_click_share" prop="top3_click_share" label="Top3周平均点击份额" min-width="140" align="center" show-overflow-tooltip>
            <template #default="{ row }">
              {{ row.top3_click_share ? (Number(row.top3_click_share) * 100).toFixed(2) + '%' : '-' }}
            </template>
          </el-table-column>
          <el-table-column v-if="columnConfig.avg_conversion_share" prop="avg_conversion_share" label="Top3周平均转化份额" min-width="150" align="center" show-overflow-tooltip>
            <template #default="{ row }">
              {{ row.avg_conversion_share ? (Number(row.avg_conversion_share) * 100).toFixed(2) + '%' : '-' }}
            </template>
          </el-table-column>
          <el-table-column v-if="columnConfig.asin_count" prop="asin_count" label="asin数量" width="100" align="center">
            <template #default="{ row }">
              {{ row.asin_count !== null ? row.asin_count : '-' }}
            </template>
          </el-table-column>
        </el-table>
      </div>

      <!-- 关键词分页 -->
      <div class="pagination" v-if="selectedProduct && viewMode === 'keywords'">
        <el-pagination
          v-model:current-page="keywordPage"
          v-model:page-size="keywordPageSize"
          :page-sizes="[20, 50, 100, 200]"
          :total="keywordTotal"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handleKeywordSizeChange"
          @current-change="handleKeywordPageChange"
        />
      </div>

      <!-- 词云视图 -->
      <div class="wordcloud-container" v-if="selectedProduct && viewMode === 'wordcloud'">
        <WordCloud
          ref="wordCloudRef"
          :roots="allRootsForCloud"
          :categories="categories"
          :loading="loadingCloud"
          @wordClick="handleWordCloudClick"
        />
      </div>

      <!-- 排名监控视图 -->
      <KeywordMonitoringTab
        v-if="selectedProduct && viewMode === 'monitoring'"
        :product-id="selectedProduct.id"
        @show-help="openHelp"
      />

      <!-- 智能文案视图 - 使用 keep-alive 保持状态 -->
      <keep-alive>
        <SmartCopyTab
          v-if="viewMode === 'smartcopy'"
          class="smart-copy-view"
          @show-help="openHelp"
        />
      </keep-alive>

      <!-- 知识库视图 - 使用 keep-alive 保持状态 -->
      <keep-alive>
        <KnowledgeBaseTab
          v-if="viewMode === 'knowledge'"
          class="knowledge-base-view"
          @show-help="openHelp"
        />
      </keep-alive>

      <!-- 智能广告视图 - 使用 keep-alive 保持状态 -->
      <keep-alive>
        <AdOptimizerTab
          v-if="viewMode === 'ads'"
          class="ad-optimizer-view"
          @show-help="openHelp"
        />
      </keep-alive>

      <!-- 智能体视图 - 使用 keep-alive 保持状态（仅开发环境可用） -->
      <keep-alive v-if="enableAgent">
        <AgentTab
          v-if="viewMode === 'agent'"
          class="agent-view"
          @show-help="openHelp"
        />
      </keep-alive>

      <!-- 仪表板视图 -->
      <DashboardTab
        v-if="viewMode === 'dashboard'"
        :selected-product="selectedProduct"
        class="dashboard-view"
        @switch-view="switchViewMode"
        @show-help="openHelp"
      />

      <!-- 词根表格 -->
      <div class="table-container" v-if="selectedProduct && viewMode === 'roots'">
        <el-table
          :data="roots"
          v-loading="loading"
          stripe
          style="width: 100%"
          :default-sort="{ prop: 'contains_count', order: 'descending' }"
        >
          <template #empty>
            <div class="table-empty-state">
              <div class="empty-icon">
                <el-icon :size="48"><Collection /></el-icon>
              </div>
              <p class="empty-title">{{ searchText ? '没有匹配的词根' : '还没有词根数据' }}</p>
              <p class="empty-desc">{{ searchText ? '尝试其他搜索关键词' : '请先在关键词视图导入数据' }}</p>
              <el-button v-if="searchText" @click="searchText = ''; loadRoots()">
                <el-icon><RefreshLeft /></el-icon>
                清空搜索
              </el-button>
              <el-button v-else type="primary" @click="viewMode = 'keywords'">
                <el-icon><ArrowLeft /></el-icon>
                返回关键词视图
              </el-button>
            </div>
          </template>
          <el-table-column type="index" label="#" width="50" />

          <el-table-column label="词根" min-width="120">
            <template #default="{ row }">
              <span class="word-cell">{{ row.word }}</span>
            </template>
          </el-table-column>

          <el-table-column label="中文翻译" min-width="150">
            <template #default="{ row }">
              <div v-if="editingId === row.id" class="edit-cell">
                <el-input
                  v-model="editingTranslation"
                  size="small"
                  @keyup.enter="saveTranslation(row)"
                  @keyup.escape="cancelEdit"
                />
                <el-button size="small" type="primary" @click="saveTranslation(row)">
                  保存
                </el-button>
                <el-button size="small" @click="cancelEdit">取消</el-button>
              </div>
              <div v-else class="translation-cell" @click="startEdit(row)">
                {{ row.translation || '-' }}
                <el-icon class="edit-icon"><Edit /></el-icon>
              </div>
            </template>
          </el-table-column>

          <el-table-column
            label="词根长度"
            width="110"
            sortable
            :sort-method="(a: Root, b: Root) => a.word.length - b.word.length"
          >
            <template #default="{ row }">
              {{ row.word.length }}
            </template>
          </el-table-column>

          <el-table-column
            prop="contains_count"
            label="包含词"
            width="110"
            sortable
            :sort-orders="['descending', 'ascending']"
          >
            <template #default="{ row }">
              {{ row.contains_count }}个
            </template>
          </el-table-column>

          <el-table-column label="词根占比" width="100">
            <template #default="{ row }">
              {{ row.percentage.toFixed(2) }}%
            </template>
          </el-table-column>

          <el-table-column label="分类" min-width="200">
            <template #default="{ row }">
              <div class="category-cell">
                <el-tag
                  v-for="catId in row.categories"
                  :key="catId"
                  size="small"
                  closable
                  @close="toggleRootCategory(row, catId)"
                >
                  {{ getCategoryName(catId) }}
                </el-tag>
                <el-dropdown trigger="click" @visible-change="(v: boolean) => categoryDropdownVisible[row.id] = v">
                  <el-button size="small" circle>
                    <el-icon><Plus /></el-icon>
                  </el-button>
                  <template #dropdown>
                    <el-dropdown-menu>
                      <el-dropdown-item
                        v-for="cat in categories"
                        :key="cat.id"
                        :disabled="row.categories.includes(cat.id)"
                        @click="toggleRootCategory(row, cat.id)"
                      >
                        {{ cat.name }}
                      </el-dropdown-item>
                    </el-dropdown-menu>
                  </template>
                </el-dropdown>
              </div>
            </template>
          </el-table-column>
        </el-table>
      </div>

      <!-- 无产品提示 -->
      <div class="empty-state main-empty" v-if="!selectedProduct">
        <div class="empty-icon">
          <el-icon :size="64"><Pointer /></el-icon>
        </div>
        <p class="empty-title">请先选择一个产品</p>
        <p class="empty-desc">从左侧列表选择产品，或创建新产品开始分析</p>
        <el-button type="primary" @click="openAddProductDialog">
          <el-icon><Plus /></el-icon>
          新建产品
        </el-button>
      </div>

      <!-- 分页 (仅词根视图显示) -->
      <div class="pagination" v-if="selectedProduct && viewMode === 'roots'">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :page-sizes="[20, 50, 100, 200]"
          :total="total"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handleSizeChange"
          @current-change="handlePageChange"
        />
      </div>
    </main>
    </div><!-- app-body -->

    <!-- 产品编辑对话框 -->
    <ProductDialog
      v-model:visible="showProductDialog"
      v-model:form="productForm"
      :is-editing="isEditingProduct"
      :country-options="countryOptions"
      @save="saveProduct"
    />

    <!-- 快捷键帮助弹窗 -->
    <ShortcutsDialog
      v-model:visible="showShortcutsDialog"
      :app-version="appVersion"
    />

    <!-- 汇率显示设置弹窗 -->
    <el-dialog
      v-model="showExchangeRateSettings"
      title="汇率显示设置"
      width="400px"
      destroy-on-close
    >
      <div class="exchange-rate-settings">
        <p class="settings-tip">选择要在概览页面显示的 3 种货币汇率</p>
        <div class="currency-options">
          <div
            v-for="currency in EXCHANGE_RATE_CURRENCIES"
            :key="currency.code"
            class="currency-option"
            :class="{ selected: selectedCurrencies.includes(currency.code) }"
            @click="toggleCurrencySelection(currency.code)"
          >
            <span class="currency-flag" v-html="currency.flag"></span>
            <span class="currency-name">{{ currency.name }}</span>
            <span class="currency-code">{{ currency.code }}</span>
            <el-icon v-if="selectedCurrencies.includes(currency.code)" class="check-icon"><Check /></el-icon>
          </div>
        </div>
        <p class="settings-counter">
          已选择 {{ selectedCurrencies.length }} / 3 种货币
        </p>
      </div>
      <template #footer>
        <el-button @click="showExchangeRateSettings = false">取消</el-button>
        <el-button type="primary" @click="saveExchangeRateSettings" :disabled="selectedCurrencies.length !== 3">保存</el-button>
      </template>
    </el-dialog>

    <!-- 列配置弹窗 -->
    <ColumnConfigDialog
      v-model:visible="showColumnConfig"
      :column-definitions="columnDefinitions"
      :column-config="columnConfig"
      :is-all-columns-selected="isAllColumnsSelected"
      @update:column-config="(key, val) => { columnConfig[key] = val; saveColumnConfig(); }"
      @toggle-all="toggleSelectAllColumns"
      @reset-default="columnConfig = getDefaultColumnConfig(); saveColumnConfig()"
    />

    <!-- 关键词导出弹窗 -->
    <KeywordExportDialog
      v-model:visible="showKeywordExportDialog"
      v-model:scope="keywordExportScope"
      :keyword-total="keywordTotal"
      :has-active-filters="hasActiveFilters"
      :column-definitions="columnDefinitions"
      :column-config="columnConfig"
      :loading="keywordExporting"
      @export="executeKeywordExport"
      @edit-columns="showColumnConfig = true"
    />

    <!-- 备份管理弹窗 -->
    <BackupDialog
      v-model:visible="showBackupDialog"
      :backups="backups"
      :restoring="restoring"
      @restore="handleRestoreBackup"
      @delete="handleDeleteBackup"
    />

    <!-- 流量设置弹窗 -->
    <TrafficSettingsDialog
      v-model:visible="showTrafficDialog"
      :product="selectedProduct"
      @applied="onTrafficApplied"
    />

    <!-- API Key 设置弹窗 -->
    <ApiKeyDialog
      v-model:visible="showApiKeyDialog"
      @update:visible="(v) => !v && checkApiKeyStatus()"
    />

    <!-- 帮助弹窗 -->
    <el-dialog
      v-model="showHelpDialog"
      title="帮助中心"
      width="950px"
      class="help-dialog"
    >
      <div class="help-layout">
        <!-- 左侧导航 -->
        <div class="help-nav">
          <el-input
            v-model="helpSearchQuery"
            placeholder="搜索帮助..."
            clearable
            class="help-search"
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
          <el-menu
            :default-active="activeHelpTab"
            @select="(key: string) => activeHelpTab = key"
            class="help-menu"
          >
            <el-menu-item
              v-for="section in filteredHelpSections"
              :key="section.id"
              :index="section.id"
            >
              <el-icon>
                <DataAnalysis v-if="section.icon === 'DataAnalysis'" />
                <Collection v-else-if="section.icon === 'Collection'" />
                <TrendCharts v-else-if="section.icon === 'TrendCharts'" />
                <EditPen v-else-if="section.icon === 'EditPen'" />
                <Promotion v-else-if="section.icon === 'Promotion'" />
                <FolderOpened v-else-if="section.icon === 'FolderOpened'" />
              </el-icon>
              <span>{{ section.title }}</span>
            </el-menu-item>
            <!-- 无搜索结果提示 -->
            <div v-if="filteredHelpSections.length === 0 && isSearchingHelp" class="help-no-result">
              <el-icon><Search /></el-icon>
              <span>未找到相关帮助</span>
            </div>
          </el-menu>
          <div class="help-shortcut-hint">
            <kbd>⌘</kbd> + <kbd>H</kbd> 快速打开
          </div>
        </div>

        <!-- 右侧内容 -->
        <div class="help-content-area">
          <!-- 首页帮助 -->
          <div v-show="activeHelpTab === 'dashboard'">
          <div class="help-content">
            <h4>功能说明</h4>
            <p>首页是数据总览面板，展示所有产品的关键指标汇总，并提供智慧大屏和备忘录等辅助工具。</p>
            <ul>
              <li><strong>关键词统计：</strong>显示各产品的关键词总数、已分类数量等</li>
              <li><strong>监控概览：</strong>显示正在监控的关键词数量和排名变化趋势</li>
              <li><strong>排名变化榜：</strong>展示排名上升/下降最多的关键词</li>
              <li><strong>待办提醒：</strong>提示需要关注的事项，如未分类关键词等</li>
              <li><strong>汇率显示：</strong>实时显示主要货币对人民币汇率，支持美元、欧元、英镑、日元，点击齿轮图标可自定义选择显示哪3个</li>
            </ul>

            <h4>智慧大屏</h4>
            <p>点击右上角"智慧大屏"按钮进入全屏数据展示模式，适合团队会议、数据展示等场景。</p>
            <ul>
              <li><strong>全屏展示：</strong>自动适配屏幕尺寸，深色主题设计</li>
              <li><strong>实时数据：</strong>展示关键词统计、排名监控、趋势图表等核心指标</li>
              <li><strong>自动刷新：</strong>数据定时更新，无需手动操作</li>
              <li><strong>按 ESC 退出：</strong>随时按 ESC 键返回普通视图</li>
            </ul>

            <h4>备忘录</h4>
            <p>点击屏幕右侧边缘的悬浮按钮打开备忘录面板，快速记录工作待办事项。</p>
            <ul>
              <li><strong>快速添加：</strong>输入内容后按回车即可添加备忘</li>
              <li><strong>完成标记：</strong>勾选复选框标记任务完成</li>
              <li><strong>截止日期：</strong>点击日历图标设置任务截止时间，支持快捷选择"今天"、"明天"、"一周后"</li>
              <li><strong>重复任务：</strong>设置截止日期后可配置重复周期
                <ul>
                  <li>每天：每日重复，截止日期顺延一天</li>
                  <li>每周：完成后自动生成下周一到期的新任务</li>
                  <li>每月：完成后自动生成下月1号到期的新任务</li>
                </ul>
              </li>
              <li><strong>过期提醒：</strong>过期任务显示红色边框和闪烁徽章，启动时自动弹出系统通知</li>
              <li><strong>拖拽排序：</strong>拖动任务前的图标调整顺序</li>
              <li><strong>筛选查看：</strong>支持"全部"、"待办"、"已完成"三种筛选</li>
            </ul>

            <h4>使用建议</h4>
            <ul>
              <li>每天打开首页快速了解整体数据变化</li>
              <li>关注排名变化榜，及时发现异常波动</li>
              <li>使用备忘录记录每日工作计划，设置重复任务管理周期性工作</li>
              <li>团队会议时打开智慧大屏展示数据成果</li>
            </ul>
          </div>
          </div>

          <!-- 关键词帮助 -->
          <div v-show="activeHelpTab === 'keywords'">
          <div class="help-content">
            <h4>功能说明</h4>
            <p>管理和分析亚马逊关键词数据，支持导入第三方工具数据、AI智能分类、流量分级等功能，帮助运营建立完整的关键词词库。</p>
            <ul>
              <li><strong>数据导入：</strong>支持导入西柚找词（推荐）、卖家精灵、H10等工具导出的关键词数据</li>
              <li><strong>多维度展示：</strong>关键词视图、词根视图、词云视图三种查看方式</li>
              <li><strong>智能分类：</strong>AI自动分析关键词并进行一级分类（品类词、功能词、场景词等）</li>
              <li><strong>流量分级：</strong>根据搜索量自动划分大词、中词、小词</li>
            </ul>

            <h4>核心功能详解</h4>
            <p style="color: var(--el-text-color-secondary); margin-bottom: 12px;">点击展开查看各功能的详细说明：</p>

            <el-collapse class="agent-prompts-collapse">
              <el-collapse-item name="dataImport">
                <template #title>
                  <span class="agent-title">数据导入</span>
                  <span class="agent-subtitle">支持多种数据源</span>
                </template>
                <div class="prompt-section">
                  <h5>支持的数据格式</h5>
                  <div class="prompt-block">
                    <ul>
                      <li><strong>西柚找词（推荐）：</strong>ASIN 反查报告，数据全面、字段丰富</li>
                      <li><strong>卖家精灵：</strong>反查 ASIN 报告、关键词挖掘报告</li>
                      <li><strong>Helium 10：</strong>Cerebro、Magnet 导出文件</li>
                      <li><strong>通用格式：</strong>包含关键词列的 Excel/CSV 文件</li>
                    </ul>
                  </div>
                  <h5>导入方式</h5>
                  <ul>
                    <li>点击"导入"按钮选择文件</li>
                    <li>直接拖拽文件到表格区域</li>
                    <li>支持批量导入多个文件</li>
                  </ul>
                  <h5>字段映射</h5>
                  <ul>
                    <li>关键词、翻译、相关性得分、相关性档位</li>
                    <li>流量总和、周平均排名、周平均搜索量</li>
                    <li>CPC建议竞价、点击转化率、竞争度等</li>
                  </ul>
                </div>
              </el-collapse-item>

              <el-collapse-item name="viewModes">
                <template #title>
                  <span class="agent-title">视图模式</span>
                  <span class="agent-subtitle">不同角度分析关键词</span>
                </template>
                <div class="prompt-section">
                  <h5>关键词视图</h5>
                  <div class="prompt-block">
                    <ul>
                      <li>展示所有导入的原始关键词数据</li>
                      <li>支持按流量级别、相关性、分类等筛选</li>
                      <li>可自定义显示的列（点击列配置按钮）</li>
                      <li>支持多选批量添加到排名监控</li>
                    </ul>
                  </div>
                  <h5>词根视图</h5>
                  <div class="prompt-block">
                    <ul>
                      <li>将关键词拆分为词根，按词根聚合展示</li>
                      <li>显示每个词根包含的关键词数量</li>
                      <li>支持词根分类和词组打标</li>
                      <li>帮助发现核心词根和长尾扩展方向</li>
                    </ul>
                  </div>
                  <h5>词云视图</h5>
                  <div class="prompt-block">
                    <ul>
                      <li>以词云形式可视化展示词根</li>
                      <li>词根大小反映包含关键词数量</li>
                      <li>颜色可区分不同分类</li>
                      <li>直观发现高频词根</li>
                    </ul>
                  </div>
                </div>
              </el-collapse-item>

              <el-collapse-item name="trafficLevel">
                <template #title>
                  <span class="agent-title">流量分级</span>
                  <span class="agent-subtitle">划分大词、中词、小词</span>
                </template>
                <div class="prompt-section">
                  <h5>分级依据</h5>
                  <div class="prompt-block">
                    <ul>
                      <li>根据"流量总和"或"周平均搜索量"字段</li>
                      <li>可自定义大词、中词的阈值</li>
                      <li>默认：大词 > 10000，中词 1000-10000，小词 < 1000</li>
                    </ul>
                  </div>
                  <h5>使用场景</h5>
                  <ul>
                    <li><strong>大词：</strong>搜索量大、竞争激烈，适合品牌词和核心词布局</li>
                    <li><strong>中词：</strong>流量适中、竞争相对较小，性价比较高</li>
                    <li><strong>小词：</strong>精准长尾词，转化率通常较高</li>
                  </ul>
                  <h5>设置方式</h5>
                  <ul>
                    <li>点击工具栏"流量设置"按钮</li>
                    <li>选择用于分级的字段</li>
                    <li>设置大词和中词的最小阈值</li>
                  </ul>
                </div>
              </el-collapse-item>

              <el-collapse-item name="aiClassification">
                <template #title>
                  <span class="agent-title">AI 智能分类</span>
                  <span class="agent-subtitle">自动分析关键词属性</span>
                </template>
                <div class="prompt-section">
                  <h5>一级分类</h5>
                  <div class="prompt-block">
                    <ul>
                      <li><strong>品类词：</strong>产品类目相关词，如"蓝牙耳机"、"运动鞋"</li>
                      <li><strong>功能词：</strong>描述产品功能特性，如"降噪"、"防水"</li>
                      <li><strong>场景词：</strong>使用场景相关，如"户外"、"办公室"</li>
                      <li><strong>属性词：</strong>颜色、材质、尺寸等，如"黑色"、"皮质"</li>
                      <li><strong>品牌词：</strong>品牌名称相关词</li>
                      <li><strong>人群词：</strong>目标人群相关，如"女士"、"儿童"</li>
                      <li><strong>受众词：</strong>特定受众群体，如"程序员"、"学生"</li>
                    </ul>
                  </div>
                  <h5>词组打标</h5>
                  <div class="prompt-block">
                    <ul>
                      <li>分析词根组成的词组模式</li>
                      <li>识别常见搭配和修饰关系</li>
                      <li>帮助理解关键词结构</li>
                    </ul>
                  </div>
                  <h5>使用方式</h5>
                  <ul>
                    <li>需要先配置 AI 服务 API Key（DeepSeek 推荐）</li>
                    <li>在词根视图点击"AI分析"按钮</li>
                    <li>支持批量分析和单个词根分析</li>
                    <li>分析结果可导出</li>
                  </ul>
                </div>
              </el-collapse-item>

              <el-collapse-item name="filterAndSearch">
                <template #title>
                  <span class="agent-title">筛选与搜索</span>
                  <span class="agent-subtitle">快速定位目标关键词</span>
                </template>
                <div class="prompt-section">
                  <h5>筛选维度</h5>
                  <div class="prompt-block">
                    <ul>
                      <li><strong>流量级别：</strong>大词 / 中词 / 小词</li>
                      <li><strong>相关性：</strong>强相关 / 高相关 / 中相关 / 弱相关</li>
                      <li><strong>一级分类：</strong>品类词 / 功能词 / 场景词 等</li>
                      <li><strong>有序性：</strong>有序 / 无序（词根视图）</li>
                    </ul>
                  </div>
                  <h5>搜索功能</h5>
                  <ul>
                    <li>支持关键词模糊搜索</li>
                    <li>支持中文翻译搜索</li>
                    <li>回车或点击搜索按钮执行</li>
                  </ul>
                  <h5>排序功能</h5>
                  <ul>
                    <li>点击表头可按该列排序</li>
                    <li>支持升序/降序切换</li>
                    <li>常用排序：流量、搜索量、相关性得分</li>
                  </ul>
                </div>
              </el-collapse-item>

              <el-collapse-item name="batchOperations">
                <template #title>
                  <span class="agent-title">批量操作</span>
                  <span class="agent-subtitle">高效管理关键词</span>
                </template>
                <div class="prompt-section">
                  <h5>关键词视图</h5>
                  <div class="prompt-block">
                    <ul>
                      <li><strong>批量添加监控：</strong>选中关键词后添加到排名监控</li>
                      <li><strong>批量导出：</strong>导出筛选后的关键词数据</li>
                      <li><strong>复制关键词：</strong>复制选中的关键词到剪贴板</li>
                    </ul>
                  </div>
                  <h5>词根视图</h5>
                  <div class="prompt-block">
                    <ul>
                      <li><strong>批量分类：</strong>为选中词根批量设置分类</li>
                      <li><strong>AI批量分析：</strong>批量进行智能分类分析</li>
                      <li><strong>词组打标：</strong>批量分析词组结构</li>
                    </ul>
                  </div>
                </div>
              </el-collapse-item>
            </el-collapse>

            <h4>数据字段说明</h4>
            <ul>
              <li><strong>关键词：</strong>原始关键词文本</li>
              <li><strong>翻译：</strong>关键词的中文翻译</li>
              <li><strong>相关性得分/档位：</strong>与目标ASIN的相关程度</li>
              <li><strong>流量总和：</strong>关键词带来的总流量估算</li>
              <li><strong>周平均排名：</strong>产品在该关键词下的平均排名</li>
              <li><strong>周平均搜索量：</strong>关键词的周搜索量</li>
              <li><strong>CPC建议竞价：</strong>广告建议出价</li>
              <li><strong>点击转化率：</strong>点击后的转化比例</li>
              <li><strong>竞争度：</strong>关键词的竞争激烈程度</li>
            </ul>

            <h4>使用流程</h4>
            <ol>
              <li>创建产品，选择对应的亚马逊站点</li>
              <li>导入关键词数据（卖家精灵/H10导出文件）</li>
              <li>设置流量分级阈值，自动划分大中小词</li>
              <li>切换到词根视图，运行 AI 分析进行智能分类</li>
              <li>使用筛选功能定位目标关键词</li>
              <li>将重点关键词添加到排名监控</li>
              <li>定期导出更新的关键词数据</li>
            </ol>

            <h4>注意事项</h4>
            <ul>
              <li>导入前确保数据格式正确，必须包含"关键词"列</li>
              <li>AI分类需要配置API Key，推荐使用DeepSeek（成本低、效果好）</li>
              <li>大量关键词分析时需要一定时间，请耐心等待</li>
              <li>流量数据来自第三方工具，仅供参考</li>
              <li>建议定期更新关键词数据，保持词库时效性</li>
            </ul>
          </div>
          </div>

          <!-- 排名监控帮助 -->
          <div v-show="activeHelpTab === 'monitoring'">
          <div class="help-content">
            <h4>功能说明</h4>
            <p>实时监控关键词在亚马逊搜索结果中的自然排名和广告排名变化，帮助运营及时发现排名波动并追踪优化效果。</p>
            <ul>
              <li><strong>双维度监控：</strong>同时监控自然排名和广告位排名，区分 SP 广告和自然搜索结果</li>
              <li><strong>多国家支持：</strong>支持美国、英国、德国、法国、意大利、西班牙等主要站点</li>
              <li><strong>多视图模式：</strong>列表视图、按产品分组、按关键词分组三种查看方式</li>
              <li><strong>趋势可视化：</strong>迷你图展示近7天排名趋势，点击查看详细历史曲线</li>
            </ul>

            <h4>核心功能详解</h4>
            <p style="color: var(--el-text-color-secondary); margin-bottom: 12px;">点击展开查看各功能的详细说明：</p>

            <el-collapse class="agent-prompts-collapse">
              <el-collapse-item name="rankingDisplay">
                <template #title>
                  <span class="agent-title">排名显示规则</span>
                  <span class="agent-subtitle">理解排名数据的含义</span>
                </template>
                <div class="prompt-section">
                  <h5>自然排名</h5>
                  <div class="prompt-block">
                    <ul>
                      <li><strong>显示格式：</strong>"第X页第Y名"，如"第1页第5名"</li>
                      <li><strong>Top 10 高亮：</strong>第1页前10名显示为绿色，表示优秀排名</li>
                      <li><strong>无排名：</strong>显示"前N页无排名"，N为设置的最大检测页数</li>
                      <li><strong>注意：</strong>自然排名不包含 SP 广告位，只统计自然搜索结果</li>
                    </ul>
                  </div>
                  <h5>广告排名</h5>
                  <div class="prompt-block">
                    <ul>
                      <li><strong>显示格式：</strong>同样为"第X页第Y名"</li>
                      <li><strong>含义：</strong>产品在该关键词下的 Sponsored 广告位置</li>
                      <li><strong>无排名：</strong>表示该关键词下没有投放广告或广告未展示</li>
                    </ul>
                  </div>
                  <h5>趋势图</h5>
                  <ul>
                    <li>绿色线条：自然排名趋势（越低越好）</li>
                    <li>蓝色线条：广告排名趋势（越低越好）</li>
                    <li>点击趋势图可查看详细的历史排名曲线和数据</li>
                  </ul>
                </div>
              </el-collapse-item>

              <el-collapse-item name="viewModes">
                <template #title>
                  <span class="agent-title">视图模式</span>
                  <span class="agent-subtitle">不同场景使用不同视图</span>
                </template>
                <div class="prompt-section">
                  <h5>列表视图（默认）</h5>
                  <div class="prompt-block">
                    <ul>
                      <li>所有监控项平铺展示，支持排序和筛选</li>
                      <li>适合快速查看所有关键词的排名情况</li>
                      <li>支持多选批量操作</li>
                    </ul>
                  </div>
                  <h5>按产品分组</h5>
                  <div class="prompt-block">
                    <ul>
                      <li>按 ASIN + 站点 分组，展示每个产品的所有监控关键词</li>
                      <li>显示产品图片、价格、评分等信息</li>
                      <li>显示组内平均排名，支持批量检测整组</li>
                    </ul>
                  </div>
                  <h5>按关键词分组</h5>
                  <div class="prompt-block">
                    <ul>
                      <li>按关键词 + 站点 分组，展示同一关键词下的多个产品</li>
                      <li>适合分析同一关键词下自己和竞品的排名对比</li>
                      <li>显示组内最佳排名</li>
                    </ul>
                  </div>
                </div>
              </el-collapse-item>

              <el-collapse-item name="keywordTags">
                <template #title>
                  <span class="agent-title">关键词标签</span>
                  <span class="agent-subtitle">分类管理监控关键词</span>
                </template>
                <div class="prompt-section">
                  <h5>标签类型</h5>
                  <div class="prompt-block">
                    <ul>
                      <li><strong>核心词：</strong>最重要的主推关键词，需要重点关注排名</li>
                      <li><strong>长尾词：</strong>精准长尾词，通常转化率较高</li>
                      <li><strong>竞品词：</strong>竞品品牌词或型号词</li>
                      <li><strong>品牌词：</strong>自己的品牌词</li>
                      <li><strong>待优化：</strong>需要优化但目前排名不理想的词</li>
                      <li><strong>测试中：</strong>正在测试效果的词</li>
                    </ul>
                  </div>
                  <h5>使用方式</h5>
                  <ul>
                    <li>在关键词列表中，悬停时显示标签小圆点</li>
                    <li>点击 "+" 号或标签区域打开标签编辑器</li>
                    <li>支持多选，一个关键词可以有多个标签</li>
                  </ul>
                </div>
              </el-collapse-item>

              <el-collapse-item name="optimizationEvents">
                <template #title>
                  <span class="agent-title">优化事件记录</span>
                  <span class="agent-subtitle">追踪操作与排名变化的关联</span>
                </template>
                <div class="prompt-section">
                  <h5>事件类型</h5>
                  <div class="prompt-block">
                    <ul>
                      <li><strong>Listing 优化：</strong>标题、五点、描述、图片、A+内容等修改</li>
                      <li><strong>价格调整：</strong>调价、优惠券、促销活动</li>
                      <li><strong>库存变化：</strong>补货、断货、FBA 入库</li>
                      <li><strong>广告调整：</strong>竞价、预算、投放词修改</li>
                      <li><strong>其他操作：</strong>差评处理、QA 更新、品牌旗舰店等</li>
                    </ul>
                  </div>
                  <h5>使用价值</h5>
                  <ul>
                    <li>在排名历史图表中显示事件标记，直观看到操作对排名的影响</li>
                    <li>支持日历视图和列表视图两种浏览方式</li>
                    <li>可关联到特定 ASIN 和关键词</li>
                    <li>帮助复盘优化效果，积累运营经验</li>
                  </ul>
                </div>
              </el-collapse-item>

              <el-collapse-item name="detectionSettings">
                <template #title>
                  <span class="agent-title">检测设置</span>
                  <span class="agent-subtitle">自定义检测行为</span>
                </template>
                <div class="prompt-section">
                  <h5>检测页数</h5>
                  <div class="prompt-block">
                    <ul>
                      <li>默认检测前 5 页（约 240 个结果）</li>
                      <li>可在设置中调整最大检测页数（1-10页）</li>
                      <li>页数越多检测越慢，建议根据实际需要设置</li>
                    </ul>
                  </div>
                  <h5>自动检测</h5>
                  <div class="prompt-block">
                    <ul>
                      <li>支持定时自动检测（每天/每周指定时间）</li>
                      <li>在"监控设置"中配置自动检测时间</li>
                      <li>建议固定时间检测，便于对比分析</li>
                    </ul>
                  </div>
                  <h5>检测优先级</h5>
                  <ul>
                    <li>支持设置关键词优先级（高/中/低）</li>
                    <li>自动检测时可设置只检测高优先级关键词</li>
                    <li>按产品分组时，组内按优先级排序显示</li>
                  </ul>
                </div>
              </el-collapse-item>
            </el-collapse>

            <h4>统计指标说明</h4>
            <ul>
              <li><strong>监控总数：</strong>添加的监控关键词总数</li>
              <li><strong>活跃监控：</strong>状态为"活跃"的监控数量（暂停的不计入）</li>
              <li><strong>Top 10：</strong>自然排名在第1页前10名的数量</li>
              <li><strong>Top 30：</strong>自然排名在前30名的数量（约前2页）</li>
              <li><strong>有广告位：</strong>有 Sponsored 广告排名的数量</li>
            </ul>

            <h4>使用流程</h4>
            <ol>
              <li>点击"添加监控"，输入关键词和 ASIN（或从词库批量添加）</li>
              <li>选择站点并设置优先级</li>
              <li>首次添加后点击"检测全部"获取初始排名</li>
              <li>后续可手动检测或配置自动检测</li>
              <li>点击趋势图查看历史排名曲线</li>
              <li>进行优化操作时记录"优化事件"</li>
              <li>观察排名变化，分析优化效果</li>
            </ol>

            <h4>注意事项</h4>
            <ul>
              <li>首次检测需要安装依赖（Python + Playwright），按提示安装即可</li>
              <li>排名数据通过模拟浏览器抓取，检测较多关键词时需要一定时间</li>
              <li>不同地区/账号看到的排名可能略有差异，以趋势变化为参考</li>
              <li>建议每天固定时间检测，便于横向对比分析</li>
              <li>SP 广告位和自然位分开统计，互不影响</li>
              <li>检测失败时检查网络连接，或尝试减少检测页数</li>
            </ul>
          </div>
          </div>

          <!-- 智能文案帮助 -->
          <div v-show="activeHelpTab === 'smartcopy'">
          <div class="help-content">
            <h4>功能说明</h4>
            <p>基于AI分析竞品数据，生成符合 A9、COSMO、Rufus 算法的优质 Listing 文案建议。</p>
            <ul>
              <li><strong>新品打造：</strong>从零开始创建 Listing，支持填写产品信息获得针对性建议</li>
              <li><strong>老品优化：</strong>基于现有文案生成优化建议，保留品牌调性</li>
              <li><strong>竞品分析：</strong>抓取竞品标题、五点、评论等信息</li>
              <li><strong>双显示模式：</strong>画布模式（并行）、经典模式（顺序）</li>
            </ul>

            <h4>AI 分析流程</h4>
            <p style="color: var(--el-text-color-secondary); margin-bottom: 12px;">点击展开查看各步骤的详细分析逻辑：</p>

            <el-collapse class="agent-prompts-collapse">
              <el-collapse-item name="reviewInsights">
                <template #title>
                  <span class="agent-title">步骤 1: 评论洞察分析</span>
                  <span class="agent-subtitle">提取用户真实反馈</span>
                </template>
                <div class="prompt-section">
                  <h5>分析内容</h5>
                  <div class="prompt-block">
                    <ul>
                      <li><strong>使用场景（5-10条）：</strong>买家实际使用产品的场景，如"户外露营时使用"</li>
                      <li><strong>爽点/卖点（5-10条）：</strong>买家喜欢的产品优点，用买家的语言描述</li>
                      <li><strong>痛点/问题（5-10条）：</strong>差评(1-3星)中的高频问题和抱怨</li>
                    </ul>
                  </div>
                  <h5>输出内容</h5>
                  <ul>
                    <li>每条洞察标注出现频次和示例评论原文</li>
                    <li>综合洞察总结（100字以内）</li>
                  </ul>
                </div>
              </el-collapse-item>

              <el-collapse-item name="listingAnalysis">
                <template #title>
                  <span class="agent-title">步骤 2: 竞品文案分析</span>
                  <span class="agent-subtitle">分析文案结构和关键词</span>
                </template>
                <div class="prompt-section">
                  <h5>分析内容</h5>
                  <div class="prompt-block">
                    <p><strong>标题结构分析：</strong></p>
                    <ul>
                      <li>品牌词、核心词、属性词、场景词的排列方式</li>
                      <li>高频词汇统计</li>
                    </ul>
                    <p><strong>五点描述分析：</strong></p>
                    <ul>
                      <li>共同主题提取</li>
                      <li>最佳实践总结</li>
                    </ul>
                    <p><strong>关键词使用分析：</strong></p>
                    <ul>
                      <li>竞品普遍使用的关键词</li>
                      <li>关键词使用模式</li>
                    </ul>
                  </div>
                </div>
              </el-collapse-item>

              <el-collapse-item name="optimization">
                <template #title>
                  <span class="agent-title">步骤 3: 优化建议生成</span>
                  <span class="agent-subtitle">生成完整 Listing 文案</span>
                </template>
                <div class="prompt-section">
                  <h5>遵循的算法原则</h5>
                  <div class="prompt-block">
                    <ul>
                      <li><strong>A9 算法：</strong>在标题前部放置核心关键词，确保完全匹配</li>
                      <li><strong>COSMO 算法：</strong>覆盖多种使用场景，匹配用户搜索意图</li>
                      <li><strong>Rufus 算法：</strong>预设性解答买家常见问题，消除购买顾虑</li>
                    </ul>
                  </div>
                  <h5>生成内容</h5>
                  <ul>
                    <li><strong>标题建议：</strong>多个版本，说明关键词选择理由和数据来源</li>
                    <li><strong>五点描述：</strong>5条，每条标注重点主题、埋入的关键词、写作理由</li>
                    <li><strong>后台关键词：</strong>选择前台未使用的长尾关键词</li>
                    <li><strong>商品描述：</strong>2个版本，包含品牌故事、核心卖点、使用场景</li>
                    <li><strong>A+内容建议：</strong>主图文案、辅图主题、模块推荐</li>
                  </ul>
                  <h5>五点主题参考</h5>
                  <div class="prompt-block">
                    <ol>
                      <li>核心卖点/主要功能</li>
                      <li>解决用户痛点（针对评论中的负面反馈）</li>
                      <li>使用场景覆盖（匹配 COSMO 场景算法）</li>
                      <li>规格参数/品质保障</li>
                      <li>售后服务/包装配件</li>
                    </ol>
                  </div>
                </div>
              </el-collapse-item>
            </el-collapse>

            <h4>使用流程</h4>
            <ol>
              <li>创建项目，选择场景（新品打造/老品优化）</li>
              <li>添加 3-5 个主要竞品 ASIN</li>
              <li>点击"批量获取"抓取竞品数据（Listing + 评论）</li>
              <li>（可选）关联产品关键词，获得更精准的关键词建议</li>
              <li>（新品打造）填写"我的产品信息"获得针对性文案</li>
              <li>点击"开始分析"，AI 会逐步生成分析报告</li>
              <li>参考建议优化自己的 Listing，支持导出 Excel</li>
            </ol>

            <h4>注意事项</h4>
            <ul>
              <li>需要先在设置中配置 AI 服务的 API Key</li>
              <li>竞品数据抓取需要一定时间，请耐心等待</li>
              <li>评论数据越多，洞察越准确</li>
              <li>建议关联产品关键词，以便 AI 选择高搜索量词</li>
            </ul>
          </div>
          </div>

          <!-- 智能广告帮助 -->
          <div v-show="activeHelpTab === 'ads'">
          <div class="help-content">
            <h4>功能说明</h4>
            <p>基于AI多智能体架构分析亚马逊广告搜索词报告，提供优化建议。</p>
            <ul>
              <li><strong>数据导入：</strong>支持导入亚马逊搜索词报告（Excel/CSV格式）</li>
              <li><strong>多国家支持：</strong>自动识别国家并按国家分组分析，正确处理不同货币</li>
              <li><strong>多智能体分析：</strong>4个AI专家并行分析，全面覆盖广告优化维度</li>
              <li><strong>增量显示：</strong>每个国家分析完成后立即显示结果，失败不影响已完成的国家</li>
            </ul>

            <h4>AI 智能体介绍</h4>
            <p style="color: var(--el-text-color-secondary); margin-bottom: 12px;">点击展开查看各智能体的详细分析逻辑：</p>

            <el-collapse class="agent-prompts-collapse">
              <el-collapse-item name="searchTermAnalyst">
                <template #title>
                  <span class="agent-title">1. 搜索词分析师</span>
                  <span class="agent-subtitle">识别无效搜索词和高潜力词</span>
                </template>
                <div class="prompt-section">
                  <h5>分析维度</h5>
                  <div class="prompt-block">
                    <p><strong>无效搜索词识别：</strong></p>
                    <ul>
                      <li>高花费零转化（spend &gt; 平均值 且 orders = 0）</li>
                      <li>极低转化率（conversion_rate &lt; 1% 且 clicks &gt; 10）</li>
                      <li>不相关词（通过语义判断与投放词不相关）</li>
                      <li>ACOS 极高（acos &gt; 200%）</li>
                    </ul>
                    <p><strong>高潜力词识别：</strong></p>
                    <ul>
                      <li>高转化低 ACOS（acos &lt; 目标 且 orders &gt;= 2）</li>
                      <li>低展示高转化（需要加大投放）</li>
                      <li>精准匹配候选词（已有好表现的广泛/词组匹配词）</li>
                    </ul>
                  </div>
                  <h5>输出内容</h5>
                  <ul>
                    <li>否定词候选（最多15条）：搜索词、原因、风险级别、浪费花费、建议匹配类型、影响的广告活动</li>
                    <li>高潜力词（最多10条）：搜索词、表现数据、建议操作、当前匹配类型</li>
                  </ul>
                </div>
              </el-collapse-item>

              <el-collapse-item name="acosExpert">
                <template #title>
                  <span class="agent-title">2. ACOS 专家</span>
                  <span class="agent-subtitle">分析广告效率，识别ACOS异常</span>
                </template>
                <div class="prompt-section">
                  <h5>ACOS 分布分析</h5>
                  <div class="prompt-block">
                    <ul>
                      <li><strong>优秀：</strong>ACOS &lt; 目标 × 0.7</li>
                      <li><strong>良好：</strong>目标 × 0.7 ~ 目标</li>
                      <li><strong>边缘：</strong>目标 ~ 目标 × 1.5</li>
                      <li><strong>较差：</strong>目标 × 1.5 ~ 100%</li>
                      <li><strong>极差：</strong>ACOS &gt; 100%</li>
                      <li><strong>无销售：</strong>有花费但销售为0</li>
                    </ul>
                  </div>
                  <h5>重点关注</h5>
                  <ul>
                    <li>超高 ACOS（&gt;100%）的搜索词</li>
                    <li>高花费无销售的搜索词</li>
                  </ul>
                  <h5>输出内容</h5>
                  <ul>
                    <li>效率分析：盈利/亏损/持平/无销售关键词统计</li>
                    <li>ACOS 分布统计：各区间的数量和花费</li>
                    <li>优化优先级列表（最多15条）</li>
                  </ul>
                </div>
              </el-collapse-item>

              <el-collapse-item name="bidStrategist">
                <template #title>
                  <span class="agent-title">3. 竞价策略师</span>
                  <span class="agent-subtitle">提供竞价调整建议</span>
                </template>
                <div class="prompt-section">
                  <h5>竞价调整原则</h5>
                  <div class="prompt-block">
                    <p><strong>加价条件（+10%~30%）：</strong></p>
                    <ul>
                      <li>ACOS &lt; 目标 × 0.7 且转化率 &gt; 5% 且展示量偏低</li>
                      <li>表现优秀但市场份额可能不足</li>
                    </ul>
                    <p><strong>降价条件（-10%~30%）：</strong></p>
                    <ul>
                      <li>ACOS &gt; 目标 × 1.3 且有转化</li>
                      <li>需要保持曝光但控制成本</li>
                    </ul>
                    <p><strong>暂停条件：</strong></p>
                    <ul>
                      <li>ACOS &gt; 150%</li>
                      <li>花费 &gt; $15 且零转化</li>
                      <li>持续亏损无改善迹象</li>
                    </ul>
                    <p><strong>维持条件：</strong>ACOS 在目标 ± 30% 范围内</p>
                  </div>
                  <h5>输出内容</h5>
                  <ul>
                    <li>竞价调整建议（最多20条）：投放词、建议、调整幅度、原因、优先级</li>
                    <li>汇总统计：加价/降价/暂停/维持数量，预计节省金额</li>
                  </ul>
                </div>
              </el-collapse-item>

              <el-collapse-item name="suggestionIntegrator">
                <template #title>
                  <span class="agent-title">4. 建议整合器</span>
                  <span class="agent-subtitle">生成最终优化报告</span>
                </template>
                <div class="prompt-section">
                  <h5>整合要求</h5>
                  <div class="prompt-block">
                    <ul>
                      <li>合并去重否定词建议，按风险级别和浪费花费排序</li>
                      <li>整合竞价调整建议并按优先级排序</li>
                      <li>提取关键词机会并评估潜力</li>
                      <li>生成执行摘要和关键洞察</li>
                    </ul>
                  </div>
                  <h5>最终输出</h5>
                  <ul>
                    <li>否定词建议（最多15条）：综合三位专家意见</li>
                    <li>竞价调整（最多15条）：按优先级排序</li>
                    <li>新词机会（最多10条）：按潜力评估排序</li>
                    <li>优化摘要：总花费、预计节省、优化评分、关键洞察</li>
                  </ul>
                </div>
              </el-collapse-item>
            </el-collapse>

            <h4>优化评分规则（0-100分）</h4>
            <p>评分基于以下维度计算：</p>
            <ul>
              <li>基础分：50 分</li>
              <li>整体 ACOS 低于目标：+20 分</li>
              <li>整体 ACOS 高于目标 50% 以上：-20 分</li>
              <li>高风险否定词占比 &lt; 5%：+10 分</li>
              <li>高风险否定词占比 &gt; 20%：-10 分</li>
              <li>存在高潜力关键词机会：+10 分</li>
              <li>大部分关键词 ACOS 在目标范围内：+10 分</li>
            </ul>

            <h4>使用流程</h4>
            <ol>
              <li>创建广告项目，设置目标 ACOS</li>
              <li>导入亚马逊搜索词报告（支持多国家混合报告）</li>
              <li>选择 AI 服务商和模型，点击"开始分析"</li>
              <li>等待分析完成，查看各国家的优化建议</li>
              <li>如有失败的国家，可点击"重试失败"按钮</li>
            </ol>

            <h4>注意事项</h4>
            <ul>
              <li>需要先在设置中配置 AI 服务的 API Key</li>
              <li>建议选择 DeepSeek 或 Gemini，性价比较高</li>
              <li>数据量大时分析时间较长，请耐心等待</li>
              <li>分析过程中请勿关闭页面，否则需要重新开始</li>
            </ul>
          </div>
          </div>

          <!-- 知识库帮助 -->
          <div v-show="activeHelpTab === 'knowledge'">
          <div class="help-content">
            <h4>功能说明</h4>
            <p>上传产品相关文档，构建企业专属知识库。通过向量检索 + 关键词搜索的混合检索技术，实现基于文档内容的精准 AI 问答。支持 Obsidian 风格的多分类管理、双向链接和知识图谱。</p>
            <ul>
              <li><strong>文档管理：</strong>支持 PDF、Word、Excel、PPT、Markdown、TXT 等格式，批量上传带进度显示</li>
              <li><strong>分类整理：</strong>创建分类管理文档，支持颜色标识、拖拽排序，一个文档可归属多个分类</li>
              <li><strong>双向链接：</strong>文档间建立关联，支持反向链接查看</li>
              <li><strong>知识图谱：</strong>可视化展示文档之间的关联关系</li>
              <li><strong>智能问答：</strong>三种对话模式，支持流式输出和来源引用</li>
              <li><strong>图片识别：</strong>自动提取文档中的图片并用 AI 识别内容</li>
            </ul>

            <h4>核心功能详解</h4>
            <p style="color: var(--el-text-color-secondary); margin-bottom: 12px;">点击展开查看详细说明：</p>

            <el-collapse class="agent-prompts-collapse">
              <el-collapse-item name="documentUpload">
                <template #title>
                  <span class="agent-title">文档上传与处理</span>
                  <span class="agent-subtitle">支持多种格式，智能解析</span>
                </template>
                <div class="prompt-section">
                  <h5>支持的文档格式</h5>
                  <div class="prompt-block">
                    <ul>
                      <li><strong>PDF：</strong>自动提取文本，文本提取失败时自动使用 AI OCR 识别</li>
                      <li><strong>Word (docx)：</strong>解析文本内容，自动提取并识别文档中的图片</li>
                      <li><strong>Excel (xlsx/xls)：</strong>提取表格数据，支持图片识别</li>
                      <li><strong>PPT (pptx)：</strong>提取幻灯片内容，支持图片识别</li>
                      <li><strong>Markdown/TXT：</strong>直接解析文本内容</li>
                    </ul>
                  </div>
                  <h5>处理流程</h5>
                  <ul>
                    <li><strong>文本解析：</strong>提取文档中的文字内容</li>
                    <li><strong>智能分块：</strong>将长文档切分为适合检索的文本块</li>
                    <li><strong>向量化：</strong>为每个文本块生成向量索引（需配置 DeepSeek 或通义千问）</li>
                    <li><strong>图片处理：</strong>提取图片并用 AI Vision 识别内容（需配置通义千问或 Gemini）</li>
                  </ul>
                </div>
              </el-collapse-item>

              <el-collapse-item name="categorySystem">
                <template #title>
                  <span class="agent-title">多分类管理</span>
                  <span class="agent-subtitle">灵活组织文档</span>
                </template>
                <div class="prompt-section">
                  <h5>分类管理</h5>
                  <div class="prompt-block">
                    <ul>
                      <li><strong>创建分类：</strong>点击左侧分类面板的"+"按钮创建新分类</li>
                      <li><strong>颜色设置：</strong>为每个分类设置不同颜色，便于视觉区分</li>
                      <li><strong>编辑/删除：</strong>右键点击分类可编辑或删除</li>
                      <li><strong>拖拽排序：</strong>拖拽分类可调整显示顺序</li>
                    </ul>
                  </div>
                  <h5>文档分类</h5>
                  <div class="prompt-block">
                    <ul>
                      <li><strong>多分类支持：</strong>一个文档可以归属于多个分类</li>
                      <li><strong>添加分类：</strong>在文档预览面板中，点击"添加分类"选择要归入的分类</li>
                      <li><strong>移除分类：</strong>点击分类标签上的关闭按钮可移除</li>
                      <li><strong>按分类筛选：</strong>点击侧边栏的分类可快速筛选该分类下的所有文档</li>
                    </ul>
                  </div>
                  <h5>使用场景</h5>
                  <ul>
                    <li>同一文档可属于多个分类，实现多维度管理</li>
                    <li>比单一文件夹更灵活，避免文档重复存储</li>
                    <li>分类颜色直观标识，快速识别文档归属</li>
                  </ul>
                </div>
              </el-collapse-item>

              <el-collapse-item name="biLinks">
                <template #title>
                  <span class="agent-title">双向链接与知识图谱</span>
                  <span class="agent-subtitle">构建知识网络</span>
                </template>
                <div class="prompt-section">
                  <h5>双向链接</h5>
                  <div class="prompt-block">
                    <ul>
                      <li><strong>添加链接：</strong>在文档预览面板点击"添加链接"，选择要关联的文档</li>
                      <li><strong>出链：</strong>当前文档主动引用的其他文档</li>
                      <li><strong>反向链接：</strong>其他文档中引用当前文档的列表</li>
                      <li><strong>快速跳转：</strong>点击链接可直接打开目标文档</li>
                    </ul>
                  </div>
                  <h5>知识图谱</h5>
                  <div class="prompt-block">
                    <ul>
                      <li><strong>打开图谱：</strong>点击工具栏的"知识图谱"按钮查看</li>
                      <li><strong>节点显示：</strong>每个文档显示为一个节点</li>
                      <li><strong>连线关系：</strong>有链接关系的文档之间会显示连线</li>
                      <li><strong>节点交互：</strong>点击节点可直接跳转到对应文档</li>
                    </ul>
                  </div>
                  <h5>使用场景</h5>
                  <ul>
                    <li>产品文档关联到竞品分析、用户反馈等文档</li>
                    <li>通过反向链接发现哪些文档引用了当前内容</li>
                    <li>知识图谱直观展示知识体系的关联结构</li>
                  </ul>
                </div>
              </el-collapse-item>

              <el-collapse-item name="callouts">
                <template #title>
                  <span class="agent-title">Callouts 提示框</span>
                  <span class="agent-subtitle">Obsidian 风格高亮块</span>
                </template>
                <div class="prompt-section">
                  <h5>语法格式</h5>
                  <div class="prompt-block">
                    <p>在 Markdown 文档中使用以下语法创建提示框：</p>
                    <pre style="background: var(--el-fill-color-light); padding: 12px; border-radius: 6px; margin: 8px 0; font-size: 13px;">&gt; [!note] 标题
&gt; 内容文字

&gt; [!warning] 警告标题
&gt; 警告内容</pre>
                  </div>
                  <h5>支持的类型</h5>
                  <div class="prompt-block">
                    <ul>
                      <li><strong>note（笔记）：</strong>蓝色，用于一般性说明</li>
                      <li><strong>tip（提示）：</strong>绿色，用于技巧和建议</li>
                      <li><strong>warning（警告）：</strong>橙色，用于注意事项</li>
                      <li><strong>danger（危险）：</strong>红色，用于重要警示</li>
                      <li><strong>info（信息）：</strong>蓝色，用于补充信息</li>
                      <li><strong>quote（引用）：</strong>灰色，用于引用内容</li>
                      <li><strong>success（成功）：</strong>绿色，用于正面信息</li>
                      <li><strong>question（疑问）：</strong>紫色，用于问题提示</li>
                    </ul>
                  </div>
                </div>
              </el-collapse-item>

              <el-collapse-item name="outline">
                <template #title>
                  <span class="agent-title">大纲导航</span>
                  <span class="agent-subtitle">快速定位长文档</span>
                </template>
                <div class="prompt-section">
                  <h5>功能说明</h5>
                  <div class="prompt-block">
                    <ul>
                      <li><strong>自动提取：</strong>系统自动解析文档中的标题（# ## ### 等）</li>
                      <li><strong>层级展示：</strong>大纲按标题层级缩进显示</li>
                      <li><strong>快速跳转：</strong>点击大纲项可滚动到对应位置</li>
                      <li><strong>位置显示：</strong>在文档预览面板右侧展示</li>
                    </ul>
                  </div>
                  <h5>使用场景</h5>
                  <ul>
                    <li>快速浏览长文档的结构</li>
                    <li>在文档各章节之间快速跳转</li>
                    <li>了解文档的内容组织方式</li>
                  </ul>
                </div>
              </el-collapse-item>

              <el-collapse-item name="saveAsNote">
                <template #title>
                  <span class="agent-title">保存 AI 回答为笔记</span>
                  <span class="agent-subtitle">知识沉淀闭环</span>
                </template>
                <div class="prompt-section">
                  <h5>功能说明</h5>
                  <div class="prompt-block">
                    <ul>
                      <li><strong>保存按钮：</strong>AI 回答后，悬停消息可见"保存为笔记"按钮</li>
                      <li><strong>自动关联：</strong>保存时自动将引用的来源文档建立双向链接</li>
                      <li><strong>编辑内容：</strong>保存前可编辑笔记标题和内容</li>
                    </ul>
                  </div>
                  <h5>使用场景</h5>
                  <ul>
                    <li>将 AI 分析的结论保存为知识库文档</li>
                    <li>整理 AI 回答，形成结构化的知识积累</li>
                    <li>通过双向链接追溯知识的来源</li>
                  </ul>
                </div>
              </el-collapse-item>

              <el-collapse-item name="chatModes">
                <template #title>
                  <span class="agent-title">三种对话模式</span>
                  <span class="agent-subtitle">灵活切换，满足不同需求</span>
                </template>
                <div class="prompt-section">
                  <h5>严格模式</h5>
                  <div class="prompt-block">
                    <ul>
                      <li>仅基于知识库中的文档内容回答问题</li>
                      <li>找不到相关内容时会明确告知</li>
                      <li>适合需要严格依据文档的场景</li>
                    </ul>
                  </div>
                  <h5>分析模式（推荐）</h5>
                  <div class="prompt-block">
                    <ul>
                      <li>结合知识库内容和 AI 的分析能力</li>
                      <li>可以对文档内容进行解读、总结、分析</li>
                      <li>平衡准确性和智能性，适合日常使用</li>
                    </ul>
                  </div>
                  <h5>直接对话模式</h5>
                  <div class="prompt-block">
                    <ul>
                      <li>不检索知识库，直接与 AI 对话</li>
                      <li>适合通用问题或不需要知识库参考的场景</li>
                    </ul>
                  </div>
                </div>
              </el-collapse-item>

              <el-collapse-item name="hybridSearch">
                <template #title>
                  <span class="agent-title">混合检索技术</span>
                  <span class="agent-subtitle">向量 + 关键词双重匹配</span>
                </template>
                <div class="prompt-section">
                  <h5>检索流程</h5>
                  <div class="prompt-block">
                    <p><strong>1. 向量搜索（语义匹配）：</strong></p>
                    <ul>
                      <li>将用户问题转换为向量</li>
                      <li>在向量库中查找语义相似的文档块</li>
                      <li>能理解同义词、近义表达</li>
                    </ul>
                    <p><strong>2. 关键词搜索（精确匹配）：</strong></p>
                    <ul>
                      <li>基于关键词的全文检索</li>
                      <li>确保包含特定术语的内容不会遗漏</li>
                    </ul>
                    <p><strong>3. Re-ranking 重排序：</strong></p>
                    <ul>
                      <li>使用 AI 对候选结果重新评估相关性</li>
                      <li>筛选出最相关的文档块提供给 AI 回答</li>
                    </ul>
                  </div>
                </div>
              </el-collapse-item>

              <el-collapse-item name="categoryManage">
                <template #title>
                  <span class="agent-title">分类管理</span>
                  <span class="agent-subtitle">灵活组织文档</span>
                </template>
                <div class="prompt-section">
                  <h5>分类功能</h5>
                  <ul>
                    <li><strong>新建分类：</strong>点击"新建分类"创建文档分类</li>
                    <li><strong>颜色标识：</strong>为每个分类设置不同颜色，便于区分</li>
                    <li><strong>拖拽排序：</strong>按住分类标签可拖拽调整顺序</li>
                    <li><strong>移动文档：</strong>右键菜单可将文档移动到其他分类</li>
                    <li><strong>重命名/删除：</strong>右键分类标签进行操作</li>
                  </ul>
                </div>
              </el-collapse-item>
            </el-collapse>

            <h4>使用流程</h4>
            <ol>
              <li>配置 AI 服务 API Key（建议配置 DeepSeek + 通义千问）</li>
              <li>点击"添加文档"或拖拽文件上传文档</li>
              <li>等待文档处理完成（解析 → 分块 → 向量化）</li>
              <li>为文档添加标签，方便后续筛选和管理</li>
              <li>建立文档间的双向链接，构建知识网络</li>
              <li>在对话界面输入问题，AI 会基于知识库内容回答</li>
              <li>点击"来源"展开查看回答的参考文档</li>
              <li>将有价值的 AI 回答保存为笔记，沉淀知识</li>
            </ol>

            <h4>注意事项</h4>
            <ul>
              <li><strong>API Key 要求：</strong>向量化需要 DeepSeek 或通义千问 API Key；图片识别需要通义千问或 Gemini</li>
              <li><strong>PDF OCR：</strong>扫描版 PDF 需要安装额外依赖，首次使用时会提示安装</li>
              <li><strong>处理时间：</strong>大文档的向量化需要一定时间，上传时可查看进度</li>
              <li><strong>向量化状态：</strong>文档列表显示向量化进度，未完成向量化的文档检索效果会受影响</li>
              <li><strong>对话上下文：</strong>保留最近 15 轮对话作为上下文，长对话建议新建会话</li>
              <li><strong>标签与分类：</strong>分类是文件夹式的单一归属，标签支持多维度打标，两者可配合使用</li>
              <li><strong>双向链接：</strong>链接是双向的，删除一个方向会同时删除另一方向的关联</li>
            </ul>
          </div>
          </div>

        </div><!-- .help-content-area -->
      </div><!-- .help-layout -->
    </el-dialog>

    <!-- 首次启动配置向导 -->
    <SetupWizardDialog
      v-model:visible="showSetupWizard"
      @complete="checkApiKeyStatus"
    />

    <!-- 监控设置弹窗 -->
    <SettingsDialog
      v-model="showSettingsDialog"
      :initial-tab="settingsInitialTab"
    />

    <!-- 快速批量添加监控弹窗 -->
    <QuickAddMonitoringDialog
      v-model="showQuickAddMonitoringDialog"
      :product-id="selectedProduct?.id ?? 0"
      :product-country="selectedProduct?.country ?? null"
      :keywords="selectedKeywords"
      @success="handleQuickAddMonitoringSuccess"
    />

    <!-- 更新下载进度弹窗 -->
    <el-dialog
      v-model="showUpdateDialog"
      title="正在下载更新"
      width="400px"
      :close-on-click-modal="false"
      :close-on-press-escape="false"
      :show-close="false"
    >
      <div class="update-progress">
        <div class="update-version">
          <el-icon class="update-icon"><Download /></el-icon>
          <span>正在下载 v{{ updateVersion }}</span>
        </div>
        <el-progress
          :percentage="updateProgress"
          :status="updateProgress >= 100 ? 'success' : undefined"
          :stroke-width="20"
          :text-inside="true"
        />
        <div class="update-hint" v-if="updateDownloading">
          请勿关闭应用，下载完成后将自动提示重启
        </div>
        <div class="update-hint success" v-else>
          下载完成！
        </div>
      </div>
    </el-dialog>

    <!-- 全局通知弹窗 -->
    <GlobalNotification @view-details="handleNotificationDetails" />

    <!-- 快捷备忘录浮动按钮 -->
    <QuickNotes />
  </div>
</template>

<style>
/* Google Fonts: Poppins + Open Sans */
@import url('https://fonts.googleapis.com/css2?family=Open+Sans:wght@300;400;500;600;700&family=Poppins:wght@400;500;600;700&display=swap');

:root {
  /* 亮色主题 */
  --bg-primary: #f5f7fa;
  --bg-secondary: #fff;
  --bg-hover: #f5f7fa;
  --bg-active: #ecf5ff;
  --text-primary: #303133;
  --text-secondary: #606266;
  --text-muted: #909399;
  --border-color: #e4e7ed;
  --accent-color: var(--accent-color);

  /* Glassmorphism 变量 - 亮色 */
  --glass-bg: rgba(255, 255, 255, 0.7);
  --glass-border: rgba(0, 0, 0, 0.08);
  --glass-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05), 0 10px 15px -3px rgba(0, 0, 0, 0.05);
  --glass-shadow-hover: 0 8px 12px -2px rgba(0, 0, 0, 0.08), 0 16px 24px -4px rgba(0, 0, 0, 0.06);

  /* 渐变背景 */
  --gradient-bg: linear-gradient(135deg, #f0f4ff 0%, #fdf4ff 50%, #fff7ed 100%);
  --gradient-primary: linear-gradient(135deg, #2563EB 0%, #3B82F6 100%);
  --gradient-success: linear-gradient(135deg, #10B981 0%, #34D399 100%);
  --gradient-purple: linear-gradient(135deg, #8B5CF6 0%, #A78BFA 100%);
  --gradient-orange: linear-gradient(135deg, #F97316 0%, #FB923C 100%);
}

html.dark {
  /* 深色主题 */
  --bg-primary: #1a1a1a;
  --bg-secondary: #242424;
  --bg-hover: #2c2c2c;
  --bg-active: #1a3a5c;
  --text-primary: #e5e5e5;
  --text-secondary: #a3a3a3;
  --text-muted: #737373;
  --border-color: #3a3a3a;
  --accent-color: var(--accent-color);

  /* Glassmorphism 变量 - 暗色 */
  --glass-bg: rgba(30, 41, 59, 0.7);
  --glass-border: rgba(255, 255, 255, 0.15);
  --glass-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.2), 0 10px 15px -3px rgba(0, 0, 0, 0.15);
  --glass-shadow-hover: 0 8px 12px -2px rgba(0, 0, 0, 0.25), 0 16px 24px -4px rgba(0, 0, 0, 0.2);

  /* 渐变背景 - 暗色 */
  --gradient-bg: linear-gradient(135deg, #1e293b 0%, #1e1b2e 50%, #1f1814 100%);
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html,
body,
#app {
  height: 100%;
  font-family: 'Open Sans', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue",
    Arial, sans-serif;
}

/* 标题字体 */
h1, h2, h3, h4, h5, h6,
.font-heading {
  font-family: 'Poppins', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
}

/* 表格排序箭头样式：放在文字后面同一行 */
.el-table th.el-table__cell > .cell {
  display: inline-flex !important;
  align-items: center;
  flex-wrap: nowrap;
}

.el-table th.el-table__cell .caret-wrapper {
  display: inline-flex;
  flex-direction: column;
  align-items: center;
  height: 14px;
  width: 24px;
  vertical-align: middle;
  overflow: initial;
  margin-left: 4px;
}

/* 更新下载进度弹窗样式 */
.update-progress {
  padding: 10px 0;
}

.update-version {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin-bottom: 20px;
  font-size: 16px;
  color: var(--text-primary);
}

.update-icon {
  font-size: 24px;
  color: var(--el-color-primary);
}

.update-hint {
  text-align: center;
  margin-top: 16px;
  font-size: 13px;
  color: var(--text-secondary);
}

.update-hint.success {
  color: var(--el-color-success);
  font-weight: 500;
}
</style>

<style scoped>
/* 汇率显示设置样式 */
.exchange-rate-settings {
  padding: 8px 0;
}

.exchange-rate-settings .settings-tip {
  font-size: 13px;
  color: var(--el-text-color-secondary);
  margin-bottom: 16px;
}

.exchange-rate-settings .currency-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.exchange-rate-settings .currency-option {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border: 1px solid var(--el-border-color);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.exchange-rate-settings .currency-option:hover {
  border-color: var(--el-color-primary-light-5);
  background: var(--el-color-primary-light-9);
}

.exchange-rate-settings .currency-option.selected {
  border-color: var(--el-color-primary);
  background: var(--el-color-primary-light-9);
}

.exchange-rate-settings .currency-flag {
  width: 24px;
  height: 16px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 2px;
  overflow: hidden;
}

.exchange-rate-settings .currency-flag :deep(svg) {
  width: 100%;
  height: 100%;
}

.exchange-rate-settings .currency-name {
  flex: 1;
  font-size: 14px;
  color: var(--el-text-color-primary);
}

.exchange-rate-settings .currency-code {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  font-family: monospace;
}

.exchange-rate-settings .check-icon {
  color: var(--el-color-primary);
  font-size: 16px;
}

.exchange-rate-settings .settings-counter {
  margin-top: 16px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
  text-align: center;
}

.app-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: var(--bg-primary);
  position: relative;
}

/* 顶部导航栏 */
.top-nav {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 10px 20px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
  position: relative;
}

.global-settings-dropdown {
  position: absolute;
  right: 80px;
}

.nav-help-btn {
  position: absolute;
  right: 20px;
}

/* 帮助对话框 - 左右分栏布局 */
.help-dialog .el-dialog__body {
  padding: 0 !important;
}

.help-layout {
  display: flex;
  height: 520px;
}

.help-nav {
  width: 200px;
  border-right: 1px solid var(--el-border-color-light);
  display: flex;
  flex-direction: column;
  background: var(--el-bg-color-page);
}

.help-search {
  margin: 12px;
  width: calc(100% - 24px);
}

.help-menu {
  flex: 1;
  border-right: none !important;
}

.help-menu .el-menu-item {
  height: 44px;
  line-height: 44px;
}

.help-menu .el-menu-item.is-active {
  background: var(--el-color-primary-light-9);
}

.help-shortcut-hint {
  padding: 12px;
  font-size: 12px;
  color: var(--el-text-color-placeholder);
  text-align: center;
  border-top: 1px solid var(--el-border-color-lighter);
}

.help-shortcut-hint kbd {
  display: inline-block;
  padding: 2px 6px;
  background: var(--el-fill-color);
  border: 1px solid var(--el-border-color);
  border-radius: 4px;
  font-family: inherit;
  font-size: 11px;
}

.help-no-result {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 24px 12px;
  color: var(--el-text-color-placeholder);
  font-size: 13px;
}

.help-no-result .el-icon {
  font-size: 24px;
}

.help-content-area {
  flex: 1;
  padding: 20px 24px;
  overflow-y: auto;
}

.help-dialog .help-content {
  line-height: 1.8;
  color: var(--el-text-color-regular);
}

.help-dialog .help-content h4 {
  margin: 16px 0 8px;
  color: var(--el-text-color-primary);
  font-size: 15px;
}

.help-dialog .help-content h4:first-child {
  margin-top: 0;
}

.help-dialog .help-content p {
  margin: 0 0 12px;
  color: var(--el-text-color-secondary);
}

.help-dialog .help-content ul,
.help-dialog .help-content ol {
  margin: 0 0 12px;
  padding-left: 20px;
}

.help-dialog .help-content li {
  margin-bottom: 6px;
}

.help-dialog .help-content strong {
  color: var(--el-text-color-primary);
}

/* AI 智能体提示词折叠面板 */
.agent-prompts-collapse {
  margin-bottom: 16px;
}

.agent-prompts-collapse .agent-title {
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin-right: 8px;
}

.agent-prompts-collapse .agent-subtitle {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.agent-prompts-collapse .prompt-section {
  padding: 0 8px;
}

.agent-prompts-collapse .prompt-section h5 {
  margin: 12px 0 6px;
  font-size: 13px;
  color: var(--el-color-primary);
  font-weight: 600;
}

.agent-prompts-collapse .prompt-section h5:first-child {
  margin-top: 0;
}

.agent-prompts-collapse .prompt-block {
  background: var(--el-fill-color-lighter);
  border-radius: 6px;
  padding: 10px 12px;
  margin-bottom: 8px;
}

.agent-prompts-collapse .prompt-block p {
  margin: 8px 0 4px;
  font-size: 13px;
}

.agent-prompts-collapse .prompt-block p:first-child {
  margin-top: 0;
}

.agent-prompts-collapse .prompt-block ul {
  margin: 4px 0 8px;
  padding-left: 18px;
}

.agent-prompts-collapse .prompt-block li {
  font-size: 12px;
  margin-bottom: 2px;
  color: var(--el-text-color-regular);
}

.agent-prompts-collapse .prompt-section > ul {
  padding-left: 18px;
}

.agent-prompts-collapse .prompt-section > ul li {
  font-size: 12px;
  margin-bottom: 4px;
}

/* 主体区域 */
.app-body {
  display: flex;
  flex: 1;
  min-height: 0;
}

.keyword-cell {
  display: flex;
  align-items: center;
  gap: 6px;
}

.keyword-link {
  color: var(--el-color-primary);
  cursor: pointer;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.keyword-link:hover {
  text-decoration: underline;
}

.copy-icon {
  color: var(--el-text-color-secondary);
  cursor: pointer;
  font-size: 14px;
  opacity: 0;
  transition: opacity 0.2s, color 0.2s;
  flex-shrink: 0;
}

.keyword-cell:hover .copy-icon {
  opacity: 1;
}

.copy-icon:hover {
  color: var(--el-color-primary);
}

.drop-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(64, 158, 255, 0.9);
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
}

.drop-content {
  text-align: center;
  color: #fff;
}

.drop-icon {
  font-size: 64px;
  margin-bottom: 16px;
}

.drop-content p {
  font-size: 20px;
  font-weight: 500;
}

/* 侧边栏样式 */
.sidebar {
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  min-width: 180px;
  max-width: 400px;
  height: 100%;
}

/* 拖动调整手柄 */
.resize-handle {
  width: 4px;
  cursor: col-resize;
  background: transparent;
  transition: background-color 0.2s;
  flex-shrink: 0;
}

.resize-handle:hover,
.resize-handle.resizing {
  background: var(--el-color-primary);
}

.sidebar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid var(--border-color);
}

.sidebar-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.product-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.product-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  margin-bottom: 4px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
}

.product-item:hover {
  background-color: var(--bg-primary);
}

.product-item.active {
  background-color: var(--bg-active);
  border: 1px solid var(--accent-color);
}

.product-info {
  flex: 1;
  min-width: 0;
}

.product-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.product-meta {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 4px;
  display: flex;
  gap: 8px;
}

.product-status {
  margin-top: 6px;
}

.product-status .el-tag {
  font-size: 11px;
}

.product-action {
  opacity: 0;
  transition: opacity 0.2s;
}

.product-item:hover .product-action {
  opacity: 1;
}

/* 空状态样式 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 40px 20px;
}

.empty-state .empty-icon {
  color: var(--el-color-info-light-3);
  margin-bottom: 16px;
}

.empty-state .empty-title {
  font-size: 16px;
  font-weight: 500;
  color: var(--text-primary);
  margin: 0 0 8px 0;
}

.empty-state .empty-desc {
  font-size: 14px;
  color: var(--text-muted);
  margin: 0 0 20px 0;
}

.sidebar-empty {
  flex: 1;
  padding: 60px 20px;
}

.sidebar-empty .empty-icon {
  color: var(--el-color-primary-light-5);
}

.main-empty {
  flex: 1;
  height: 100%;
}

.main-empty .empty-icon {
  color: var(--el-color-primary-light-3);
}

.table-empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
}

.table-empty-state .empty-icon {
  color: var(--el-color-info-light-3);
  margin-bottom: 16px;
}

.table-empty-state .empty-title {
  font-size: 15px;
  font-weight: 500;
  color: var(--text-primary);
  margin: 0 0 8px 0;
}

.table-empty-state .empty-desc {
  font-size: 13px;
  color: var(--text-muted);
  margin: 0 0 16px 0;
}

.sidebar-footer {
  padding: 12px 16px;
  border-top: 1px solid var(--border-color);
}

.theme-toggle {
  width: 100%;
  justify-content: flex-start;
  gap: 8px;
  color: var(--text-secondary);
}

.theme-toggle:hover {
  color: var(--accent-color);
}

.app-version {
  font-size: 11px;
  color: var(--text-muted);
  text-align: center;
  padding: 8px 0 4px;
}

/* 主内容区 */
.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
}

.header {
  display: flex;
  align-items: center;
  padding: 16px 24px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  gap: 24px;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: baseline;
  gap: 12px;
}

.title {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary);
}

.category-tabs {
  display: flex;
  gap: 8px;
  flex: 1;
  flex-wrap: nowrap;
  align-items: center;
}

.category-tag {
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 0 10px;
  height: 28px;
  font-size: 13px;
}

.more-tag {
  background-color: var(--bg-hover);
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-stats {
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: var(--text-secondary);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: auto;
}

.header-actions .help-btn {
  color: var(--el-text-color-secondary);
  border-color: var(--el-border-color-light);
}

.header-actions .help-btn:hover {
  color: var(--el-color-primary);
  border-color: var(--el-color-primary-light-5);
}

.table-container {
  flex: 1;
  overflow: auto;
  padding: 16px;
  min-height: 0;
}

.keyword-table-container {
  flex: 1;
  overflow: hidden;
  padding: 16px;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.keyword-table-container :deep(.el-table) {
  flex: 1;
}

.keyword-table-container :deep(.el-table__row) {
  height: 48px;
}

.empty-cell {
  color: var(--text-muted);
}

.wordcloud-container {
  flex: 1;
  overflow: auto;
  padding: 16px;
  min-height: 0;
  background: var(--el-bg-color);
  border-radius: 8px;
  margin: 0 16px;
}

.knowledge-base-view {
  flex: 1;
  overflow: hidden;
  min-height: 0;
  margin: 0 16px;
  background: var(--el-bg-color);
  border-radius: 8px;
}

.smart-copy-view {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
  margin: 0 16px;
  background: var(--el-bg-color);
  border-radius: 8px;
}

.ad-optimizer-view {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
  margin: 0 16px;
  background: var(--el-bg-color);
  border-radius: 8px;
}

.dashboard-view {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
  margin: 0 16px;
  background: var(--el-bg-color);
  border-radius: 8px;
}

.view-toggle {
  margin-right: 8px;
}

.table-container :deep(.el-table__row) {
  height: 52px;
}

.table-container :deep(.el-table__cell) {
  padding: 12px 0;
}

.table-container :deep(.el-table__header th .cell) {
  white-space: nowrap;
}

.word-cell {
  font-weight: 500;
  color: var(--accent-color);
}

.translation-cell {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  color: var(--text-secondary);
}

.translation-cell:hover .edit-icon {
  opacity: 1;
}

.edit-icon {
  opacity: 0;
  transition: opacity 0.2s;
  color: var(--text-muted);
}

.edit-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.sortable-header {
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
}

.category-cell {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  align-items: center;
}

.pagination {
  display: flex;
  justify-content: center;
  padding: 16px;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
  flex-shrink: 0;
}

/* 关键词筛选栏 */
.keyword-filter-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 20px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
}

.filter-result-count {
  margin-left: auto;
  color: var(--el-text-color-secondary);
  font-size: 13px;
}

.workflow-steps-header {
  flex: 1;
  display: flex;
  align-items: center;
  padding: 0 20px;
}

.workflow-steps-header .el-steps {
  flex: 1;
}

.workflow-steps-header .el-step__title {
  font-size: 13px;
}

/* 可编辑单元格 */
.editable-cell {
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 2px;
  display: inline-block;
  min-width: 20px;
  white-space: nowrap;
}

.editable-cell:hover {
  background: var(--el-fill-color-light);
}

/* 国家选择器 */
.country-option {
  display: flex;
  align-items: center;
  gap: 8px;
}

.country-flag {
  width: 24px;
  height: 16px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 2px;
  overflow: hidden;
  flex-shrink: 0;
}

.country-flag svg {
  width: 100%;
  height: 100%;
}

/* 侧边栏产品列表中的小国旗 */
.country-flag-small {
  width: 16px;
  height: 11px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 1px;
  overflow: hidden;
  flex-shrink: 0;
}

.country-flag-small svg {
  width: 100%;
  height: 100%;
}

.product-meta {
  display: flex;
  align-items: center;
  gap: 4px;
}

/* 排序图标动画 */
.keyword-table-container :deep(.el-table .caret-wrapper) {
  transition: transform 0.3s ease;
}

.keyword-table-container :deep(.el-table .ascending .caret-wrapper) {
  animation: sort-bounce 0.3s ease;
}

.keyword-table-container :deep(.el-table .descending .caret-wrapper) {
  animation: sort-bounce 0.3s ease;
}

.keyword-table-container :deep(.el-table .sort-caret) {
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.keyword-table-container :deep(.el-table .ascending .sort-caret.ascending) {
  transform: translateY(-2px) scale(1.2);
}

.keyword-table-container :deep(.el-table .descending .sort-caret.descending) {
  transform: translateY(2px) scale(1.2);
}

@keyframes sort-bounce {
  0% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.3);
  }
  100% {
    transform: scale(1);
  }
}
</style>

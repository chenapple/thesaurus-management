<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, defineAsyncComponent } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";
import { getVersion } from "@tauri-apps/api/app";
import * as XLSX from "xlsx";
import * as api from "./api";
import { batchAnalyzeWords, batchAnalyzeKeywordCategories } from "./deepseek";
import type { BackupInfo, Category, KeywordData, Product, Root, WorkflowStatus } from "./types";
import { EXCHANGE_RATE_CURRENCIES } from "./types";

// Composables
import { useSidebarResize } from "./composables/useSidebarResize";
import { useAutoUpdate } from "./composables/useAutoUpdate";
import { useDataImport } from "./composables/useDataImport";

// Lazy-loaded components
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
const OnboardingTour = defineAsyncComponent(() => import("./components/OnboardingTour.vue"));
const DashboardTab = defineAsyncComponent(() => import("./components/DashboardTab.vue"));
const SmartCopyTab = defineAsyncComponent(() => import("./components/SmartCopyTab.vue"));
const AdOptimizerTab = defineAsyncComponent(() => import("./components/AdOptimizerTab.vue"));
const AgentTab = defineAsyncComponent(() => import("./components/AgentTab.vue"));
const GlobalNotification = defineAsyncComponent(() => import("./components/GlobalNotification.vue"));
const QuickNotes = defineAsyncComponent(() => import("./components/QuickNotes.vue"));
const WeeklyReportTab = defineAsyncComponent(() => import("./components/WeeklyReportTab.vue"));

// New extracted components
const TopNavBar = defineAsyncComponent(() => import("./components/TopNavBar.vue"));
const ProductSidebar = defineAsyncComponent(() => import("./components/ProductSidebar.vue"));
const HelpDialog = defineAsyncComponent(() => import("./components/HelpDialog.vue"));
const KeywordsTab = defineAsyncComponent(() => import("./components/KeywordsTab.vue"));
const RootsTab = defineAsyncComponent(() => import("./components/RootsTab.vue"));

// ==================== Product state ====================
const products = ref<Product[]>([]);
const selectedProduct = ref<Product | null>(null);
const showProductDialog = ref(false);
const productForm = ref({ id: 0, name: "", country: "" });
const isEditingProduct = ref(false);

// Country options with SVG flags
const countryOptions = [
  { code: "US", name: "美国", flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="20" fill="#B22234"/><rect y="1.54" width="30" height="1.54" fill="white"/><rect y="4.62" width="30" height="1.54" fill="white"/><rect y="7.69" width="30" height="1.54" fill="white"/><rect y="10.77" width="30" height="1.54" fill="white"/><rect y="13.85" width="30" height="1.54" fill="white"/><rect y="16.92" width="30" height="1.54" fill="white"/><rect width="12" height="10.77" fill="#3C3B6E"/></svg>` },
  { code: "UK", name: "英国", flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="20" fill="#012169"/><path d="M0,0 L30,20 M30,0 L0,20" stroke="white" stroke-width="4"/><path d="M0,0 L30,20 M30,0 L0,20" stroke="#C8102E" stroke-width="2.5"/><path d="M15,0 V20 M0,10 H30" stroke="white" stroke-width="6"/><path d="M15,0 V20 M0,10 H30" stroke="#C8102E" stroke-width="3.5"/></svg>` },
  { code: "DE", name: "德国", flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="6.67" fill="#000"/><rect y="6.67" width="30" height="6.67" fill="#DD0000"/><rect y="13.33" width="30" height="6.67" fill="#FFCE00"/></svg>` },
  { code: "FR", name: "法国", flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="10" height="20" fill="#002395"/><rect x="10" width="10" height="20" fill="white"/><rect x="20" width="10" height="20" fill="#ED2939"/></svg>` },
  { code: "IT", name: "意大利", flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="10" height="20" fill="#009246"/><rect x="10" width="10" height="20" fill="white"/><rect x="20" width="10" height="20" fill="#CE2B37"/></svg>` },
  { code: "ES", name: "西班牙", flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="5" fill="#AA151B"/><rect y="5" width="30" height="10" fill="#F1BF00"/><rect y="15" width="30" height="5" fill="#AA151B"/></svg>` },
  { code: "JP", name: "日本", flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="20" fill="white"/><circle cx="15" cy="10" r="6" fill="#BC002D"/></svg>` },
];

// Amazon domain mapping
const amazonDomains: Record<string, string> = {
  US: "www.amazon.com",
  UK: "www.amazon.co.uk",
  DE: "www.amazon.de",
  FR: "www.amazon.fr",
  IT: "www.amazon.it",
  ES: "www.amazon.es",
  JP: "www.amazon.co.jp",
};

// ==================== Root state ====================
const categories = ref<Category[]>([]);
const roots = ref<Root[]>([]);
const total = ref(0);
const loading = ref(false);
const analyzing = ref(false);
const analysisProgress = ref({ current: 0, total: 0 });
const exporting = ref(false);
const analysisAbortController = ref<AbortController | null>(null);

// ==================== Keyword state ====================
const keywordData = ref<KeywordData[]>([]);
const keywordTotal = ref(0);
const keywordLoading = ref(false);
const keywordPage = ref(1);
const keywordPageSize = ref(50);
const keywordSearch = ref("");
const keywordSortBy = ref("id");
const keywordSortOrder = ref("asc");
const selectedKeywords = ref<KeywordData[]>([]);

// Keyword filters
const keywordFilters = ref({
  trafficLevel: [] as string[],
  relevanceLevel: [] as string[],
  primaryCategory: [] as string[],
  orderliness: [] as string[],
});

// Keyword export
const showKeywordExportDialog = ref(false);
const keywordExportScope = ref<'filtered' | 'all'>('filtered');
const keywordExporting = ref(false);

// Backup management
const showBackupDialog = ref(false);
const backups = ref<BackupInfo[]>([]);
const restoring = ref(false);

// Keyword classification
const classifying = ref(false);
const classifyProgress = ref({ current: 0, total: 0 });
const classifyAbortController = ref<AbortController | null>(null);

// Phrase tagging
const phraseTagging = ref(false);

// Workflow status
const workflowStatus = ref<WorkflowStatus>({
  has_data: false,
  has_traffic_level: false,
  has_category: false,
  has_phrase_tag: false,
  has_orderliness: false,
});

// Root filters and pagination
const searchText = ref("");
const selectedCategories = ref<number[]>([]);
const currentPage = ref(1);
const pageSize = ref(50);
const sortBy = ref("contains_count");
const sortOrder = ref("desc");

// Stats
const stats = ref({ keywordCount: 0, rootCount: 0 });
const categoryCounts = ref<Map<number, number>>(new Map());

// ==================== UI State ====================
const isDarkMode = ref(false);

// Use sidebar resize composable
const { sidebarWidth, isResizing, startResize } = useSidebarResize();

// Use auto update composable
const { showUpdateDialog, updateVersion, updateDownloading, updateProgress, checkForUpdates } = useAutoUpdate();

// Dialog states
const showShortcutsDialog = ref(false);
const showApiKeyDialog = ref(false);
const showExchangeRateSettings = ref(false);
const selectedCurrencies = ref<string[]>(['USD', 'EUR', 'GBP']);
const showHelpDialog = ref(false);
const showSetupWizard = ref(false);
const showSettingsDialog = ref(false);
const settingsInitialTab = ref<'monitoring' | 'auto' | 'logs'>('monitoring');
const showTrafficDialog = ref(false);
const showColumnConfig = ref(false);
const showQuickAddMonitoringDialog = ref(false);

// Onboarding tour
const onboardingTourRef = ref<InstanceType<typeof OnboardingTour> | null>(null);
const triggerOnboarding = ref(false);

// App version
const appVersion = ref("");

// View mode
const viewMode = ref<'dashboard' | 'keywords' | 'roots' | 'wordcloud' | 'monitoring' | 'smartcopy' | 'knowledge' | 'ads' | 'agent' | 'weekly_report'>('dashboard');
const enableAgent = import.meta.env.VITE_ENABLE_AGENT === 'true';

// Word cloud
const wordCloudRef = ref<InstanceType<typeof WordCloud> | null>(null);
const allRootsForCloud = ref<Root[]>([]);
const loadingCloud = ref(false);

// API Key status
const apiKeyStatus = ref({
  deepseek: false,
  qwen: false,
});

// Column configuration
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

const getDefaultColumnConfig = () => {
  const config: Record<string, boolean> = {};
  columnDefinitions.forEach(col => {
    config[col.key] = true;
  });
  return config;
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

const isAllColumnsSelected = computed(() => {
  return columnDefinitions.filter(col => !col.required).every(col => columnConfig.value[col.key]);
});

const toggleSelectAllColumns = (val: boolean) => {
  columnDefinitions.forEach(col => {
    if (!col.required) {
      columnConfig.value[col.key] = val;
    }
  });
  saveColumnConfig();
};

// Categories
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

// Has active filters
const hasActiveFilters = computed(() => {
  return !!(keywordSearch.value ||
    keywordFilters.value.trafficLevel.length > 0 ||
    keywordFilters.value.relevanceLevel.length > 0 ||
    keywordFilters.value.primaryCategory.length > 0 ||
    keywordFilters.value.orderliness.length > 0);
});

// ==================== Data Import (using composable) ====================
const {
  importing,
  isDragging,
  handleImport,
  setupDragDrop,
} = useDataImport({
  selectedProduct,
  products,
  viewMode,
  stats,
  loadKeywordData,
  loadRoots,
  loadStats,
  loadWorkflowStatus,
});

// ==================== Product Management ====================

async function loadProducts() {
  try {
    products.value = await api.getProducts();
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
  allRootsForCloud.value = [];
  keywordData.value = [];
  loadStats();
  loadWorkflowStatus();

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

    if (selectedProduct.value?.id === product.id) {
      selectedProduct.value = null;
    }
    await loadProducts();

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

// ==================== Categories and Roots ====================

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
      categoryIds: selectedCategories.value.length ? selectedCategories.value : undefined,
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
      pageSize: 500,
    });
    allRootsForCloud.value = data;
  } catch (e) {
    console.error("加载词云数据失败:", e);
  } finally {
    loadingCloud.value = false;
  }
}

// ==================== Keyword Data Management ====================

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

function handleKeywordSortChange({ prop, order }: { prop: string; order: string | null }) {
  if (order) {
    keywordSortBy.value = prop;
    keywordSortOrder.value = order === "ascending" ? "asc" : "desc";
  } else {
    keywordSortBy.value = "id";
    keywordSortOrder.value = "asc";
  }
  keywordPage.value = 1;
  loadKeywordData();
}

function handleKeywordSelectionChange(rows: KeywordData[]) {
  selectedKeywords.value = rows;
}

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

function handleFilterChange() {
  keywordPage.value = 1;
  loadKeywordData();
}

function clearKeywordSelection() {
  selectedKeywords.value = [];
}

function handleQuickAddMonitoringSuccess() {
  clearKeywordSelection();
  ElMessage.success('添加成功，可在「排名监控」标签页查看');
}

// ==================== Stats and Workflow ====================

async function loadStats() {
  if (!selectedProduct.value) {
    stats.value = { keywordCount: 0, rootCount: 0 };
    categoryCounts.value = new Map();
    return;
  }

  try {
    const [keywordCount, rootCount] = await api.getStats(selectedProduct.value.id);
    stats.value = { keywordCount, rootCount };

    const counts = await api.getCategoryCounts(selectedProduct.value.id);
    categoryCounts.value = new Map(counts);
  } catch (e) {
    console.error("加载统计失败:", e);
  }
}

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

// ==================== View Mode ====================

function switchViewMode(mode: 'dashboard' | 'keywords' | 'roots' | 'wordcloud' | 'monitoring' | 'smartcopy' | 'knowledge' | 'ads' | 'agent' | 'weekly_report') {
  viewMode.value = mode;
  if (mode === 'wordcloud' && allRootsForCloud.value.length === 0) {
    loadAllRootsForCloud();
  } else if (mode === 'keywords' && keywordData.value.length === 0) {
    loadKeywordData();
  } else if (mode === 'roots' && roots.value.length === 0) {
    loadRoots();
  }
}

function handleWordCloudClick(word: string) {
  searchText.value = word;
  viewMode.value = 'roots';
  handleSearch();
}

function handleNotificationDetails(_notification: { taskId: number; taskName: string; runId: number }) {
  if (enableAgent) {
    switchViewMode('agent');
  }
}

// ==================== Traffic Settings ====================

function openTrafficDialog() {
  if (!selectedProduct.value) {
    ElMessage.warning("请先选择一个产品");
    return;
  }
  showTrafficDialog.value = true;
}

async function onTrafficApplied(bigThreshold: number, mediumThreshold: number) {
  if (!selectedProduct.value) return;

  selectedProduct.value.big_word_threshold = bigThreshold;
  selectedProduct.value.medium_word_threshold = mediumThreshold;

  const idx = products.value.findIndex(p => p.id === selectedProduct.value!.id);
  if (idx >= 0) {
    products.value[idx].big_word_threshold = bigThreshold;
    products.value[idx].medium_word_threshold = mediumThreshold;
  }

  await loadKeywordData();
}

// ==================== AI Analysis ====================

async function handleAIAnalysis() {
  if (!selectedProduct.value) return;

  const unanalyzedRoots = roots.value.filter(r => !r.translation);
  if (unanalyzedRoots.length === 0) {
    ElMessage.info("所有词根已有翻译，无需分析");
    return;
  }

  // Build a map to look up root by word
  const rootMap = new Map(unanalyzedRoots.map(r => [r.word, r]));

  analyzing.value = true;
  analysisProgress.value = { current: 0, total: unanalyzedRoots.length };
  analysisAbortController.value = new AbortController();

  let negativeCount = 0;  // 统计否词数量

  try {
    await batchAnalyzeWords(
      unanalyzedRoots.map(r => r.word),
      {
        signal: analysisAbortController.value.signal,
        productName: selectedProduct.value.name,  // 传入产品名，用于否词判断
        onProgress: (current: number) => {
          analysisProgress.value.current = current;
        },
        onBatchComplete: async (results) => {
          for (const result of results) {
            const root = rootMap.get(result.word);
            if (root) {
              await api.updateRootTranslation(root.id, result.translation);
              // Map category names to IDs
              const categoryIds: number[] = [];
              for (const catName of result.categories) {
                const category = categories.value.find(c => c.name === catName);
                if (category) {
                  await api.addRootCategory(root.id, category.id);
                  categoryIds.push(category.id);
                }
              }
              root.translation = result.translation;
              root.categories = [...root.categories, ...categoryIds];

              // 处理否词标记
              if (result.is_negative && !root.is_negative) {
                await api.setRootNegative(root.id, true);
                root.is_negative = true;
                negativeCount++;
              }
            }
          }
        },
      }
    );
    const negativeMsg = negativeCount > 0 ? `，识别 ${negativeCount} 个否词` : "";
    ElMessage.success(`分析完成${negativeMsg}`);
  } catch (e) {
    if ((e as Error).name !== 'AbortError') {
      ElMessage.error("分析失败: " + e);
    }
  } finally {
    analyzing.value = false;
    analysisAbortController.value = null;
    await loadStats();
    await loadWorkflowStatus();
  }
}

function cancelAnalysis() {
  analysisAbortController.value?.abort();
  analyzing.value = false;
}

async function handleKeywordClassify() {
  if (!selectedProduct.value) return;

  const unclassifiedKeywords = keywordData.value.filter(k => !k.primary_category);
  if (unclassifiedKeywords.length === 0) {
    ElMessage.info("当前页所有关键词已分类");
    return;
  }

  // Build a map to look up keyword by keyword text
  const keywordMap = new Map(unclassifiedKeywords.map(k => [k.keyword, k]));

  classifying.value = true;
  classifyProgress.value = { current: 0, total: unclassifiedKeywords.length };
  classifyAbortController.value = new AbortController();

  try {
    await batchAnalyzeKeywordCategories(
      unclassifiedKeywords.map(k => ({ keyword: k.keyword, translation: k.translation })),
      {
        signal: classifyAbortController.value.signal,
        onProgress: (current: number) => {
          classifyProgress.value.current = current;
        },
        onBatchComplete: async (results) => {
          // Batch update to database
          const updates: [string, string, string, string][] = results.map(r => [
            r.keyword,
            r.primary_category,
            r.secondary_category,
            r.search_intent,
          ]);
          await api.batchUpdateKeywordCategories(selectedProduct.value!.id, updates);

          // Update local state
          for (const result of results) {
            const kw = keywordMap.get(result.keyword);
            if (kw) {
              kw.primary_category = result.primary_category;
              kw.secondary_category = result.secondary_category;
              kw.search_intent = result.search_intent;
            }
          }
        },
      }
    );
    ElMessage.success("分类完成");
  } catch (e) {
    if ((e as Error).name !== 'AbortError') {
      ElMessage.error("分类失败: " + e);
    }
  } finally {
    classifying.value = false;
    classifyAbortController.value = null;
    await loadWorkflowStatus();
  }
}

function cancelClassify() {
  classifyAbortController.value?.abort();
  classifying.value = false;
}

async function handlePhraseTagging() {
  if (!selectedProduct.value) return;

  phraseTagging.value = true;
  try {
    await api.calculatePhraseTags(selectedProduct.value.id);
    ElMessage.success("词组打标完成");
    await loadKeywordData();
    await loadWorkflowStatus();
  } catch (e) {
    ElMessage.error("打标失败: " + e);
  } finally {
    phraseTagging.value = false;
  }
}

// ==================== Export ====================

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
    const defaultFileName = `${selectedProduct.value.name}_词库分析_${new Date().toISOString().slice(0, 10)}.xlsx`;
    const filePath = await save({
      defaultPath: defaultFileName,
      filters: [{ name: "Excel文件", extensions: ["xlsx"] }],
    });

    if (!filePath) return;

    exporting.value = true;

    const [allRoots] = await api.getRoots({
      productId: selectedProduct.value.id,
      page: 1,
      pageSize: 100000,
    });

    const exportData = allRoots.map((root) => ({
      词根: root.word,
      中文翻译: root.translation || "",
      词根长度: root.word.length,
      包含词数: root.contains_count,
      词根占比: root.percentage.toFixed(2) + "%",
      分类: root.categories.map((id) => getCategoryName(id)).join(", "),
    }));

    const worksheet = XLSX.utils.json_to_sheet(exportData);
    const workbook = XLSX.utils.book_new();
    XLSX.utils.book_append_sheet(workbook, worksheet, "词根分析");

    worksheet["!cols"] = [
      { wch: 20 },
      { wch: 25 },
      { wch: 10 },
      { wch: 10 },
      { wch: 10 },
      { wch: 30 },
    ];

    const excelBuffer = XLSX.write(workbook, { bookType: "xlsx", type: "array" });
    await writeFile(filePath, new Uint8Array(excelBuffer));

    ElMessage.success(`成功导出 ${allRoots.length} 条词根数据`);
  } catch (e) {
    ElMessage.error("导出失败: " + e);
  } finally {
    exporting.value = false;
  }
}

function handleKeywordExport() {
  if (!selectedProduct.value) {
    ElMessage.warning("请先选择一个产品");
    return;
  }

  if (keywordTotal.value === 0) {
    ElMessage.warning("当前产品没有关键词数据可导出");
    return;
  }

  keywordExportScope.value = hasActiveFilters.value ? 'filtered' : 'all';
  showKeywordExportDialog.value = true;
}

async function executeKeywordExport() {
  if (!selectedProduct.value) return;

  try {
    const safeName = selectedProduct.value.name.replace(/[<>:"/\\|?*]/g, '_');
    const scopeSuffix = keywordExportScope.value === 'filtered' ? '_筛选结果' : '';
    const defaultFileName = `${safeName}_关键词数据${scopeSuffix}_${new Date().toISOString().slice(0, 10)}.xlsx`;

    const filePath = await save({
      defaultPath: defaultFileName,
      filters: [{ name: "Excel文件", extensions: ["xlsx"] }],
    });

    if (!filePath) return;

    keywordExporting.value = true;
    showKeywordExportDialog.value = false;

    let exportData: KeywordData[];

    if (keywordExportScope.value === 'all') {
      const [allData] = await api.getKeywordData({
        productId: selectedProduct.value.id,
        page: 1,
        pageSize: 1000000,
      });
      exportData = allData;
    } else {
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

    const enabledColumns = columnDefinitions.filter(col => columnConfig.value[col.key]);

    const formattedData = exportData.map(item => {
      const row: Record<string, string | number | null> = {};
      enabledColumns.forEach(col => {
        const value = item[col.key as keyof KeywordData];
        row[col.label] = formatExportCellValue(col.key, value);
      });
      return row;
    });

    const worksheet = XLSX.utils.json_to_sheet(formattedData);
    const workbook = XLSX.utils.book_new();
    XLSX.utils.book_append_sheet(workbook, worksheet, "关键词数据");

    worksheet["!cols"] = enabledColumns.map(col => ({
      wch: getExportColumnWidth(col.key),
    }));

    const excelBuffer = XLSX.write(workbook, { bookType: "xlsx", type: "array" });
    await writeFile(filePath, new Uint8Array(excelBuffer));

    ElMessage.success(`成功导出 ${exportData.length} 条关键词数据`);
  } catch (e) {
    ElMessage.error("导出失败: " + e);
  } finally {
    keywordExporting.value = false;
  }
}

function formatExportCellValue(key: string, value: unknown): string | number | null {
  if (value === null || value === undefined) return null;

  switch (key) {
    case 'traffic_share':
      return typeof value === 'number' ? value.toFixed(2) + '%' : String(value);
    case 'click_rate':
    case 'top3_click_share':
    case 'avg_conversion_share':
      return typeof value === 'number' || !isNaN(Number(value))
        ? (Number(value) * 100).toFixed(2) + '%'
        : String(value);
    case 'traffic_total':
    case 'avg_search_volume':
    case 'asin_count':
      return typeof value === 'number' ? value : String(value);
    default:
      return String(value);
  }
}

function getExportColumnWidth(key: string): number {
  const widthMap: Record<string, number> = {
    keyword: 30, translation: 25, traffic_level: 10, negative_word: 10,
    orderliness: 10, phrase_tag: 20, primary_category: 12, secondary_category: 12,
    search_intent: 15, traffic_share: 12, relevance_score: 12, relevance_level: 12,
    traffic_total: 12, avg_keyword_rank: 15, avg_search_volume: 15, cpc_bid: 12,
    bid_range: 15, click_rate: 12, conversion_competition: 12, competition_level: 12,
    natural_position_flow: 15, top3_click_share: 15, avg_conversion_share: 15, asin_count: 10,
  };
  return widthMap[key] || 12;
}

// ==================== Backup Management ====================

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
      { confirmButtonText: '确认回滚', cancelButtonText: '取消', type: 'warning' }
    );

    restoring.value = true;
    await api.restoreBackup(backup.id);
    ElMessage.success('数据已成功回滚');

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
      { confirmButtonText: '确认删除', cancelButtonText: '取消', type: 'warning' }
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

async function handleClearData() {
  if (!selectedProduct.value) {
    ElMessage.warning("请先选择一个产品");
    return;
  }

  try {
    await ElMessageBox.confirm(
      `确定要重置"${selectedProduct.value.name}"的词库吗？所有关键词和词根数据都会被删除，此操作不可恢复！`,
      "重置词库",
      { confirmButtonText: "确定重置", cancelButtonText: "取消", type: "warning" }
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

// ==================== Root Category ====================

async function toggleRootCategory(rootId: number, categoryId: number) {
  const root = roots.value.find(r => r.id === rootId);
  if (!root) return;

  try {
    if (root.categories.includes(categoryId)) {
      await api.removeRootCategory(root.id, categoryId);
      root.categories = root.categories.filter(c => c !== categoryId);
    } else {
      await api.addRootCategory(root.id, categoryId);
      root.categories.push(categoryId);
    }
    const counts = await api.getCategoryCounts(selectedProduct.value!.id);
    categoryCounts.value = new Map(counts);
  } catch (e) {
    ElMessage.error("操作失败: " + e);
  }
}

function toggleCategory(categoryId: number) {
  const idx = selectedCategories.value.indexOf(categoryId);
  if (idx >= 0) {
    selectedCategories.value.splice(idx, 1);
  } else {
    selectedCategories.value.push(categoryId);
  }
  currentPage.value = 1;
  loadRoots();
}

function handleSearch() {
  currentPage.value = 1;
  loadRoots();
}

function getCategoryName(id: number): string {
  const cat = categories.value.find((c) => c.id === id);
  return cat?.name || "";
}

function getCategoryCount(categoryId: number): number {
  return categoryCounts.value.get(categoryId) || 0;
}

// ==================== Theme ====================

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

// ==================== Exchange Rate Settings ====================

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

function saveExchangeRateSettings() {
  if (selectedCurrencies.value.length < 1 || selectedCurrencies.value.length > 5) {
    ElMessage.warning('请选择 1-5 个货币');
    return;
  }
  localStorage.setItem('exchange_rate_currencies', JSON.stringify(selectedCurrencies.value));
  window.dispatchEvent(new CustomEvent('exchange-rate-settings-changed'));
  showExchangeRateSettings.value = false;
  ElMessage.success('汇率显示设置已保存');
}

function toggleCurrencySelection(code: string) {
  const index = selectedCurrencies.value.indexOf(code);
  if (index > -1) {
    if (selectedCurrencies.value.length > 1) {
      selectedCurrencies.value.splice(index, 1);
    } else {
      ElMessage.warning('至少需要选择 1 个货币');
    }
  } else {
    if (selectedCurrencies.value.length < 5) {
      selectedCurrencies.value.push(code);
    } else {
      ElMessage.warning('最多只能选择 5 个货币');
    }
  }
}

// ==================== Settings ====================

function openSettingsTab(tab: 'monitoring' | 'auto' | 'logs') {
  settingsInitialTab.value = tab;
  showSettingsDialog.value = true;
}

async function checkApiKeyStatus() {
  try {
    apiKeyStatus.value.deepseek = await api.hasApiKey('deepseek');
    apiKeyStatus.value.qwen = await api.hasApiKey('qwen');
  } catch (e) {
    console.error('检查 API Key 状态失败:', e);
  }
}

async function checkSetupWizard() {
  try {
    const completed = await api.hasApiKey('__setup_wizard_completed');
    if (completed) return;

    const hasDeepseek = await api.hasApiKey('deepseek');
    const hasQwen = await api.hasApiKey('qwen');

    if (!hasDeepseek && !hasQwen) {
      showSetupWizard.value = true;
    }
  } catch (e) {
    console.error('检查向导状态失败:', e);
  }
}

// ==================== Onboarding Tour ====================

function handleStartOnboarding() {
  // 延迟 500ms 后启动教程，等待对话框关闭动画完成
  setTimeout(() => {
    triggerOnboarding.value = true;
  }, 500);
}

function handleOnboardingCompleted() {
  triggerOnboarding.value = false;
}

function restartOnboardingTour() {
  onboardingTourRef.value?.restartTour();
}

// ==================== Keyboard Shortcuts ====================

function handleKeyboard(e: KeyboardEvent) {
  const isMod = e.ctrlKey || e.metaKey;

  const isInputting =
    document.activeElement?.tagName === "INPUT" ||
    document.activeElement?.tagName === "TEXTAREA";

  if (isInputting && e.key !== "Escape") return;

  if (isMod && e.key === "n") {
    e.preventDefault();
    openAddProductDialog();
    return;
  }

  if (isMod && e.key === "i") {
    e.preventDefault();
    if (selectedProduct.value) handleImport();
    return;
  }

  if (isMod && e.key === "e") {
    e.preventDefault();
    if (selectedProduct.value) {
      if (viewMode.value === 'keywords') handleKeywordExport();
      else handleExport();
    }
    return;
  }

  if (isMod && e.key === "f") {
    e.preventDefault();
    const searchInput = document.querySelector(".header-right .el-input__inner") as HTMLInputElement;
    searchInput?.focus();
    return;
  }

  if (isMod && e.key === "Enter") {
    e.preventDefault();
    if (selectedProduct.value) {
      if (viewMode.value === 'keywords' && !classifying.value) handleKeywordClassify();
      else if (viewMode.value === 'roots' && !analyzing.value) handleAIAnalysis();
    }
    return;
  }

  if (isMod && e.key === "d") {
    e.preventDefault();
    toggleTheme();
    return;
  }

  if (isMod && e.key === "h") {
    e.preventDefault();
    showHelpDialog.value = true;
    return;
  }

  if (e.key === "?" || (isMod && e.key === "/")) {
    e.preventDefault();
    showShortcutsDialog.value = true;
    return;
  }

  if (e.key === "Escape") {
    if (showShortcutsDialog.value) showShortcutsDialog.value = false;
    return;
  }

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

// ==================== Initialization ====================

onMounted(async () => {
  initTheme();
  loadExchangeRateSettings();

  // 自动迁移旧版 API Key 到系统密钥链（首次升级时执行）
  try {
    const migrated = await api.migrateApiKeys();
    if (migrated.length > 0) {
      console.log('已迁移 API Key 到系统密钥链:', migrated);
    }
  } catch (e) {
    console.warn('API Key 迁移失败，将继续使用旧存储:', e);
  }

  await checkSetupWizard();
  await checkApiKeyStatus();
  appVersion.value = await getVersion();
  window.addEventListener("keydown", handleKeyboard);

  await loadProducts();
  await loadCategories();
  if (selectedProduct.value) {
    await loadStats();
    await loadWorkflowStatus();
    if (viewMode.value === 'keywords') {
      await loadKeywordData();
    } else if (viewMode.value === 'roots') {
      await loadRoots();
    }
  }
  await setupDragDrop();
  checkForUpdates();
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeyboard);
});
</script>

<template>
  <div class="app-container">
    <!-- Drag overlay -->
    <div v-if="isDragging" class="drop-overlay">
      <div class="drop-content">
        <el-icon class="drop-icon"><Upload /></el-icon>
        <p>释放以导入Excel文件</p>
      </div>
    </div>

    <!-- Top Navigation Bar -->
    <TopNavBar
      :view-mode="viewMode"
      :enable-agent="enableAgent"
      @switch-view="switchViewMode"
      @show-api-key-dialog="showApiKeyDialog = true"
      @show-shortcuts-dialog="showShortcutsDialog = true"
      @show-exchange-rate-settings="showExchangeRateSettings = true"
      @show-help-dialog="showHelpDialog = true"
    />

    <!-- Main body area -->
    <div class="app-body">
      <!-- Product Sidebar -->
      <ProductSidebar
        v-if="viewMode !== 'knowledge' && viewMode !== 'dashboard' && viewMode !== 'smartcopy' && viewMode !== 'ads' && viewMode !== 'agent' && viewMode !== 'weekly_report'"
        :products="products"
        :selected-product="selectedProduct"
        :sidebar-width="sidebarWidth"
        :is-dark-mode="isDarkMode"
        :workflow-status="workflowStatus"
        :country-options="countryOptions"
        @select-product="selectProduct"
        @add-product="openAddProductDialog"
        @edit-product="openEditProductDialog"
        @delete-product="deleteProduct"
        @toggle-theme="toggleTheme"
      />

      <!-- Resize handle -->
      <div
        v-if="viewMode !== 'knowledge' && viewMode !== 'dashboard' && viewMode !== 'smartcopy' && viewMode !== 'ads' && viewMode !== 'agent' && viewMode !== 'weekly_report'"
        class="resize-handle"
        :class="{ resizing: isResizing }"
        @mousedown="startResize"
      ></div>

      <!-- Main content -->
      <main class="main-content">
        <!-- Top toolbar -->
        <header v-if="viewMode !== 'knowledge' && viewMode !== 'dashboard' && viewMode !== 'smartcopy' && viewMode !== 'ads' && viewMode !== 'agent' && viewMode !== 'weekly_report'" class="header">
          <div class="header-left">
            <h1 class="title">{{ selectedProduct?.name || '请选择产品' }}</h1>
            <div class="header-stats" v-if="selectedProduct">
              <span>关键词: {{ stats.keywordCount }}</span>
              <span>词根: {{ stats.rootCount }}</span>
            </div>
          </div>

          <!-- Category tabs - roots view only -->
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

          <!-- Search box - roots view only -->
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

          <!-- Action buttons -->
          <div class="header-actions" v-if="selectedProduct">
            <!-- Keyword view actions -->
            <template v-if="viewMode === 'keywords'">
              <el-button size="small" @click="openTrafficDialog">流量设置</el-button>
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
              <el-button v-else size="small" type="danger" @click="cancelClassify">
                <el-icon><Close /></el-icon>
                停止 ({{ classifyProgress.current }}/{{ classifyProgress.total }})
              </el-button>
              <el-button size="small" :loading="phraseTagging" @click="handlePhraseTagging">
                {{ phraseTagging ? '打标中...' : '词组打标' }}
              </el-button>
            </template>

            <!-- Roots view actions -->
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
              <el-button v-else size="small" type="danger" @click="cancelAnalysis">
                <el-icon><Close /></el-icon>
                停止 ({{ analysisProgress.current }}/{{ analysisProgress.total }})
              </el-button>
            </template>

            <!-- Monitoring view actions -->
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

            <!-- Other views - data dropdown -->
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
                    <el-dropdown-item v-else @click="handleExport" :disabled="exporting">
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

        <!-- Keywords Tab -->
        <KeywordsTab
          v-if="selectedProduct && viewMode === 'keywords'"
          :selected-product="selectedProduct"
          :keyword-data="keywordData"
          :keyword-total="keywordTotal"
          :keyword-loading="keywordLoading"
          :keyword-page="keywordPage"
          :keyword-page-size="keywordPageSize"
          :keyword-search="keywordSearch"
          :keyword-filters="keywordFilters"
          :column-config="columnConfig"
          :has-active-filters="hasActiveFilters"
          :amazon-domains="amazonDomains"
          @update:keyword-page="keywordPage = $event"
          @update:keyword-page-size="keywordPageSize = $event"
          @update:keyword-search="keywordSearch = $event"
          @update:keyword-filters="keywordFilters = $event"
          @load-data="loadKeywordData"
          @filter-change="handleFilterChange"
          @reset-filters="resetKeywordFilters"
          @handle-import="handleImport"
          @sort-change="handleKeywordSortChange"
          @selection-change="handleKeywordSelectionChange"
          @show-quick-add-monitoring="showQuickAddMonitoringDialog = true"
        />

        <!-- Roots Tab -->
        <RootsTab
          v-if="selectedProduct && viewMode === 'roots'"
          :selected-product="selectedProduct"
          :roots="roots"
          :total="total"
          :loading="loading"
          :current-page="currentPage"
          :page-size="pageSize"
          :search-text="searchText"
          :categories="categories"
          :category-counts="categoryCounts"
          @update:current-page="currentPage = $event"
          @update:page-size="pageSize = $event"
          @update:search-text="searchText = $event"
          @load-roots="loadRoots"
          @toggle-category="toggleRootCategory"
          @switch-to-keywords="viewMode = 'keywords'"
          @negative-changed="loadKeywordData"
        />

        <!-- Word cloud view -->
        <div class="wordcloud-container" v-if="selectedProduct && viewMode === 'wordcloud'">
          <WordCloud
            ref="wordCloudRef"
            :roots="allRootsForCloud"
            :categories="categories"
            :loading="loadingCloud"
            @wordClick="handleWordCloudClick"
          />
        </div>

        <!-- Keyword Monitoring view -->
        <KeywordMonitoringTab
          v-if="selectedProduct && viewMode === 'monitoring'"
          :product-id="selectedProduct.id"
          @show-help="showHelpDialog = true"
        />

        <!-- Smart Copy view -->
        <keep-alive>
          <SmartCopyTab
            v-if="viewMode === 'smartcopy'"
            class="smart-copy-view"
            @show-help="showHelpDialog = true"
          />
        </keep-alive>

        <!-- Knowledge Base view -->
        <keep-alive>
          <KnowledgeBaseTab
            v-if="viewMode === 'knowledge'"
            class="knowledge-base-view"
            @show-help="showHelpDialog = true"
          />
        </keep-alive>

        <!-- Ad Optimizer view -->
        <keep-alive>
          <AdOptimizerTab
            v-if="viewMode === 'ads'"
            class="ad-optimizer-view"
            @show-help="showHelpDialog = true"
          />
        </keep-alive>

        <!-- Agent view -->
        <keep-alive v-if="enableAgent">
          <AgentTab
            v-if="viewMode === 'agent'"
            class="agent-view"
            @show-help="showHelpDialog = true"
          />
        </keep-alive>

        <!-- Weekly Report view -->
        <keep-alive>
          <WeeklyReportTab
            v-if="viewMode === 'weekly_report'"
            class="weekly-report-view"
            @show-help="showHelpDialog = true"
          />
        </keep-alive>

        <!-- Dashboard view -->
        <DashboardTab
          v-if="viewMode === 'dashboard'"
          :selected-product="selectedProduct"
          class="dashboard-view"
          @switch-view="switchViewMode"
          @show-help="showHelpDialog = true"
        />

        <!-- No product selected -->
        <div class="empty-state main-empty" v-if="!selectedProduct && viewMode !== 'dashboard' && viewMode !== 'smartcopy' && viewMode !== 'knowledge' && viewMode !== 'ads' && viewMode !== 'weekly_report'">
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
      </main>
    </div>

    <!-- Dialogs -->
    <ProductDialog
      v-model:visible="showProductDialog"
      v-model:form="productForm"
      :is-editing="isEditingProduct"
      :country-options="countryOptions"
      @save="saveProduct"
    />

    <ShortcutsDialog
      v-model:visible="showShortcutsDialog"
      :app-version="appVersion"
    />

    <!-- Exchange rate settings dialog -->
    <el-dialog
      v-model="showExchangeRateSettings"
      title="汇率显示设置"
      width="400px"
      destroy-on-close
    >
      <div class="exchange-rate-settings">
        <p class="settings-tip">选择要在概览页面轮播显示的货币汇率（最多 5 种）</p>
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
        <p class="settings-counter">已选择 {{ selectedCurrencies.length }} / 5 种货币</p>
      </div>
      <template #footer>
        <el-button @click="showExchangeRateSettings = false">取消</el-button>
        <el-button type="primary" @click="saveExchangeRateSettings" :disabled="selectedCurrencies.length < 1 || selectedCurrencies.length > 5">保存</el-button>
      </template>
    </el-dialog>

    <ColumnConfigDialog
      v-model:visible="showColumnConfig"
      :column-definitions="columnDefinitions"
      :column-config="columnConfig"
      :is-all-columns-selected="isAllColumnsSelected"
      @update:column-config="(key, val) => { columnConfig[key] = val; saveColumnConfig(); }"
      @toggle-all="toggleSelectAllColumns"
      @reset-default="columnConfig = getDefaultColumnConfig(); saveColumnConfig()"
    />

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

    <BackupDialog
      v-model:visible="showBackupDialog"
      :backups="backups"
      :restoring="restoring"
      @restore="handleRestoreBackup"
      @delete="handleDeleteBackup"
    />

    <TrafficSettingsDialog
      v-model:visible="showTrafficDialog"
      :product="selectedProduct"
      @applied="onTrafficApplied"
    />

    <ApiKeyDialog
      v-model:visible="showApiKeyDialog"
      @update:visible="(v) => !v && checkApiKeyStatus()"
    />

    <HelpDialog
      v-model:visible="showHelpDialog"
      @start-onboarding="restartOnboardingTour"
    />

    <SetupWizardDialog
      v-model:visible="showSetupWizard"
      @complete="checkApiKeyStatus"
      @start-onboarding="handleStartOnboarding"
    />

    <!-- Onboarding Tour -->
    <OnboardingTour
      ref="onboardingTourRef"
      :trigger="triggerOnboarding"
      @completed="handleOnboardingCompleted"
      @switch-view="switchViewMode"
    />

    <SettingsDialog
      v-model="showSettingsDialog"
      :initial-tab="settingsInitialTab"
    />

    <QuickAddMonitoringDialog
      v-model="showQuickAddMonitoringDialog"
      :product-id="selectedProduct?.id ?? 0"
      :product-country="selectedProduct?.country ?? null"
      :keywords="selectedKeywords"
      @success="handleQuickAddMonitoringSuccess"
    />

    <!-- Update download progress dialog -->
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

    <!-- Global notification -->
    <GlobalNotification @view-details="handleNotificationDetails" />

    <!-- Quick Notes floating button -->
    <QuickNotes />
  </div>
</template>

<style>
/* Google Fonts: Poppins + Open Sans */
@import url('https://fonts.googleapis.com/css2?family=Open+Sans:wght@300;400;500;600;700&family=Poppins:wght@400;500;600;700&display=swap');

:root {
  --bg-primary: #f5f7fa;
  --bg-secondary: #fff;
  --bg-hover: #f5f7fa;
  --bg-active: #ecf5ff;
  --text-primary: #303133;
  --text-secondary: #606266;
  --text-muted: #909399;
  --border-color: #e4e7ed;
  --accent-color: var(--accent-color);
  --glass-bg: rgba(255, 255, 255, 0.7);
  --glass-border: rgba(0, 0, 0, 0.08);
  --glass-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05), 0 10px 15px -3px rgba(0, 0, 0, 0.05);
  --glass-shadow-hover: 0 8px 12px -2px rgba(0, 0, 0, 0.08), 0 16px 24px -4px rgba(0, 0, 0, 0.06);
  --gradient-bg: linear-gradient(135deg, #f0f4ff 0%, #fdf4ff 50%, #fff7ed 100%);
  --gradient-primary: linear-gradient(135deg, #2563EB 0%, #3B82F6 100%);
  --gradient-success: linear-gradient(135deg, #10B981 0%, #34D399 100%);
  --gradient-purple: linear-gradient(135deg, #8B5CF6 0%, #A78BFA 100%);
  --gradient-orange: linear-gradient(135deg, #F97316 0%, #FB923C 100%);
}

html.dark {
  --bg-primary: #1a1a1a;
  --bg-secondary: #242424;
  --bg-hover: #2c2c2c;
  --bg-active: #1a3a5c;
  --text-primary: #e5e5e5;
  --text-secondary: #a3a3a3;
  --text-muted: #737373;
  --border-color: #3a3a3a;
  --accent-color: var(--accent-color);
  --glass-bg: rgba(30, 41, 59, 0.7);
  --glass-border: rgba(255, 255, 255, 0.15);
  --glass-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.2), 0 10px 15px -3px rgba(0, 0, 0, 0.15);
  --glass-shadow-hover: 0 8px 12px -2px rgba(0, 0, 0, 0.25), 0 16px 24px -4px rgba(0, 0, 0, 0.2);
  --gradient-bg: linear-gradient(135deg, #1e293b 0%, #1e1b2e 50%, #1f1814 100%);
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  height: 100%;
  font-family: 'Open Sans', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
}

h1, h2, h3, h4, h5, h6, .font-heading {
  font-family: 'Poppins', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
}

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

.update-progress { padding: 10px 0; }
.update-version { display: flex; align-items: center; justify-content: center; gap: 8px; margin-bottom: 20px; font-size: 16px; color: var(--text-primary); }
.update-icon { font-size: 24px; color: var(--el-color-primary); }
.update-hint { text-align: center; margin-top: 16px; font-size: 13px; color: var(--text-secondary); }
.update-hint.success { color: var(--el-color-success); font-weight: 500; }
</style>

<style scoped>
.exchange-rate-settings { padding: 8px 0; }
.exchange-rate-settings .settings-tip { font-size: 13px; color: var(--el-text-color-secondary); margin-bottom: 16px; }
.exchange-rate-settings .currency-options { display: flex; flex-direction: column; gap: 8px; }
.exchange-rate-settings .currency-option { display: flex; align-items: center; gap: 12px; padding: 10px 12px; border: 1px solid var(--el-border-color); border-radius: 8px; cursor: pointer; transition: all 0.2s; }
.exchange-rate-settings .currency-option:hover { border-color: var(--el-color-primary-light-5); background: var(--el-color-primary-light-9); }
.exchange-rate-settings .currency-option.selected { border-color: var(--el-color-primary); background: var(--el-color-primary-light-9); }
.exchange-rate-settings .currency-flag { width: 24px; height: 16px; display: inline-flex; align-items: center; justify-content: center; border-radius: 2px; overflow: hidden; }
.exchange-rate-settings .currency-flag :deep(svg) { width: 100%; height: 100%; }
.exchange-rate-settings .currency-name { flex: 1; font-size: 14px; color: var(--el-text-color-primary); }
.exchange-rate-settings .currency-code { font-size: 12px; color: var(--el-text-color-secondary); font-family: monospace; }
.exchange-rate-settings .check-icon { color: var(--el-color-primary); font-size: 16px; }
.exchange-rate-settings .settings-counter { margin-top: 16px; font-size: 13px; color: var(--el-text-color-secondary); text-align: center; }

.app-container { height: 100vh; display: flex; flex-direction: column; background-color: var(--bg-primary); position: relative; }
.app-body { display: flex; flex: 1; min-height: 0; }

.drop-overlay { position: absolute; top: 0; left: 0; right: 0; bottom: 0; background: rgba(64, 158, 255, 0.9); z-index: 9999; display: flex; align-items: center; justify-content: center; pointer-events: none; }
.drop-content { text-align: center; color: #fff; }
.drop-icon { font-size: 64px; margin-bottom: 16px; }
.drop-content p { font-size: 20px; font-weight: 500; }

.resize-handle { width: 4px; cursor: col-resize; background: transparent; transition: background-color 0.2s; flex-shrink: 0; }
.resize-handle:hover, .resize-handle.resizing { background: var(--el-color-primary); }

.main-content { flex: 1; display: flex; flex-direction: column; min-width: 0; overflow: hidden; }

.header { display: flex; align-items: center; padding: 16px 24px; background: var(--bg-secondary); border-bottom: 1px solid var(--border-color); gap: 24px; flex-shrink: 0; }
.header-left { display: flex; align-items: baseline; gap: 12px; }
.title { font-size: 20px; font-weight: 600; color: var(--text-primary); }
.header-stats { display: flex; gap: 16px; font-size: 13px; color: var(--text-secondary); }
.header-right { display: flex; align-items: center; gap: 12px; }
.header-actions { display: flex; align-items: center; gap: 8px; margin-left: auto; }

.category-tabs { display: flex; gap: 8px; flex: 1; flex-wrap: nowrap; align-items: center; }
.category-tag { cursor: pointer; display: flex; align-items: center; gap: 4px; padding: 0 10px; height: 28px; font-size: 13px; }
.more-tag { background-color: var(--bg-hover); }

.empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center; text-align: center; padding: 40px 20px; }
.empty-state .empty-icon { color: var(--el-color-info-light-3); margin-bottom: 16px; }
.empty-state .empty-title { font-size: 16px; font-weight: 500; color: var(--text-primary); margin: 0 0 8px 0; }
.empty-state .empty-desc { font-size: 14px; color: var(--text-muted); margin: 0 0 20px 0; }
.main-empty { flex: 1; height: 100%; }
.main-empty .empty-icon { color: var(--el-color-primary-light-3); }

.wordcloud-container { flex: 1; overflow: auto; padding: 16px; min-height: 0; background: var(--el-bg-color); border-radius: 8px; margin: 0 16px; }
.knowledge-base-view { flex: 1; overflow: hidden; min-height: 0; margin: 0 16px; background: var(--el-bg-color); border-radius: 8px; }
.smart-copy-view { flex: 1; overflow-y: auto; min-height: 0; margin: 0 16px; background: var(--el-bg-color); border-radius: 8px; }
.ad-optimizer-view { flex: 1; overflow-y: auto; min-height: 0; margin: 0 16px; background: var(--el-bg-color); border-radius: 8px; }
.dashboard-view { flex: 1; overflow-y: auto; min-height: 0; margin: 0 16px; background: var(--el-bg-color); border-radius: 8px; }
.agent-view { flex: 1; overflow-y: auto; min-height: 0; margin: 0 16px; background: var(--el-bg-color); border-radius: 8px; }
.weekly-report-view { flex: 1; overflow-y: auto; min-height: 0; margin: 0 16px; background: var(--el-bg-color); border-radius: 8px; }
</style>

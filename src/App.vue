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
];

// Amazon 域名映射
const amazonDomains: Record<string, string> = {
  US: "www.amazon.com",
  UK: "www.amazon.co.uk",
  DE: "www.amazon.de",
  FR: "www.amazon.fr",
  IT: "www.amazon.it",
  ES: "www.amazon.es",
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

// 应用版本
const appVersion = ref("");

// 自动更新相关状态
const showUpdateDialog = ref(false);
const updateVersion = ref("");
const updateDownloading = ref(false);
const updateProgress = ref(0);
const updateTotal = ref(0);

// 视图模式: 'keywords' | 'roots' | 'wordcloud'
const viewMode = ref<'keywords' | 'roots' | 'wordcloud'>('keywords');
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
function switchViewMode(mode: 'keywords' | 'roots' | 'wordcloud') {
  viewMode.value = mode;
  if (mode === 'wordcloud' && allRootsForCloud.value.length === 0) {
    loadAllRootsForCloud();
  } else if (mode === 'keywords' && keywordData.value.length === 0) {
    loadKeywordData();
  } else if (mode === 'roots' && roots.value.length === 0) {
    loadRoots();
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

// 计算当前流程步骤 (0-4)
const currentWorkflowStep = computed(() => {
  const s = workflowStatus.value;
  if (!s.has_data) return 0;
  if (!s.has_traffic_level) return 1;
  if (!s.has_category) return 2;
  if (!s.has_phrase_tag) return 3;
  if (!s.has_orderliness) return 4;
  return 5; // 全部完成
});

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
      const confirm = await ElMessageBox.confirm(
        `发现新版本 ${update.version}，是否立即更新？`,
        "版本更新",
        {
          confirmButtonText: "立即更新",
          cancelButtonText: "稍后提醒",
          type: "info",
        }
      );

      if (confirm) {
        // 显示下载进度弹窗
        updateVersion.value = update.version;
        updateProgress.value = 0;
        updateTotal.value = 0;
        updateDownloading.value = true;
        showUpdateDialog.value = true;

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
          confirmButtonText: "确定",
        });

        showUpdateDialog.value = false;
        await relaunch();
      }
    }
  } catch (e) {
    // 检查更新失败时静默处理，不影响用户使用
    console.log("检查更新失败:", e);
    showUpdateDialog.value = false;
    updateDownloading.value = false;
  }
}

// ==================== API Key 迁移 ====================

async function migrateApiKey() {
  try {
    // 检查是否已经配置了 API Key
    const hasKey = await api.hasApiKey("deepseek");
    if (!hasKey) {
      // 迁移旧的 API Key 到系统密钥链
      const oldApiKey = "sk-260241b985f243a78114c8f8d360c34c";
      await api.setApiKey("deepseek", oldApiKey);
      console.log("API Key 已迁移到系统密钥链");
    }
  } catch (e) {
    console.error("API Key 迁移失败:", e);
  }
}

// ==================== 初始化 ====================

onMounted(async () => {
  // 初始化主题
  initTheme();

  // 迁移 API Key（一次性）
  await migrateApiKey();

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

    <!-- 侧边栏 - 产品列表 -->
    <aside class="sidebar" :style="{ width: sidebarWidth + 'px' }">
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

    <!-- 拖动调整手柄 -->
    <div
      class="resize-handle"
      :class="{ resizing: isResizing }"
      @mousedown="startResize"
    ></div>

    <!-- 主内容区 -->
    <main class="main-content">
      <!-- 顶部工具栏 -->
      <header class="header">
        <div class="header-left">
          <h1 class="title">{{ selectedProduct?.name || '请选择产品' }}</h1>
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

        <!-- 流程进度条 - 仅关键词视图显示 -->
        <div class="workflow-steps-header" v-if="selectedProduct && viewMode === 'keywords'">
          <el-steps :active="currentWorkflowStep" finish-status="success" simple>
            <el-step title="导入数据" :status="workflowStatus.has_data ? 'success' : 'wait'" />
            <el-step title="流量级别" :status="workflowStatus.has_traffic_level ? 'success' : (workflowStatus.has_data ? 'process' : 'wait')" />
            <el-step title="AI分类" :status="workflowStatus.has_category ? 'success' : (workflowStatus.has_traffic_level ? 'process' : 'wait')" />
            <el-step title="词组打标" :status="workflowStatus.has_phrase_tag ? 'success' : (workflowStatus.has_category ? 'process' : 'wait')" />
            <el-step title="有序性" :status="workflowStatus.has_orderliness ? 'success' : (workflowStatus.has_phrase_tag ? 'process' : 'wait')" />
          </el-steps>
        </div>
      </header>

      <!-- 统计和操作栏 -->
      <div class="toolbar" v-if="selectedProduct">
        <div class="stats">
          <span>关键词: {{ stats.keywordCount }}</span>
          <span>词根: {{ stats.rootCount }}</span>
        </div>
        <div class="actions">
          <!-- 视图切换按钮组 -->
          <el-button-group class="view-toggle">
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
          </el-button-group>

          <el-divider direction="vertical" />

          <!-- 工作流操作 - 关键词视图 -->
          <template v-if="viewMode === 'keywords'">
            <el-button @click="openTrafficDialog">
              流量设置
            </el-button>
            <el-button
              v-if="!classifying"
              @click="handleKeywordClassify"
            >
              AI分类
            </el-button>
            <el-button
              v-else
              type="danger"
              @click="cancelClassify"
            >
              <el-icon><Close /></el-icon>
              停止分类 ({{ classifyProgress.current }}/{{ classifyProgress.total }})
            </el-button>
            <el-button
              :loading="phraseTagging"
              @click="handlePhraseTagging"
            >
              {{ phraseTagging ? '打标中...' : '词组打标' }}
            </el-button>
          </template>

          <!-- 工作流操作 - 词根视图 -->
          <template v-if="viewMode === 'roots'">
            <el-button
              v-if="!analyzing"
              type="success"
              @click="handleAIAnalysis"
            >
              <el-icon><MagicStick /></el-icon>
              智能分析
            </el-button>
            <el-button
              v-else
              type="danger"
              @click="cancelAnalysis"
            >
              <el-icon><Close /></el-icon>
              停止分析 ({{ analysisProgress.current }}/{{ analysisProgress.total }})
            </el-button>
          </template>

          <el-divider direction="vertical" />

          <!-- 数据管理下拉菜单 -->
          <el-dropdown trigger="click">
            <el-button>
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
                <el-dropdown-item @click="showApiKeyDialog = true">
                  <el-icon><Key /></el-icon> API Key 设置
                </el-dropdown-item>
                <el-dropdown-item divided @click="handleClearData">
                  <el-icon color="#f56c6c"><Delete /></el-icon>
                  <span style="color: #f56c6c">重置词库</span>
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>

          <el-button @click="showColumnConfig = true">
            <el-icon><Setting /></el-icon>
            列配置
          </el-button>
        </div>
      </div>

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
      </div>

      <!-- 关键词表格 -->
      <div class="keyword-table-container" v-if="selectedProduct && viewMode === 'keywords'">
        <el-table
          :data="keywordData"
          v-loading="keywordLoading"
          stripe
          style="width: 100%"
          height="100%"
          @sort-change="handleKeywordSortChange"
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
  </div>
</template>

<style>
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
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue",
    Arial, sans-serif;
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
.app-container {
  height: 100vh;
  display: flex;
  background-color: var(--bg-primary);
  position: relative;
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
  gap: 12px;
  flex: 1;
  flex-wrap: wrap;
}

.category-tag {
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 16px;
  height: 36px;
  font-size: 14px;
}

.more-tag {
  background-color: var(--bg-hover);
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 20px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.stats {
  display: flex;
  gap: 20px;
  font-size: 14px;
  color: var(--text-secondary);
}

.actions {
  display: flex;
  gap: 10px;
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

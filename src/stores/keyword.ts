import { defineStore } from "pinia";
import { ref, computed } from "vue";
import * as api from "../api";
import type { KeywordData, Category, Root, TrafficLevelStats } from "../types";
import { useProductStore } from "./product";

// 筛选选项
export const trafficLevelOptions = ["大词", "中词", "小词"];
export const relevanceLevelOptions = ["强相关", "高相关", "中相关", "弱相关"];
export const orderlinessOptions = ["有序", "无序"];
export const primaryCategoryOptions = ["品类词", "功能词", "场景词", "属性词", "品牌词", "人群词", "受众词", "其他"];

export const useKeywordStore = defineStore("keyword", () => {
  const productStore = useProductStore();

  // ==================== 关键词数据状态 ====================
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
    trafficLevel: [] as string[],
    relevanceLevel: [] as string[],
    primaryCategory: [] as string[],
    orderliness: [] as string[],
  });

  // ==================== 词根数据状态 ====================
  const categories = ref<Category[]>([]);
  const roots = ref<Root[]>([]);
  const rootTotal = ref(0);
  const rootLoading = ref(false);
  const rootPage = ref(1);
  const rootPageSize = ref(50);
  const rootSearch = ref("");
  const rootSortBy = ref("contains_count");
  const rootSortOrder = ref("desc");
  const selectedCategories = ref<number[]>([]);
  const categoryCounts = ref<Map<number, number>>(new Map());

  // ==================== 流量设置 ====================
  const trafficStats = ref<TrafficLevelStats>({
    big_count: 0,
    medium_count: 0,
    small_count: 0,
  });

  // ==================== 计算属性 ====================
  const hasActiveFilters = computed(() => {
    return !!(
      keywordSearch.value ||
      keywordFilters.value.trafficLevel.length > 0 ||
      keywordFilters.value.relevanceLevel.length > 0 ||
      keywordFilters.value.primaryCategory.length > 0 ||
      keywordFilters.value.orderliness.length > 0
    );
  });

  // ==================== 关键词方法 ====================

  // 加载关键词数据
  async function loadKeywordData() {
    if (!productStore.selectedProduct) return;

    keywordLoading.value = true;
    try {
      const [data, total] = await api.getKeywordData({
        productId: productStore.selectedProduct.id,
        search: keywordSearch.value || undefined,
        trafficLevels: keywordFilters.value.trafficLevel.length > 0 ? keywordFilters.value.trafficLevel : undefined,
        relevanceLevels: keywordFilters.value.relevanceLevel.length > 0 ? keywordFilters.value.relevanceLevel : undefined,
        primaryCategories: keywordFilters.value.primaryCategory.length > 0 ? keywordFilters.value.primaryCategory : undefined,
        orderlinessValues: keywordFilters.value.orderliness.length > 0 ? keywordFilters.value.orderliness : undefined,
        sortBy: keywordSortBy.value,
        sortOrder: keywordSortOrder.value,
        page: keywordPage.value,
        pageSize: keywordPageSize.value,
      });
      keywordData.value = data;
      keywordTotal.value = total;
    } finally {
      keywordLoading.value = false;
    }
  }

  // 重置关键词筛选
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

  // 更新关键词字段
  async function updateKeywordField(id: number, field: string, value: string) {
    await api.updateKeywordField(id, field, value);
    // 更新本地数据
    const item = keywordData.value.find(k => k.id === id);
    if (item) {
      (item as Record<string, unknown>)[field] = value;
    }
  }

  // 清空关键词数据
  async function clearKeywordData() {
    if (!productStore.selectedProduct) return;
    await api.clearKeywordData(productStore.selectedProduct.id);
    keywordData.value = [];
    keywordTotal.value = 0;
  }

  // ==================== 词根方法 ====================

  // 加载分类
  async function loadCategories() {
    categories.value = await api.getCategories();
  }

  // 加载词根数据
  async function loadRoots() {
    if (!productStore.selectedProduct) return;

    rootLoading.value = true;
    try {
      const [data, total] = await api.getRoots({
        productId: productStore.selectedProduct.id,
        search: rootSearch.value || undefined,
        categoryIds: selectedCategories.value.length > 0 ? selectedCategories.value : undefined,
        sortBy: rootSortBy.value,
        sortOrder: rootSortOrder.value,
        page: rootPage.value,
        pageSize: rootPageSize.value,
      });
      roots.value = data;
      rootTotal.value = total;
    } finally {
      rootLoading.value = false;
    }
  }

  // 加载分类统计
  async function loadCategoryCounts() {
    if (!productStore.selectedProduct) return;
    const counts = await api.getCategoryCounts(productStore.selectedProduct.id);
    categoryCounts.value = new Map(counts);
  }

  // 更新词根翻译
  async function updateRootTranslation(id: number, translation: string) {
    await api.updateRootTranslation(id, translation);
    const root = roots.value.find(r => r.id === id);
    if (root) {
      root.translation = translation;
    }
  }

  // 添加词根分类
  async function addRootCategory(rootId: number, categoryId: number) {
    await api.addRootCategory(rootId, categoryId);
    const root = roots.value.find(r => r.id === rootId);
    if (root && !root.categories.includes(categoryId)) {
      root.categories.push(categoryId);
    }
    await loadCategoryCounts();
  }

  // 移除词根分类
  async function removeRootCategory(rootId: number, categoryId: number) {
    await api.removeRootCategory(rootId, categoryId);
    const root = roots.value.find(r => r.id === rootId);
    if (root) {
      root.categories = root.categories.filter(c => c !== categoryId);
    }
    await loadCategoryCounts();
  }

  // ==================== 流量设置方法 ====================

  // 加载流量统计
  async function loadTrafficStats() {
    if (!productStore.selectedProduct) return;
    trafficStats.value = await api.getTrafficLevelStats(productStore.selectedProduct.id);
  }

  // 计算流量级别
  async function calculateTrafficLevels(bigThreshold: number, mediumThreshold: number) {
    if (!productStore.selectedProduct) return;
    await api.calculateTrafficLevels(productStore.selectedProduct.id, bigThreshold, mediumThreshold);
    await loadTrafficStats();
    await productStore.loadWorkflowStatus();
  }

  // 推荐阈值
  async function recommendThreshold(targetBigCount: number = 20): Promise<number> {
    if (!productStore.selectedProduct) return 20000;
    return await api.recommendThreshold(productStore.selectedProduct.id, targetBigCount);
  }

  // 计算流量占比
  async function calculateTrafficShare() {
    if (!productStore.selectedProduct) return;
    await api.calculateTrafficShare(productStore.selectedProduct.id);
  }

  // 计算词组标签
  async function calculatePhraseTags() {
    if (!productStore.selectedProduct) return;
    await api.calculatePhraseTags(productStore.selectedProduct.id);
    await productStore.loadWorkflowStatus();
  }

  // 计算有序性
  async function calculateOrderliness() {
    if (!productStore.selectedProduct) return;
    await api.calculateOrderliness(productStore.selectedProduct.id);
    await productStore.loadWorkflowStatus();
  }

  // ==================== 重置状态 ====================
  function resetState() {
    keywordData.value = [];
    keywordTotal.value = 0;
    keywordPage.value = 1;
    keywordSearch.value = "";
    keywordFilters.value = {
      trafficLevel: [],
      relevanceLevel: [],
      primaryCategory: [],
      orderliness: [],
    };

    roots.value = [];
    rootTotal.value = 0;
    rootPage.value = 1;
    rootSearch.value = "";
    selectedCategories.value = [];

    trafficStats.value = { big_count: 0, medium_count: 0, small_count: 0 };
  }

  return {
    // 关键词状态
    keywordData,
    keywordTotal,
    keywordLoading,
    keywordPage,
    keywordPageSize,
    keywordSearch,
    keywordSortBy,
    keywordSortOrder,
    keywordFilters,
    hasActiveFilters,

    // 词根状态
    categories,
    roots,
    rootTotal,
    rootLoading,
    rootPage,
    rootPageSize,
    rootSearch,
    rootSortBy,
    rootSortOrder,
    selectedCategories,
    categoryCounts,

    // 流量状态
    trafficStats,

    // 关键词方法
    loadKeywordData,
    resetKeywordFilters,
    handleFilterChange,
    updateKeywordField,
    clearKeywordData,

    // 词根方法
    loadCategories,
    loadRoots,
    loadCategoryCounts,
    updateRootTranslation,
    addRootCategory,
    removeRootCategory,

    // 流量方法
    loadTrafficStats,
    calculateTrafficLevels,
    recommendThreshold,
    calculateTrafficShare,
    calculatePhraseTags,
    calculateOrderliness,

    // 工具方法
    resetState,
  };
});

import { defineStore } from "pinia";
import { ref, computed } from "vue";
import * as api from "../api";
import type { Product, BackupInfo, WorkflowStatus } from "../types";

// 国家选项（使用SVG国旗）
export const countryOptions = [
  { code: "US", name: "美国", flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="20" fill="#B22234"/><rect y="1.54" width="30" height="1.54" fill="white"/><rect y="4.62" width="30" height="1.54" fill="white"/><rect y="7.69" width="30" height="1.54" fill="white"/><rect y="10.77" width="30" height="1.54" fill="white"/><rect y="13.85" width="30" height="1.54" fill="white"/><rect y="16.92" width="30" height="1.54" fill="white"/><rect width="12" height="10.77" fill="#3C3B6E"/></svg>` },
  { code: "UK", name: "英国", flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="20" fill="#012169"/><path d="M0,0 L30,20 M30,0 L0,20" stroke="white" stroke-width="4"/><path d="M0,0 L30,20 M30,0 L0,20" stroke="#C8102E" stroke-width="2.5"/><path d="M15,0 V20 M0,10 H30" stroke="white" stroke-width="6"/><path d="M15,0 V20 M0,10 H30" stroke="#C8102E" stroke-width="3.5"/></svg>` },
  { code: "DE", name: "德国", flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="6.67" fill="#000"/><rect y="6.67" width="30" height="6.67" fill="#DD0000"/><rect y="13.33" width="30" height="6.67" fill="#FFCE00"/></svg>` },
  { code: "FR", name: "法国", flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="10" height="20" fill="#002395"/><rect x="10" width="10" height="20" fill="white"/><rect x="20" width="10" height="20" fill="#ED2939"/></svg>` },
  { code: "IT", name: "意大利", flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="10" height="20" fill="#009246"/><rect x="10" width="10" height="20" fill="white"/><rect x="20" width="10" height="20" fill="#CE2B37"/></svg>` },
  { code: "ES", name: "西班牙", flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="5" fill="#AA151B"/><rect y="5" width="30" height="10" fill="#F1BF00"/><rect y="15" width="30" height="5" fill="#AA151B"/></svg>` },
];

// Amazon 域名映射
export const amazonDomains: Record<string, string> = {
  US: "www.amazon.com",
  UK: "www.amazon.co.uk",
  DE: "www.amazon.de",
  FR: "www.amazon.fr",
  IT: "www.amazon.it",
  ES: "www.amazon.es",
};

export const useProductStore = defineStore("product", () => {
  // ==================== 状态 ====================
  const products = ref<Product[]>([]);
  const selectedProduct = ref<Product | null>(null);
  const loading = ref(false);

  // 备份相关
  const backups = ref<BackupInfo[]>([]);
  const restoring = ref(false);

  // 流程状态
  const workflowStatus = ref<WorkflowStatus>({
    has_data: false,
    has_traffic_level: false,
    has_category: false,
    has_phrase_tag: false,
    has_orderliness: false,
  });

  // 统计
  const stats = ref({ keywordCount: 0, rootCount: 0 });

  // ==================== 计算属性 ====================
  const hasSelectedProduct = computed(() => !!selectedProduct.value);

  const selectedProductId = computed(() => selectedProduct.value?.id || null);

  // ==================== 方法 ====================

  // 加载产品列表
  async function loadProducts() {
    try {
      loading.value = true;
      products.value = await api.getProducts();
    } finally {
      loading.value = false;
    }
  }

  // 选择产品
  async function selectProduct(product: Product | null) {
    selectedProduct.value = product;
    if (product) {
      await Promise.all([
        loadStats(),
        loadWorkflowStatus(),
      ]);
    } else {
      stats.value = { keywordCount: 0, rootCount: 0 };
      workflowStatus.value = {
        has_data: false,
        has_traffic_level: false,
        has_category: false,
        has_phrase_tag: false,
        has_orderliness: false,
      };
    }
  }

  // 创建产品
  async function createProduct(name: string, country?: string) {
    const id = await api.createProduct(name, country);
    await loadProducts();
    return id;
  }

  // 更新产品
  async function updateProduct(id: number, name: string, country?: string) {
    await api.updateProduct(id, name, country);
    await loadProducts();
    // 如果更新的是当前选中的产品，更新 selectedProduct
    if (selectedProduct.value?.id === id) {
      const updated = products.value.find(p => p.id === id);
      if (updated) {
        selectedProduct.value = updated;
      }
    }
  }

  // 删除产品
  async function deleteProduct(id: number) {
    await api.deleteProduct(id);
    if (selectedProduct.value?.id === id) {
      selectedProduct.value = null;
    }
    await loadProducts();
  }

  // 加载统计
  async function loadStats() {
    if (!selectedProduct.value) return;
    const [keywordCount, rootCount] = await api.getStats(selectedProduct.value.id);
    stats.value = { keywordCount, rootCount };
  }

  // 加载流程状态
  async function loadWorkflowStatus() {
    if (!selectedProduct.value) return;
    workflowStatus.value = await api.getWorkflowStatus(selectedProduct.value.id);
  }

  // 加载备份列表
  async function loadBackups() {
    if (!selectedProduct.value) {
      backups.value = [];
      return;
    }
    try {
      backups.value = await api.getBackups(selectedProduct.value.id);
    } catch (e) {
      console.error("Failed to load backups:", e);
    }
  }

  // 创建备份
  async function createBackup(backupName?: string) {
    if (!selectedProduct.value) return;
    await api.createBackup(selectedProduct.value.id, backupName);
  }

  // 恢复备份
  async function restoreBackup(backupId: number) {
    restoring.value = true;
    try {
      await api.restoreBackup(backupId);
      await loadStats();
      await loadWorkflowStatus();
    } finally {
      restoring.value = false;
    }
  }

  // 删除备份
  async function deleteBackup(backupId: number) {
    await api.deleteBackup(backupId);
    await loadBackups();
  }

  // 清空产品数据
  async function clearProductData() {
    if (!selectedProduct.value) return;
    await api.clearProductData(selectedProduct.value.id);
    await loadStats();
    await loadWorkflowStatus();
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

  // 获取 Amazon 域名
  function getAmazonDomain(): string | null {
    if (!selectedProduct.value?.country) return null;
    return amazonDomains[selectedProduct.value.country] || null;
  }

  return {
    // 状态
    products,
    selectedProduct,
    loading,
    backups,
    restoring,
    workflowStatus,
    stats,

    // 计算属性
    hasSelectedProduct,
    selectedProductId,

    // 方法
    loadProducts,
    selectProduct,
    createProduct,
    updateProduct,
    deleteProduct,
    loadStats,
    loadWorkflowStatus,
    loadBackups,
    createBackup,
    restoreBackup,
    deleteBackup,
    clearProductData,
    getCountryName,
    getCountryFlag,
    getAmazonDomain,
  };
});

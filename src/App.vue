<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { open, save } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { readFile, writeFile } from "@tauri-apps/plugin-fs";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import * as XLSX from "xlsx";
import * as api from "./api";
import { batchAnalyzeWords } from "./deepseek";
import type { Category, Product, Root } from "./types";
import type { UnlistenFn } from "@tauri-apps/api/event";
import WordCloud from "./components/WordCloud.vue";

// ==================== 产品相关状态 ====================
const products = ref<Product[]>([]);
const selectedProduct = ref<Product | null>(null);
const showProductDialog = ref(false);
const productForm = ref({ id: 0, name: "", sku: "", asin: "" });
const isEditingProduct = ref(false);

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

// 快捷键帮助弹窗
const showShortcutsDialog = ref(false);

// 视图模式: 'table' | 'wordcloud'
const viewMode = ref<'table' | 'wordcloud'>('table');
const wordCloudRef = ref<InstanceType<typeof WordCloud> | null>(null);
const allRootsForCloud = ref<Root[]>([]);

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
  loadRoots();
  loadStats();
}

function openAddProductDialog() {
  productForm.value = { id: 0, name: "", sku: "", asin: "" };
  isEditingProduct.value = false;
  showProductDialog.value = true;
}

function openEditProductDialog(product: Product) {
  productForm.value = {
    id: product.id,
    name: product.name,
    sku: product.sku || "",
    asin: product.asin || "",
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
        productForm.value.sku || undefined,
        productForm.value.asin || undefined
      );
      ElMessage.success("产品已更新");
    } else {
      const newId = await api.createProduct(
        productForm.value.name,
        productForm.value.sku || undefined,
        productForm.value.asin || undefined
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
  }
}

// 切换视图模式
function switchViewMode(mode: 'table' | 'wordcloud') {
  viewMode.value = mode;
  if (mode === 'wordcloud' && allRootsForCloud.value.length === 0) {
    loadAllRootsForCloud();
  }
}

// 词云点击处理
function handleWordCloudClick(word: string) {
  searchText.value = word;
  viewMode.value = 'table';
  handleSearch();
}

// 导出词云图片
function exportWordCloudImage() {
  wordCloudRef.value?.exportImage();
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

// ==================== 导入功能 ====================

async function processExcelBuffer(buffer: ArrayBuffer) {
  if (!selectedProduct.value) {
    ElMessage.warning("请先选择或创建一个产品");
    return;
  }

  const workbook = XLSX.read(buffer, { type: "array" });
  const sheetName = workbook.SheetNames[0];
  const sheet = workbook.Sheets[sheetName];
  const data = XLSX.utils.sheet_to_json<{ [key: string]: string }>(sheet);

  const keywords: string[] = [];
  for (const row of data) {
    const firstKey = Object.keys(row)[0];
    if (firstKey && row[firstKey]) {
      keywords.push(String(row[firstKey]));
    }
  }

  if (keywords.length === 0) {
    ElMessage.warning("Excel中没有找到关键词");
    return;
  }

  await api.importKeywords(selectedProduct.value.id, keywords);
  ElMessage.success(`成功导入 ${keywords.length} 个关键词到"${selectedProduct.value.name}"`);

  await loadRoots();
  await loadStats();
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
    const response = await fetch(`file://${selected}`);
    const buffer = await response.arrayBuffer();
    await processExcelBuffer(buffer);
  } catch (e) {
    ElMessage.error("导入失败: " + e);
  } finally {
    importing.value = false;
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

function handleSort(column: string) {
  if (sortBy.value === column) {
    sortOrder.value = sortOrder.value === "asc" ? "desc" : "asc";
  } else {
    sortBy.value = column;
    sortOrder.value = "desc";
  }
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
      handleExport();
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

  // Ctrl/Cmd + Enter: 开始智能分析
  if (isMod && e.key === "Enter") {
    e.preventDefault();
    if (selectedProduct.value && !analyzing.value) {
      handleAIAnalysis();
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
        ElMessage.info("正在下载更新...");

        await update.downloadAndInstall((progress) => {
          if (progress.event === "Progress" && progress.data) {
            console.log(`已下载: ${progress.data.chunkLength} bytes`);
          }
        });

        await ElMessageBox.alert("更新已下载完成，点击确定重启应用", "更新完成", {
          confirmButtonText: "确定",
        });

        await relaunch();
      }
    }
  } catch (e) {
    // 检查更新失败时静默处理，不影响用户使用
    console.log("检查更新失败:", e);
  }
}

// ==================== 初始化 ====================

onMounted(async () => {
  // 初始化主题
  initTheme();

  // 注册键盘快捷键
  window.addEventListener("keydown", handleKeyboard);

  await loadProducts();
  await loadCategories();
  if (selectedProduct.value) {
    await loadRoots();
    await loadStats();
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
    <aside class="sidebar">
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
            <div class="product-meta" v-if="product.sku || product.asin">
              <span v-if="product.sku">SKU: {{ product.sku }}</span>
              <span v-if="product.asin">ASIN: {{ product.asin }}</span>
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
        <div v-if="products.length === 0" class="no-product">
          <p>暂无产品</p>
          <el-button type="primary" size="small" @click="openAddProductDialog">
            创建产品
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

    <!-- 主内容区 -->
    <main class="main-content">
      <!-- 顶部工具栏 -->
      <header class="header">
        <div class="header-left">
          <h1 class="title">{{ selectedProduct?.name || '请选择产品' }}</h1>
        </div>

        <!-- 一级分类标签 -->
        <div class="category-tabs" v-if="selectedProduct">
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

        <div class="header-right" v-if="selectedProduct">
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
              :type="viewMode === 'table' ? 'primary' : 'default'"
              @click="switchViewMode('table')"
            >
              <el-icon><Grid /></el-icon>
              表格
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

          <el-button type="primary" :loading="importing" @click="handleImport">
            <el-icon><Upload /></el-icon>
            导入Excel
          </el-button>
          <el-button :loading="exporting" @click="handleExport">
            <el-icon><Download /></el-icon>
            导出Excel
          </el-button>
          <el-button
            v-if="viewMode === 'wordcloud'"
            @click="exportWordCloudImage"
          >
            <el-icon><Picture /></el-icon>
            导出词云
          </el-button>
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
          <el-button type="warning" plain @click="handleClearData">
            <el-icon><RefreshRight /></el-icon>
            重置词库
          </el-button>
        </div>
      </div>

      <!-- 词云视图 -->
      <div class="wordcloud-container" v-if="selectedProduct && viewMode === 'wordcloud'">
        <WordCloud
          ref="wordCloudRef"
          :roots="allRootsForCloud"
          :categories="categories"
          :loading="loading"
          @wordClick="handleWordCloudClick"
        />
      </div>

      <!-- 词根表格 -->
      <div class="table-container" v-if="selectedProduct && viewMode === 'table'">
        <el-table
          :data="roots"
          v-loading="loading"
          stripe
          style="width: 100%"
          :default-sort="{ prop: 'contains_count', order: 'descending' }"
        >
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
            width="100"
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
            width="100"
            sortable
            :sort-orders="['descending', 'ascending']"
          >
            <template #header>
              <span class="sortable-header" @click="handleSort('contains_count')">
                包含词
                <el-icon v-if="sortBy === 'contains_count'">
                  <ArrowUp v-if="sortOrder === 'asc'" />
                  <ArrowDown v-else />
                </el-icon>
              </span>
            </template>
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
      <div class="no-product-main" v-if="!selectedProduct">
        <el-empty description="请先选择或创建一个产品">
          <el-button type="primary" @click="openAddProductDialog">创建产品</el-button>
        </el-empty>
      </div>

      <!-- 分页 (仅表格视图显示) -->
      <div class="pagination" v-if="selectedProduct && viewMode === 'table'">
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
    <el-dialog
      v-model="showProductDialog"
      :title="isEditingProduct ? '编辑产品' : '创建产品'"
      width="400px"
    >
      <el-form :model="productForm" label-width="80px">
        <el-form-item label="产品名称" required>
          <el-input v-model="productForm.name" placeholder="请输入产品名称" />
        </el-form-item>
        <el-form-item label="SKU">
          <el-input v-model="productForm.sku" placeholder="请输入SKU（可选）" />
        </el-form-item>
        <el-form-item label="ASIN">
          <el-input v-model="productForm.asin" placeholder="请输入ASIN（可选）" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showProductDialog = false">取消</el-button>
        <el-button type="primary" @click="saveProduct">确定</el-button>
      </template>
    </el-dialog>

    <!-- 快捷键帮助弹窗 -->
    <el-dialog
      v-model="showShortcutsDialog"
      title="键盘快捷键"
      width="480px"
      :show-close="true"
    >
      <div class="shortcuts-list">
        <div class="shortcut-group">
          <h4>通用操作</h4>
          <div class="shortcut-item">
            <span class="shortcut-desc">创建新产品</span>
            <span class="shortcut-key"><kbd>⌘</kbd> <kbd>N</kbd></span>
          </div>
          <div class="shortcut-item">
            <span class="shortcut-desc">切换深色模式</span>
            <span class="shortcut-key"><kbd>⌘</kbd> <kbd>D</kbd></span>
          </div>
          <div class="shortcut-item">
            <span class="shortcut-desc">显示快捷键帮助</span>
            <span class="shortcut-key"><kbd>?</kbd></span>
          </div>
        </div>
        <div class="shortcut-group">
          <h4>数据操作</h4>
          <div class="shortcut-item">
            <span class="shortcut-desc">导入 Excel</span>
            <span class="shortcut-key"><kbd>⌘</kbd> <kbd>I</kbd></span>
          </div>
          <div class="shortcut-item">
            <span class="shortcut-desc">导出 Excel</span>
            <span class="shortcut-key"><kbd>⌘</kbd> <kbd>E</kbd></span>
          </div>
          <div class="shortcut-item">
            <span class="shortcut-desc">开始智能分析</span>
            <span class="shortcut-key"><kbd>⌘</kbd> <kbd>↵</kbd></span>
          </div>
        </div>
        <div class="shortcut-group">
          <h4>导航</h4>
          <div class="shortcut-item">
            <span class="shortcut-desc">聚焦搜索框</span>
            <span class="shortcut-key"><kbd>⌘</kbd> <kbd>F</kbd></span>
          </div>
          <div class="shortcut-item">
            <span class="shortcut-desc">切换产品</span>
            <span class="shortcut-key"><kbd>↑</kbd> <kbd>↓</kbd></span>
          </div>
          <div class="shortcut-item">
            <span class="shortcut-desc">取消/关闭</span>
            <span class="shortcut-key"><kbd>Esc</kbd></span>
          </div>
        </div>
      </div>
      <template #footer>
        <span class="shortcuts-tip">Windows 用户请将 ⌘ 替换为 Ctrl</span>
      </template>
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
</style>

<style scoped>
.app-container {
  height: 100vh;
  display: flex;
  background-color: var(--bg-primary);
  position: relative;
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
  width: 240px;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
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

.no-product {
  text-align: center;
  padding: 40px 20px;
  color: var(--text-muted);
}

.no-product p {
  margin-bottom: 16px;
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

.no-product-main {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* 快捷键帮助弹窗 */
.shortcuts-list {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.shortcut-group h4 {
  font-size: 13px;
  color: var(--text-muted);
  margin-bottom: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.shortcut-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid var(--border-color);
}

.shortcut-item:last-child {
  border-bottom: none;
}

.shortcut-desc {
  color: var(--text-primary);
  font-size: 14px;
}

.shortcut-key {
  display: flex;
  gap: 4px;
}

.shortcut-key kbd {
  display: inline-block;
  padding: 4px 8px;
  font-size: 12px;
  font-family: inherit;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-secondary);
  min-width: 28px;
  text-align: center;
}

.shortcuts-tip {
  font-size: 12px;
  color: var(--text-muted);
}
</style>

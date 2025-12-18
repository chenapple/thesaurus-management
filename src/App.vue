<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { open, save } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { readFile, writeFile } from "@tauri-apps/plugin-fs";
import * as XLSX from "xlsx";
import * as api from "./api";
import { batchAnalyzeWords } from "./deepseek";
import type { Category, Product, Root } from "./types";
import type { UnlistenFn } from "@tauri-apps/api/event";

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

// 编辑翻译
const editingId = ref<number | null>(null);
const editingTranslation = ref("");

// 分类下拉
const categoryDropdownVisible = ref<{ [key: number]: boolean }>({});

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

async function loadStats() {
  if (!selectedProduct.value) {
    stats.value = { keywordCount: 0, rootCount: 0 };
    return;
  }

  try {
    const [keywordCount, rootCount] = await api.getStats(selectedProduct.value.id);
    stats.value = { keywordCount, rootCount };
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
    } else if (event.payload.type === "leave" || event.payload.type === "cancel") {
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
  return roots.value.filter((r) => r.categories.includes(categoryId)).length;
}

// ==================== 初始化 ====================

onMounted(async () => {
  await loadProducts();
  await loadCategories();
  if (selectedProduct.value) {
    await loadRoots();
    await loadStats();
  }
  await setupDragDrop();
});

onUnmounted(() => {
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
          <el-button type="primary" :loading="importing" @click="handleImport">
            <el-icon><Upload /></el-icon>
            导入Excel
          </el-button>
          <el-button :loading="exporting" @click="handleExport">
            <el-icon><Download /></el-icon>
            导出Excel
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

      <!-- 词根表格 -->
      <div class="table-container" v-if="selectedProduct">
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
      <div class="no-product-main" v-else>
        <el-empty description="请先选择或创建一个产品">
          <el-button type="primary" @click="openAddProductDialog">创建产品</el-button>
        </el-empty>
      </div>

      <!-- 分页 -->
      <div class="pagination" v-if="selectedProduct">
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
  </div>
</template>

<style>
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
  background-color: #f5f7fa;
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
  background: #fff;
  border-right: 1px solid #e4e7ed;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.sidebar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid #e4e7ed;
}

.sidebar-title {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
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
  background-color: #f5f7fa;
}

.product-item.active {
  background-color: #ecf5ff;
  border: 1px solid #409eff;
}

.product-info {
  flex: 1;
  min-width: 0;
}

.product-name {
  font-size: 14px;
  font-weight: 500;
  color: #303133;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.product-meta {
  font-size: 12px;
  color: #909399;
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
  color: #909399;
}

.no-product p {
  margin-bottom: 16px;
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
  background: #fff;
  border-bottom: 1px solid #e4e7ed;
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
  color: #303133;
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
  background-color: #f0f0f0;
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
  background: #fff;
  border-bottom: 1px solid #e4e7ed;
  flex-shrink: 0;
}

.stats {
  display: flex;
  gap: 20px;
  font-size: 14px;
  color: #606266;
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

.table-container :deep(.el-table__row) {
  height: 52px;
}

.table-container :deep(.el-table__cell) {
  padding: 12px 0;
}

.word-cell {
  font-weight: 500;
  color: #409eff;
}

.translation-cell {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  color: #606266;
}

.translation-cell:hover .edit-icon {
  opacity: 1;
}

.edit-icon {
  opacity: 0;
  transition: opacity 0.2s;
  color: #909399;
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
  background: #fff;
  border-top: 1px solid #e4e7ed;
  flex-shrink: 0;
}

.no-product-main {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>

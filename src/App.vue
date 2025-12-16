<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { readFile } from "@tauri-apps/plugin-fs";
import * as XLSX from "xlsx";
import * as api from "./api";
import type { Category, Root } from "./types";
import type { UnlistenFn } from "@tauri-apps/api/event";

// 状态
const categories = ref<Category[]>([]);
const roots = ref<Root[]>([]);
const total = ref(0);
const loading = ref(false);
const importing = ref(false);
const isDragging = ref(false);

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

// 加载分类
async function loadCategories() {
  try {
    categories.value = await api.getCategories();
  } catch (e) {
    ElMessage.error("加载分类失败: " + e);
  }
}

// 加载词根
async function loadRoots() {
  loading.value = true;
  try {
    const [data, count] = await api.getRoots({
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

// 加载统计
async function loadStats() {
  try {
    const [keywordCount, rootCount] = await api.getStats();
    stats.value = { keywordCount, rootCount };
  } catch (e) {
    console.error("加载统计失败:", e);
  }
}

// 处理Excel文件（通用函数）
async function processExcelBuffer(buffer: ArrayBuffer) {
  const workbook = XLSX.read(buffer, { type: "array" });

  // 获取第一个工作表
  const sheetName = workbook.SheetNames[0];
  const sheet = workbook.Sheets[sheetName];

  // 转换为JSON
  const data = XLSX.utils.sheet_to_json<{ [key: string]: string }>(sheet);

  // 提取关键词（第一列）
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

  // 导入
  await api.importKeywords(keywords);
  ElMessage.success(`成功导入 ${keywords.length} 个关键词`);

  // 刷新数据
  await loadRoots();
  await loadStats();
}

// 导入Excel（点击按钮）
async function handleImport() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Excel", extensions: ["xlsx", "xls"] }],
    });

    if (!selected) return;

    importing.value = true;

    // 读取文件
    const response = await fetch(`file://${selected}`);
    const buffer = await response.arrayBuffer();
    await processExcelBuffer(buffer);
  } catch (e) {
    ElMessage.error("导入失败: " + e);
  } finally {
    importing.value = false;
  }
}

// 拖拽导入 (Tauri 原生事件)
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

// 清空数据
async function handleClearData() {
  try {
    await ElMessageBox.confirm("确定要清空所有数据吗？此操作不可恢复！", "警告", {
      confirmButtonText: "确定",
      cancelButtonText: "取消",
      type: "warning",
    });

    await api.clearAllData();
    ElMessage.success("数据已清空");
    await loadRoots();
    await loadStats();
  } catch (e) {
    if (e !== "cancel") {
      ElMessage.error("清空失败: " + e);
    }
  }
}

// 搜索
function handleSearch() {
  currentPage.value = 1;
  loadRoots();
}

// 分类筛选
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

// 排序
function handleSort(column: string) {
  if (sortBy.value === column) {
    sortOrder.value = sortOrder.value === "asc" ? "desc" : "asc";
  } else {
    sortBy.value = column;
    sortOrder.value = "desc";
  }
  loadRoots();
}

// 分页
function handlePageChange(page: number) {
  currentPage.value = page;
  loadRoots();
}

function handleSizeChange(size: number) {
  pageSize.value = size;
  currentPage.value = 1;
  loadRoots();
}

// 编辑翻译
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

// 分类标签管理
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

// 初始化
onMounted(async () => {
  await loadCategories();
  await loadRoots();
  await loadStats();
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
    <!-- 顶部工具栏 -->
    <header class="header">
      <div class="header-left">
        <h1 class="title">词根</h1>
        <span class="subtitle">属性词分类</span>
      </div>

      <!-- 一级分类标签 -->
      <div class="category-tabs">
        <el-tag
          v-for="cat in primaryCategories"
          :key="cat.id"
          :type="selectedCategories.includes(cat.id) ? '' : 'info'"
          :effect="selectedCategories.includes(cat.id) ? 'dark' : 'plain'"
          class="category-tag"
          @click="toggleCategory(cat.id)"
        >
          <el-icon><PriceTag /></el-icon>
          {{ cat.name }}({{ getCategoryCount(cat.id) }})
        </el-tag>

        <el-dropdown trigger="click">
          <el-tag type="info" effect="plain" class="category-tag more-tag">
            <el-icon><Grid /></el-icon>
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

      <div class="header-right">
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
    <div class="toolbar">
      <div class="stats">
        <span>关键词: {{ stats.keywordCount }}</span>
        <span>词根: {{ stats.rootCount }}</span>
      </div>
      <div class="actions">
        <el-button type="primary" :loading="importing" @click="handleImport">
          <el-icon><Upload /></el-icon>
          导入Excel
        </el-button>
        <el-button type="danger" plain @click="handleClearData">
          <el-icon><Delete /></el-icon>
          清空数据
        </el-button>
      </div>
    </div>

    <!-- 词根表格 -->
    <div class="table-container">
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

    <!-- 分页 -->
    <div class="pagination">
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
  flex-direction: column;
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

.header {
  display: flex;
  align-items: center;
  padding: 12px 20px;
  background: #fff;
  border-bottom: 1px solid #e4e7ed;
  gap: 20px;
}

.header-left {
  display: flex;
  align-items: baseline;
  gap: 12px;
}

.title {
  font-size: 18px;
  font-weight: 600;
  color: #303133;
}

.subtitle {
  font-size: 14px;
  color: #e6a23c;
  font-weight: 500;
}

.category-tabs {
  display: flex;
  gap: 8px;
  flex: 1;
}

.category-tag {
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
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
}
</style>

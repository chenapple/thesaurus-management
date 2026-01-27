<script setup lang="ts">
import { ref, computed } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { Plus, RefreshLeft, Collection, ArrowLeft, Edit, CircleClose, CircleCheck } from "@element-plus/icons-vue";
import * as api from "../api";
import type { Root, Category, Product } from "../types";

const props = defineProps<{
  selectedProduct: Product | null;
  roots: Root[];
  total: number;
  loading: boolean;
  currentPage: number;
  pageSize: number;
  searchText: string;
  categories: Category[];
  categoryCounts: Map<number, number>;
}>();

const emit = defineEmits<{
  (e: 'update:currentPage', value: number): void;
  (e: 'update:pageSize', value: number): void;
  (e: 'update:searchText', value: string): void;
  (e: 'load-roots'): void;
  (e: 'toggle-category', rootId: number, categoryId: number): void;
  (e: 'switch-to-keywords'): void;
  (e: 'negative-changed'): void;
}>();

// Translation editing
const editingId = ref<number | null>(null);
const editingTranslation = ref("");

// Category dropdown
const categoryDropdownVisible = ref<{ [key: number]: boolean }>({});

// Selection for batch operations
const selectedRoots = ref<Root[]>([]);
const tableRef = ref<InstanceType<typeof import('element-plus')['ElTable']> | null>(null);

function startEdit(row: Root) {
  editingId.value = row.id;
  editingTranslation.value = row.translation || "";
}

async function saveTranslation(row: Root) {
  if (editingId.value === null) return;
  try {
    await api.updateRootTranslation(row.id, editingTranslation.value);
    row.translation = editingTranslation.value;
    ElMessage.success("已保存");
  } catch (e) {
    ElMessage.error("保存失败: " + e);
  }
  editingId.value = null;
}

function cancelEdit() {
  editingId.value = null;
}

function getCategoryName(id: number, categories: Category[]): string {
  const cat = categories.find((c) => c.id === id);
  return cat?.name || "";
}

function handleToggleCategory(root: Root, categoryId: number) {
  emit('toggle-category', root.id, categoryId);
}

function handlePageChange(page: number) {
  emit('update:currentPage', page);
  emit('load-roots');
}

function handleSizeChange(size: number) {
  emit('update:pageSize', size);
  emit('update:currentPage', 1);
  emit('load-roots');
}

function clearSearch() {
  emit('update:searchText', '');
  emit('load-roots');
}

// Selection handling
function handleSelectionChange(rows: Root[]) {
  selectedRoots.value = rows;
}

function clearSelection() {
  selectedRoots.value = [];
  tableRef.value?.clearSelection();
}

// Toggle negative for single root
async function handleToggleNegative(row: Root, isNegative: boolean) {
  try {
    const affected = await api.setRootNegative(row.id, isNegative);
    row.is_negative = isNegative;
    ElMessage.success(
      isNegative
        ? `已标记为否词，${affected} 个关键词受影响`
        : `已取消否词标记，${affected} 个关键词已恢复`
    );
    // 通知父组件刷新关键词数据
    emit('negative-changed');
  } catch (e) {
    // Revert UI state on error
    row.is_negative = !isNegative;
    ElMessage.error("操作失败: " + e);
  }
}

// Batch set negative
async function batchSetNegative(isNegative: boolean) {
  if (selectedRoots.value.length === 0) return;

  const ids = selectedRoots.value.map((r) => r.id);
  const action = isNegative ? "标记为否词" : "取消否词标记";

  try {
    await ElMessageBox.confirm(
      `确定要将选中的 ${selectedRoots.value.length} 个词根${action}吗？`,
      "批量操作",
      { confirmButtonText: "确定", cancelButtonText: "取消", type: "warning" }
    );

    const affected = await api.batchSetRootsNegative(ids, isNegative);

    // Update local state
    for (const root of selectedRoots.value) {
      root.is_negative = isNegative;
    }

    ElMessage.success(`已${action} ${selectedRoots.value.length} 个词根，共 ${affected} 个关键词受影响`);
    clearSelection();
    // 通知父组件刷新关键词数据
    emit('negative-changed');
  } catch (e) {
    if (e !== 'cancel') {
      ElMessage.error("操作失败: " + e);
    }
  }
}

// Computed for negative count
const negativeCount = computed(() => {
  return props.roots.filter((r) => r.is_negative).length;
});
</script>

<template>
  <div class="roots-tab">
    <!-- Batch operations toolbar -->
    <div v-if="selectedRoots.length > 0" class="batch-toolbar">
      <span class="batch-info">已选中 {{ selectedRoots.length }} 个词根</span>
      <el-button type="danger" size="small" @click="batchSetNegative(true)">
        <el-icon><CircleClose /></el-icon>
        批量标记否词
      </el-button>
      <el-button size="small" @click="batchSetNegative(false)">
        <el-icon><CircleCheck /></el-icon>
        批量取消否词
      </el-button>
      <el-button text @click="clearSelection">取消选择</el-button>
    </div>

    <!-- Info bar (shown when no selection) -->
    <div v-else-if="negativeCount > 0" class="info-bar">
      <span class="negative-info">
        已标记 {{ negativeCount }} 个否词词根
      </span>
    </div>

    <!-- Roots table -->
    <div class="table-container">
      <el-table
        ref="tableRef"
        :data="roots"
        v-loading="loading"
        stripe
        style="width: 100%"
        :default-sort="{ prop: 'contains_count', order: 'descending' }"
        @selection-change="handleSelectionChange"
      >
        <template #empty>
          <div class="table-empty-state">
            <div class="empty-icon">
              <el-icon :size="48"><Collection /></el-icon>
            </div>
            <p class="empty-title">{{ searchText ? '没有匹配的词根' : '还没有词根数据' }}</p>
            <p class="empty-desc">{{ searchText ? '尝试其他搜索关键词' : '请先在关键词视图导入数据' }}</p>
            <el-button v-if="searchText" @click="clearSearch">
              <el-icon><RefreshLeft /></el-icon>
              清空搜索
            </el-button>
            <el-button v-else type="primary" @click="emit('switch-to-keywords')">
              <el-icon><ArrowLeft /></el-icon>
              返回关键词视图
            </el-button>
          </div>
        </template>

        <el-table-column type="selection" width="40" />
        <el-table-column type="index" label="#" width="50" />

        <el-table-column label="词根" min-width="120">
          <template #default="{ row }">
            <span class="word-cell" :class="{ 'is-negative': row.is_negative }">
              {{ row.word }}
            </span>
          </template>
        </el-table-column>

        <el-table-column label="否词" width="80" align="center">
          <template #default="{ row }">
            <el-switch
              :model-value="row.is_negative"
              size="small"
              @change="(val: boolean) => handleToggleNegative(row, val)"
            />
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
                @close="handleToggleCategory(row, catId)"
              >
                {{ getCategoryName(catId, categories) }}
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
                      @click="handleToggleCategory(row, cat.id)"
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

    <!-- Pagination -->
    <div class="pagination">
      <el-pagination
        :current-page="currentPage"
        :page-size="pageSize"
        :page-sizes="[20, 50, 100, 200]"
        :total="total"
        layout="total, sizes, prev, pager, next, jumper"
        @size-change="handleSizeChange"
        @current-change="handlePageChange"
      />
    </div>
  </div>
</template>

<style scoped>
.roots-tab {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.info-bar {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.batch-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: var(--el-color-warning-light-9);
  border-bottom: 1px solid var(--el-color-warning-light-5);
  flex-shrink: 0;
}

.batch-info {
  font-size: 14px;
  color: var(--el-color-warning-dark-2);
  font-weight: 500;
}

.negative-info {
  font-size: 13px;
  color: var(--el-color-danger);
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

.table-container :deep(.el-table__header th .cell) {
  white-space: nowrap;
}

.word-cell {
  font-weight: 500;
  color: var(--accent-color);
}

.word-cell.is-negative {
  color: var(--el-color-danger);
  text-decoration: line-through;
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

.category-cell {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  align-items: center;
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

.pagination {
  display: flex;
  justify-content: center;
  padding: 16px;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
  flex-shrink: 0;
}
</style>

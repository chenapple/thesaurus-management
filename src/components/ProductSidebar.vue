<script setup lang="ts">
import { Plus, MoreFilled, Edit, Delete, Box, Sunny, Moon } from "@element-plus/icons-vue";
import type { Product, WorkflowStatus } from "../types";

defineProps<{
  products: Product[];
  selectedProduct: Product | null;
  sidebarWidth: number;
  isDarkMode: boolean;
  workflowStatus: WorkflowStatus;
  countryOptions: { code: string; name: string; flag: string }[];
}>();

const emit = defineEmits<{
  (e: 'select-product', product: Product): void;
  (e: 'add-product'): void;
  (e: 'edit-product', product: Product): void;
  (e: 'delete-product', product: Product): void;
  (e: 'toggle-theme'): void;
}>();

function getCountryName(code: string | null, countryOptions: { code: string; name: string; flag: string }[]): string {
  if (!code) return "";
  const country = countryOptions.find(c => c.code === code);
  return country?.name || code;
}

function getCountryFlag(code: string | null, countryOptions: { code: string; name: string; flag: string }[]): string {
  if (!code) return "";
  const country = countryOptions.find(c => c.code === code);
  return country?.flag || "";
}

function getWorkflowStatusText(status: WorkflowStatus): { text: string; type: 'warning' | 'info' | 'success' } {
  if (!status.has_data) return { text: '待导入', type: 'warning' };
  if (!status.has_traffic_level) return { text: '待设流量', type: 'info' };
  if (!status.has_category) return { text: '待分类', type: 'info' };
  if (!status.has_phrase_tag) return { text: '待打标', type: 'info' };
  if (!status.has_orderliness) return { text: '待排序', type: 'info' };
  return { text: '已完成 ✓', type: 'success' };
}
</script>

<template>
  <aside class="sidebar" :style="{ width: sidebarWidth + 'px' }">
    <div class="sidebar-header">
      <span class="sidebar-title">产品列表</span>
      <el-button type="primary" size="small" circle @click="emit('add-product')">
        <el-icon><Plus /></el-icon>
      </el-button>
    </div>
    <div class="product-list">
      <div
        v-for="product in products"
        :key="product.id"
        class="product-item"
        :class="{ active: selectedProduct?.id === product.id }"
        @click="emit('select-product', product)"
      >
        <div class="product-info">
          <div class="product-name">{{ product.name }}</div>
          <div class="product-meta" v-if="product.country">
            <span class="country-flag-small" v-html="getCountryFlag(product.country, countryOptions)"></span>
            <span>{{ getCountryName(product.country, countryOptions) }}</span>
          </div>
          <!-- Workflow status tag -->
          <div class="product-status" v-if="selectedProduct?.id === product.id">
            <el-tag size="small" :type="getWorkflowStatusText(workflowStatus).type">
              {{ getWorkflowStatusText(workflowStatus).text }}
            </el-tag>
          </div>
        </div>
        <el-dropdown trigger="click" @click.stop>
          <el-button size="small" text class="product-action">
            <el-icon><MoreFilled /></el-icon>
          </el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item @click="emit('edit-product', product)">
                <el-icon><Edit /></el-icon> 编辑
              </el-dropdown-item>
              <el-dropdown-item @click="emit('delete-product', product)" divided>
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
        <el-button type="primary" @click="emit('add-product')">
          <el-icon><Plus /></el-icon>
          新建产品
        </el-button>
      </div>
    </div>
    <div class="sidebar-footer">
      <el-button text @click="emit('toggle-theme')" class="theme-toggle">
        <el-icon><Sunny v-if="isDarkMode" /><Moon v-else /></el-icon>
        <span>{{ isDarkMode ? '浅色模式' : '深色模式' }}</span>
      </el-button>
    </div>
  </aside>
</template>

<style scoped>
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
  align-items: center;
  gap: 4px;
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

/* Empty state */
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

/* Country flag */
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

.country-flag-small :deep(svg) {
  width: 100%;
  height: 100%;
}
</style>

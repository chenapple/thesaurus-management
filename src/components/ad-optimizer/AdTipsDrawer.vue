<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Search, Reading, MagicStick, Aim, Coin, DataAnalysis, Wallet, Promotion } from '@element-plus/icons-vue';
import type { Component } from 'vue';
import { AD_TIPS, searchTips, type AdTip, type AdTipCategory } from '../../data/ad-tips';
import { renderSimpleMarkdown } from '../../utils/sanitize';

// 图标映射
const iconMap: Record<string, Component> = {
  MagicStick,
  Aim,
  Coin,
  DataAnalysis,
  Wallet,
  Promotion,
};

const props = defineProps<{
  modelValue: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (v) => emit('update:modelValue', v),
});

// 搜索
const searchKeyword = ref('');
const searchResults = computed(() => searchTips(searchKeyword.value));
const isSearching = computed(() => searchKeyword.value.trim().length > 0);

// 展开的分类
const expandedCategories = ref<string[]>([AD_TIPS[0]?.id || '']);

// 当前选中查看的 Tip
const selectedTip = ref<AdTip | null>(null);
const selectedCategory = ref<AdTipCategory | null>(null);

// 打开时重置状态
watch(visible, (v) => {
  if (v) {
    searchKeyword.value = '';
    selectedTip.value = null;
    selectedCategory.value = null;
  }
});

// 选择一个 Tip 查看详情
function selectTip(tip: AdTip, category?: AdTipCategory) {
  selectedTip.value = tip;
  selectedCategory.value = category || null;
}

// 返回列表
function backToList() {
  selectedTip.value = null;
  selectedCategory.value = null;
}

// 从搜索结果选择
function selectSearchResult(result: AdTip & { categoryName: string }) {
  // 找到对应的分类
  const category = AD_TIPS.find(c => c.name === result.categoryName);
  selectTip(result, category);
}

// 渲染 Markdown 内容
function renderContent(content: string): string {
  return renderSimpleMarkdown(content);
}
</script>

<template>
  <el-drawer
    v-model="visible"
    title=""
    direction="rtl"
    size="420px"
    :show-close="true"
    class="ad-tips-drawer"
  >
    <template #header>
      <div class="drawer-header">
        <el-icon class="header-icon"><Reading /></el-icon>
        <span class="header-title">亚马逊广告指南</span>
        <el-tag size="small" type="info">持续更新中</el-tag>
      </div>
    </template>

    <!-- 搜索框 -->
    <div class="search-box">
      <el-input
        v-model="searchKeyword"
        placeholder="搜索广告技巧..."
        :prefix-icon="Search"
        clearable
      />
    </div>

    <!-- 详情视图 -->
    <div v-if="selectedTip" class="tip-detail">
      <div class="detail-header">
        <button class="back-btn" @click="backToList">
          ← 返回列表
        </button>
        <el-tag v-if="selectedCategory" size="small" class="category-tag">
          <el-icon class="tag-icon"><component :is="iconMap[selectedCategory.icon]" /></el-icon>
          {{ selectedCategory.name }}
        </el-tag>
      </div>
      <h3 class="detail-title">{{ selectedTip.title }}</h3>
      <div class="detail-content markdown-content" v-html="renderContent(selectedTip.content)"></div>
    </div>

    <!-- 搜索结果 -->
    <div v-else-if="isSearching" class="search-results">
      <div v-if="searchResults.length === 0" class="empty-search">
        <p>未找到相关内容</p>
        <span>试试其他关键词</span>
      </div>
      <div v-else class="results-list">
        <div class="results-count">找到 {{ searchResults.length }} 条相关内容</div>
        <div
          v-for="result in searchResults"
          :key="result.id"
          class="result-item"
          @click="selectSearchResult(result)"
        >
          <div class="result-category">
            <el-icon class="result-icon"><component :is="iconMap[result.categoryIcon]" /></el-icon>
            {{ result.categoryName }}
          </div>
          <div class="result-title">{{ result.title }}</div>
        </div>
      </div>
    </div>

    <!-- 分类列表 -->
    <div v-else class="category-list">
      <el-collapse v-model="expandedCategories">
        <el-collapse-item
          v-for="category in AD_TIPS"
          :key="category.id"
          :name="category.id"
        >
          <template #title>
            <div class="category-header">
              <el-icon class="category-icon"><component :is="iconMap[category.icon]" /></el-icon>
              <span class="category-name">{{ category.name }}</span>
              <span class="category-count">{{ category.tips.length }}</span>
            </div>
          </template>
          <div v-if="category.description" class="category-desc">
            {{ category.description }}
          </div>
          <div class="tip-list">
            <div
              v-for="tip in category.tips"
              :key="tip.id"
              class="tip-item"
              @click="selectTip(tip, category)"
            >
              <span class="tip-title">{{ tip.title }}</span>
              <el-icon class="tip-arrow"><svg viewBox="0 0 24 24" width="14" height="14"><path fill="currentColor" d="M8.59 16.59L13.17 12 8.59 7.41 10 6l6 6-6 6-1.41-1.41z"/></svg></el-icon>
            </div>
          </div>
        </el-collapse-item>
      </el-collapse>

      <!-- 底部提示 -->
      <div class="footer-hint">
        <p>💡 内容持续更新中，敬请期待更多技巧</p>
      </div>
    </div>
  </el-drawer>
</template>

<style scoped>
.ad-tips-drawer :deep(.el-drawer__header) {
  margin-bottom: 0;
  padding: 16px 20px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.drawer-header {
  display: flex;
  align-items: center;
  gap: 10px;
}

.header-icon {
  font-size: 22px;
  color: var(--el-color-primary);
}

.header-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.search-box {
  padding: 16px 20px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

/* 分类列表 */
.category-list {
  padding: 12px 0;
}

.category-list :deep(.el-collapse) {
  border: none;
}

.category-list :deep(.el-collapse-item__header) {
  padding: 0 20px;
  height: 52px;
  background: transparent;
  border-bottom: none;
}

.category-list :deep(.el-collapse-item__wrap) {
  border-bottom: none;
}

.category-list :deep(.el-collapse-item__content) {
  padding: 0 20px 12px;
}

.category-header {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
}

.category-icon {
  font-size: 18px;
  color: var(--el-color-primary);
}

.result-icon {
  font-size: 14px;
  margin-right: 4px;
  vertical-align: middle;
}

.category-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.tag-icon {
  font-size: 12px;
}

.category-name {
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.category-count {
  margin-left: auto;
  padding-right: 8px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.category-desc {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 12px;
  padding: 8px 12px;
  background: var(--el-fill-color-light);
  border-radius: 6px;
}

.tip-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.tip-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
  background: var(--el-fill-color-lighter);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.tip-item:hover {
  background: var(--el-color-primary-light-9);
}

.tip-title {
  font-size: 14px;
  color: var(--el-text-color-primary);
}

.tip-arrow {
  color: var(--el-text-color-placeholder);
}

/* 搜索结果 */
.search-results {
  padding: 16px 20px;
}

.empty-search {
  text-align: center;
  padding: 40px 20px;
  color: var(--el-text-color-secondary);
}

.empty-search p {
  font-size: 14px;
  margin-bottom: 8px;
}

.empty-search span {
  font-size: 12px;
}

.results-count {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 12px;
}

.results-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.result-item {
  padding: 12px 14px;
  background: var(--el-fill-color-lighter);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.result-item:hover {
  background: var(--el-color-primary-light-9);
}

.result-category {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 4px;
}

.result-title {
  font-size: 14px;
  color: var(--el-text-color-primary);
  font-weight: 500;
}

/* 详情视图 */
.tip-detail {
  padding: 16px 20px;
}

.detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.back-btn {
  background: none;
  border: none;
  color: var(--el-color-primary);
  font-size: 14px;
  cursor: pointer;
  padding: 4px 0;
}

.back-btn:hover {
  text-decoration: underline;
}

.detail-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin-bottom: 20px;
  line-height: 1.4;
}

.detail-content {
  font-size: 14px;
  line-height: 1.8;
  color: var(--el-text-color-regular);
}

.detail-content :deep(h1),
.detail-content :deep(h2),
.detail-content :deep(h3),
.detail-content :deep(h4) {
  margin-top: 20px;
  margin-bottom: 12px;
  color: var(--el-text-color-primary);
}

.detail-content :deep(p) {
  margin-bottom: 12px;
}

.detail-content :deep(ul),
.detail-content :deep(ol) {
  padding-left: 20px;
  margin-bottom: 12px;
}

.detail-content :deep(li) {
  margin-bottom: 6px;
}

.detail-content :deep(code) {
  background: var(--el-fill-color);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: monospace;
}

.detail-content :deep(pre) {
  background: var(--el-fill-color);
  padding: 12px;
  border-radius: 8px;
  overflow-x: auto;
  margin-bottom: 12px;
}

.detail-content :deep(table) {
  width: 100%;
  border-collapse: collapse;
  margin-bottom: 16px;
  font-size: 13px;
}

.detail-content :deep(th),
.detail-content :deep(td) {
  padding: 10px 12px;
  border: 1px solid var(--el-border-color-lighter);
  text-align: left;
}

.detail-content :deep(th) {
  background: var(--el-fill-color-light);
  font-weight: 600;
}

.detail-content :deep(strong) {
  color: var(--el-text-color-primary);
}

/* 底部提示 */
.footer-hint {
  padding: 20px;
  text-align: center;
}

.footer-hint p {
  font-size: 13px;
  color: var(--el-text-color-secondary);
  margin: 0;
}
</style>

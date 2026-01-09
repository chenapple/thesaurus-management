<template>
  <el-tabs v-model="activeTab" class="result-tabs">
    <!-- 否定词建议 -->
    <el-tab-pane label="否定词建议" name="negative">
      <div class="tab-header">
        <span class="tab-count">共 {{ countryResult.negative_words.length }} 个建议</span>
        <el-button size="small" @click="emit('export', 'negative_words')">
          <el-icon><Download /></el-icon>
          导出 Excel
        </el-button>
      </div>

      <el-table :data="countryResult.negative_words" style="width: 100%">
        <el-table-column prop="search_term" label="搜索词" min-width="200">
          <template #default="{ row }">
            <div class="search-term-cell">
              <span class="copyable-text" @click="copyToClipboard(row.search_term)">
                {{ formatAsin(row.search_term) }}
                <el-icon class="copy-icon"><CopyDocument /></el-icon>
              </span>
              <el-tag
                size="small"
                :type="getRiskType(row.risk_level)"
              >
                {{ row.risk_level === 'high' ? '高风险' : row.risk_level === 'medium' ? '中风险' : '低风险' }}
              </el-tag>
            </div>
          </template>
        </el-table-column>
        <el-table-column prop="reason" label="原因" min-width="200" />
        <el-table-column prop="spend_wasted" label="浪费花费" width="100">
          <template #default="{ row }">
            <span class="money-red">{{ countryResult.currency.symbol }}{{ row.spend_wasted.toFixed(2) }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="match_type_suggestion" label="建议类型" width="100">
          <template #default="{ row }">
            <el-tag size="small">{{ row.match_type_suggestion === 'exact' ? '精准否定' : '词组否定' }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="campaigns_affected" label="影响活动" min-width="200">
          <template #default="{ row }">
            <div v-if="row.campaigns_affected && row.campaigns_affected.length > 0" class="campaigns-list">
              <!-- 显示前3个活动 -->
              <div
                v-for="(campaign, idx) in row.campaigns_affected.slice(0, 3)"
                :key="idx"
                class="campaign-item"
              >
                {{ campaign }}
              </div>
              <!-- 如果超过3个，显示剩余数量 -->
              <el-tooltip
                v-if="row.campaigns_affected.length > 3"
                :content="row.campaigns_affected.slice(3).join(', ')"
                placement="top"
              >
                <div class="campaign-more">
                  +{{ row.campaigns_affected.length - 3 }} 个活动
                </div>
              </el-tooltip>
            </div>
            <span v-else class="no-campaign">-</span>
          </template>
        </el-table-column>
      </el-table>
    </el-tab-pane>

    <!-- 竞价调整 -->
    <el-tab-pane label="竞价调整" name="bid">
      <div class="tab-header">
        <span class="tab-count">共 {{ countryResult.bid_adjustments.length }} 个建议</span>
        <el-button size="small" @click="emit('export', 'bid_adjustments')">
          <el-icon><Download /></el-icon>
          导出 Excel
        </el-button>
      </div>

      <el-table :data="countryResult.bid_adjustments" style="width: 100%">
        <el-table-column prop="targeting" label="投放词" min-width="180">
          <template #default="{ row }">
            <span class="copyable-text" @click="copyToClipboard(row.targeting)">
              {{ formatAsin(row.targeting) }}
              <el-icon class="copy-icon"><CopyDocument /></el-icon>
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="campaign_name" label="广告活动" min-width="150" />
        <el-table-column label="当前表现" width="200">
          <template #default="{ row }">
            <div class="performance-cell">
              <span>ACOS: {{ row.current_performance.acos.toFixed(1) }}%</span>
              <span>转化: {{ row.current_performance.conversion_rate.toFixed(1) }}%</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column prop="suggestion" label="建议" width="100">
          <template #default="{ row }">
            <el-tag :type="getSuggestionType(row.suggestion)" size="small">
              {{ getSuggestionText(row.suggestion) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="adjustment_percent" label="调整幅度" width="100">
          <template #default="{ row }">
            <span :class="row.adjustment_percent > 0 ? 'money-green' : 'money-red'">
              {{ row.adjustment_percent > 0 ? '+' : '' }}{{ row.adjustment_percent }}%
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="priority" label="优先级" width="80">
          <template #default="{ row }">
            <el-tag :type="getPriorityType(row.priority)" size="small">
              {{ row.priority === 'high' ? '高' : row.priority === 'medium' ? '中' : '低' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="reason" label="原因" min-width="200" />
      </el-table>
    </el-tab-pane>

    <!-- 新词机会 -->
    <el-tab-pane label="新词机会" name="opportunity">
      <div class="tab-header">
        <span class="tab-count">共 {{ countryResult.keyword_opportunities.length }} 个机会</span>
      </div>

      <el-table :data="countryResult.keyword_opportunities" style="width: 100%">
        <el-table-column prop="search_term" label="搜索词" min-width="200">
          <template #default="{ row }">
            <span class="copyable-text" @click="copyToClipboard(row.search_term)">
              {{ formatAsin(row.search_term) }}
              <el-icon class="copy-icon"><CopyDocument /></el-icon>
            </span>
          </template>
        </el-table-column>
        <el-table-column label="表现数据" width="200">
          <template #default="{ row }">
            <div class="performance-cell">
              <span>订单: {{ row.performance.orders }}</span>
              <span>ACOS: {{ row.performance.acos.toFixed(1) }}%</span>
              <span>转化: {{ row.performance.conversion_rate?.toFixed(1) || '-' }}%</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column prop="match_type" label="建议匹配" width="100">
          <template #default="{ row }">
            <el-tag size="small">{{ formatMatchType(row.match_type) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="estimated_potential" label="潜力评估" width="100">
          <template #default="{ row }">
            <el-tag
              :type="getPotentialType(row.estimated_potential)"
              size="small"
            >
              {{ formatPotential(row.estimated_potential) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="suggestion" label="建议操作" min-width="200" />
      </el-table>
    </el-tab-pane>
  </el-tabs>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { Download, CopyDocument } from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import type { CountryAnalysisResult } from '../../types';

defineProps<{
  countryResult: CountryAnalysisResult;
}>();

const emit = defineEmits<{
  (e: 'export', type: 'negative_words' | 'bid_adjustments' | 'all'): void;
}>();

const activeTab = ref('negative');

function getRiskType(level: string): 'danger' | 'warning' | 'info' {
  switch (level) {
    case 'high': return 'danger';
    case 'medium': return 'warning';
    default: return 'info';
  }
}

function getSuggestionType(suggestion: string): 'success' | 'warning' | 'danger' | 'info' {
  switch (suggestion) {
    case 'increase': return 'success';
    case 'decrease': return 'warning';
    case 'pause': return 'danger';
    default: return 'info';
  }
}

function getSuggestionText(suggestion: string): string {
  switch (suggestion) {
    case 'increase': return '加价';
    case 'decrease': return '降价';
    case 'pause': return '暂停';
    case 'maintain': return '维持';
    default: return suggestion;
  }
}

function getPriorityType(priority: string): 'danger' | 'warning' | 'info' {
  switch (priority) {
    case 'high': return 'danger';
    case 'medium': return 'warning';
    default: return 'info';
  }
}

// 格式化ASIN：如果是ASIN（B0开头的10位字符），则转为大写
function formatAsin(text: string): string {
  if (!text) return text;
  // ASIN格式：B0 + 8位字母数字，共10位
  if (/^[bB]0[a-zA-Z0-9]{8}$/i.test(text)) {
    return text.toUpperCase();
  }
  return text;
}

// 复制文本到剪贴板
async function copyToClipboard(text: string) {
  try {
    await navigator.clipboard.writeText(formatAsin(text));
    ElMessage.success('已复制');
  } catch {
    ElMessage.error('复制失败');
  }
}

// 匹配类型转中文
function formatMatchType(type: string): string {
  const map: Record<string, string> = {
    'exact': '精准',
    'phrase': '词组',
    'broad': '广泛',
  };
  return map[type?.toLowerCase()] || type;
}

// 潜力评估转中文
function formatPotential(potential: string): string {
  const map: Record<string, string> = {
    'high': '高',
    'medium': '中',
    'low': '低',
  };
  return map[potential?.toLowerCase()] || potential;
}

// 获取潜力评估标签类型
function getPotentialType(potential: string): 'success' | 'warning' | 'info' {
  const normalized = potential?.toLowerCase();
  if (normalized === 'high' || potential === '高') return 'success';
  if (normalized === 'medium' || potential === '中') return 'warning';
  return 'info';
}
</script>

<style scoped>
.result-tabs {
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-light);
  border-radius: 8px;
  padding: 16px;
}

.tab-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.tab-count {
  color: var(--el-text-color-secondary);
  font-size: 14px;
}

.search-term-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.performance-cell {
  display: flex;
  flex-direction: column;
  gap: 2px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.money-red {
  color: var(--el-color-danger);
  font-weight: 600;
}

.money-green {
  color: var(--el-color-success);
  font-weight: 600;
}

/* 可复制文本样式 */
.copyable-text {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.copyable-text:hover {
  background-color: var(--el-fill-color-light);
}

.copyable-text .copy-icon {
  font-size: 14px;
  color: var(--el-text-color-secondary);
  opacity: 0;
  transition: opacity 0.2s;
}

.copyable-text:hover .copy-icon {
  opacity: 1;
}

/* 活动列表样式 */
.campaigns-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.campaign-item {
  font-size: 12px;
  color: var(--el-text-color-regular);
  line-height: 1.4;
  padding: 2px 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 180px;
}

.campaign-more {
  font-size: 12px;
  color: var(--el-color-primary);
  cursor: pointer;
  padding: 2px 0;
}

.campaign-more:hover {
  text-decoration: underline;
}

.no-campaign {
  color: var(--el-text-color-placeholder);
}
</style>

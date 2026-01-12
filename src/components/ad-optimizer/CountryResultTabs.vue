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
        <el-table-column prop="search_term" min-width="280">
          <template #header>
            <el-tooltip placement="top" :show-after="300">
              <template #content>
                <div class="risk-tooltip">
                  <div class="risk-title">风险等级判定规则</div>
                  <div class="risk-item high">
                    <span class="risk-label">高风险：</span>
                    <span>高花费零转化（花费 > 平均值且无订单），或 ACOS > 200%</span>
                  </div>
                  <div class="risk-item medium">
                    <span class="risk-label">中风险：</span>
                    <span>低转化率（< 1% 且点击 > 10），或 ACOS 明显偏高</span>
                  </div>
                  <div class="risk-item low">
                    <span class="risk-label">低风险：</span>
                    <span>与投放词相关性较低，或效果略差</span>
                  </div>
                </div>
              </template>
              <span class="header-with-tip">搜索词 <el-icon class="tip-icon"><QuestionFilled /></el-icon></span>
            </el-tooltip>
          </template>
          <template #default="{ row }">
            <div class="search-term-cell-vertical">
              <el-tooltip :content="formatAsin(row.search_term)" placement="top" :show-after="500">
                <span class="copyable-text search-term-text" @click="copyToClipboard(row.search_term)">
                  {{ formatAsin(row.search_term) }}
                  <el-icon class="copy-icon"><CopyDocument /></el-icon>
                </span>
              </el-tooltip>
              <el-tag
                size="small"
                :type="getRiskType(row.risk_level)"
                class="risk-tag-below"
              >
                {{ row.risk_level === 'high' ? '高风险' : row.risk_level === 'medium' ? '中风险' : '低风险' }}
              </el-tag>
            </div>
          </template>
        </el-table-column>
        <el-table-column prop="ad_group_name" label="广告组" min-width="150">
          <template #default="{ row }">
            <span v-if="row.ad_group_name">{{ row.ad_group_name }}</span>
            <span v-else class="no-data">-</span>
          </template>
        </el-table-column>
        <el-table-column prop="targeting" label="投放词" min-width="150">
          <template #default="{ row }">
            <span v-if="row.targeting" class="copyable-text" @click="copyToClipboard(row.targeting)">
              {{ formatAsin(row.targeting) }}
              <el-icon class="copy-icon"><CopyDocument /></el-icon>
            </span>
            <span v-else class="no-data">-</span>
          </template>
        </el-table-column>
        <el-table-column prop="reason_category" label="原因分类" width="110">
          <template #default="{ row }">
            <el-tag v-if="row.reason_category" :type="getReasonCategoryType(row.reason_category)" size="small">
              {{ formatReasonCategory(row.reason_category) }}
            </el-tag>
            <span v-else class="no-data">-</span>
          </template>
        </el-table-column>
        <el-table-column prop="negation_level" label="否定层级" width="100">
          <template #default="{ row }">
            <el-tooltip v-if="row.negation_level" :content="row.negation_level_reason || ''" placement="top" :disabled="!row.negation_level_reason">
              <el-tag :type="getNegationLevelType(row.negation_level)" size="small">
                {{ formatNegationLevel(row.negation_level) }}
              </el-tag>
            </el-tooltip>
            <span v-else class="no-data">-</span>
          </template>
        </el-table-column>
        <el-table-column prop="reason" label="原因" min-width="220" />
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
        <el-table-column prop="campaigns_affected" label="影响活动" min-width="250">
          <template #default="{ row }">
            <div v-if="row.campaigns_affected && row.campaigns_affected.length > 0" class="campaigns-list">
              <!-- 显示所有活动 -->
              <div
                v-for="(campaign, idx) in row.campaigns_affected"
                :key="idx"
                class="campaign-item"
              >
                {{ campaign }}
              </div>
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
        <el-table-column prop="ad_group_name" label="广告组" min-width="150">
          <template #default="{ row }">
            <span v-if="row.ad_group_name">{{ row.ad_group_name }}</span>
            <span v-else class="no-data">-</span>
          </template>
        </el-table-column>
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
        <el-table-column prop="adjustment_level" label="等级" width="70">
          <template #default="{ row }">
            <el-tag v-if="row.adjustment_level" :type="getAdjustmentLevelType(row.adjustment_level)" size="small">
              {{ row.adjustment_level }}
            </el-tag>
            <span v-else class="no-data">-</span>
          </template>
        </el-table-column>
        <el-table-column prop="confidence" label="信心" width="80">
          <template #default="{ row }">
            <el-tooltip v-if="row.confidence != null" :content="row.confidence_factors?.join('、') || ''" placement="top" :disabled="!row.confidence_factors?.length">
              <span :style="{ color: getConfidenceColor(row.confidence) }">
                {{ (row.confidence * 100).toFixed(0) }}%
              </span>
            </el-tooltip>
            <span v-else class="no-data">-</span>
          </template>
        </el-table-column>
        <el-table-column prop="reason" label="原因" min-width="220" />
      </el-table>
    </el-tab-pane>

    <!-- 新词机会 -->
    <el-tab-pane label="新词机会" name="opportunity">
      <div class="tab-header">
        <span class="tab-count">共 {{ countryResult.keyword_opportunities.length }} 个机会</span>
      </div>

      <el-table :data="countryResult.keyword_opportunities" style="width: 100%">
        <el-table-column prop="search_term" label="搜索词" min-width="280">
          <template #default="{ row }">
            <el-tooltip :content="formatAsin(row.search_term)" placement="top" :show-after="500">
              <span class="copyable-text search-term-text" @click="copyToClipboard(row.search_term)">
                {{ formatAsin(row.search_term) }}
                <el-icon class="copy-icon"><CopyDocument /></el-icon>
              </span>
            </el-tooltip>
          </template>
        </el-table-column>
        <el-table-column prop="campaign_name" label="广告活动" min-width="150">
          <template #default="{ row }">
            <span v-if="row.campaign_name">{{ row.campaign_name }}</span>
            <span v-else class="no-data">-</span>
          </template>
        </el-table-column>
        <el-table-column prop="ad_group_name" label="广告组" min-width="150">
          <template #default="{ row }">
            <span v-if="row.ad_group_name">{{ row.ad_group_name }}</span>
            <span v-else class="no-data">-</span>
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
        <el-table-column prop="opportunity_type" label="机会类型" width="100">
          <template #default="{ row }">
            <el-tag v-if="row.opportunity_type" :type="getOpportunityTypeStyle(row.opportunity_type)" size="small">
              {{ formatOpportunityType(row.opportunity_type) }}
            </el-tag>
            <el-tag v-else :type="getPotentialType(row.estimated_potential)" size="small">
              {{ formatPotential(row.estimated_potential) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="recommended_match_type" label="推荐匹配" width="100">
          <template #default="{ row }">
            <el-tooltip v-if="row.recommended_match_type" :content="row.match_type_reason || ''" placement="top" :disabled="!row.match_type_reason">
              <el-tag size="small" type="info">{{ formatMatchType(row.recommended_match_type) }}</el-tag>
            </el-tooltip>
            <el-tag v-else size="small">{{ formatMatchType(row.match_type) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="suggestion" label="建议操作" min-width="200" />
      </el-table>
    </el-tab-pane>
  </el-tabs>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { Download, CopyDocument, QuestionFilled } from '@element-plus/icons-vue';
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

// ========== 新增字段格式化函数 ==========

// 否定原因分类格式化
function formatReasonCategory(category: string): string {
  const map: Record<string, string> = {
    'wrong_category': '错品类',
    'wrong_scenario': '错场景',
    'non_target_customer': '非目标',
    'low_intent': '低意图',
    'competitor': '竞品词',
    'other': '数据驱动',
  };
  return map[category] || category;
}

// 否定原因分类标签类型
function getReasonCategoryType(category: string): 'danger' | 'warning' | 'info' | 'success' {
  switch (category) {
    case 'wrong_category':
    case 'wrong_scenario':
      return 'danger';
    case 'non_target_customer':
    case 'competitor':
      return 'warning';
    case 'low_intent':
      return 'info';
    default:
      return 'info';
  }
}

// 否定层级格式化
function formatNegationLevel(level: string): string {
  const map: Record<string, string> = {
    'ad_group': '广告组',
    'campaign': '活动',
    'account': '账户⚠️',
  };
  return map[level] || level;
}

// 否定层级标签类型
function getNegationLevelType(level: string): 'danger' | 'warning' | 'info' {
  switch (level) {
    case 'account': return 'danger';
    case 'campaign': return 'warning';
    default: return 'info';
  }
}

// 调整等级标签类型
function getAdjustmentLevelType(level: string): 'success' | 'warning' | 'danger' {
  switch (level) {
    case 'L1': return 'success';
    case 'L2': return 'warning';
    case 'L3': return 'danger';
    default: return 'warning';
  }
}

// 信心值颜色
function getConfidenceColor(confidence: number): string {
  if (confidence >= 0.8) return 'var(--el-color-success)';
  if (confidence >= 0.6) return 'var(--el-color-warning)';
  return 'var(--el-text-color-secondary)';
}

// 机会类型格式化
function formatOpportunityType(type: string): string {
  const map: Record<string, string> = {
    'expansion': '扩量词',
    'testing': '测试词',
    'structure': '结构词',
  };
  return map[type] || type;
}

// 机会类型标签样式
function getOpportunityTypeStyle(type: string): 'success' | 'warning' | 'info' {
  switch (type) {
    case 'expansion': return 'success';
    case 'testing': return 'warning';
    case 'structure': return 'info';
    default: return 'info';
  }
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

/* 垂直布局：标签在搜索词下方 */
.search-term-cell-vertical {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
}

.risk-tag-below {
  margin-top: 2px;
}

/* 搜索词文本截断 */
.search-term-text {
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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
  padding: 2px 6px;
  background: var(--el-fill-color-light);
  border-radius: 4px;
  word-break: break-all;
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

/* 表头提示样式 */
.header-with-tip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  cursor: help;
}

.tip-icon {
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

.header-with-tip:hover .tip-icon {
  color: var(--el-color-primary);
}

/* 风险等级提示框样式 */
.risk-tooltip {
  max-width: 320px;
  line-height: 1.6;
}

.risk-title {
  font-weight: 600;
  margin-bottom: 8px;
  padding-bottom: 6px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.2);
}

.risk-item {
  margin-bottom: 6px;
  font-size: 13px;
}

.risk-item:last-child {
  margin-bottom: 0;
}

.risk-label {
  font-weight: 500;
}

.risk-item.high .risk-label {
  color: #f56c6c;
}

.risk-item.medium .risk-label {
  color: #e6a23c;
}

.risk-item.low .risk-label {
  color: #67c23a;
}

/* 无数据占位符 */
.no-data {
  color: var(--el-text-color-placeholder);
}
</style>

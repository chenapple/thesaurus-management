<template>
  <div class="ad-analysis-results">
    <!-- 增量分析提示 -->
    <el-alert
      v-if="isPartial"
      type="info"
      :closable="false"
      class="partial-alert"
    >
      <template #title>
        <div class="partial-alert-content">
          <el-icon class="loading-icon"><Loading /></el-icon>
          <span>正在分析更多国家，已完成的结果实时显示...</span>
        </div>
      </template>
    </el-alert>

    <!-- 多国家标签页 -->
    <el-tabs
      v-if="hasMultipleCountries"
      v-model="activeCountry"
      class="country-tabs"
      type="border-card"
    >
      <!-- 概览标签页 -->
      <el-tab-pane label="总览" name="overview">
        <div class="summary-section">
          <div class="summary-header">
            <h4>多国家优化分析报告</h4>
            <el-tag :type="getScoreType(result.summary.optimization_score)">
              平均优化评分: {{ result.summary.optimization_score }}/100
            </el-tag>
          </div>

          <div class="summary-stats">
            <div class="stat-item">
              <div class="stat-value">{{ result.by_country?.length || 1 }}</div>
              <div class="stat-label">分析国家</div>
            </div>
            <div class="stat-item">
              <div class="stat-value">{{ result.negative_words.length }}</div>
              <div class="stat-label">否定词建议</div>
            </div>
            <div class="stat-item">
              <div class="stat-value">{{ result.bid_adjustments.length }}</div>
              <div class="stat-label">竞价调整</div>
            </div>
            <div class="stat-item">
              <div class="stat-value">{{ result.keyword_opportunities.length }}</div>
              <div class="stat-label">新词机会</div>
            </div>
          </div>

          <div class="key-insights">
            <div class="insights-title">各国关键发现</div>
            <ul>
              <li v-for="(insight, index) in result.summary.key_insights" :key="index">
                {{ insight }}
              </li>
            </ul>
          </div>
        </div>
      </el-tab-pane>

      <!-- 各国家标签页 -->
      <el-tab-pane
        v-for="countryResult in result.by_country"
        :key="countryResult.country"
        :name="countryResult.country"
      >
        <template #label>
          <span class="country-tab-label">
            <span class="country-flag" v-html="getCountryFlag(countryResult.country)"></span>
            {{ getCountryLabel(countryResult.country) }}
          </span>
        </template>

        <!-- 单个国家的摘要 -->
        <div class="summary-section">
          <div class="summary-header">
            <h4><span v-html="getCountryFlag(countryResult.country)"></span> {{ getCountryLabel(countryResult.country) }} 优化分析</h4>
            <el-tag :type="getScoreType(countryResult.summary.optimization_score)">
              优化评分: {{ countryResult.summary.optimization_score }}/100
            </el-tag>
          </div>

          <div class="summary-stats">
            <div class="stat-item">
              <div class="stat-value">{{ countryResult.currency.symbol }}{{ countryResult.summary.total_spend_analyzed.toFixed(2) }}</div>
              <div class="stat-label">分析花费</div>
            </div>
            <div class="stat-item highlight">
              <div class="stat-value">{{ countryResult.currency.symbol }}{{ countryResult.summary.potential_savings.toFixed(2) }}</div>
              <div class="stat-label">预计可节省</div>
            </div>
            <div class="stat-item">
              <div class="stat-value">{{ countryResult.negative_words.length }}</div>
              <div class="stat-label">否定词建议</div>
            </div>
            <div class="stat-item">
              <div class="stat-value">{{ countryResult.bid_adjustments.length }}</div>
              <div class="stat-label">竞价调整</div>
            </div>
            <div class="stat-item">
              <div class="stat-value">{{ countryResult.keyword_opportunities.length }}</div>
              <div class="stat-label">新词机会</div>
            </div>
          </div>

          <div class="key-insights">
            <div class="insights-title">关键发现</div>
            <ul>
              <li v-for="(insight, index) in countryResult.summary.key_insights" :key="index">
                {{ insight }}
              </li>
            </ul>
          </div>
        </div>

        <!-- 该国家的详细结果 -->
        <CountryResultTabs
          :country-result="countryResult"
          @export="(type) => emit('export', type)"
        />
      </el-tab-pane>
    </el-tabs>

    <!-- 单国家模式 - 保持原有布局 -->
    <template v-else>
      <!-- 摘要卡片 -->
      <div class="summary-section">
        <div class="summary-header">
          <h4>
            <template v-if="singleCountryResult">
              <span v-html="getCountryFlag(singleCountryResult.country)"></span>
              {{ getCountryLabel(singleCountryResult.country) }} 优化分析报告
            </template>
            <template v-else>优化分析报告</template>
          </h4>
          <el-tag :type="getScoreType(currentSummary.optimization_score)">
            优化评分: {{ currentSummary.optimization_score }}/100
          </el-tag>
        </div>

        <div class="summary-stats">
          <div class="stat-item">
            <div class="stat-value">{{ currentCurrency }}{{ currentSummary.total_spend_analyzed.toFixed(2) }}</div>
            <div class="stat-label">分析花费</div>
          </div>
          <div class="stat-item highlight">
            <div class="stat-value">{{ currentCurrency }}{{ currentSummary.potential_savings.toFixed(2) }}</div>
            <div class="stat-label">预计可节省</div>
          </div>
          <div class="stat-item">
            <div class="stat-value">{{ currentNegativeWords.length }}</div>
            <div class="stat-label">否定词建议</div>
          </div>
          <div class="stat-item">
            <div class="stat-value">{{ currentBidAdjustments.length }}</div>
            <div class="stat-label">竞价调整</div>
          </div>
          <div class="stat-item">
            <div class="stat-value">{{ currentKeywordOpportunities.length }}</div>
            <div class="stat-label">新词机会</div>
          </div>
        </div>

        <div class="key-insights">
          <div class="insights-title">关键发现</div>
          <ul>
            <li v-for="(insight, index) in currentSummary.key_insights" :key="index">
              {{ insight }}
            </li>
          </ul>
        </div>
      </div>

    <!-- 详细结果 Tabs -->
    <el-tabs v-model="activeTab" class="result-tabs">
      <!-- 否定词建议 -->
      <el-tab-pane label="否定词建议" name="negative">
        <div class="tab-header">
          <span class="tab-count">共 {{ currentNegativeWords.length }} 个建议</span>
          <el-button size="small" @click="emit('export', 'negative_words')">
            <el-icon><Download /></el-icon>
            导出 Excel
          </el-button>
        </div>

        <el-table :data="currentNegativeWords" style="width: 100%">
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
              <div class="search-term-cell">
                <el-tooltip :content="formatAsin(row.search_term)" placement="top" :show-after="500">
                  <span class="copyable-text search-term-text" @click="copyToClipboard(row.search_term)">
                    {{ formatAsin(row.search_term) }}
                    <el-icon class="copy-icon"><CopyDocument /></el-icon>
                  </span>
                </el-tooltip>
                <el-tag
                  size="small"
                  :type="getRiskType(row.risk_level)"
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
          <el-table-column prop="reason" label="原因" min-width="280" />
          <el-table-column prop="spend_wasted" label="浪费花费" width="100">
            <template #default="{ row }">
              <span class="money-red">{{ currentCurrency }}{{ row.spend_wasted.toFixed(2) }}</span>
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
          <span class="tab-count">共 {{ currentBidAdjustments.length }} 个建议</span>
          <el-button size="small" @click="emit('export', 'bid_adjustments')">
            <el-icon><Download /></el-icon>
            导出 Excel
          </el-button>
        </div>

        <el-table :data="currentBidAdjustments" style="width: 100%">
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
          <el-table-column prop="reason" label="原因" min-width="280" />
        </el-table>
      </el-tab-pane>

      <!-- 新词机会 -->
      <el-tab-pane label="新词机会" name="opportunity">
        <div class="tab-header">
          <span class="tab-count">共 {{ currentKeywordOpportunities.length }} 个机会</span>
        </div>

        <el-table :data="currentKeywordOpportunities" style="width: 100%">
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { Download, Loading, CopyDocument, QuestionFilled } from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import type { AdAnalysisResult, CountryAnalysisResult } from '../../types';
import { getCountryFlag, getCountryLabel } from '../../types';
import CountryResultTabs from './CountryResultTabs.vue';

const props = defineProps<{
  result: AdAnalysisResult;
  targetAcos: number;
  isPartial?: boolean;  // 是否还在分析中（增量显示）
}>();

const emit = defineEmits<{
  (e: 'export', type: 'negative_words' | 'bid_adjustments' | 'all'): void;
}>();

const activeTab = ref('negative');
const activeCountry = ref('overview');

// 是否有多个国家
const hasMultipleCountries = computed(() => {
  return props.result.by_country && props.result.by_country.length > 1;
});

// 单国家结果（用于单国家模式）
const singleCountryResult = computed((): CountryAnalysisResult | null => {
  if (props.result.by_country && props.result.by_country.length === 1) {
    return props.result.by_country[0];
  }
  return null;
});

// 当前显示的摘要
const currentSummary = computed(() => {
  if (singleCountryResult.value) {
    return singleCountryResult.value.summary;
  }
  return props.result.summary;
});

// 当前货币符号
const currentCurrency = computed(() => {
  if (singleCountryResult.value) {
    return singleCountryResult.value.currency.symbol;
  }
  return '$';
});

// 当前否定词列表
const currentNegativeWords = computed(() => {
  if (singleCountryResult.value) {
    return singleCountryResult.value.negative_words;
  }
  return props.result.negative_words;
});

// 当前竞价调整列表
const currentBidAdjustments = computed(() => {
  if (singleCountryResult.value) {
    return singleCountryResult.value.bid_adjustments;
  }
  return props.result.bid_adjustments;
});

// 当前新词机会列表
const currentKeywordOpportunities = computed(() => {
  if (singleCountryResult.value) {
    return singleCountryResult.value.keyword_opportunities;
  }
  return props.result.keyword_opportunities;
});

function getScoreType(score: number): 'success' | 'warning' | 'danger' {
  if (score >= 70) return 'success';
  if (score >= 40) return 'warning';
  return 'danger';
}

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

// 切换到指定国家标签页
function switchToCountry(country: string) {
  // 检查该国家是否已分析
  const countryExists = props.result.by_country?.some(c => c.country === country);
  if (countryExists) {
    activeCountry.value = country;
    return true;
  }
  return false;
}

// 检查国家是否已分析
function hasCountryResult(country: string): boolean {
  return props.result.by_country?.some(c => c.country === country) || false;
}

// 暴露方法供父组件调用
defineExpose({
  switchToCountry,
  hasCountryResult,
});
</script>

<style scoped>
.ad-analysis-results {
  margin-top: 20px;
}

/* 摘要区 */
.summary-section {
  background: linear-gradient(135deg, var(--el-color-primary-light-9), var(--el-fill-color-light));
  border: 1px solid var(--el-border-color-light);
  border-radius: 12px;
  padding: 24px;
  margin-bottom: 24px;
}

.summary-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.summary-header h4 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.summary-stats {
  display: flex;
  gap: 24px;
  margin-bottom: 20px;
}

.stat-item {
  flex: 1;
  text-align: center;
  padding: 16px;
  background: var(--el-bg-color);
  border-radius: 8px;
}

.stat-item.highlight {
  background: var(--el-color-success-light-9);
}

.stat-item.highlight .stat-value {
  color: var(--el-color-success);
}

.stat-item .stat-value {
  font-size: 24px;
  font-weight: 600;
  color: var(--el-color-primary);
  margin-bottom: 4px;
}

.stat-item .stat-label {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.key-insights {
  background: var(--el-bg-color);
  border-radius: 8px;
  padding: 16px;
}

.insights-title {
  font-weight: 600;
  margin-bottom: 12px;
  color: var(--el-text-color-primary);
}

.key-insights ul {
  margin: 0;
  padding-left: 20px;
}

.key-insights li {
  margin-bottom: 8px;
  color: var(--el-text-color-regular);
  line-height: 1.6;
}

/* Tabs */
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

/* 表格单元格 */
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

/* 搜索词文本截断 */
.search-term-text {
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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

.no-campaign,
.no-data {
  color: var(--el-text-color-placeholder);
}

/* 国家标签页 */
.country-tabs {
  margin-bottom: 24px;
}

.country-tab-label {
  display: flex;
  align-items: center;
  gap: 6px;
}

.country-tab-label .country-flag {
  display: inline-flex;
  align-items: center;
  width: 24px;
  height: 16px;
}

.country-tab-label .country-flag :deep(svg) {
  width: 100%;
  height: 100%;
  border-radius: 2px;
}

/* 标题中的国旗 */
.summary-header h4 span {
  display: inline-flex;
  align-items: center;
  vertical-align: middle;
  width: 28px;
  height: 18px;
  margin-right: 6px;
}

.summary-header h4 span :deep(svg) {
  width: 100%;
  height: 100%;
  border-radius: 3px;
}

/* 增量分析提示 */
.partial-alert {
  margin-bottom: 16px;
}

.partial-alert-content {
  display: flex;
  align-items: center;
  gap: 8px;
}

.partial-alert-content .loading-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
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
</style>

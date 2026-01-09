<template>
  <div class="ad-analysis-canvas">
    <!-- 国家分析进度（仅在分析进行中且多国家时显示） -->
    <div v-if="session?.status === 'running' && session.countryProgress && session.countryProgress.total > 1" class="country-progress-bar">
      <div class="country-progress-header">
        <span class="country-flag">{{ getCountryFlag(session.currentCountry) }}</span>
        <span class="country-label">
          正在分析 <strong>{{ getCountryLabel(session.currentCountry) }}</strong> 市场
        </span>
        <span class="country-count">
          ({{ session.countryProgress.completed + 1 }} / {{ session.countryProgress.total }})
        </span>
      </div>
      <el-progress
        :percentage="((session.countryProgress.completed) / session.countryProgress.total) * 100"
        :stroke-width="8"
        :show-text="false"
        color="#67c23a"
      />
      <div class="country-list">
        <span
          v-for="country in session.countryProgress.countries"
          :key="country"
          class="country-tag"
          :class="{
            'completed': isCountryCompleted(country),
            'current': country === session.currentCountry,
            'failed': isCountryFailed(country)
          }"
        >
          {{ getCountryFlag(country) }} {{ country }}
          <span v-if="isCountryFailed(country)" class="failed-icon">✕</span>
        </span>
      </div>
    </div>

    <div class="canvas-container">
      <!-- 数据源节点 -->
      <div class="node-row">
        <div class="agent-node data-source" :class="{ 'active': session?.status !== 'idle' }">
          <div class="node-icon">
            <el-icon><Document /></el-icon>
          </div>
          <div class="node-info">
            <div class="node-name">广告报表数据</div>
            <div class="node-status">已导入</div>
          </div>
        </div>
      </div>

      <!-- 连接线 -->
      <div class="connector-row">
        <div class="connector-line vertical"></div>
        <div class="connector-branch">
          <div class="branch-line left"></div>
          <div class="branch-line center"></div>
          <div class="branch-line right"></div>
        </div>
      </div>

      <!-- 分析智能体节点 -->
      <div class="node-row agents-row">
        <div
          class="agent-node"
          :class="getNodeClass(session?.agents?.searchTermAnalyst)"
        >
          <div class="node-icon">
            <el-icon><Search /></el-icon>
          </div>
          <div class="node-info">
            <div class="node-name">搜索词分析师</div>
            <div class="node-status">{{ getStatusText(session?.agents?.searchTermAnalyst) }}</div>
          </div>
          <div v-if="session?.agents?.searchTermAnalyst?.status === 'running'" class="node-progress">
            <el-progress
              :percentage="session.agents.searchTermAnalyst.progress"
              :show-text="false"
              :stroke-width="3"
            />
          </div>
        </div>

        <div
          class="agent-node"
          :class="getNodeClass(session?.agents?.acosExpert)"
        >
          <div class="node-icon">
            <el-icon><TrendCharts /></el-icon>
          </div>
          <div class="node-info">
            <div class="node-name">ACOS 专家</div>
            <div class="node-status">{{ getStatusText(session?.agents?.acosExpert) }}</div>
          </div>
          <div v-if="session?.agents?.acosExpert?.status === 'running'" class="node-progress">
            <el-progress
              :percentage="session.agents.acosExpert.progress"
              :show-text="false"
              :stroke-width="3"
            />
          </div>
        </div>

        <div
          class="agent-node"
          :class="getNodeClass(session?.agents?.bidStrategist)"
        >
          <div class="node-icon">
            <el-icon><Money /></el-icon>
          </div>
          <div class="node-info">
            <div class="node-name">竞价策略师</div>
            <div class="node-status">{{ getStatusText(session?.agents?.bidStrategist) }}</div>
          </div>
          <div v-if="session?.agents?.bidStrategist?.status === 'running'" class="node-progress">
            <el-progress
              :percentage="session.agents.bidStrategist.progress"
              :show-text="false"
              :stroke-width="3"
            />
          </div>
        </div>
      </div>

      <!-- 连接线 -->
      <div class="connector-row">
        <div class="connector-branch">
          <div class="branch-line-up left"></div>
          <div class="branch-line-up center"></div>
          <div class="branch-line-up right"></div>
        </div>
        <div class="connector-line vertical"></div>
      </div>

      <!-- 整合器节点 -->
      <div class="node-row">
        <div
          class="agent-node integrator"
          :class="getNodeClass(session?.agents?.suggestionIntegrator)"
        >
          <div class="node-icon">
            <el-icon><Merge /></el-icon>
          </div>
          <div class="node-info">
            <div class="node-name">建议整合器</div>
            <div class="node-status">{{ getStatusText(session?.agents?.suggestionIntegrator) }}</div>
          </div>
          <div v-if="session?.agents?.suggestionIntegrator?.status === 'running'" class="node-progress">
            <el-progress
              :percentage="session.agents.suggestionIntegrator.progress"
              :show-text="false"
              :stroke-width="3"
            />
          </div>
        </div>
      </div>

      <!-- 连接线 -->
      <div class="connector-row">
        <div class="connector-line vertical"></div>
        <div class="connector-branch">
          <div class="branch-line left"></div>
          <div class="branch-line center"></div>
          <div class="branch-line right"></div>
        </div>
      </div>

      <!-- 结果节点 -->
      <div class="node-row results-row">
        <div class="result-node" :class="{ 'active': session?.status === 'completed' }">
          <el-icon><CircleClose /></el-icon>
          <span>否定词建议</span>
        </div>
        <div class="result-node" :class="{ 'active': session?.status === 'completed' }">
          <el-icon><Setting /></el-icon>
          <span>竞价调整</span>
        </div>
        <div class="result-node" :class="{ 'active': session?.status === 'completed' }">
          <el-icon><Star /></el-icon>
          <span>新词机会</span>
        </div>
      </div>
    </div>

    <!-- 实时日志 -->
    <div v-if="session?.status === 'running'" class="analysis-log">
      <div class="log-header">
        <el-icon class="loading-icon"><Loading /></el-icon>
        <span>分析进行中...</span>
      </div>
      <div class="log-content">
        <div v-for="agent in activeAgents" :key="agent.id" class="log-item">
          <span class="agent-name">{{ agent.name }}:</span>
          <span class="agent-message">{{ getAgentMessage(agent) }}</span>
        </div>
      </div>

      <!-- 流式输出预览 -->
      <div v-if="currentStreamingContent" class="streaming-preview">
        <div class="streaming-header">
          <span class="streaming-title">AI 输出中...</span>
          <span class="typing-cursor">▋</span>
        </div>
        <pre class="streaming-content">{{ currentStreamingContent }}</pre>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import {
  Document,
  Search,
  TrendCharts,
  Money,
  CircleClose,
  Setting,
  Star,
  Loading,
} from '@element-plus/icons-vue';
import type { AnalysisSession, AgentState } from '../../ad-prompts';

// 使用 Connection 图标作为 Merge 的替代
import { Connection as Merge } from '@element-plus/icons-vue';

const props = defineProps<{
  session?: AnalysisSession | null;
}>();

const activeAgents = computed(() => {
  if (!props.session?.agents) return [];
  return Object.values(props.session.agents).filter(
    (agent) => agent.status === 'running' || agent.status === 'completed'
  );
});

// 获取当前正在流式输出的内容（取所有运行中智能体的流式内容）
const currentStreamingContent = computed(() => {
  if (!props.session?.agents) return '';
  const runningAgents = Object.values(props.session.agents).filter(
    (agent) => agent.status === 'running' && agent.streamingContent
  );
  if (runningAgents.length === 0) return '';
  // 合并所有运行中智能体的流式内容
  return runningAgents
    .map(agent => `[${agent.name}]\n${agent.streamingContent}`)
    .join('\n\n');
});

function getNodeClass(agent?: AgentState): Record<string, boolean> {
  return {
    'pending': agent?.status === 'pending',
    'running': agent?.status === 'running',
    'completed': agent?.status === 'completed',
    'error': agent?.status === 'error',
  };
}

function getStatusText(agent?: AgentState): string {
  if (!agent) return '等待中';
  switch (agent.status) {
    case 'pending': return '等待中';
    case 'running': return '执行中...';
    case 'completed': return '已完成';
    case 'error': return '出错';
    default: return '未知';
  }
}

function getAgentMessage(agent: AgentState): string {
  if (agent.message) {
    return agent.message;
  }
  if (agent.status === 'running') {
    return `正在分析... ${agent.progress}%`;
  }
  if (agent.status === 'completed') {
    return '分析完成';
  }
  if (agent.status === 'error') {
    return agent.error || '分析出错';
  }
  return '等待中...';
}

// 国家相关辅助函数
const COUNTRY_FLAGS: Record<string, string> = {
  'US': '🇺🇸',
  'UK': '🇬🇧',
  'DE': '🇩🇪',
  'FR': '🇫🇷',
  'IT': '🇮🇹',
  'ES': '🇪🇸',
  'CA': '🇨🇦',
  'MX': '🇲🇽',
  'JP': '🇯🇵',
  'AU': '🇦🇺',
};

const COUNTRY_LABELS: Record<string, string> = {
  'US': '美国',
  'UK': '英国',
  'DE': '德国',
  'FR': '法国',
  'IT': '意大利',
  'ES': '西班牙',
  'CA': '加拿大',
  'MX': '墨西哥',
  'JP': '日本',
  'AU': '澳大利亚',
};

function getCountryFlag(country?: string): string {
  if (!country) return '🌍';
  return COUNTRY_FLAGS[country] || '🌍';
}

function getCountryLabel(country?: string): string {
  if (!country) return '准备中...';
  if (country === 'Unknown') return '未知市场';
  return COUNTRY_LABELS[country] || country;
}

function isCountryCompleted(country: string): boolean {
  if (!props.session?.countryProgress) return false;
  const { completed, countries, failedCountries } = props.session.countryProgress;
  // 失败的国家不算完成
  if (failedCountries?.includes(country)) return false;
  const index = countries.indexOf(country);
  return index < completed;
}

function isCountryFailed(country: string): boolean {
  if (!props.session?.countryProgress?.failedCountries) return false;
  return props.session.countryProgress.failedCountries.includes(country);
}
</script>

<style scoped>
.ad-analysis-canvas {
  padding: 20px;
}

.canvas-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0;
}

.node-row {
  display: flex;
  justify-content: center;
  gap: 40px;
}

.agents-row {
  gap: 60px;
}

/* 智能体节点 */
.agent-node {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  background: var(--el-fill-color-light);
  border: 2px solid var(--el-border-color);
  border-radius: 12px;
  min-width: 180px;
  transition: all 0.3s;
  position: relative;
}

.agent-node.data-source,
.agent-node.integrator {
  background: var(--el-color-primary-light-9);
  border-color: var(--el-color-primary-light-5);
}

.agent-node.pending {
  opacity: 0.6;
}

.agent-node.running {
  border-color: var(--el-color-primary);
  box-shadow: 0 0 0 4px var(--el-color-primary-light-8);
  animation: pulse 1.5s infinite;
}

.agent-node.completed {
  border-color: var(--el-color-success);
  background: var(--el-color-success-light-9);
}

.agent-node.error {
  border-color: var(--el-color-danger);
  background: var(--el-color-danger-light-9);
}

.agent-node.active {
  border-color: var(--el-color-success);
}

@keyframes pulse {
  0%, 100% {
    box-shadow: 0 0 0 4px var(--el-color-primary-light-8);
  }
  50% {
    box-shadow: 0 0 0 8px var(--el-color-primary-light-9);
  }
}

.node-icon {
  font-size: 24px;
  color: var(--el-color-primary);
}

.agent-node.completed .node-icon {
  color: var(--el-color-success);
}

.agent-node.error .node-icon {
  color: var(--el-color-danger);
}

.node-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.node-name {
  font-weight: 600;
  font-size: 14px;
  color: var(--el-text-color-primary);
}

.node-status {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.node-progress {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 0 4px 4px;
}

/* 连接线 */
.connector-row {
  display: flex;
  flex-direction: column;
  align-items: center;
  height: 40px;
}

.connector-line.vertical {
  width: 2px;
  height: 20px;
  background: var(--el-border-color);
}

.connector-branch {
  display: flex;
  justify-content: center;
  gap: 120px;
}

.branch-line {
  width: 2px;
  height: 20px;
  background: var(--el-border-color);
  position: relative;
}

.branch-line::before {
  content: '';
  position: absolute;
  top: 0;
  width: 120px;
  height: 2px;
  background: var(--el-border-color);
}

.branch-line.left::before {
  right: 0;
}

.branch-line.right::before {
  left: 0;
}

.branch-line.center::before {
  display: none;
}

.branch-line-up {
  width: 2px;
  height: 20px;
  background: var(--el-border-color);
  position: relative;
}

.branch-line-up::before {
  content: '';
  position: absolute;
  bottom: 0;
  width: 120px;
  height: 2px;
  background: var(--el-border-color);
}

.branch-line-up.left::before {
  right: 0;
}

.branch-line-up.right::before {
  left: 0;
}

.branch-line-up.center::before {
  display: none;
}

/* 结果节点 */
.results-row {
  gap: 40px;
}

.result-node {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: var(--el-fill-color-light);
  border: 1px solid var(--el-border-color);
  border-radius: 8px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
  transition: all 0.3s;
}

.result-node.active {
  background: var(--el-color-success-light-9);
  border-color: var(--el-color-success);
  color: var(--el-color-success);
}

/* 日志区 */
.analysis-log {
  margin-top: 24px;
  padding: 16px;
  background: var(--el-fill-color-darker);
  border-radius: 8px;
}

.log-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.loading-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.log-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.log-item {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.agent-name {
  color: var(--el-color-primary);
  margin-right: 8px;
}

/* 流式输出预览 */
.streaming-preview {
  margin-top: 16px;
  padding: 12px;
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color);
  border-radius: 8px;
  max-height: 150px;
  overflow-y: auto;
}

.streaming-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.streaming-title {
  font-size: 12px;
  color: var(--el-color-primary);
  font-weight: 500;
}

.typing-cursor {
  animation: blink 1s infinite;
  color: var(--el-color-primary);
}

@keyframes blink {
  0%, 50% { opacity: 1; }
  51%, 100% { opacity: 0; }
}

.streaming-content {
  margin: 0;
  font-size: 11px;
  line-height: 1.4;
  color: var(--el-text-color-secondary);
  white-space: pre-wrap;
  word-break: break-all;
  font-family: monospace;
}

/* 国家分析进度 */
.country-progress-bar {
  margin-bottom: 20px;
  padding: 16px;
  background: linear-gradient(135deg, var(--el-color-primary-light-9), var(--el-fill-color-light));
  border: 1px solid var(--el-color-primary-light-7);
  border-radius: 12px;
}

.country-progress-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.country-flag {
  font-size: 24px;
}

.country-label {
  font-size: 14px;
  color: var(--el-text-color-primary);
}

.country-label strong {
  color: var(--el-color-primary);
}

.country-count {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-left: auto;
}

.country-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 12px;
}

.country-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  background: var(--el-fill-color);
  border: 1px solid var(--el-border-color);
  border-radius: 16px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  transition: all 0.3s;
}

.country-tag.completed {
  background: var(--el-color-success-light-9);
  border-color: var(--el-color-success);
  color: var(--el-color-success);
}

.country-tag.current {
  background: var(--el-color-primary-light-9);
  border-color: var(--el-color-primary);
  color: var(--el-color-primary);
  animation: pulse-tag 1.5s infinite;
}

@keyframes pulse-tag {
  0%, 100% {
    box-shadow: 0 0 0 2px var(--el-color-primary-light-8);
  }
  50% {
    box-shadow: 0 0 0 4px var(--el-color-primary-light-9);
  }
}

.country-tag.failed {
  background: var(--el-color-danger-light-9);
  border-color: var(--el-color-danger);
  color: var(--el-color-danger);
}

.failed-icon {
  margin-left: 4px;
  font-weight: bold;
}
</style>

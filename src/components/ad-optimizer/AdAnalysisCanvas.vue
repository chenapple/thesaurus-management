<template>
  <div class="ad-analysis-canvas">
    <!-- 国家分析进度（仅在分析进行中且多国家时显示） -->
    <div v-if="session?.status === 'running' && session.countryProgress && session.countryProgress.total > 1" class="country-progress-bar">
      <div class="country-progress-header">
        <span class="country-flag" v-html="getCountryFlag(session.currentCountry)"></span>
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
          <span class="country-flag-mini" v-html="getCountryFlag(country)"></span> {{ getCountryLabel(country) }}
          <span v-if="isCountryFailed(country)" class="failed-icon">✕</span>
        </span>
      </div>
    </div>

    <!-- 流程图容器 -->
    <div class="flow-container" ref="flowContainer">
      <!-- SVG 连接线层 -->
      <svg class="flow-lines" :viewBox="`0 0 ${svgWidth} ${svgHeight}`" preserveAspectRatio="xMidYMid meet">
        <defs>
          <!-- 渐变定义 -->
          <linearGradient id="line-gradient-active" x1="0%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" stop-color="var(--el-color-primary)" />
            <stop offset="100%" stop-color="var(--el-color-primary-light-3)" />
          </linearGradient>
          <linearGradient id="line-gradient-success" x1="0%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" stop-color="var(--el-color-success)" />
            <stop offset="100%" stop-color="var(--el-color-success-light-3)" />
          </linearGradient>
          <!-- 箭头标记 -->
          <marker id="arrow" markerWidth="8" markerHeight="8" refX="6" refY="4" orient="auto">
            <path d="M0,0 L8,4 L0,8 L2,4 Z" fill="var(--el-border-color-darker)" />
          </marker>
          <marker id="arrow-active" markerWidth="8" markerHeight="8" refX="6" refY="4" orient="auto">
            <path d="M0,0 L8,4 L0,8 L2,4 Z" fill="var(--el-color-primary)" />
          </marker>
          <marker id="arrow-success" markerWidth="8" markerHeight="8" refX="6" refY="4" orient="auto">
            <path d="M0,0 L8,4 L0,8 L2,4 Z" fill="var(--el-color-success)" />
          </marker>
          <!-- 流动动画 -->
          <filter id="glow">
            <feGaussianBlur stdDeviation="2" result="coloredBlur"/>
            <feMerge>
              <feMergeNode in="coloredBlur"/>
              <feMergeNode in="SourceGraphic"/>
            </feMerge>
          </filter>
        </defs>

        <!-- 数据源 -> 三个分析师 -->
        <g class="connection-group" :class="{ 'active': isDataSourceActive }">
          <!-- 左分支 -->
          <path
            :d="`M ${centerX} ${row1Bottom}
                 L ${centerX} ${row1Bottom + 15}
                 L ${agent1X} ${row1Bottom + 15}
                 L ${agent1X} ${row2Top}`"
            :class="getLineClass('source-to-agents')"
            fill="none"
            marker-end="url(#arrow)"
          />
          <!-- 中分支 -->
          <path
            :d="`M ${centerX} ${row1Bottom} L ${centerX} ${row2Top}`"
            :class="getLineClass('source-to-agents')"
            fill="none"
            marker-end="url(#arrow)"
          />
          <!-- 右分支 -->
          <path
            :d="`M ${centerX} ${row1Bottom}
                 L ${centerX} ${row1Bottom + 15}
                 L ${agent3X} ${row1Bottom + 15}
                 L ${agent3X} ${row2Top}`"
            :class="getLineClass('source-to-agents')"
            fill="none"
            marker-end="url(#arrow)"
          />
        </g>

        <!-- 三个分析师 -> 整合器 -->
        <g class="connection-group">
          <!-- 左分支 -->
          <path
            :d="`M ${agent1X} ${row2Bottom}
                 L ${agent1X} ${row2Bottom + 15}
                 L ${centerX} ${row2Bottom + 15}
                 L ${centerX} ${row3Top}`"
            :class="getLineClass('agents-to-integrator', 'searchTermAnalyst')"
            fill="none"
            marker-end="url(#arrow)"
          />
          <!-- 中分支 -->
          <path
            :d="`M ${centerX} ${row2Bottom} L ${centerX} ${row3Top}`"
            :class="getLineClass('agents-to-integrator', 'acosExpert')"
            fill="none"
            marker-end="url(#arrow)"
          />
          <!-- 右分支 -->
          <path
            :d="`M ${agent3X} ${row2Bottom}
                 L ${agent3X} ${row2Bottom + 15}
                 L ${centerX} ${row2Bottom + 15}
                 L ${centerX} ${row3Top}`"
            :class="getLineClass('agents-to-integrator', 'bidStrategist')"
            fill="none"
            marker-end="url(#arrow)"
          />
        </g>

        <!-- 整合器 -> 三个结果 -->
        <g class="connection-group">
          <!-- 左分支 -->
          <path
            :d="`M ${centerX} ${row3Bottom}
                 L ${centerX} ${row3Bottom + 15}
                 L ${result1X} ${row3Bottom + 15}
                 L ${result1X} ${row4Top}`"
            :class="getLineClass('integrator-to-results')"
            fill="none"
            marker-end="url(#arrow)"
          />
          <!-- 中分支 -->
          <path
            :d="`M ${centerX} ${row3Bottom} L ${centerX} ${row4Top}`"
            :class="getLineClass('integrator-to-results')"
            fill="none"
            marker-end="url(#arrow)"
          />
          <!-- 右分支 -->
          <path
            :d="`M ${centerX} ${row3Bottom}
                 L ${centerX} ${row3Bottom + 15}
                 L ${result3X} ${row3Bottom + 15}
                 L ${result3X} ${row4Top}`"
            :class="getLineClass('integrator-to-results')"
            fill="none"
            marker-end="url(#arrow)"
          />
        </g>

        <!-- 运行时的动态流动效果 -->
        <g v-if="session?.status === 'running'" class="flow-particles">
          <circle
            v-for="(particle, idx) in flowParticles"
            :key="idx"
            :cx="particle.x"
            :cy="particle.y"
            r="3"
            class="particle"
            :style="{ animationDelay: `${idx * 0.3}s` }"
          />
        </g>
      </svg>

      <!-- 节点层 -->
      <div class="nodes-layer">
        <!-- 数据源节点 -->
        <div class="node-row row-1">
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

        <!-- 分析智能体节点 -->
        <div class="node-row row-2">
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

        <!-- 整合器节点 -->
        <div class="node-row row-3">
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

        <!-- 结果节点 -->
        <div class="node-row row-4">
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
          <span class="typing-cursor">|</span>
        </div>
        <pre class="streaming-content">{{ currentStreamingContent }}</pre>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import {
  Document,
  Search,
  TrendCharts,
  Money,
  CircleClose,
  Setting,
  Star,
  Loading,
  Connection as Merge,
} from '@element-plus/icons-vue';
import type { AnalysisSession, AgentState } from '../../ad-prompts';

const props = defineProps<{
  session?: AnalysisSession | null;
}>();

const flowContainer = ref<HTMLElement>();

// SVG 尺寸和坐标计算
const svgWidth = 720;
const svgHeight = 440;
const centerX = svgWidth / 2;

// 节点位置 (基于 180px 节点宽度 + 40px 间距)
const nodeWidth = 180;
const nodeGap = 40;
const totalRowWidth = nodeWidth * 3 + nodeGap * 2;
const startX = (svgWidth - totalRowWidth) / 2;

const agent1X = startX + nodeWidth / 2;
const agent3X = startX + totalRowWidth - nodeWidth / 2;
const result1X = agent1X;
const result3X = agent3X;

// 行的 Y 坐标
const row1Bottom = 80;
const row2Top = 120;
const row2Bottom = 200;
const row3Top = 240;
const row3Bottom = 320;
const row4Top = 360;

// 流动粒子（用于动画效果）
const flowParticles = computed(() => {
  return [
    { x: centerX, y: row1Bottom + 10 },
    { x: agent1X, y: row2Top - 10 },
    { x: agent3X, y: row2Top - 10 },
  ];
});

// 数据源是否激活
const isDataSourceActive = computed(() => {
  return props.session?.status !== 'idle';
});

const activeAgents = computed(() => {
  if (!props.session?.agents) return [];
  return Object.values(props.session.agents).filter(
    (agent) => agent.status === 'running' || agent.status === 'completed'
  );
});

// 获取当前正在流式输出的内容
const currentStreamingContent = computed(() => {
  if (!props.session?.agents) return '';
  const runningAgents = Object.values(props.session.agents).filter(
    (agent) => agent.status === 'running' && agent.streamingContent
  );
  if (runningAgents.length === 0) return '';
  return runningAgents
    .map(agent => `[${agent.name}]\n${agent.streamingContent}`)
    .join('\n\n');
});

// 获取连接线的样式类
function getLineClass(section: string, agentKey?: string): string {
  const classes = ['flow-line'];

  if (section === 'source-to-agents') {
    if (props.session?.status === 'running' || props.session?.status === 'completed') {
      classes.push('active');
    }
    if (props.session?.status === 'completed') {
      classes.push('success');
    }
  } else if (section === 'agents-to-integrator' && agentKey) {
    const agent = props.session?.agents?.[agentKey as keyof typeof props.session.agents];
    if (agent?.status === 'running') {
      classes.push('active');
    }
    if (agent?.status === 'completed') {
      classes.push('success');
    }
  } else if (section === 'integrator-to-results') {
    const integrator = props.session?.agents?.suggestionIntegrator;
    if (integrator?.status === 'running') {
      classes.push('active');
    }
    if (integrator?.status === 'completed' || props.session?.status === 'completed') {
      classes.push('success');
    }
  }

  return classes.join(' ');
}

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

// 使用 types.ts 中的函数（支持代码和名称查找）
import { getCountryFlag as getFlag, getCountryLabel as getLabel } from '../../types';

function getCountryFlag(country?: string): string {
  if (!country) return '';
  const flag = getFlag(country);
  return flag || '';  // SVG 或空
}

function getCountryLabel(country?: string): string {
  if (!country) return '准备中...';
  if (country === 'Unknown') return '未知市场';
  return getLabel(country);
}

function isCountryCompleted(country: string): boolean {
  if (!props.session?.countryProgress) return false;
  const { completed, countries, failedCountries } = props.session.countryProgress;
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

/* 流程图容器 */
.flow-container {
  position: relative;
  width: 100%;
  max-width: 720px;
  margin: 0 auto;
  height: 440px;
}

/* SVG 连接线 */
.flow-lines {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 1;
}

.flow-line {
  stroke: var(--el-border-color-darker);
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
  transition: stroke 0.3s, stroke-width 0.3s;
}

.flow-line.active {
  stroke: var(--el-color-primary);
  stroke-width: 2.5;
  animation: line-pulse 1.5s ease-in-out infinite;
}

.flow-line.success {
  stroke: var(--el-color-success);
  stroke-width: 2;
  animation: none;
}

@keyframes line-pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.6;
  }
}

/* 流动粒子 */
.particle {
  fill: var(--el-color-primary);
  opacity: 0;
  animation: particle-flow 2s ease-in-out infinite;
}

@keyframes particle-flow {
  0% {
    opacity: 0;
    transform: translateY(0);
  }
  20% {
    opacity: 1;
  }
  80% {
    opacity: 1;
  }
  100% {
    opacity: 0;
    transform: translateY(80px);
  }
}

/* 节点层 */
.nodes-layer {
  position: relative;
  z-index: 2;
  display: flex;
  flex-direction: column;
  height: 100%;
}

.node-row {
  display: flex;
  justify-content: center;
  gap: 40px;
}

.row-1 {
  padding-top: 20px;
  margin-bottom: 40px;
}

.row-2 {
  margin-bottom: 40px;
}

.row-3 {
  margin-bottom: 40px;
}

.row-4 {
  /* 最后一行 */
}

/* 智能体节点 */
.agent-node {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  background: var(--el-bg-color);
  border: 2px solid var(--el-border-color);
  border-radius: 12px;
  min-width: 180px;
  transition: all 0.3s ease;
  position: relative;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
}

.agent-node:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
  transform: translateY(-2px);
}

.agent-node.data-source,
.agent-node.integrator {
  background: linear-gradient(135deg, var(--el-color-primary-light-9), var(--el-bg-color));
  border-color: var(--el-color-primary-light-5);
}

.agent-node.pending {
  opacity: 0.6;
}

.agent-node.running {
  border-color: var(--el-color-primary);
  box-shadow: 0 0 0 4px var(--el-color-primary-light-8),
              0 4px 12px rgba(64, 158, 255, 0.2);
  animation: node-pulse 2s ease-in-out infinite;
}

.agent-node.completed {
  border-color: var(--el-color-success);
  background: linear-gradient(135deg, var(--el-color-success-light-9), var(--el-bg-color));
}

.agent-node.error {
  border-color: var(--el-color-danger);
  background: linear-gradient(135deg, var(--el-color-danger-light-9), var(--el-bg-color));
}

.agent-node.active {
  border-color: var(--el-color-success);
}

@keyframes node-pulse {
  0%, 100% {
    box-shadow: 0 0 0 4px var(--el-color-primary-light-8),
                0 4px 12px rgba(64, 158, 255, 0.2);
  }
  50% {
    box-shadow: 0 0 0 8px var(--el-color-primary-light-9),
                0 4px 16px rgba(64, 158, 255, 0.3);
  }
}

.node-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: 10px;
  background: var(--el-color-primary-light-8);
  color: var(--el-color-primary);
  font-size: 20px;
  flex-shrink: 0;
}

.agent-node.completed .node-icon {
  background: var(--el-color-success-light-8);
  color: var(--el-color-success);
}

.agent-node.error .node-icon {
  background: var(--el-color-danger-light-8);
  color: var(--el-color-danger);
}

.node-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
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
  bottom: 4px;
  left: 12px;
  right: 12px;
}

/* 结果节点 */
.result-node {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 14px 20px;
  background: var(--el-bg-color);
  border: 2px solid var(--el-border-color);
  border-radius: 10px;
  font-size: 13px;
  font-weight: 500;
  color: var(--el-text-color-secondary);
  transition: all 0.3s ease;
  min-width: 140px;
  justify-content: center;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
}

.result-node:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

.result-node.active {
  background: linear-gradient(135deg, var(--el-color-success-light-9), var(--el-bg-color));
  border-color: var(--el-color-success);
  color: var(--el-color-success);
}

.result-node .el-icon {
  font-size: 18px;
}

/* 日志区 */
.analysis-log {
  margin-top: 24px;
  padding: 16px;
  background: var(--el-fill-color-darker);
  border-radius: 12px;
  border: 1px solid var(--el-border-color-light);
}

.log-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.log-header .stop-btn {
  margin-left: auto;
}

.loading-icon {
  animation: spin 1s linear infinite;
  color: var(--el-color-primary);
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
  padding: 4px 0;
}

.agent-name {
  color: var(--el-color-primary);
  font-weight: 500;
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
  gap: 4px;
  margin-bottom: 8px;
}

.streaming-title {
  font-size: 12px;
  color: var(--el-color-primary);
  font-weight: 500;
}

.typing-cursor {
  animation: cursor-blink 1s step-end infinite;
  color: var(--el-color-primary);
  font-weight: bold;
}

@keyframes cursor-blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}

.streaming-content {
  margin: 0;
  font-size: 11px;
  line-height: 1.5;
  color: var(--el-text-color-secondary);
  white-space: pre-wrap;
  word-break: break-all;
  font-family: 'SF Mono', Monaco, Consolas, monospace;
}

/* 国家分析进度 */
.country-progress-bar {
  margin-bottom: 20px;
  padding: 16px 20px;
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
  display: inline-flex;
  align-items: center;
}

.country-flag :deep(svg) {
  width: 24px;
  height: 16px;
  border-radius: 2px;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

.country-flag-mini {
  display: inline-flex;
  align-items: center;
}

.country-flag-mini :deep(svg) {
  width: 18px;
  height: 12px;
  border-radius: 2px;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
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
  padding: 6px 12px;
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color);
  border-radius: 20px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  transition: all 0.3s ease;
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
  animation: tag-pulse 2s ease-in-out infinite;
}

@keyframes tag-pulse {
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

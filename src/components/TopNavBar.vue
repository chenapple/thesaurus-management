<script setup lang="ts">
import { ArrowDown, DataLine, Document, Grid, PieChart, TrendCharts, EditPen, Promotion, ChatDotRound, Cpu, Setting, QuestionFilled } from "@element-plus/icons-vue";

type ViewMode = 'dashboard' | 'keywords' | 'roots' | 'wordcloud' | 'monitoring' | 'smartcopy' | 'knowledge' | 'ads' | 'agent';

defineProps<{
  viewMode: ViewMode;
  enableAgent: boolean;
}>();

const emit = defineEmits<{
  (e: 'switch-view', mode: ViewMode): void;
  (e: 'show-api-key-dialog'): void;
  (e: 'show-shortcuts-dialog'): void;
  (e: 'show-exchange-rate-settings'): void;
  (e: 'show-help-dialog'): void;
}>();

function switchView(mode: ViewMode) {
  emit('switch-view', mode);
}
</script>

<template>
  <nav class="top-nav">
    <el-button-group class="view-toggle">
      <el-button
        :type="viewMode === 'dashboard' ? 'primary' : 'default'"
        @click="switchView('dashboard')"
      >
        <el-icon><DataLine /></el-icon>
        概览
      </el-button>
      <el-button
        :type="viewMode === 'keywords' ? 'primary' : 'default'"
        @click="switchView('keywords')"
      >
        <el-icon><Document /></el-icon>
        关键词
      </el-button>
      <el-button
        :type="viewMode === 'roots' ? 'primary' : 'default'"
        @click="switchView('roots')"
      >
        <el-icon><Grid /></el-icon>
        词根
      </el-button>
      <el-button
        :type="viewMode === 'wordcloud' ? 'primary' : 'default'"
        @click="switchView('wordcloud')"
      >
        <el-icon><PieChart /></el-icon>
        词云
      </el-button>
      <el-button
        :type="viewMode === 'monitoring' ? 'primary' : 'default'"
        @click="switchView('monitoring')"
      >
        <el-icon><TrendCharts /></el-icon>
        排名监控
      </el-button>
      <el-button
        :type="viewMode === 'smartcopy' ? 'primary' : 'default'"
        @click="switchView('smartcopy')"
      >
        <el-icon><EditPen /></el-icon>
        智能文案
      </el-button>
      <el-button
        :type="viewMode === 'ads' ? 'primary' : 'default'"
        @click="switchView('ads')"
      >
        <el-icon><Promotion /></el-icon>
        智能广告
      </el-button>
      <el-button
        :type="viewMode === 'knowledge' ? 'primary' : 'default'"
        @click="switchView('knowledge')"
      >
        <el-icon><ChatDotRound /></el-icon>
        知识库
      </el-button>
      <el-button
        v-if="enableAgent"
        :type="viewMode === 'agent' ? 'primary' : 'default'"
        @click="switchView('agent')"
      >
        <el-icon><Cpu /></el-icon>
        智能体
      </el-button>
    </el-button-group>
    <el-dropdown trigger="click" class="global-settings-dropdown">
      <el-button>
        <el-icon><Setting /></el-icon>
        设置
        <el-icon class="el-icon--right"><ArrowDown /></el-icon>
      </el-button>
      <template #dropdown>
        <el-dropdown-menu>
          <el-dropdown-item @click="emit('show-api-key-dialog')">API Key</el-dropdown-item>
          <el-dropdown-item @click="emit('show-shortcuts-dialog')">快捷键</el-dropdown-item>
          <el-dropdown-item @click="emit('show-exchange-rate-settings')">汇率显示</el-dropdown-item>
        </el-dropdown-menu>
      </template>
    </el-dropdown>
    <el-button class="nav-help-btn" @click="emit('show-help-dialog')">
      <el-icon><QuestionFilled /></el-icon>
      帮助
    </el-button>
  </nav>
</template>

<style scoped>
.top-nav {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 10px 20px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
  position: relative;
}

.global-settings-dropdown {
  position: absolute;
  right: 80px;
}

.nav-help-btn {
  position: absolute;
  right: 20px;
}

.view-toggle {
  margin-right: 8px;
}
</style>

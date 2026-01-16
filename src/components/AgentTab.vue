<script setup lang="ts">
import { ref, computed, onUnmounted, watch, onMounted } from 'vue';
import {
  Cpu,
  DocumentCopy,
  CircleCheck,
  Warning,
  Loading,
  QuestionFilled,
  Setting,
  Refresh,
  Tools,
  ArrowRight,
  FolderOpened,
  Search,
  Plus,
  Edit,
  Delete,
  Clock,
  VideoPlay,
} from '@element-plus/icons-vue';
import MarketResearchTaskDialog from './MarketResearchTaskDialog.vue';
import type { MarketResearchTask } from './MarketResearchTaskDialog.vue';
import { ElMessage } from 'element-plus';
import { invoke } from '@tauri-apps/api/core';
import {
  createMarketResearchAgent,
  createWeeklyReportTask,
  createQuickScanTask,
} from '../agent';
import type { AgentEvent, TaskResult } from '../agent';
import type { AIProvider } from '../types';
import { AI_PROVIDERS, COUNTRY_OPTIONS, getCountryLabel } from '../types';

// 子类目信息类型
interface SubcategoryInfo {
  name: string;
  category_id: string;
  url: string;
}

// 类目层级
interface CategoryLevel {
  name: string;
  category_id: string;
  subcategories: SubcategoryInfo[];
}

// Emits
const emit = defineEmits<{
  (e: 'showHelp', tab: string): void;
}>();

// ==================== 配置状态 ====================

// 表单数据
const selectedMarketplace = ref('US');
const customCategoryId = ref('');
const customCategoryName = ref('');
const useCustomCategory = ref(false);
const selectedProvider = ref<AIProvider>('deepseek');

// ==================== 类目浏览器状态 ====================

const showCategoryBrowser = ref(false);
const categoryLevels = ref<CategoryLevel[]>([]);
const isDiscovering = ref(false);
const discoveringLevel = ref(-1);

// 子类目缓存 (marketplace:category_id -> subcategories)
const subcategoryCache = new Map<string, SubcategoryInfo[]>();

// 一级类目列表（各站点通用）
const rootCategories = [
  { id: 'beauty', name: 'Beauty & Personal Care' },
  { id: 'electronics', name: 'Electronics' },
  { id: 'home-garden', name: 'Home & Garden' },
  { id: 'kitchen', name: 'Kitchen & Dining' },
  { id: 'pet-supplies', name: 'Pet Supplies' },
  { id: 'sports-outdoors', name: 'Sports & Outdoors' },
  { id: 'toys-games', name: 'Toys & Games' },
  { id: 'office-products', name: 'Office Products' },
  { id: 'automotive', name: 'Automotive' },
  { id: 'baby-products', name: 'Baby' },
  { id: 'clothing-shoes-jewelry', name: 'Clothing, Shoes & Jewelry' },
  { id: 'grocery', name: 'Grocery & Gourmet Food' },
  { id: 'health-personal-care', name: 'Health & Household' },
  { id: 'industrial-scientific', name: 'Industrial & Scientific' },
  { id: 'lawn-garden', name: 'Patio, Lawn & Garden' },
  { id: 'musical-instruments', name: 'Musical Instruments' },
  { id: 'tools', name: 'Tools & Home Improvement' },
];

// 当站点改变时，清空类目浏览器
watch(selectedMarketplace, () => {
  categoryLevels.value = [];
});

// AI Provider 和模型选择
const selectedModel = ref(AI_PROVIDERS.deepseek.defaultModel);

const availableModels = computed(() => {
  return AI_PROVIDERS[selectedProvider.value].models;
});

// 当切换 provider 时，更新默认 model
watch(selectedProvider, (newProvider) => {
  selectedModel.value = AI_PROVIDERS[newProvider].defaultModel;
});

// ==================== 监控任务管理 ====================

const monitoringTasks = ref<MarketResearchTask[]>([]);
const showTaskDialog = ref(false);
const showTaskListDialog = ref(false);  // 任务列表弹窗
const editingTask = ref<MarketResearchTask | null>(null);
const loadingTasks = ref(false);

async function loadMonitoringTasks() {
  loadingTasks.value = true;
  try {
    const tasks = await invoke<MarketResearchTask[]>('get_market_research_tasks');
    monitoringTasks.value = tasks;
  } catch (error) {
    console.error('加载监控任务失败:', error);
  } finally {
    loadingTasks.value = false;
  }
}

function openCreateTaskDialog() {
  editingTask.value = null;
  showTaskDialog.value = true;
}

function openEditTaskDialog(task: MarketResearchTask) {
  editingTask.value = task;
  showTaskDialog.value = true;
}

async function deleteTask(task: MarketResearchTask) {
  try {
    await invoke('delete_market_research_task', { id: task.id });
    ElMessage.success('任务已删除');
    await loadMonitoringTasks();
  } catch (error) {
    ElMessage.error(`删除失败: ${error}`);
  }
}

async function runTaskNow(task: MarketResearchTask) {
  // 使用任务配置运行 Agent
  selectedMarketplace.value = task.marketplace;
  customCategoryId.value = task.category_id;
  customCategoryName.value = task.category_name || '';
  selectedProvider.value = task.ai_provider as AIProvider;
  // 设置模型，如果任务没有指定则使用该 provider 的默认模型
  selectedModel.value = task.ai_model || AI_PROVIDERS[selectedProvider.value].defaultModel;

  // 设置当前执行的任务 ID
  runningTaskId.value = task.id || null;

  // 执行周报生成
  await runAgent(false);
}

function formatSchedule(task: MarketResearchTask): string {
  if (task.schedule_type === 'daily') {
    return `每天 ${task.schedule_time}`;
  }
  const daysMap: { [key: number]: string } = {
    0: '日', 1: '一', 2: '二', 3: '三', 4: '四', 5: '五', 6: '六'
  };
  try {
    const days = JSON.parse(task.schedule_days || '[]');
    const dayNames = days.map((d: number) => `周${daysMap[d]}`).join('、');
    return `${dayNames} ${task.schedule_time}`;
  } catch {
    return task.schedule_time;
  }
}

onMounted(() => {
  loadMonitoringTasks();
  loadLastResult(); // 加载上次的执行结果
});

// ==================== 执行状态 ====================

const isRunning = ref(false);
const currentPhase = ref<'idle' | 'thinking' | 'tool_call' | 'completed' | 'error'>('idle');
const currentIteration = ref(0);
const currentToolName = ref('');
const thinkingContent = ref('');
const toolCallHistory = ref<{ name: string; result: any; error?: string }[]>([]);
const finalResult = ref<TaskResult | null>(null);
const abortController = ref<AbortController | null>(null);
const runningTaskId = ref<number | null>(null); // 当前执行的任务 ID
const lastResultTime = ref<string>(''); // 上次结果的时间

// localStorage key
const LAST_RESULT_KEY = 'market_research_last_result';

// 计算属性
const categoryInfo = computed(() => {
  return {
    id: customCategoryId.value,
    name: customCategoryName.value || customCategoryId.value,
  };
});

const canRun = computed(() => {
  // 只要有类目 ID 就可以运行
  return customCategoryId.value.trim() !== '';
});

// 当前选中的完整类目路径
const selectedCategoryPath = computed(() => {
  if (categoryLevels.value.length === 0) return '';
  return categoryLevels.value.map(l => l.name).join(' › ');
});

// 当前选中的类目 ID
const selectedCategoryId = computed(() => {
  if (categoryLevels.value.length === 0) return '';
  return categoryLevels.value[categoryLevels.value.length - 1].category_id;
});

// ==================== 类目浏览器功能 ====================

// 打开类目浏览器
function openCategoryBrowser() {
  showCategoryBrowser.value = true;
  // 如果还没有选择一级类目，不自动发现
}

// 选择一级类目
async function selectRootCategory(category: { id: string; name: string }) {
  categoryLevels.value = [{
    name: category.name,
    category_id: category.id,
    subcategories: [],
  }];
  await discoverSubcategories(0);
}

// 发现子类目
async function discoverSubcategories(levelIndex: number) {
  if (isDiscovering.value) return;

  const level = categoryLevels.value[levelIndex];
  if (!level) return;

  // 获取当前路径中所有已选类目的 ID（用于过滤，避免循环）
  const pathCategoryIds = new Set(
    categoryLevels.value.map(l => l.category_id)
  );

  // 检查缓存
  const cacheKey = `${selectedMarketplace.value}:${level.category_id}`;
  const cached = subcategoryCache.get(cacheKey);
  if (cached) {
    console.log(`[Cache] 使用缓存的子类目: ${cacheKey}`);
    // 过滤掉已经在路径中的类目
    const filtered = cached.filter(s => !pathCategoryIds.has(s.category_id));
    categoryLevels.value[levelIndex].subcategories = filtered;
    if (filtered.length === 0) {
      ElMessage.info('该类目下没有更多子类目');
    }
    return;
  }

  isDiscovering.value = true;
  discoveringLevel.value = levelIndex;

  try {
    const result = await invoke<{
      marketplace: string;
      parent_category: string;
      subcategories: SubcategoryInfo[];
      error: string | null;
    }>('discover_subcategories', {
      marketplace: selectedMarketplace.value,
      parentCategory: level.category_id,
    });

    if (result.error) {
      ElMessage.error(`发现子类目失败: ${result.error}`);
      return;
    }

    // 存入缓存（原始数据）
    subcategoryCache.set(cacheKey, result.subcategories);
    console.log(`[Cache] 缓存子类目: ${cacheKey}, ${result.subcategories.length} 个`);

    // 过滤掉已经在路径中的类目（避免循环）
    const filtered = result.subcategories.filter(s => !pathCategoryIds.has(s.category_id));
    categoryLevels.value[levelIndex].subcategories = filtered;

    if (filtered.length === 0) {
      ElMessage.info('该类目下没有更多子类目');
    } else if (filtered.length < result.subcategories.length) {
      console.log(`[Filter] 过滤掉 ${result.subcategories.length - filtered.length} 个已在路径中的类目`);
    }
  } catch (error) {
    console.error('发现子类目失败:', error);
    ElMessage.error('发现子类目失败');
  } finally {
    isDiscovering.value = false;
    discoveringLevel.value = -1;
  }
}

// 选择子类目
async function selectSubcategory(levelIndex: number, subcategory: SubcategoryInfo) {
  // 移除该层级之后的所有层级
  categoryLevels.value = categoryLevels.value.slice(0, levelIndex + 1);

  // 添加新层级
  categoryLevels.value.push({
    name: subcategory.name,
    category_id: subcategory.category_id,
    subcategories: [],
  });

  // 自动发现下一级子类目
  await discoverSubcategories(levelIndex + 1);
}

// 确认选择类目
function confirmCategorySelection() {
  if (categoryLevels.value.length === 0) {
    ElMessage.warning('请先选择类目');
    return;
  }

  const lastLevel = categoryLevels.value[categoryLevels.value.length - 1];
  customCategoryId.value = lastLevel.category_id;
  customCategoryName.value = selectedCategoryPath.value;
  useCustomCategory.value = true;
  showCategoryBrowser.value = false;

  ElMessage.success(`已选择类目: ${selectedCategoryPath.value}`);
}

// 返回上一级
function goBackLevel(toIndex: number) {
  categoryLevels.value = categoryLevels.value.slice(0, toIndex + 1);
}

// ==================== Agent 执行 ====================

async function runAgent(quickScan = false) {
  if (!canRun.value || isRunning.value) return;

  // 重置状态
  isRunning.value = true;
  currentPhase.value = 'thinking';
  currentIteration.value = 0;
  thinkingContent.value = '';
  toolCallHistory.value = [];
  finalResult.value = null;

  // 创建 AbortController
  abortController.value = new AbortController();

  // 如果是监控任务执行，创建执行记录
  let runId: number | null = null;
  if (runningTaskId.value) {
    try {
      runId = await invoke<number>('create_research_run', {
        taskId: runningTaskId.value,
      });
    } catch (error) {
      console.error('创建执行记录失败:', error);
    }
  }

  try {
    // 创建 Agent
    const agent = createMarketResearchAgent(selectedProvider.value, selectedModel.value);

    // 监听事件
    const unsubscribe = agent.onEvent((event: AgentEvent) => {
      handleAgentEvent(event);
    });

    // 创建任务
    const task = quickScan
      ? createQuickScanTask(
          selectedMarketplace.value,
          categoryInfo.value.id,
          categoryInfo.value.name
        )
      : createWeeklyReportTask(
          selectedMarketplace.value,
          categoryInfo.value.id,
          categoryInfo.value.name
        );

    // 执行任务
    const result = await agent.execute(task, abortController.value.signal);
    finalResult.value = result;
    currentPhase.value = result.success ? 'completed' : 'error';

    // 取消订阅
    unsubscribe();

    // 保存结果
    await saveResult(result, runId);

    if (result.success) {
      ElMessage.success('Agent 任务完成');
    } else {
      ElMessage.error(`任务失败: ${result.error}`);
    }
  } catch (error) {
    console.error('Agent 执行错误:', error);
    currentPhase.value = 'error';
    const errorResult: TaskResult = {
      success: false,
      output: '',
      error: error instanceof Error ? error.message : String(error),
      toolsUsed: [],
      iterations: currentIteration.value,
    };
    finalResult.value = errorResult;

    // 保存错误结果
    await saveResult(errorResult, runId);

    ElMessage.error('Agent 执行出错');
  } finally {
    isRunning.value = false;
    abortController.value = null;
    runningTaskId.value = null; // 重置任务 ID
  }
}

// 保存执行结果
async function saveResult(result: TaskResult, runId: number | null) {
  // 1. 保存到 localStorage（用于快速恢复）
  const timestamp = new Date().toISOString();
  const resultData = {
    result,
    marketplace: selectedMarketplace.value,
    categoryId: customCategoryId.value,
    categoryName: customCategoryName.value,
    provider: selectedProvider.value,
    model: selectedModel.value,
    timestamp,
  };
  try {
    localStorage.setItem(LAST_RESULT_KEY, JSON.stringify(resultData));
    lastResultTime.value = formatResultTime(timestamp);
  } catch (error) {
    console.error('保存到 localStorage 失败:', error);
  }

  // 2. 如果有执行记录 ID，更新数据库
  if (runId) {
    try {
      if (result.success) {
        // 提取摘要（取前200个字符）
        const summary = result.output.slice(0, 200).replace(/[#\n]/g, ' ').trim();
        await invoke('update_research_run', {
          runId,
          status: 'completed',
          summary,
          content: result.output,
          snapshotId: null,
        });
      } else {
        await invoke('fail_research_run', {
          runId,
          errorMessage: result.error || '未知错误',
        });
      }
    } catch (error) {
      console.error('更新执行记录失败:', error);
    }
  }
}

// 加载上次的结果
function loadLastResult() {
  try {
    const savedData = localStorage.getItem(LAST_RESULT_KEY);
    if (savedData) {
      const data = JSON.parse(savedData);
      finalResult.value = data.result;
      // 如果结果存在，设置为已完成状态
      if (data.result) {
        currentPhase.value = data.result.success ? 'completed' : 'error';
      }
      // 加载时间戳
      if (data.timestamp) {
        lastResultTime.value = formatResultTime(data.timestamp);
      }
      console.log('已加载上次的执行结果');
    }
  } catch (error) {
    console.error('加载上次结果失败:', error);
  }
}

// 格式化结果时间
function formatResultTime(isoString: string): string {
  try {
    const date = new Date(isoString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffMins < 1) {
      return '刚刚';
    } else if (diffMins < 60) {
      return `${diffMins} 分钟前`;
    } else if (diffHours < 24) {
      return `${diffHours} 小时前`;
    } else if (diffDays < 7) {
      return `${diffDays} 天前`;
    } else {
      return date.toLocaleDateString('zh-CN', {
        month: 'numeric',
        day: 'numeric',
        hour: 'numeric',
        minute: 'numeric',
      });
    }
  } catch {
    return '';
  }
}

function handleAgentEvent(event: AgentEvent) {
  switch (event.type) {
    case 'thinking_start':
      currentPhase.value = 'thinking';
      currentIteration.value = event.data.iteration;
      thinkingContent.value = '';
      break;

    case 'thinking_chunk':
      thinkingContent.value += event.data.content;
      break;

    case 'thinking_end':
      thinkingContent.value = event.data.content;
      break;

    case 'tool_call_start':
      currentPhase.value = 'tool_call';
      currentToolName.value = event.data.toolName;
      break;

    case 'tool_call_end':
      toolCallHistory.value.push({
        name: event.data.toolName,
        result: event.data.result,
        error: event.data.error,
      });
      break;

    case 'task_complete':
      currentPhase.value = event.data.success ? 'completed' : 'error';
      break;

    case 'error':
      currentPhase.value = 'error';
      break;
  }
}

function stopAgent() {
  if (abortController.value) {
    abortController.value.abort();
    ElMessage.warning('正在停止 Agent...');
  }
}

// 清理
onUnmounted(() => {
  if (abortController.value) {
    abortController.value.abort();
  }
});

// ==================== UI 辅助 ====================

function getPhaseIcon() {
  switch (currentPhase.value) {
    case 'thinking':
      return Loading;
    case 'tool_call':
      return Tools;
    case 'completed':
      return CircleCheck;
    case 'error':
      return Warning;
    default:
      return Cpu;
  }
}

function getPhaseText() {
  switch (currentPhase.value) {
    case 'thinking':
      return `正在思考 (迭代 ${currentIteration.value})...`;
    case 'tool_call':
      return `执行工具: ${currentToolName.value}`;
    case 'completed':
      return '任务完成';
    case 'error':
      return '执行出错';
    default:
      return '准备就绪';
  }
}

// 简单的 Markdown 渲染（仅处理基本格式）
function renderMarkdown(text: string): string {
  if (!text) return '';

  return text
    // 代码块
    .replace(/```(\w*)\n([\s\S]*?)```/g, '<pre><code>$2</code></pre>')
    // 标题
    .replace(/^### (.+)$/gm, '<h4>$1</h4>')
    .replace(/^## (.+)$/gm, '<h3>$1</h3>')
    .replace(/^# (.+)$/gm, '<h2>$1</h2>')
    // 粗体
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    // 斜体
    .replace(/\*(.+?)\*/g, '<em>$1</em>')
    // 列表
    .replace(/^- (.+)$/gm, '<li>$1</li>')
    .replace(/(<li>.*<\/li>\n?)+/g, '<ul>$&</ul>')
    // 换行
    .replace(/\n/g, '<br>');
}
</script>

<template>
  <div class="agent-tab">
    <!-- 顶部标题 -->
    <div class="tab-header">
      <div class="header-left">
        <el-icon :size="24"><Cpu /></el-icon>
        <span class="title">智能体</span>
        <el-tag size="small" type="warning">Beta</el-tag>
      </div>
      <div class="header-right">
        <el-button :icon="QuestionFilled" text @click="emit('showHelp', 'agent')">
          帮助
        </el-button>
      </div>
    </div>

    <!-- 主内容区 -->
    <div class="main-content">
      <!-- 左侧：配置面板 -->
      <div class="config-panel">
        <el-card shadow="never">
          <template #header>
            <div class="card-header">
              <div class="header-title">
                <el-icon><Setting /></el-icon>
                <span>任务配置</span>
              </div>
              <el-button
                :icon="Clock"
                size="small"
                @click="showTaskListDialog = true"
              >
                监控任务
                <el-badge v-if="monitoringTasks.length > 0" :value="monitoringTasks.length" class="task-badge" />
              </el-button>
            </div>
          </template>

          <el-form label-position="top">
            <!-- AI Provider -->
            <el-form-item label="AI 服务">
              <el-select v-model="selectedProvider" :disabled="isRunning" style="width: 100%">
                <el-option
                  v-for="(config, key) in AI_PROVIDERS"
                  :key="key"
                  :value="key"
                  :label="config.name"
                />
              </el-select>
            </el-form-item>

            <!-- AI Model -->
            <el-form-item label="模型">
              <el-select v-model="selectedModel" :disabled="isRunning" style="width: 100%">
                <el-option
                  v-for="model in availableModels"
                  :key="model"
                  :value="model"
                  :label="model"
                />
              </el-select>
            </el-form-item>

            <!-- 站点选择 -->
            <el-form-item label="Amazon 站点">
              <el-select v-model="selectedMarketplace" :disabled="isRunning" style="width: 100%">
                <el-option
                  v-for="country in COUNTRY_OPTIONS"
                  :key="country.value"
                  :value="country.value"
                  :label="country.label"
                >
                  <div style="display: flex; align-items: center; gap: 8px;">
                    <span class="country-flag" v-html="country.flag"></span>
                    <span>{{ country.label }}</span>
                  </div>
                </el-option>
              </el-select>
            </el-form-item>

            <!-- 类目选择 -->
            <el-form-item label="监控类目">
              <!-- 类目浏览器按钮 -->
              <el-button
                :icon="FolderOpened"
                :disabled="isRunning"
                style="width: 100%; margin-bottom: 8px;"
                @click="openCategoryBrowser"
              >
                浏览类目
              </el-button>

              <!-- 已选类目显示 -->
              <div v-if="customCategoryId" class="selected-category">
                <div class="category-path">{{ customCategoryName || customCategoryId }}</div>
                <div class="category-id">ID: {{ customCategoryId }}</div>
              </div>

              <!-- 手动输入（折叠） -->
              <el-collapse v-model="useCustomCategory" accordion style="margin-top: 8px;">
                <el-collapse-item name="manual" title="手动输入类目 ID">
                  <el-input
                    v-model="customCategoryId"
                    :disabled="isRunning"
                    placeholder="例如: beauty 或 beauty/211005031"
                    style="margin-bottom: 8px;"
                  />
                  <el-input
                    v-model="customCategoryName"
                    :disabled="isRunning"
                    placeholder="类目名称（可选）"
                  />
                  <div class="category-hint">
                    类目 ID 可从 Amazon BSR 页面 URL 获取
                  </div>
                </el-collapse-item>
              </el-collapse>
            </el-form-item>

            <!-- 操作按钮 -->
            <el-form-item>
              <div class="action-buttons">
                <el-button
                  type="primary"
                  :icon="Cpu"
                  :loading="isRunning"
                  :disabled="!canRun"
                  @click="runAgent(false)"
                >
                  生成周报
                </el-button>
                <el-button
                  :icon="Refresh"
                  :loading="isRunning"
                  :disabled="!canRun"
                  @click="runAgent(true)"
                >
                  快速扫描
                </el-button>
                <el-button
                  v-if="isRunning"
                  type="danger"
                  @click="stopAgent"
                >
                  停止
                </el-button>
              </div>
            </el-form-item>
          </el-form>
        </el-card>
      </div>

      <!-- 右侧：结果展示 -->
      <div class="result-panel">
        <el-card shadow="never" class="result-card">
          <template #header>
            <div class="card-header">
              <div class="header-title">
                <el-icon><DocumentCopy /></el-icon>
                <span>执行结果</span>
                <el-tag v-if="finalResult" :type="finalResult.success ? 'success' : 'danger'" size="small">
                  {{ finalResult.success ? '成功' : '失败' }}
                </el-tag>
              </div>
              <span v-if="lastResultTime && finalResult && !isRunning" class="result-time">
                {{ lastResultTime }}
              </span>
            </div>
          </template>

          <!-- 空状态 -->
          <div v-if="!finalResult && currentPhase === 'idle'" class="empty-state">
            <el-icon :size="48"><Cpu /></el-icon>
            <p>配置任务参数后点击"生成周报"或"快速扫描"开始</p>
          </div>

          <!-- 执行中 -->
          <div v-else-if="isRunning" class="running-state">
            <!-- 当前执行阶段 -->
            <div class="phase-status">
              <el-icon :class="{ spinning: currentPhase === 'thinking' || currentPhase === 'tool_call' }" :size="24">
                <component :is="getPhaseIcon()" />
              </el-icon>
              <span class="phase-text">{{ getPhaseText() }}</span>
            </div>

            <!-- 工具调用进度 -->
            <div v-if="toolCallHistory.length > 0" class="tool-progress">
              <div class="tool-progress-header">
                <el-icon><Tools /></el-icon>
                <span>已调用 {{ toolCallHistory.length }} 个工具</span>
              </div>
              <div class="tool-tags">
                <el-tooltip
                  v-for="(call, index) in toolCallHistory"
                  :key="index"
                  :content="call.error || '调用成功'"
                  placement="top"
                >
                  <el-tag
                    :type="call.error ? 'danger' : 'success'"
                    size="small"
                    class="tool-tag"
                  >
                    {{ call.name }}
                  </el-tag>
                </el-tooltip>
              </div>
            </div>

            <div v-if="thinkingContent" class="thinking-preview">
              <pre>{{ thinkingContent.slice(0, 500) }}{{ thinkingContent.length > 500 ? '...' : '' }}</pre>
            </div>
          </div>

          <!-- 结果展示 -->
          <div v-else-if="finalResult" class="result-content">
            <div v-if="finalResult.success" class="success-result">
              <div class="result-meta">
                <span>迭代次数: {{ finalResult.iterations }}</span>
                <span>使用工具: {{ finalResult.toolsUsed.join(', ') || '无' }}</span>
              </div>
              <div class="markdown-content" v-html="renderMarkdown(finalResult.output)"></div>
            </div>
            <div v-else class="error-result">
              <el-alert type="error" :title="finalResult.error" show-icon :closable="false" />
            </div>
          </div>
        </el-card>
      </div>
    </div>

    <!-- 类目浏览器对话框 -->
    <el-dialog
      v-model="showCategoryBrowser"
      title="类目浏览器"
      width="700px"
      :close-on-click-modal="false"
    >
      <div class="category-browser">
        <!-- 面包屑导航 -->
        <div v-if="categoryLevels.length > 0" class="breadcrumb">
          <span
            v-for="(level, index) in categoryLevels"
            :key="index"
            class="breadcrumb-item"
            :class="{ active: index === categoryLevels.length - 1 }"
            @click="goBackLevel(index)"
          >
            {{ level.name }}
            <el-icon v-if="index < categoryLevels.length - 1"><ArrowRight /></el-icon>
          </span>
        </div>

        <!-- 一级类目选择 -->
        <div v-if="categoryLevels.length === 0" class="root-categories">
          <div class="section-title">选择一级类目</div>
          <div class="category-grid">
            <div
              v-for="cat in rootCategories"
              :key="cat.id"
              class="category-item"
              @click="selectRootCategory(cat)"
            >
              <el-icon><FolderOpened /></el-icon>
              <span>{{ cat.name }}</span>
            </div>
          </div>
        </div>

        <!-- 子类目列表 -->
        <div v-else class="subcategories">
          <div class="section-title">
            <span>子类目列表</span>
            <el-button
              v-if="categoryLevels[categoryLevels.length - 1].subcategories.length === 0"
              :icon="Search"
              :loading="isDiscovering"
              size="small"
              @click="discoverSubcategories(categoryLevels.length - 1)"
            >
              发现子类目
            </el-button>
          </div>

          <!-- 加载中 -->
          <div v-if="isDiscovering" class="loading-state">
            <el-icon class="spinning"><Loading /></el-icon>
            <span>正在发现子类目...</span>
          </div>

          <!-- 子类目网格 -->
          <div
            v-else-if="categoryLevels[categoryLevels.length - 1].subcategories.length > 0"
            class="category-grid"
          >
            <div
              v-for="sub in categoryLevels[categoryLevels.length - 1].subcategories"
              :key="sub.category_id"
              class="category-item"
              @click="selectSubcategory(categoryLevels.length - 1, sub)"
            >
              <el-icon><FolderOpened /></el-icon>
              <span>{{ sub.name }}</span>
            </div>
          </div>

          <!-- 无子类目 -->
          <div v-else class="empty-state">
            <p>该类目下没有更多子类目</p>
            <p class="hint">您可以直接使用当前类目进行分析</p>
          </div>
        </div>

        <!-- 当前选中信息 -->
        <div v-if="categoryLevels.length > 0" class="selection-info">
          <div class="info-label">当前选中:</div>
          <div class="info-path">{{ selectedCategoryPath }}</div>
          <div class="info-id">ID: {{ selectedCategoryId }}</div>
        </div>
      </div>

      <template #footer>
        <el-button @click="showCategoryBrowser = false">取消</el-button>
        <el-button
          type="primary"
          :disabled="categoryLevels.length === 0"
          @click="confirmCategorySelection"
        >
          确认选择
        </el-button>
      </template>
    </el-dialog>

    <!-- 监控任务列表弹窗 -->
    <el-dialog
      v-model="showTaskListDialog"
      title="监控任务"
      width="500px"
    >
      <div class="task-list-dialog">
        <div v-if="loadingTasks" class="loading-state">
          <el-icon class="spinning"><Loading /></el-icon>
          <span>加载中...</span>
        </div>

        <div v-else-if="monitoringTasks.length === 0" class="empty-task-state">
          <el-icon :size="40"><Clock /></el-icon>
          <p>暂无监控任务</p>
          <p class="hint">创建任务后可自动定时执行市场调研</p>
        </div>

        <div v-else class="task-list">
          <div
            v-for="task in monitoringTasks"
            :key="task.id"
            class="task-item"
            :class="{ disabled: !task.is_enabled }"
          >
            <div class="task-info">
              <div class="task-name">{{ task.name }}</div>
              <div class="task-meta">
                <el-tag size="small" type="info">{{ getCountryLabel(task.marketplace) }}</el-tag>
                <span class="task-schedule">{{ formatSchedule(task) }}</span>
              </div>
            </div>
            <div class="task-actions">
              <el-tooltip content="立即运行" placement="top">
                <el-button
                  :icon="VideoPlay"
                  size="small"
                  circle
                  :disabled="isRunning"
                  @click="runTaskNow(task); showTaskListDialog = false;"
                />
              </el-tooltip>
              <el-tooltip content="编辑" placement="top">
                <el-button
                  :icon="Edit"
                  size="small"
                  circle
                  @click="openEditTaskDialog(task)"
                />
              </el-tooltip>
              <el-popconfirm
                title="确定要删除此任务吗？"
                @confirm="deleteTask(task)"
              >
                <template #reference>
                  <el-button
                    :icon="Delete"
                    size="small"
                    circle
                    type="danger"
                  />
                </template>
              </el-popconfirm>
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <el-button @click="showTaskListDialog = false">关闭</el-button>
        <el-button type="primary" :icon="Plus" @click="openCreateTaskDialog">
          添加任务
        </el-button>
      </template>
    </el-dialog>

    <!-- 监控任务编辑对话框 -->
    <MarketResearchTaskDialog
      v-model="showTaskDialog"
      :task="editingTask"
      @success="loadMonitoringTasks"
    />
  </div>
</template>


<style scoped>
.agent-tab {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 16px;
  box-sizing: border-box;
}

.country-flag {
  display: inline-flex;
  width: 20px;
  height: 14px;
  border-radius: 2px;
  overflow: hidden;
  flex-shrink: 0;
}

.country-flag :deep(svg) {
  width: 100%;
  height: 100%;
}

.tab-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-left .title {
  font-size: 18px;
  font-weight: 600;
}

.main-content {
  flex: 1;
  display: flex;
  gap: 16px;
  min-height: 0;
}

.config-panel {
  width: 320px;
  flex-shrink: 0;
}

.config-panel > .el-card {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.config-panel > .el-card :deep(.el-card__body) {
  flex: 1;
}

.result-panel {
  flex: 1;
  min-width: 0;
}

.result-card {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.result-card :deep(.el-card__body) {
  flex: 1;
  overflow: auto;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.result-time {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.action-buttons {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

/* 工具调用进度 - 显示在右侧结果面板 */
.tool-progress {
  margin: 16px 0;
  padding: 12px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
  text-align: left;
}

.tool-progress-header {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
  margin-bottom: 8px;
}

.tool-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.tool-tag {
  cursor: default;
  font-size: 11px;
}

.empty-state,
.running-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px;
  color: var(--el-text-color-secondary);
  text-align: center;
}

.empty-state p,
.running-state p {
  margin-top: 12px;
}

/* 执行阶段状态 */
.phase-status {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 20px;
  background: var(--el-color-primary-light-9);
  border: 1px solid var(--el-color-primary-light-5);
  border-radius: 8px;
  margin-bottom: 16px;
}

.phase-status .phase-text {
  font-size: 15px;
  font-weight: 500;
  color: var(--el-color-primary);
}

.thinking-preview {
  margin-top: 16px;
  padding: 12px;
  background: var(--el-fill-color-light);
  border-radius: 4px;
  max-height: 200px;
  overflow: auto;
  width: 100%;
}

.thinking-preview pre {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
  font-size: 12px;
}

.result-content {
  padding: 8px;
}

.result-meta {
  display: flex;
  gap: 16px;
  margin-bottom: 16px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.markdown-content {
  line-height: 1.6;
}

.markdown-content :deep(h2) {
  font-size: 18px;
  margin: 16px 0 8px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  padding-bottom: 4px;
}

.markdown-content :deep(h3) {
  font-size: 16px;
  margin: 12px 0 6px;
}

.markdown-content :deep(h4) {
  font-size: 14px;
  margin: 10px 0 4px;
}

.markdown-content :deep(ul) {
  padding-left: 20px;
  margin: 8px 0;
}

.markdown-content :deep(li) {
  margin: 4px 0;
}

.markdown-content :deep(pre) {
  background: var(--el-fill-color-light);
  padding: 12px;
  border-radius: 4px;
  overflow-x: auto;
}

.markdown-content :deep(code) {
  font-family: monospace;
  font-size: 13px;
}

.error-result {
  padding: 16px;
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.category-hint {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 6px;
  line-height: 1.4;
}

.category-hint strong {
  color: var(--el-color-primary);
}

/* 已选类目显示 */
.selected-category {
  padding: 10px 12px;
  background: var(--el-fill-color-light);
  border-radius: 6px;
  border: 1px solid var(--el-border-color-lighter);
}

.selected-category .category-path {
  font-size: 13px;
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 4px;
}

.selected-category .category-id {
  font-size: 11px;
  color: var(--el-text-color-secondary);
  font-family: monospace;
}

/* 类目浏览器 */
.category-browser {
  min-height: 400px;
}

.breadcrumb {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px;
  padding: 12px;
  background: var(--el-fill-color-light);
  border-radius: 6px;
  margin-bottom: 16px;
}

.breadcrumb-item {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  color: var(--el-color-primary);
  cursor: pointer;
}

.breadcrumb-item:hover {
  text-decoration: underline;
}

.breadcrumb-item.active {
  color: var(--el-text-color-primary);
  font-weight: 500;
  cursor: default;
}

.breadcrumb-item.active:hover {
  text-decoration: none;
}

.section-title {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-secondary);
  margin-bottom: 12px;
}

.category-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
}

.category-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  background: var(--el-fill-color-lighter);
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
}

.category-item:hover {
  background: var(--el-color-primary-light-9);
  border-color: var(--el-color-primary-light-5);
}

.category-item span {
  font-size: 13px;
  line-height: 1.3;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px;
  color: var(--el-text-color-secondary);
  gap: 12px;
}

.category-browser .empty-state {
  padding: 32px;
}

.category-browser .empty-state .hint {
  font-size: 12px;
  margin-top: 4px;
}

.selection-info {
  margin-top: 16px;
  padding: 12px;
  background: var(--el-color-success-light-9);
  border: 1px solid var(--el-color-success-light-5);
  border-radius: 6px;
}

.selection-info .info-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 4px;
}

.selection-info .info-path {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.selection-info .info-id {
  font-size: 11px;
  color: var(--el-text-color-secondary);
  font-family: monospace;
  margin-top: 4px;
}

/* 监控任务按钮 */
.task-badge {
  margin-left: 8px;
}

/* 监控任务弹窗 */
.task-list-dialog {
  min-height: 150px;
}

.empty-task-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 32px 0;
  color: var(--el-text-color-secondary);
}

.empty-task-state p {
  margin: 4px 0;
}

.empty-task-state .hint {
  font-size: 12px;
}

.task-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.task-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  background: var(--el-fill-color-light);
  border-radius: 6px;
  transition: all 0.2s;
}

.task-item:hover {
  background: var(--el-fill-color);
}

.task-item.disabled {
  opacity: 0.5;
}

.task-info {
  flex: 1;
  min-width: 0;
}

.task-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.task-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.task-schedule {
  font-size: 11px;
}

.task-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}
</style>

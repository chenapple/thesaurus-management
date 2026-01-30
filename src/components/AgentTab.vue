<script setup lang="ts">
import { ref, computed, onUnmounted, watch, onMounted } from 'vue';
import {
  Cpu,
  DocumentCopy,
  CircleCheck,
  Warning,
  Loading,
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
  Close,
  DataBoard,
  Aim,
} from '@element-plus/icons-vue';
import MarketResearchTaskDialog from './MarketResearchTaskDialog.vue';
import MarketResearchDashboard from './MarketResearchDashboard.vue';
import CompetitorTaskDialog from './CompetitorTaskDialog.vue';
import CompetitorDashboard from './CompetitorDashboard.vue';
import type { MarketResearchTask } from './MarketResearchTaskDialog.vue';
import type { CompetitorTask } from '../types';
import { ElMessage } from 'element-plus';
import { invoke } from '@tauri-apps/api/core';
import {
  createMarketResearchAgent,
  createWeeklyReportTask,
  createQuickScanTask,
  createCompetitorIntelligenceAgent,
  createCompetitorMonitorTask,
  createQuickCompetitorAnalysisTask,
} from '../agent';
import type { AgentEvent, TaskResult } from '../agent';
import { renderSimpleMarkdown } from '../utils/sanitize';
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

// ==================== Agent 类型与视图模式 ====================
type AgentType = 'market-research' | 'competitor-intelligence';
type ViewMode = 'dashboard' | 'execute';

const agentType = ref<AgentType>('market-research');
const viewMode = ref<ViewMode>('dashboard');
const dashboardRef = ref<InstanceType<typeof MarketResearchDashboard> | null>(null);
const competitorDashboardRef = ref<InstanceType<typeof CompetitorDashboard> | null>(null);

// Agent 类型配置
const AGENT_TYPES = [
  { value: 'market-research', label: '市场调研' },
  { value: 'competitor-intelligence', label: '竞品情报' },
] as const;

// ==================== 配置状态 ====================

// 表单数据
const selectedMarketplace = ref('US');
const customCategoryId = ref('');
const customCategoryName = ref('');
const useCustomCategory = ref(false);
const selectedProvider = ref<AIProvider>('deepseek');

// ==================== 最近使用记录 ====================
interface RecentSelection {
  marketplace: string;
  categoryId: string;
  categoryName: string;
  timestamp: number;
}

const RECENT_SELECTIONS_KEY = 'agent_recent_selections';
const MAX_RECENT_SELECTIONS = 8;

const recentSelections = ref<RecentSelection[]>([]);

// 加载最近使用记录
function loadRecentSelections() {
  try {
    const saved = localStorage.getItem(RECENT_SELECTIONS_KEY);
    if (saved) {
      recentSelections.value = JSON.parse(saved);
    }
  } catch (e) {
    console.error('加载最近使用记录失败:', e);
  }
}

// 保存最近使用记录
function saveRecentSelection(marketplace: string, categoryId: string, categoryName: string) {
  if (!categoryId) return;

  // 移除重复项
  const filtered = recentSelections.value.filter(
    r => !(r.marketplace === marketplace && r.categoryId === categoryId)
  );

  // 添加到开头
  filtered.unshift({
    marketplace,
    categoryId,
    categoryName: categoryName || categoryId,
    timestamp: Date.now(),
  });

  // 限制数量
  recentSelections.value = filtered.slice(0, MAX_RECENT_SELECTIONS);

  // 保存到 localStorage
  try {
    localStorage.setItem(RECENT_SELECTIONS_KEY, JSON.stringify(recentSelections.value));
  } catch (e) {
    console.error('保存最近使用记录失败:', e);
  }
}

// 快速选择
function quickSelect(selection: RecentSelection) {
  selectedMarketplace.value = selection.marketplace;
  customCategoryId.value = selection.categoryId;
  customCategoryName.value = selection.categoryName;
}

// 删除最近使用记录
function removeRecentSelection(index: number) {
  recentSelections.value.splice(index, 1);
  try {
    localStorage.setItem(RECENT_SELECTIONS_KEY, JSON.stringify(recentSelections.value));
  } catch (e) {
    console.error('保存失败:', e);
  }
}

// 获取站点 flag
function getMarketplaceFlag(marketplace: string): string {
  const country = COUNTRY_OPTIONS.find(c => c.value === marketplace);
  return country?.flag || '';
}

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

// 从仪表盘运行任务
function handleDashboardRunTask(task: MarketResearchTask) {
  viewMode.value = 'execute';
  runTaskNow(task);
}

// 从仪表盘编辑任务
function handleDashboardEditTask(task: MarketResearchTask) {
  openEditTaskDialog(task);
}

// 任务对话框成功回调（刷新数据）
async function handleTaskDialogSuccess() {
  await loadMonitoringTasks();
  // 同时刷新仪表盘数据
  if (dashboardRef.value) {
    dashboardRef.value.refresh();
  }
}

// ==================== 竞品情报任务管理 ====================

const showCompetitorTaskDialog = ref(false);
const showCompetitorTaskListDialog = ref(false);  // 任务列表弹窗
const editingCompetitorTask = ref<CompetitorTask | null>(null);
const runningCompetitorTask = ref<CompetitorTask | null>(null);
const competitorResult = ref<TaskResult | null>(null);
const competitorRunId = ref<number | null>(null);
const competitorTasks = ref<CompetitorTask[]>([]);
const loadingCompetitorTasks = ref(false);

// 竞品情报配置状态（用于快速执行）
const competitorMarketplace = ref('US');
const competitorMyAsin = ref('');
const competitorProvider = ref<AIProvider>('deepseek');
const competitorModel = ref('');

// 竞品 ASIN 列表
interface CompetitorAsinItem {
  asin: string;
  title?: string;
}
const competitorAsins = ref<CompetitorAsinItem[]>([]);
const newCompetitorAsin = ref('');

// 添加竞品 ASIN
function addCompetitorAsin() {
  const input = newCompetitorAsin.value.trim();
  if (!input) return;

  // 支持多种分隔符
  const asins = input
    .split(/[\n,\s]+/)
    .map(s => s.trim().toUpperCase())
    .filter(s => s && /^[A-Z0-9]{10}$/.test(s));

  if (asins.length === 0) {
    ElMessage.warning('请输入有效的 ASIN（10位字母数字）');
    return;
  }

  for (const asin of asins) {
    if (!competitorAsins.value.find(a => a.asin === asin)) {
      competitorAsins.value.push({ asin });
    }
  }

  newCompetitorAsin.value = '';
}

// 移除竞品 ASIN
function removeCompetitorAsin(index: number) {
  competitorAsins.value.splice(index, 1);
}

// 竞品情报可用模型
const competitorAvailableModels = computed(() => {
  const config = AI_PROVIDERS[competitorProvider.value];
  return config?.models || [];
});

// 监听 provider 变化，重置 model
watch(competitorProvider, (newProvider) => {
  const config = AI_PROVIDERS[newProvider];
  competitorModel.value = config?.defaultModel || '';
});

// 竞品情报是否可运行（需要我的 ASIN 或竞品 ASIN）
const canRunCompetitor = computed(() => {
  return competitorMyAsin.value.trim().length > 0 || competitorAsins.value.length > 0;
});

// 快速运行竞品监控
async function runCompetitorQuick() {
  if (isRunning.value) {
    ElMessage.warning('有任务正在执行中');
    return;
  }

  // 检查是否有输入
  const hasMyAsin = competitorMyAsin.value.trim().length > 0;
  const hasCompetitorAsins = competitorAsins.value.length > 0;

  if (!hasMyAsin && !hasCompetitorAsins) {
    ElMessage.warning('请输入我的 ASIN 或添加竞品 ASIN');
    return;
  }

  // 重置状态
  isRunning.value = true;
  currentPhase.value = 'thinking';
  currentIteration.value = 0;
  thinkingContent.value = '';
  toolCallHistory.value = [];
  competitorResult.value = null;
  runningCompetitorTask.value = null;

  // 创建 AbortController
  abortController.value = new AbortController();

  try {
    // 创建 Agent
    const provider = competitorProvider.value;
    const model = competitorModel.value || AI_PROVIDERS[provider].defaultModel;
    const agent = createCompetitorIntelligenceAgent(provider, model);

    // 监听事件
    const unsubscribe = agent.onEvent((event: AgentEvent) => {
      handleAgentEvent(event);
    });

    // 创建快速分析任务（直接传入 ASIN 列表）
    const agentTask = createQuickCompetitorAnalysisTask(
      competitorMarketplace.value,
      hasMyAsin ? competitorMyAsin.value.trim() : undefined,
      competitorAsins.value.map(a => a.asin)
    );

    // 执行任务
    const result = await agent.execute(agentTask, abortController.value.signal);
    competitorResult.value = result;
    finalResult.value = result;
    currentPhase.value = result.success ? 'completed' : 'error';

    // 取消订阅
    unsubscribe();

    if (result.success) {
      ElMessage.success('竞品分析完成');
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
    competitorResult.value = errorResult;
    finalResult.value = errorResult;
    ElMessage.error('Agent 执行出错');
  } finally {
    isRunning.value = false;
    abortController.value = null;
  }
}

// 从仪表盘创建任务
function handleCompetitorCreateTask() {
  editingCompetitorTask.value = null;
  showCompetitorTaskDialog.value = true;
}

// 从仪表盘编辑任务
function handleCompetitorEditTask(task: CompetitorTask) {
  editingCompetitorTask.value = task;
  showCompetitorTaskDialog.value = true;
}

// 从仪表盘运行任务
async function handleCompetitorRunTask(task: CompetitorTask) {
  if (isRunning.value) {
    ElMessage.warning('有任务正在执行中');
    return;
  }

  // 切换到执行视图
  viewMode.value = 'execute';
  runningCompetitorTask.value = task;

  // 重置状态
  isRunning.value = true;
  currentPhase.value = 'thinking';
  currentIteration.value = 0;
  thinkingContent.value = '';
  toolCallHistory.value = [];
  competitorResult.value = null;

  // 创建 AbortController
  abortController.value = new AbortController();

  // 创建执行记录
  try {
    competitorRunId.value = await invoke<number>('create_competitor_run', {
      taskId: task.id,
    });
  } catch (error) {
    console.error('创建执行记录失败:', error);
  }

  try {
    // 创建 Agent
    const provider = (task.ai_provider || 'deepseek') as AIProvider;
    const model = task.ai_model || AI_PROVIDERS[provider].defaultModel;
    const agent = createCompetitorIntelligenceAgent(provider, model);

    // 监听事件
    const unsubscribe = agent.onEvent((event: AgentEvent) => {
      handleAgentEvent(event);
    });

    // 创建任务
    const agentTask = createCompetitorMonitorTask(
      task.id,
      task.name,
      task.marketplace,
      task.my_asin
    );

    // 执行任务
    const result = await agent.execute(agentTask, abortController.value.signal);
    competitorResult.value = result;
    finalResult.value = result;
    currentPhase.value = result.success ? 'completed' : 'error';

    // 取消订阅
    unsubscribe();

    // 更新执行记录
    if (competitorRunId.value) {
      if (result.success) {
        const summary = result.output.slice(0, 200).replace(/[#\n]/g, ' ').trim();
        await invoke('update_competitor_run', {
          runId: competitorRunId.value,
          status: 'completed',
          reportSummary: summary,
          reportContent: result.output,
        });
      } else {
        await invoke('fail_competitor_run', {
          runId: competitorRunId.value,
          errorMessage: result.error || '未知错误',
        });
      }
    }

    if (result.success) {
      ElMessage.success('竞品监控任务完成');
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
    competitorResult.value = errorResult;
    finalResult.value = errorResult;

    // 更新执行记录
    if (competitorRunId.value) {
      await invoke('fail_competitor_run', {
        runId: competitorRunId.value,
        errorMessage: errorResult.error || '未知错误',
      });
    }

    ElMessage.error('Agent 执行出错');
  } finally {
    isRunning.value = false;
    abortController.value = null;
  }
}

// 加载竞品监控任务列表
async function loadCompetitorTasks() {
  loadingCompetitorTasks.value = true;
  try {
    const tasks = await invoke<CompetitorTask[]>('get_competitor_tasks');
    competitorTasks.value = tasks;
  } catch (error) {
    console.error('加载竞品监控任务失败:', error);
  } finally {
    loadingCompetitorTasks.value = false;
  }
}

// 立即运行竞品任务
async function runCompetitorTaskNow(task: CompetitorTask) {
  showCompetitorTaskListDialog.value = false;
  await handleCompetitorRunTask(task);
}

// 编辑竞品任务（从列表弹窗）
function openEditCompetitorTaskDialog(task: CompetitorTask) {
  editingCompetitorTask.value = task;
  showCompetitorTaskDialog.value = true;
}

// 删除竞品任务
async function deleteCompetitorTask(task: CompetitorTask) {
  try {
    await invoke('delete_competitor_task', { id: task.id });
    ElMessage.success('任务已删除');
    await loadCompetitorTasks();
  } catch (error) {
    ElMessage.error(`删除失败: ${error}`);
  }
}

// 格式化竞品任务计划
function formatCompetitorSchedule(task: CompetitorTask): string {
  if (task.schedule_type === 'daily') {
    return `每天 ${task.schedule_time}`;
  }
  return `每周 ${task.schedule_time}`;
}

// 竞品任务对话框成功回调
function handleCompetitorTaskDialogSuccess() {
  // 刷新仪表盘数据
  if (competitorDashboardRef.value) {
    competitorDashboardRef.value.refresh();
  }
  // 刷新任务列表
  loadCompetitorTasks();
}

onMounted(() => {
  loadMonitoringTasks();
  loadCompetitorTasks(); // 加载竞品监控任务
  loadLastResult(); // 加载上次的执行结果
  loadRecentSelections(); // 加载最近使用记录
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
      // 保存到最近使用记录
      saveRecentSelection(selectedMarketplace.value, customCategoryId.value, customCategoryName.value);
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
  // 优先从 generate_weekly_report 工具调用结果中提取 HTML 报告内容
  let reportContent = result.output;
  const reportToolCall = toolCallHistory.value.find(
    call => call.name === 'generate_weekly_report' && call.result?.report_content
  );
  if (reportToolCall?.result?.report_content) {
    reportContent = reportToolCall.result.report_content;
    // 同时更新 finalResult 的 output，以便页面正确显示
    if (finalResult.value) {
      finalResult.value.output = reportContent;
    }
    console.log('使用工具调用返回的 HTML 报告内容');
  }

  // 1. 保存到 localStorage（用于快速恢复）
  const timestamp = new Date().toISOString();
  const resultData = {
    result: { ...result, output: reportContent },
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
        // 提取摘要（取前200个字符，去除HTML标签）
        const plainText = reportContent.replace(/<[^>]*>/g, ' ').replace(/\s+/g, ' ');
        const summary = plainText.slice(0, 200).trim();

        // 检查报告内容是否为空
        if (!reportContent || reportContent.trim().length === 0) {
          console.warn('警告: 任务成功但报告内容为空', { runId, toolsUsed: result.toolsUsed });
        }

        await invoke('update_research_run', {
          runId,
          status: 'completed',
          summary: summary || '报告生成完成',
          content: reportContent,
          snapshotId: null,
        });
        console.log('市场调研报告已保存到数据库', { runId, contentLength: reportContent.length });
      } else {
        await invoke('fail_research_run', {
          runId,
          errorMessage: result.error || '未知错误',
        });
      }
    } catch (error) {
      console.error('更新执行记录失败:', error, { runId, result });
    }
  } else if (runningTaskId.value) {
    // 有任务 ID 但没有 runId，说明创建执行记录失败
    console.warn('警告: 任务执行完成但无法保存，runId 为空');
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

// Markdown 渲染 - 使用安全的 sanitize 工具
function renderMarkdown(text: string): string {
  return renderSimpleMarkdown(text);
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

        <!-- Agent 类型选择 -->
        <el-radio-group v-model="agentType" size="small" class="agent-type-selector">
          <el-radio-button
            v-for="agent in AGENT_TYPES"
            :key="agent.value"
            :value="agent.value"
          >
            {{ agent.label }}
          </el-radio-button>
        </el-radio-group>
      </div>
      <div class="header-right">
        <!-- 视图模式选择 -->
        <el-radio-group v-model="viewMode" size="small">
          <el-radio-button value="dashboard">
            <el-icon><DataBoard /></el-icon>
            <span>仪表盘</span>
          </el-radio-button>
          <el-radio-button value="execute">
            <el-icon><Cpu /></el-icon>
            <span>执行</span>
          </el-radio-button>
        </el-radio-group>
      </div>
    </div>

    <!-- ==================== 市场调研 Agent ==================== -->
    <!-- 市场调研 - 仪表盘视图 -->
    <MarketResearchDashboard
      v-if="agentType === 'market-research' && viewMode === 'dashboard'"
      ref="dashboardRef"
      @run-task="handleDashboardRunTask"
      @edit-task="handleDashboardEditTask"
    />

    <!-- 市场调研 - 执行任务视图 -->
    <div v-if="agentType === 'market-research' && viewMode === 'execute'" class="main-content">
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
            <!-- 最近使用 -->
            <el-form-item v-if="recentSelections.length > 0" label="快速选择">
              <div class="recent-selections">
                <div
                  v-for="(item, index) in recentSelections"
                  :key="`${item.marketplace}-${item.categoryId}`"
                  class="recent-item"
                  :class="{ active: selectedMarketplace === item.marketplace && customCategoryId === item.categoryId }"
                  @click="quickSelect(item)"
                >
                  <span class="recent-flag" v-html="getMarketplaceFlag(item.marketplace)"></span>
                  <span class="recent-name">{{ item.categoryName }}</span>
                  <el-icon class="recent-remove" @click.stop="removeRecentSelection(index)"><Close /></el-icon>
                </div>
              </div>
            </el-form-item>

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

    <!-- ==================== 竞品情报 Agent ==================== -->
    <!-- 竞品情报 - 仪表盘视图 -->
    <CompetitorDashboard
      v-if="agentType === 'competitor-intelligence' && viewMode === 'dashboard'"
      ref="competitorDashboardRef"
      @edit-task="handleCompetitorEditTask"
      @run-task="handleCompetitorRunTask"
    />

    <!-- 竞品情报 - 执行视图 -->
    <div v-if="agentType === 'competitor-intelligence' && viewMode === 'execute'" class="main-content">
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
                @click="showCompetitorTaskListDialog = true"
              >
                监控任务
                <el-badge v-if="competitorTasks.length > 0" :value="competitorTasks.length" class="task-badge" />
              </el-button>
            </div>
          </template>

          <!-- 当前任务信息（从仪表盘运行时显示） -->
          <div v-if="runningCompetitorTask" class="competitor-task-info">
            <div class="info-item">
              <span class="label">任务名称</span>
              <span class="value">{{ runningCompetitorTask.name }}</span>
            </div>
            <div class="info-item">
              <span class="label">站点</span>
              <span class="value">{{ getCountryLabel(runningCompetitorTask.marketplace) }}</span>
            </div>
            <div class="info-item" v-if="runningCompetitorTask.my_asin">
              <span class="label">我的 ASIN</span>
              <span class="value">{{ runningCompetitorTask.my_asin }}</span>
            </div>
            <div class="info-item">
              <span class="label">AI 服务</span>
              <span class="value">{{ runningCompetitorTask.ai_provider }}</span>
            </div>
            <!-- 停止按钮 -->
            <div v-if="isRunning" class="action-buttons" style="margin-top: 16px;">
              <el-button type="danger" @click="stopAgent" style="width: 100%;">
                停止执行
              </el-button>
            </div>
          </div>

          <!-- 快速配置表单 -->
          <el-form v-else label-position="top" size="default">
            <!-- AI Provider -->
            <el-form-item label="AI 服务">
              <el-select v-model="competitorProvider" :disabled="isRunning" style="width: 100%">
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
              <el-select v-model="competitorModel" :disabled="isRunning" style="width: 100%">
                <el-option
                  v-for="model in competitorAvailableModels"
                  :key="model"
                  :value="model"
                  :label="model"
                />
              </el-select>
            </el-form-item>

            <!-- 站点选择 -->
            <el-form-item label="Amazon 站点">
              <el-select v-model="competitorMarketplace" :disabled="isRunning" style="width: 100%">
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

            <!-- 我的 ASIN -->
            <el-form-item label="我的 ASIN">
              <el-input
                v-model="competitorMyAsin"
                :disabled="isRunning"
                placeholder="可选，用于对比分析"
                clearable
              />
              <div class="category-hint">
                填写您的产品 ASIN，用于与竞品进行对比分析
              </div>
            </el-form-item>

            <!-- 竞品 ASIN -->
            <el-form-item label="竞品 ASIN">
              <el-input
                v-model="newCompetitorAsin"
                :disabled="isRunning"
                placeholder="输入竞品 ASIN，按回车添加"
                @keyup.enter="addCompetitorAsin"
              >
                <template #append>
                  <el-button :disabled="isRunning" @click="addCompetitorAsin">添加</el-button>
                </template>
              </el-input>
              <div class="category-hint">支持批量粘贴（每行一个或逗号分隔）</div>

              <!-- 已添加的竞品列表 -->
              <div v-if="competitorAsins.length > 0" class="competitor-asin-list">
                <el-tag
                  v-for="(item, index) in competitorAsins"
                  :key="index"
                  closable
                  type="info"
                  size="small"
                  @close="removeCompetitorAsin(index)"
                >
                  {{ item.asin }}
                </el-tag>
              </div>
            </el-form-item>

            <!-- 操作按钮 -->
            <el-form-item>
              <div class="action-buttons">
                <el-button
                  type="primary"
                  :icon="Aim"
                  :loading="isRunning"
                  :disabled="!canRunCompetitor"
                  style="flex: 1;"
                  @click="runCompetitorQuick"
                >
                  开始监控
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
                <el-tag v-if="competitorResult" :type="competitorResult.success ? 'success' : 'danger'" size="small">
                  {{ competitorResult.success ? '成功' : '失败' }}
                </el-tag>
              </div>
            </div>
          </template>

          <!-- 空状态 -->
          <div v-if="!competitorResult && currentPhase === 'idle'" class="empty-state">
            <el-icon :size="48"><Aim /></el-icon>
            <p>输入 ASIN 后点击"开始监控"分析竞品情况</p>
          </div>

          <!-- 执行中 -->
          <div v-else-if="isRunning" class="running-state">
            <div class="phase-status">
              <el-icon :class="{ spinning: currentPhase === 'thinking' || currentPhase === 'tool_call' }" :size="24">
                <component :is="getPhaseIcon()" />
              </el-icon>
              <span class="phase-text">{{ getPhaseText() }}</span>
            </div>

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
          <div v-else-if="competitorResult" class="result-content">
            <div v-if="competitorResult.success" class="success-result">
              <div class="result-meta">
                <span>迭代次数: {{ competitorResult.iterations }}</span>
                <span>使用工具: {{ competitorResult.toolsUsed.join(', ') || '无' }}</span>
              </div>
              <div class="markdown-content" v-html="renderMarkdown(competitorResult.output)"></div>
            </div>
            <div v-else class="error-result">
              <el-alert type="error" :title="competitorResult.error" show-icon :closable="false" />
            </div>
          </div>
        </el-card>
      </div>
    </div>

    <!-- 竞品任务编辑对话框 -->
    <CompetitorTaskDialog
      v-model="showCompetitorTaskDialog"
      :task="editingCompetitorTask"
      @success="handleCompetitorTaskDialogSuccess"
    />

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
      @success="handleTaskDialogSuccess"
    />

    <!-- 竞品监控任务列表弹窗 -->
    <el-dialog
      v-model="showCompetitorTaskListDialog"
      title="竞品监控任务"
      width="500px"
    >
      <div class="task-list-dialog">
        <div v-if="loadingCompetitorTasks" class="loading-state">
          <el-icon class="spinning"><Loading /></el-icon>
          <span>加载中...</span>
        </div>

        <div v-else-if="competitorTasks.length === 0" class="empty-task-state">
          <el-icon :size="40"><Aim /></el-icon>
          <p>暂无监控任务</p>
          <p class="hint">创建任务后可自动定时执行竞品监控</p>
        </div>

        <div v-else class="task-list">
          <div
            v-for="task in competitorTasks"
            :key="task.id"
            class="task-item"
            :class="{ disabled: !task.is_enabled }"
          >
            <div class="task-info">
              <div class="task-name">{{ task.name }}</div>
              <div class="task-meta">
                <el-tag size="small" type="info">{{ getCountryLabel(task.marketplace) }}</el-tag>
                <span class="task-schedule">{{ formatCompetitorSchedule(task) }}</span>
              </div>
            </div>
            <div class="task-actions">
              <el-tooltip content="立即运行" placement="top">
                <el-button
                  :icon="VideoPlay"
                  size="small"
                  circle
                  :disabled="isRunning"
                  @click="runCompetitorTaskNow(task)"
                />
              </el-tooltip>
              <el-tooltip content="编辑" placement="top">
                <el-button
                  :icon="Edit"
                  size="small"
                  circle
                  @click="openEditCompetitorTaskDialog(task)"
                />
              </el-tooltip>
              <el-popconfirm
                title="确定要删除此任务吗？"
                @confirm="deleteCompetitorTask(task)"
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
        <el-button @click="showCompetitorTaskListDialog = false">关闭</el-button>
        <el-button type="primary" :icon="Plus" @click="handleCompetitorCreateTask(); showCompetitorTaskListDialog = false">
          添加任务
        </el-button>
      </template>
    </el-dialog>
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
  gap: 12px;
}

.header-left .title {
  font-size: 18px;
  font-weight: 600;
}

.header-right {
  display: flex;
  align-items: center;
}

.header-right .el-radio-button :deep(.el-radio-button__inner) {
  display: flex;
  align-items: center;
  gap: 6px;
}

.agent-type-selector {
  margin-left: 8px;
}

.agent-type-selector :deep(.el-radio-button__inner) {
  display: flex;
  align-items: center;
  gap: 6px;
  font-weight: 500;
}

.agent-type-selector :deep(.el-radio-button.is-active .el-radio-button__inner) {
  background: var(--el-color-primary);
  border-color: var(--el-color-primary);
}

/* Coming Soon Placeholder */
.coming-soon-container {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 400px;
}

.coming-soon-card {
  text-align: center;
  padding: 48px 60px;
  background: var(--glass-bg);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border-radius: 20px;
  box-shadow: var(--glass-shadow);
  border: 1px solid var(--glass-border);
  max-width: 480px;
}

.coming-soon-icon {
  color: var(--el-color-primary);
  margin-bottom: 16px;
}

.coming-soon-card h2 {
  margin: 0 0 12px;
  font-size: 24px;
  font-weight: 700;
  color: var(--el-text-color-primary);
}

.coming-soon-card h3 {
  margin: 0 0 8px;
  font-size: 18px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.coming-soon-desc {
  color: var(--el-text-color-secondary);
  font-size: 14px;
  margin-bottom: 24px;
  line-height: 1.6;
}

.coming-soon-features {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  margin-bottom: 24px;
  text-align: left;
}

.feature-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--el-text-color-regular);
}

.feature-item .el-icon {
  color: var(--el-color-success);
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
  /* 报告使用固定浅色主题，确保深色模式下也能清晰显示 */
  background: #ffffff;
  padding: 16px;
  border-radius: 8px;
  color: #1f2937;
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

.markdown-content :deep(h5) {
  font-size: 14px;
  font-weight: 600;
  margin: 10px 0 4px;
  color: #374151;
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

/* 最近使用快速选择 */
.recent-selections {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.recent-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  background: var(--el-fill-color-light);
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 6px;
  font-size: 12px;
  color: var(--el-text-color-regular);
  cursor: pointer;
  transition: all 0.2s;
}

.recent-item:hover {
  background: var(--el-fill-color);
  border-color: var(--el-color-primary-light-5);
}

.recent-item.active {
  background: var(--el-color-primary-light-9);
  border-color: var(--el-color-primary);
  color: var(--el-color-primary);
}

.recent-flag {
  font-size: 14px;
}

.recent-name {
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.recent-remove {
  font-size: 12px;
  color: var(--el-text-color-placeholder);
  opacity: 0;
  transition: opacity 0.2s;
}

.recent-item:hover .recent-remove {
  opacity: 1;
}

.recent-remove:hover {
  color: var(--el-color-danger);
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

/* 竞品任务信息 */
.competitor-task-info {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.competitor-task-info .info-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.competitor-task-info .info-item:last-child {
  border-bottom: none;
}

.competitor-task-info .label {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.competitor-task-info .value {
  font-size: 13px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.no-task-hint {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px 16px;
  color: var(--el-text-color-secondary);
  text-align: center;
}

.no-task-hint p {
  margin: 12px 0;
  font-size: 13px;
}

/* 竞品 ASIN 列表 */
.competitor-asin-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 12px;
  padding: 12px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
  max-height: 150px;
  overflow-y: auto;
}

.competitor-asin-list .el-tag {
  font-family: monospace;
}
</style>

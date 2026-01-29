<template>
  <div class="ad-optimizer-tab">
    <!-- 项目列表视图 -->
    <div v-if="viewMode === 'list'" class="project-list-view">
      <div class="list-header">
        <h2>智能广告分析</h2>
        <div class="header-actions">
          <el-button type="primary" @click="showCreateDialog = true">
            <el-icon><Plus /></el-icon>
            新建项目
          </el-button>
        </div>
      </div>

      <!-- 项目管理工具栏（有项目时显示） -->
      <div v-if="projects.length > 0" class="project-toolbar">
        <el-input
          v-model="projectSearch"
          placeholder="搜索项目名称..."
          clearable
          style="width: 240px"
          :prefix-icon="Search"
        />
        <el-select v-model="projectSort" style="width: 150px">
          <el-option label="最近更新" value="updated" />
          <el-option label="名称 A-Z" value="name_asc" />
          <el-option label="名称 Z-A" value="name_desc" />
          <el-option label="搜索词数量" value="terms" />
          <el-option label="目标 ACOS" value="acos" />
        </el-select>
        <span class="project-count">共 {{ filteredProjects.length }} 个项目</span>
      </div>

      <div v-if="projects.length === 0" class="empty-state">
        <el-empty description="暂无广告分析项目">
          <el-button type="primary" @click="showCreateDialog = true">创建第一个项目</el-button>
        </el-empty>
      </div>

      <div v-else-if="filteredProjects.length > 0" class="project-grid">
        <div
          v-for="project in filteredProjects"
          :key="project.id"
          class="project-card"
          @click="enterProject(project)"
        >
          <div class="card-header">
            <span class="project-name">{{ project.name }}</span>
            <div class="more-btn" @click.stop>
              <el-dropdown trigger="click">
                <el-icon class="more-icon"><MoreFilled /></el-icon>
                <template #dropdown>
                  <el-dropdown-menu>
                    <el-dropdown-item @click="editProject(project)">编辑</el-dropdown-item>
                    <el-dropdown-item @click="deleteProject(project)" divided style="color: var(--el-color-danger)">删除</el-dropdown-item>
                  </el-dropdown-menu>
                </template>
              </el-dropdown>
            </div>
          </div>
          <div class="card-meta">
            <span class="meta-item">
              目标 ACOS: {{ project.target_acos }}%
            </span>
          </div>
          <div class="card-stats">
            <span class="stat">{{ project.search_term_count || 0 }} 条搜索词</span>
            <span class="stat-time">{{ formatDate(project.updated_at || project.created_at) }}</span>
          </div>
        </div>
      </div>

      <!-- 搜索无结果 -->
      <div v-else-if="projectSearch" class="empty-state">
        <el-empty :description="`未找到包含 '${projectSearch}' 的项目`">
          <el-button @click="projectSearch = ''">清除搜索</el-button>
        </el-empty>
      </div>
    </div>

    <!-- 项目详情视图 -->
    <div v-else class="project-detail-view">
      <div class="detail-header">
        <el-button link @click="viewMode = 'list'">
          <el-icon><ArrowLeft /></el-icon>
          返回列表
        </el-button>
        <div class="header-info">
          <h3>{{ currentProject?.name }}</h3>
          <el-tag size="small">目标 ACOS: {{ currentProject?.target_acos }}%</el-tag>
          <el-button link size="small" @click="editProject(currentProject!)" style="margin-left: 8px">
            编辑
          </el-button>
        </div>
        <el-button type="primary" @click="showImportDialog = true">
          <el-icon><Upload /></el-icon>
          导入数据
        </el-button>
      </div>

      <!-- 数据导入区 -->
      <AdDataImport
        v-if="!hasData"
        :project-id="currentProject?.id"
        @imported="onDataImported"
      />

      <!-- 有数据时显示概览和分析 -->
      <template v-else>
        <!-- 数据概览 -->
        <div class="data-overview">
          
           <!-- 全球/合计 摘要条 -->
           <div class="global-summary-banner">
               <div class="summary-item">
                   <div class="label">总花费</div>
                   <div class="value">{{ formatCurrency(stats.total_spend, stats.by_country[0]?.country || 'US') }}</div>
               </div>
               <div class="divider"></div>
               <div class="summary-item">
                   <div class="label">总销售额</div>
                   <div class="value">{{ formatCurrency(stats.total_sales, stats.by_country[0]?.country || 'US') }}</div>
               </div>
               <div class="divider"></div>
               <div class="summary-item">
                   <div class="label">平均 ACOS</div>
                   <div class="value" :class="{ 'warning': stats.avg_acos > (currentProject?.target_acos || 30) }">{{ stats.avg_acos.toFixed(1) }}%</div>
               </div>
               <div class="divider"></div>
               <div class="summary-item">
                    <div class="label">市场覆盖</div>
                    <div class="value">{{ stats.by_country.length }} 个国家</div>
               </div>
           </div>

          <!-- 国家卡片网格 -->
          <div class="market-grid-container">
             <div class="grid-header">
                 <h4>各市场表现</h4>
             </div>
             
             <!-- 如果只有一个国家，也用 Grid 但限制宽度 -->
             <div class="cards-grid">
                <AdCountryCard 
                    v-for="countryStat in stats.by_country" 
                    :key="countryStat.country"
                    :stats="countryStat"
                    :target-acos="currentProject?.target_acos || 30"
                    @analyze="startAnalysisForCountry(countryStat.country)"
                />
             </div>
          </div>
        </div>

        <!-- 数据可视化图表 -->
        <AdDataCharts
          ref="dataChartsRef"
          :terms="searchTerms"
          :target-acos="currentProject?.target_acos || 30"
          v-model:selected-country="selectedChartCountry"
          @select="handleChartSelect"
        />

        <!-- AI 分析区 -->
        <div class="ai-analysis-section">
          <!-- 分析范围信息 -->
          <div class="analysis-scope-info">
            <div class="scope-header">
              <span class="scope-label">分析范围</span>
            </div>
            <div class="scope-details">
              <div class="scope-item">
                <el-icon class="scope-icon"><Calendar /></el-icon>
                <span>时间：{{ analysisDateRangeLabel }}</span>
              </div>
              <div class="scope-item">
                <el-icon class="scope-icon"><Location /></el-icon>
                <span>国家：{{ analysisCountryLabel }}</span>
              </div>
              <div class="scope-item">
                <el-icon class="scope-icon"><DataAnalysis /></el-icon>
                <span>数据量：{{ analysisTermsCount }} 条搜索词</span>
              </div>
            </div>
            <div v-if="needsSampling" class="scope-warning">
              <el-icon><Warning /></el-icon>
              <span>每个国家超过 200 条时将智能采样（高花费、低转化、高ACOS、高潜力各占比例）</span>
            </div>
          </div>

          <div class="section-header">
            <span class="section-title">AI 多智能体分析</span>
            <div class="controls">
              <span class="control-label">AI 服务:</span>
              <el-select v-model="selectedProvider" size="small" style="width: 110px" :disabled="isAnalyzing">
                <el-option v-for="(config, key) in AI_PROVIDERS" :key="key" :label="config.name" :value="key" />
              </el-select>
              <span class="control-label">模型:</span>
              <el-select v-model="selectedModel" size="small" style="width: 150px" :disabled="isAnalyzing">
                <el-option v-for="model in availableModels" :key="model" :label="model" :value="model" />
              </el-select>
              <el-button
                v-if="!isAnalyzing"
                type="primary"
                @click="startAnalysis"
              >
                {{ analysisResult ? '重新分析' : '开始分析' }}
              </el-button>
              <el-button
                v-else
                type="danger"
                @click="handleStopAnalysis"
              >
                停止分析
              </el-button>
              <el-button
                v-if="failedCountries.length > 0 && !isAnalyzing"
                type="warning"
                @click="retryFailed"
              >
                重试失败 ({{ failedCountries.length }})
              </el-button>
            </div>
          </div>

          <!-- 失败国家提示 -->
          <el-alert
            v-if="failedCountries.length > 0 && !isAnalyzing"
            type="warning"
            :closable="false"
            style="margin-bottom: 16px"
          >
            <template #title>
              <span>以下国家分析失败: {{ failedCountries.join(', ') }}</span>
            </template>
            <span>已完成的国家结果不受影响，可点击"重试失败"按钮重新分析失败的国家</span>
          </el-alert>

          <!-- 可视化画布（分析进行中） -->
          <AdAnalysisCanvas
            v-if="isAnalyzing || analysisSession"
            :session="analysisSession"
            @stop="handleStopAnalysis"
          />

          <!-- 分析结果（支持边分析边显示） -->
          <AdAnalysisResults
            ref="analysisResultsRef"
            v-if="analysisResult"
            :result="analysisResult"
            :target-acos="currentProject?.target_acos || 30"
            :is-partial="isAnalyzing"
            v-model:selected-country="selectedChartCountry"
            @export="exportResults"
          />
        </div>
      </template>
    </div>

    <!-- 创建/编辑项目对话框 -->
    <el-dialog
      v-model="showCreateDialog"
      :title="editingProject ? '编辑项目' : '新建广告分析项目'"
      width="500px"
    >
      <el-form :model="projectForm" label-width="100px">
        <el-form-item label="项目名称" required>
          <el-input v-model="projectForm.name" placeholder="例如：产品A广告优化" />
        </el-form-item>
        <el-form-item label="目标 ACOS" required>
          <el-input-number
            v-model="projectForm.targetAcos"
            :min="1"
            :max="100"
            :step="5"
          />
          <span style="margin-left: 8px">%</span>
        </el-form-item>
        <div class="form-tip">
          国家/地区信息将从导入的搜索词报告中自动识别
        </div>
      </el-form>
      <template #footer>
        <el-button @click="showCreateDialog = false">取消</el-button>
        <el-button type="primary" @click="saveProject" :loading="saving">
          {{ editingProject ? '保存' : '创建' }}
        </el-button>
      </template>
    </el-dialog>

    <!-- 导入数据对话框 -->
    <el-dialog
      v-model="showImportDialog"
      title="导入搜索词数据"
      width="800px"
      destroy-on-close
    >
      <AdDataImport
        :project-id="currentProject?.id"
        @imported="onImportDialogImported"
      />
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, nextTick } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Plus, MoreFilled, ArrowLeft, Search, Upload, Warning, Calendar, Location, DataAnalysis } from '@element-plus/icons-vue';
import type { AdProject, AdSearchTerm, AdAnalysisResult, AIProvider, SearchTermsStatsResult } from '../types';
import { AI_PROVIDERS, COUNTRY_CURRENCY_MAP, getCountryLabel } from '../types';
import type { AnalysisSession } from '../ad-prompts';
import { checkApiKeyConfigured } from '../ai-service';
import {
  adGetProjects,
  adCreateProject,
  adUpdateProject,
  adDeleteProject,
  adGetSearchTermsStats,
  adGetSearchTerms,
  adSaveAnalysis,
  adGetAnalysis,
} from '../api';
import { runMultiAgentAnalysis, retryFailedCountries, stopAnalysis } from '../ad-analysis';
import type { CountryAnalysisResult } from '../types';
import AdDataImport from './ad-optimizer/AdDataImport.vue';
import AdAnalysisCanvas from './ad-optimizer/AdAnalysisCanvas.vue';
import AdAnalysisResults from './ad-optimizer/AdAnalysisResults.vue';
import AdCountryCard from './ad-optimizer/AdCountryCard.vue';
import AdDataCharts from './ad-optimizer/AdDataCharts.vue';

// 视图模式
const viewMode = ref<'list' | 'detail'>('list');
const currentProject = ref<AdProject | null>(null);

// 项目列表
const projects = ref<AdProject[]>([]);
const loading = ref(false);

// 项目搜索和排序
const projectSearch = ref('');
const projectSort = ref('updated');

// 过滤和排序后的项目列表
const filteredProjects = computed(() => {
  let result = [...projects.value];

  // 搜索过滤
  if (projectSearch.value) {
    const search = projectSearch.value.toLowerCase();
    result = result.filter(p => p.name.toLowerCase().includes(search));
  }

  // 排序
  switch (projectSort.value) {
    case 'name_asc':
      result.sort((a, b) => a.name.localeCompare(b.name, 'zh'));
      break;
    case 'name_desc':
      result.sort((a, b) => b.name.localeCompare(a.name, 'zh'));
      break;
    case 'terms':
      result.sort((a, b) => (b.search_term_count || 0) - (a.search_term_count || 0));
      break;
    case 'acos':
      result.sort((a, b) => a.target_acos - b.target_acos);
      break;
    case 'updated':
    default:
      result.sort((a, b) => {
        const dateA = new Date(a.updated_at || a.created_at).getTime();
        const dateB = new Date(b.updated_at || b.created_at).getTime();
        return dateB - dateA;
      });
      break;
  }

  return result;
});

function startAnalysisForCountry(country: string) {
    // 检查是否有分析结果
    if (!analysisResult.value) {
        ElMessage.warning('请先进行 AI 分析');
        // 滚动到 AI 分析区
        const aiSection = document.querySelector('.ai-analysis-section');
        if (aiSection) {
            aiSection.scrollIntoView({ behavior: 'smooth' });
        }
        return;
    }

    // 检查该国家是否已分析
    const hasResult = analysisResultsRef.value?.hasCountryResult(country);
    if (!hasResult) {
        const countryLabel = getCountryLabel(country);
        ElMessage.warning(`${countryLabel} 尚未分析，请先进行 AI 分析`);
        return;
    }

    // 切换到对应国家标签页
    analysisResultsRef.value?.switchToCountry(country);

    // 滚动到分析结果区域
    nextTick(() => {
        const resultsSection = document.querySelector('.ad-analysis-results');
        if (resultsSection) {
            resultsSection.scrollIntoView({ behavior: 'smooth' });
        }
    });
}

// 创建/编辑对话框
const showCreateDialog = ref(false);
const showImportDialog = ref(false);
const editingProject = ref<AdProject | null>(null);
const saving = ref(false);
const projectForm = ref({
  name: '',
  targetAcos: 30,
});

// 数据统计
const stats = ref<SearchTermsStatsResult>({
  total_spend: 0,
  total_sales: 0,
  avg_acos: 0,
  count: 0,
  by_country: [],
});
const hasData = computed(() => stats.value.count > 0);

// 格式化带货币的金额
function formatCurrency(amount: number, country: string): string {
  const currency = COUNTRY_CURRENCY_MAP[country];
  const symbol = currency?.symbol || '$';
  return `${symbol}${amount.toFixed(2)}`;
}

// AI 分析
const selectedProvider = ref<AIProvider>('deepseek');
const selectedModel = ref(AI_PROVIDERS.deepseek.defaultModel);
const availableModels = computed(() => AI_PROVIDERS[selectedProvider.value].models);
const isAnalyzing = ref(false);
const analysisSession = ref<AnalysisSession | null>(null);
const analysisResult = ref<AdAnalysisResult | null>(null);
const searchTerms = ref<AdSearchTerm[]>([]);
const failedCountries = ref<string[]>([]);
const completedResults = ref<CountryAnalysisResult[]>([]);
const analysisResultsRef = ref<InstanceType<typeof AdAnalysisResults> | null>(null);
const dataChartsRef = ref<InstanceType<typeof AdDataCharts> | null>(null);
const selectedChartCountry = ref<string>('all');  // 图表国家筛选

// Provider 变化时更新默认模型
watch(selectedProvider, (newProvider) => {
  selectedModel.value = AI_PROVIDERS[newProvider].defaultModel;
});

// 分析范围信息 - 从图表组件获取筛选后的数据
const analysisTermsCount = computed(() => {
  return dataChartsRef.value?.filteredTerms?.length || searchTerms.value.length;
});

const analysisDateRangeLabel = computed(() => {
  return dataChartsRef.value?.dateRangeLabel || '全部时间';
});

const analysisCountryLabel = computed(() => {
  if (selectedChartCountry.value === 'all') {
    const countryCount = stats.value.by_country.length;
    return countryCount > 1 ? `全部 (${countryCount} 个国家)` : stats.value.by_country[0]?.country || '全部';
  }
  return getCountryLabel(selectedChartCountry.value);
});

// 检查是否有国家需要采样（任何国家超过 200 条）
const needsSampling = computed(() => {
  // 检查各国家数据量
  const termsToCheck = dataChartsRef.value?.filteredTerms || searchTerms.value;
  const countryMap = new Map<string, number>();
  termsToCheck.forEach(term => {
    const country = term.country || 'Unknown';
    countryMap.set(country, (countryMap.get(country) || 0) + 1);
  });
  // 任何一个国家超过 200 条就需要采样
  for (const count of countryMap.values()) {
    if (count > 200) return true;
  }
  return false;
});

// 加载项目列表
async function loadProjects() {
  loading.value = true;
  try {
    projects.value = await adGetProjects();
  } catch (error) {
    ElMessage.error('加载项目列表失败');
    console.error(error);
  } finally {
    loading.value = false;
  }
}

// 进入项目
async function enterProject(project: AdProject) {
  currentProject.value = project;
  viewMode.value = 'detail';

  // 重置分析相关状态（防止显示上一个项目的数据）
  analysisResult.value = null;
  completedResults.value = [];
  searchTerms.value = [];
  failedCountries.value = [];
  analysisSession.value = null;
  stats.value = { total_spend: 0, total_sales: 0, avg_acos: 0, count: 0, by_country: [] };
  selectedChartCountry.value = 'all';  // 重置国家筛选

  // 加载项目数据统计
  await loadProjectStats();

  // 加载搜索词数据（用于图表展示）
  await loadSearchTerms();

  // 加载已有的分析结果
  await loadExistingAnalysis();
}

// 加载项目统计
async function loadProjectStats() {
  if (!currentProject.value) return;

  try {
    stats.value = await adGetSearchTermsStats(currentProject.value.id);
  } catch (error) {
    console.error('加载统计数据失败:', error);
  }
}

// 加载搜索词数据（用于图表展示）
async function loadSearchTerms() {
  if (!currentProject.value) return;

  try {
    searchTerms.value = await adGetSearchTerms(currentProject.value.id);
  } catch (error) {
    console.error('加载搜索词数据失败:', error);
  }
}

// 处理图表选中搜索词
function handleChartSelect(searchTerm: string) {
  // TODO: 实现跳转到对应搜索词的功能
  console.log('选中搜索词:', searchTerm);
}

// 加载已有分析结果
async function loadExistingAnalysis() {
  if (!currentProject.value) return;

  try {
    const result = await adGetAnalysis(currentProject.value.id, 'final_result');
    if (result) {
      analysisResult.value = JSON.parse(result.result_json);
    }
  } catch (error) {
    console.error('加载分析结果失败:', error);
  }
}

// 数据导入完成
async function onDataImported() {
  await loadProjectStats();
  await loadSearchTerms();
  ElMessage.success('数据导入成功');
}

// 导入对话框导入完成
async function onImportDialogImported() {
  showImportDialog.value = false;
  await loadProjectStats();
  await loadSearchTerms();
  ElMessage.success('数据导入成功');
}


// 停止 AI 分析
function handleStopAnalysis() {
  ElMessageBox.confirm(
    '确定要停止当前的分析吗？已完成的国家结果会保留。',
    '停止分析',
    {
      confirmButtonText: '停止',
      cancelButtonText: '继续分析',
      type: 'warning',
    }
  ).then(() => {
    const stopped = stopAnalysis();
    if (stopped) {
      ElMessage.info('分析已停止');
      isAnalyzing.value = false;
    }
  }).catch(() => {
    // 用户取消，继续分析
  });
}

// 开始 AI 分析
async function startAnalysis() {
  if (!currentProject.value) return;

  // 检查 API Key
  const hasKey = await checkApiKeyConfigured(selectedProvider.value);
  if (!hasKey) {
    ElMessage.warning(`请先在设置中配置 ${AI_PROVIDERS[selectedProvider.value].name} API Key`);
    return;
  }

  isAnalyzing.value = true;
  analysisResult.value = null;
  failedCountries.value = [];
  completedResults.value = [];

  try {
    // 加载搜索词数据
    searchTerms.value = await adGetSearchTerms(currentProject.value.id);

    if (searchTerms.value.length === 0) {
      ElMessage.warning('没有搜索词数据，请先导入数据');
      isAnalyzing.value = false;
      return;
    }

    // 使用筛选后的数据（与图表一致）
    const termsToAnalyze = dataChartsRef.value?.filteredTerms || searchTerms.value;

    if (termsToAnalyze.length === 0) {
      ElMessage.warning('当前筛选条件下没有数据，请调整时间或国家筛选');
      isAnalyzing.value = false;
      return;
    }

    console.log(`[Analysis] 分析数据量: ${termsToAnalyze.length} 条 (原始: ${searchTerms.value.length} 条)`);

    // 运行多智能体分析（支持增量回调）
    const result = await runMultiAgentAnalysis(
      termsToAnalyze,
      currentProject.value.target_acos,
      selectedProvider.value,
      selectedModel.value,
      (session) => {
        analysisSession.value = session;
        // 实时更新失败国家列表
        if (session.countryProgress?.failedCountries) {
          failedCountries.value = session.countryProgress.failedCountries;
        }
        // 实时更新结果（增量显示）
        if (session.finalResult) {
          analysisResult.value = session.finalResult;
        }
        // 检查是否被取消
        if (session.status === 'cancelled') {
          isAnalyzing.value = false;
        }
      },
      (country, countryResult) => {
        // 单个国家完成回调
        console.log(`国家 ${country} 分析完成`);
        completedResults.value.push(countryResult);
      }
    );

    if (result) {
      analysisResult.value = result;

      // 保存分析结果（无论是完成还是取消，都保存已有结果）
      await adSaveAnalysis(
        currentProject.value.id,
        'final_result',
        JSON.stringify(result),
        selectedProvider.value,
        selectedModel.value
      );

      // 根据状态显示不同消息
      if (analysisSession.value?.status === 'cancelled') {
        if (completedResults.value.length > 0) {
          ElMessage.info(`分析已停止，已保存 ${completedResults.value.length} 个国家的结果`);
        }
      } else if (failedCountries.value.length > 0) {
        ElMessage.warning(`分析部分完成，${failedCountries.value.length} 个国家失败，可点击重试`);
      } else {
        ElMessage.success('分析完成');
      }
    }
  } catch (error) {
    // 忽略取消导致的错误
    if (error instanceof Error && error.name === 'AbortError') {
      return;
    }
    console.error('分析失败:', error);
    const errorMsg = typeof error === 'string' ? error : (error as Error)?.message || String(error);
    ElMessage.error('分析失败: ' + errorMsg);
  } finally {
    isAnalyzing.value = false;
  }
}

// 重试失败的国家
async function retryFailed() {
  if (!currentProject.value || failedCountries.value.length === 0) return;

  // 检查 API Key
  const hasKey = await checkApiKeyConfigured(selectedProvider.value);
  if (!hasKey) {
    ElMessage.warning(`请先在设置中配置 ${AI_PROVIDERS[selectedProvider.value].name} API Key`);
    return;
  }

  isAnalyzing.value = true;
  const countriesToRetry = [...failedCountries.value];
  failedCountries.value = [];

  try {
    ElMessage.info(`正在重试 ${countriesToRetry.join(', ')}...`);

    const result = await retryFailedCountries(
      searchTerms.value,
      currentProject.value.target_acos,
      selectedProvider.value,
      selectedModel.value,
      countriesToRetry,
      [...completedResults.value],  // 传入副本，避免回调修改时造成重复
      (session) => {
        analysisSession.value = session;
        if (session.countryProgress?.failedCountries) {
          failedCountries.value = session.countryProgress.failedCountries;
        }
        if (session.finalResult) {
          analysisResult.value = session.finalResult;
        }
      },
      (country, countryResult) => {
        console.log(`重试: 国家 ${country} 分析完成`);
        completedResults.value.push(countryResult);
      }
    );

    if (result) {
      analysisResult.value = result;

      // 保存更新后的结果
      await adSaveAnalysis(
        currentProject.value.id,
        'final_result',
        JSON.stringify(result),
        selectedProvider.value,
        selectedModel.value
      );

      if (failedCountries.value.length > 0) {
        ElMessage.warning(`仍有 ${failedCountries.value.length} 个国家失败`);
      } else {
        ElMessage.success('所有国家分析完成');
      }
    }
  } catch (error) {
    console.error('重试失败:', error);
    const errorMsg = typeof error === 'string' ? error : (error as Error)?.message || String(error);
    ElMessage.error('重试失败: ' + errorMsg);
  } finally {
    isAnalyzing.value = false;
  }
}

// 导出结果
function exportResults(_type: 'negative_words' | 'bid_adjustments' | 'all') {
  if (!analysisResult.value) return;

  // TODO: 实现导出功能
  ElMessage.info('导出功能开发中...');
}

// 保存项目
async function saveProject() {
  if (!projectForm.value.name.trim()) {
    ElMessage.warning('请输入项目名称');
    return;
  }

  saving.value = true;
  try {
    if (editingProject.value) {
      await adUpdateProject(
        editingProject.value.id,
        projectForm.value.name,
        '',  // marketplace 从搜索词数据中自动获取
        projectForm.value.targetAcos
      );

      // 如果编辑的是当前正在查看的项目，更新 currentProject
      if (currentProject.value && currentProject.value.id === editingProject.value.id) {
        currentProject.value = {
          ...currentProject.value,
          name: projectForm.value.name,
          target_acos: projectForm.value.targetAcos,
        };
      }

      ElMessage.success('项目更新成功');
    } else {
      await adCreateProject(
        null,
        projectForm.value.name,
        '',  // marketplace 从搜索词数据中自动获取
        projectForm.value.targetAcos
      );
      ElMessage.success('项目创建成功');
    }

    showCreateDialog.value = false;
    resetForm();
    await loadProjects();
  } catch (error) {
    ElMessage.error('保存失败');
    console.error(error);
  } finally {
    saving.value = false;
  }
}

// 编辑项目
function editProject(project: AdProject) {
  editingProject.value = project;
  projectForm.value = {
    name: project.name,
    targetAcos: project.target_acos,
  };
  showCreateDialog.value = true;
}

// 删除项目
async function deleteProject(project: AdProject) {
  try {
    await ElMessageBox.confirm(
      `确定要删除项目 "${project.name}" 吗？所有相关数据将被删除。`,
      '删除确认',
      { type: 'warning' }
    );

    await adDeleteProject(project.id);
    ElMessage.success('项目已删除');
    await loadProjects();
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除失败');
      console.error(error);
    }
  }
}

// 格式化日期
function formatDate(dateStr: string): string {
  if (!dateStr) return '';
  const date = new Date(dateStr);
  return date.toLocaleDateString('zh-CN');
}

// 重置表单
function resetForm() {
  projectForm.value = {
    name: '',
    targetAcos: 30,
  };
  editingProject.value = null;
}

onMounted(() => {
  loadProjects();
});
</script>

<style scoped>
.ad-optimizer-tab {
  height: 100%;
  padding: 20px;
  overflow-y: auto;
}

/* 列表视图 */
.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.list-header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.help-btn {
  color: var(--el-text-color-secondary);
  border-color: var(--el-border-color-light);
}

.help-btn:hover {
  color: var(--el-color-primary);
  border-color: var(--el-color-primary-light-5);
}

.project-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
  padding: 12px 16px;
  background: var(--el-fill-color-lighter);
  border-radius: 8px;
}

.project-toolbar .project-count {
  margin-left: auto;
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 400px;
}

.project-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.project-card {
  background: var(--glass-bg, var(--el-bg-color));
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  border: 1px solid var(--glass-border, var(--el-border-color-lighter));
  border-radius: 12px;
  padding: 16px;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.project-card:hover {
  border-color: var(--el-color-primary-light-5);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
  transform: translateY(-2px);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.project-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.more-btn {
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.more-btn:hover {
  background: var(--el-fill-color);
}

.more-icon {
  color: var(--el-text-color-secondary);
  cursor: pointer;
  font-size: 16px;
}

.more-icon:hover {
  color: var(--el-text-color-primary);
}

.card-meta {
  display: flex;
  gap: 16px;
  margin-bottom: 12px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.card-stats {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: var(--el-text-color-placeholder);
}

/* 详情视图 */
.detail-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--el-border-color-light);
}

.header-info {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-info h3 {
  margin: 0;
  font-size: 18px;
}

/* 数据概览 */
.data-overview {
  margin-bottom: 24px;
}

.stat-card {
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-light);
  border-radius: 8px;
  padding: 16px;
  text-align: center;
}

.stat-card.warning .stat-value {
  color: var(--el-color-warning);
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: var(--el-color-primary);
  margin-bottom: 4px;
}

.stat-label {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

/* AI 分析区 */
.ai-analysis-section {
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-light);
  border-radius: 8px;
  padding: 20px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
}

.controls {
  display: flex;
  gap: 8px;
  align-items: center;
}

.control-label {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.form-tip {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 8px;
  padding: 8px 12px;
  background: var(--el-fill-color-light);
  border-radius: 4px;
}

/* Global Summary Banner */
.global-summary-banner {
    display: flex;
    align-items: center;
    background: var(--el-bg-color);
    border: 1px solid var(--el-border-color-lighter);
    border-radius: 12px;
    padding: 24px;
    margin-bottom: 32px;
    box-shadow: var(--el-box-shadow-light);
}

.summary-item {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
}

.summary-item .label {
    font-size: 13px;
    color: var(--el-text-color-secondary);
    margin-bottom: 8px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.summary-item .value {
    font-size: 28px;
    font-weight: 700;
    color: var(--el-text-color-primary);
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}

.summary-item .value.warning {
    color: var(--el-color-warning);
}

.divider {
    width: 1px;
    height: 40px;
    background: var(--el-border-color-lighter);
    margin: 0 16px;
}

/* Market Grid */
.market-grid-container {
    margin-bottom: 32px;
}

.grid-header {
    margin-bottom: 16px;
}

.grid-header h4 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--el-text-color-primary);
}

.cards-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 20px;
}

/* 分析范围信息 */
.analysis-scope-info {
    background: var(--el-fill-color-lighter);
    border: 1px solid var(--el-border-color-lighter);
    border-radius: 8px;
    padding: 16px 20px;
    margin-bottom: 20px;
}

.scope-header {
    margin-bottom: 12px;
}

.scope-label {
    font-size: 14px;
    font-weight: 600;
    color: var(--el-text-color-primary);
}

.scope-details {
    display: flex;
    gap: 24px;
    flex-wrap: wrap;
}

.scope-item {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--el-text-color-regular);
}

.scope-icon {
    font-size: 16px;
    color: var(--el-text-color-secondary);
}

.scope-warning {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 12px;
    padding: 10px 12px;
    background: var(--el-color-warning-light-9);
    border: 1px solid var(--el-color-warning-light-5);
    border-radius: 6px;
    font-size: 13px;
    color: var(--el-color-warning-dark-2);
}

.scope-warning .el-icon {
    font-size: 16px;
    color: var(--el-color-warning);
}
</style>

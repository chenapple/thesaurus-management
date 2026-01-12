<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { Plus, Delete, Back, Refresh, ArrowRight, CopyDocument, DataLine, Select, Search, MoreFilled, QuestionFilled } from '@element-plus/icons-vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import {
  scGetProjects, scCreateProject, scDeleteProject, scGetProject, scUpdateProject,
  scAddCompetitor, scGetCompetitors, scDeleteCompetitor, scUpdateCompetitorType,
  scFetchCompetitorListing, scFetchCompetitorsBatch, scFetchCompetitorReviews, scGetReviewsSummary,
  getProducts, getKeywordDataStats,
  scSaveAnalysis, scGetAllAnalysis, scGetProjectKeywords, scGetCompetitorReviews,
  scUpdateMyProductInfo, scUpdateProjectStatus, scUpdateMyListing
} from '../api';
import type { ScProject, ScCompetitor, ScReviewSummary, Product, ReviewInsights, ListingAnalysis, OptimizationResult, MyProductInfo } from '../types';
import { COUNTRY_OPTIONS, getCountryLabel, COMPETITOR_TYPE_OPTIONS, AI_PROVIDERS, type AIProvider } from '../types';
import { chatStream } from '../ai-service';
import { checkApiKeyConfigured } from '../ai-service';
import {
  buildReviewInsightsPrompt,
  buildListingAnalysisPrompt,
  buildOptimizationPrompt,
  parseAIResponse,
  validateReviewInsights,
  validateListingAnalysis
} from '../sc-prompts';
import * as XLSX from 'xlsx';
import { save } from '@tauri-apps/plugin-dialog';
import { writeFile } from '@tauri-apps/plugin-fs';
import iconRocket from '../assets/icons/rocket_3d.png';
import iconChart from '../assets/icons/chart_3d.png';
import AnalysisCanvas from './analysis/AnalysisCanvas.vue';
import type { AnalysisStatus } from './analysis/AnalysisCanvas.vue';

// Emits
const emit = defineEmits<{
  (e: 'showHelp', tab: string): void;
}>();

// ==================== 视图状态 ====================
const viewMode = ref<'list' | 'detail'>('list');
const currentProject = ref<ScProject | null>(null);

// ==================== 项目列表视图 ====================
const scenarioType = ref<'new' | 'optimize'>('new');
const projects = ref<ScProject[]>([]);
const loading = ref(false);

// 搜索和排序
const projectSearch = ref('');
const projectSort = ref<'updated' | 'created' | 'name_asc' | 'name_desc'>('updated');

const filteredProjects = computed(() => {
  let result = projects.value.filter(p => p.scenario_type === scenarioType.value);

  // 搜索过滤
  if (projectSearch.value.trim()) {
    const search = projectSearch.value.toLowerCase().trim();
    result = result.filter(p =>
      p.name.toLowerCase().includes(search) ||
      (p.my_asin && p.my_asin.toLowerCase().includes(search))
    );
  }

  // 排序
  switch (projectSort.value) {
    case 'name_asc':
      result.sort((a, b) => a.name.localeCompare(b.name, 'zh'));
      break;
    case 'name_desc':
      result.sort((a, b) => b.name.localeCompare(a.name, 'zh'));
      break;
    case 'created':
      result.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime());
      break;
    case 'updated':
    default:
      result.sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime());
      break;
  }

  return result;
});

async function loadProjects() {
  loading.value = true;
  try {
    projects.value = await scGetProjects();
  } catch (error) {
    console.error('加载项目失败:', error);
    ElMessage.error('加载项目失败');
  } finally {
    loading.value = false;
  }
}

// 新建项目弹窗
const showCreateDialog = ref(false);
const createForm = ref({ name: '', marketplace: 'US', myAsin: '', productId: null as number | null });
const creating = ref(false);
const products = ref<Product[]>([]);  // 产品列表（用于关联关键词数据）

async function handleCreateProject() {
  createForm.value = { name: '', marketplace: 'US', myAsin: '', productId: null };
  showCreateDialog.value = true;
  // 加载产品列表
  try {
    products.value = await getProducts();
  } catch (error) {
    console.error('加载产品列表失败:', error);
  }
}

async function confirmCreate() {
  if (!createForm.value.name.trim()) {
    ElMessage.warning('请输入项目名称');
    return;
  }
  if (scenarioType.value === 'optimize' && !createForm.value.myAsin.trim()) {
    ElMessage.warning('老品优化场景需要输入您的 ASIN');
    return;
  }

  creating.value = true;
  try {
    const projectId = await scCreateProject(
      createForm.value.name.trim(),
      scenarioType.value,
      createForm.value.marketplace,
      scenarioType.value === 'optimize' ? createForm.value.myAsin.trim() : undefined,
      createForm.value.productId || undefined
    );
    ElMessage.success('创建成功');
    showCreateDialog.value = false;
    await loadProjects();
    // 直接进入新创建的项目
    const newProject = await scGetProject(projectId);
    if (newProject) {
      enterProject(newProject);
    }
  } catch (error) {
    console.error('创建项目失败:', error);
    ElMessage.error('创建项目失败');
  } finally {
    creating.value = false;
  }
}

async function handleDeleteProject(project: ScProject) {
  try {
    await ElMessageBox.confirm(
      `确定删除项目「${project.name}」吗？此操作不可恢复。`,
      '删除确认',
      { confirmButtonText: '删除', cancelButtonText: '取消', type: 'warning' }
    );
    await scDeleteProject(project.id);
    ElMessage.success('删除成功');
    await loadProjects();
  } catch (error) {
    if (error !== 'cancel') {
      console.error('删除项目失败:', error);
      ElMessage.error('删除项目失败');
    }
  }
}

// 编辑项目
const showEditDialog = ref(false);
const editingProject = ref<ScProject | null>(null);
const editForm = ref({ name: '', marketplace: 'US', myAsin: '' });
const saving = ref(false);

function handleEditProject(project: ScProject, event?: Event) {
  event?.stopPropagation();  // 防止触发卡片点击
  editingProject.value = project;
  editForm.value = {
    name: project.name,
    marketplace: project.marketplace,
    myAsin: project.my_asin || '',
  };
  showEditDialog.value = true;
}

async function confirmEdit() {
  if (!editForm.value.name.trim()) {
    ElMessage.warning('请输入项目名称');
    return;
  }
  if (!editingProject.value) return;

  saving.value = true;
  try {
    await scUpdateProject(
      editingProject.value.id,
      editForm.value.name.trim(),
      editForm.value.marketplace,
      editForm.value.myAsin.trim() || undefined
    );
    ElMessage.success('更新成功');
    showEditDialog.value = false;

    // 如果正在编辑当前打开的项目，也更新 currentProject
    if (currentProject.value && editingProject.value && currentProject.value.id === editingProject.value.id) {
      currentProject.value = {
        ...currentProject.value,
        name: editForm.value.name.trim(),
        marketplace: editForm.value.marketplace,
        my_asin: editForm.value.myAsin.trim() || null,
      };
    }

    editingProject.value = null;
    await loadProjects();
  } catch (error) {
    console.error('更新项目失败:', error);
    ElMessage.error('更新项目失败');
  } finally {
    saving.value = false;
  }
}

// 关联产品信息
const linkedProduct = ref<Product | null>(null);
const linkedKeywordCount = ref(0);

// ==================== 我的产品信息（新品打造）====================
const showMyProductForm = ref(false);
const myProductInfo = ref<MyProductInfo>({
  brand_name: '',
  product_name: '',
  key_features: [],
  differentiators: '',
  specifications: '',
  target_audience: '',
  package_contents: '',
  additional_notes: '',
});
const newFeature = ref('');  // 用于添加新卖点
const savingMyProduct = ref(false);

// 添加卖点
function addKeyFeature() {
  const feature = newFeature.value.trim();
  if (!feature) return;
  if (myProductInfo.value.key_features.length >= 5) {
    ElMessage.warning('最多添加 5 条核心卖点');
    return;
  }
  if (myProductInfo.value.key_features.includes(feature)) {
    ElMessage.warning('该卖点已存在');
    return;
  }
  myProductInfo.value.key_features.push(feature);
  newFeature.value = '';
}

// 删除卖点
function removeKeyFeature(index: number) {
  myProductInfo.value.key_features.splice(index, 1);
}

// 保存产品信息
async function saveMyProductInfo() {
  if (!currentProject.value) return;

  // 验证必填字段
  if (!myProductInfo.value.brand_name.trim()) {
    ElMessage.warning('请输入品牌名称');
    return;
  }
  if (!myProductInfo.value.product_name.trim()) {
    ElMessage.warning('请输入产品名称');
    return;
  }
  if (myProductInfo.value.key_features.length === 0) {
    ElMessage.warning('请至少添加 1 条核心卖点');
    return;
  }

  savingMyProduct.value = true;
  try {
    await scUpdateMyProductInfo(currentProject.value.id, myProductInfo.value);
    ElMessage.success('产品信息已保存');
    // 更新当前项目的数据
    currentProject.value.my_product_info = JSON.stringify(myProductInfo.value);
  } catch (error) {
    console.error('保存产品信息失败:', error);
    ElMessage.error('保存产品信息失败');
  } finally {
    savingMyProduct.value = false;
  }
}

// 加载已保存的产品信息
function loadMyProductInfo(project: ScProject) {
  if (project.my_product_info) {
    try {
      const parsed = JSON.parse(project.my_product_info) as MyProductInfo;
      myProductInfo.value = {
        brand_name: parsed.brand_name || '',
        product_name: parsed.product_name || '',
        key_features: parsed.key_features || [],
        differentiators: parsed.differentiators || '',
        specifications: parsed.specifications || '',
        target_audience: parsed.target_audience || '',
        package_contents: parsed.package_contents || '',
        additional_notes: parsed.additional_notes || '',
      };
      showMyProductForm.value = true;
    } catch (e) {
      console.error('解析产品信息失败:', e);
      resetMyProductInfo();
    }
  } else {
    resetMyProductInfo();
  }
}

// 重置产品信息
function resetMyProductInfo() {
  myProductInfo.value = {
    brand_name: '',
    product_name: '',
    key_features: [],
    differentiators: '',
    specifications: '',
    target_audience: '',
    package_contents: '',
    additional_notes: '',
  };
  showMyProductForm.value = false;
}

async function enterProject(project: ScProject) {
  currentProject.value = project;
  viewMode.value = 'detail';
  // 保存当前项目ID，用于视图切换后恢复
  localStorage.setItem('sc_current_project_id', String(project.id));
  loadCompetitorsWithReviews();
  // 加载关联产品信息
  linkedProduct.value = null;
  linkedKeywordCount.value = 0;
  if (project.product_id) {
    try {
      // 从产品列表中查找（如果products为空，重新加载）
      if (products.value.length === 0) {
        products.value = await getProducts();
      }
      linkedProduct.value = products.value.find(p => p.id === project.product_id) || null;
      // 获取关键词数量
      if (linkedProduct.value) {
        linkedKeywordCount.value = await getKeywordDataStats(project.product_id);
      }
    } catch (error) {
      console.error('加载关联产品信息失败:', error);
    }
  }
  // 加载我的产品信息（仅新品打造）
  if (project.scenario_type === 'new') {
    loadMyProductInfo(project);
  }
  // 重置并加载 AI 分析结果
  analysisStep.value = 0;
  analysisError.value = '';
  reviewInsights.value = null;
  listingAnalysis.value = null;
  optimizationResult.value = null;
  loadAnalysisResults();
}

function backToList() {
  viewMode.value = 'list';
  currentProject.value = null;
  // 清除保存的项目ID
  localStorage.removeItem('sc_current_project_id');
  loadProjects();
}

// ==================== 项目详情视图 ====================
const competitors = ref<ScCompetitor[]>([]);
const loadingCompetitors = ref(false);
const fetchingCompetitorIds = ref<Set<number>>(new Set()); // 正在获取信息的竞品ID
const fetchingMyListing = ref(false); // 正在获取用户 ASIN 的 Listing
const showBullets = ref(false); // 五点描述展开状态

// 计算最大五点数量（用于对比表格）
const maxBulletCount = computed(() => {
  let max = 0;
  for (const comp of competitors.value) {
    const bullets = parseBullets(comp.bullets);
    if (bullets.length > max) max = bullets.length;
  }
  return max || 5; // 默认5条
});

async function loadCompetitors() {
  if (!currentProject.value) return;
  loadingCompetitors.value = true;
  try {
    competitors.value = await scGetCompetitors(currentProject.value.id);
  } catch (error) {
    console.error('加载竞品失败:', error);
    ElMessage.error('加载竞品失败');
  } finally {
    loadingCompetitors.value = false;
  }
}

// 获取用户的 ASIN Listing（老品优化时使用）
async function handleFetchMyListing() {
  if (!currentProject.value || !currentProject.value.my_asin) return;
  if (fetchingMyListing.value) return;

  fetchingMyListing.value = true;
  try {
    ElMessage.info(`正在获取 ${currentProject.value.my_asin} 的 Listing 信息...`);
    // 复用竞品获取逻辑，但不需要传入 id（因为不是存到竞品表）
    const result = await scFetchCompetitorListing(
      0, // 占位符，不会使用
      currentProject.value.my_asin,
      currentProject.value.marketplace
    );

    if (result.error) {
      ElMessage.error(`获取失败: ${result.error}`);
    } else {
      // 保存到项目表
      await scUpdateMyListing(
        currentProject.value.id,
        result.title,
        result.bullets,
        result.description
      );
      // 更新本地状态
      currentProject.value.my_title = result.title;
      currentProject.value.my_bullets = result.bullets ? JSON.stringify(result.bullets) : null;
      currentProject.value.my_description = result.description;
      currentProject.value.my_listing_fetched_at = new Date().toISOString();
      // 更新项目状态为采集中（单独捕获异常，不影响成功提示）
      if (currentProject.value.status === 'draft') {
        try {
          await scUpdateProjectStatus(currentProject.value.id, 'collecting');
          currentProject.value.status = 'collecting';
        } catch (e) {
          console.error('更新项目状态失败:', e);
        }
      }
      ElMessage.success('获取成功');
    }
  } catch (error) {
    console.error('获取用户 ASIN Listing 失败:', error);
    ElMessage.error('获取失败');
  } finally {
    fetchingMyListing.value = false;
  }
}

// 添加竞品弹窗
const showAddCompetitorDialog = ref(false);
const addCompetitorForm = ref({ asin: '', competitorType: 'direct' as 'top' | 'direct' | 'rising' });
const addingCompetitor = ref(false);

function handleAddCompetitor() {
  addCompetitorForm.value = { asin: '', competitorType: 'direct' };
  showAddCompetitorDialog.value = true;
}

async function confirmAddCompetitor() {
  const asin = addCompetitorForm.value.asin.trim().toUpperCase();
  if (!asin) {
    ElMessage.warning('请输入竞品 ASIN');
    return;
  }
  if (!/^B0[A-Z0-9]{8}$/.test(asin)) {
    ElMessage.warning('ASIN 格式不正确，应为 B0 开头的 10 位字符');
    return;
  }
  // 检查是否已存在
  if (competitors.value.some(c => c.asin === asin)) {
    ElMessage.warning('该 ASIN 已添加');
    return;
  }
  // 检查数量限制
  if (competitors.value.length >= 5) {
    ElMessage.warning('最多添加 5 个竞品');
    return;
  }

  addingCompetitor.value = true;
  try {
    await scAddCompetitor(currentProject.value!.id, asin, addCompetitorForm.value.competitorType);
    ElMessage.success('添加成功');
    showAddCompetitorDialog.value = false;
  } catch (error) {
    console.error('添加竞品失败:', error);
    ElMessage.error('添加竞品失败');
    addingCompetitor.value = false;
    return;
  }

  // 后续操作单独处理，不影响添加成功的提示
  try {
    await loadCompetitors();
    // 更新项目状态为采集中
    if (currentProject.value?.status === 'draft') {
      await scUpdateProjectStatus(currentProject.value.id, 'collecting');
      currentProject.value.status = 'collecting';
    }
  } catch (error) {
    console.error('加载竞品列表失败:', error);
  } finally {
    addingCompetitor.value = false;
  }
}

async function handleDeleteCompetitor(competitor: ScCompetitor) {
  try {
    await ElMessageBox.confirm(
      `确定删除竞品 ${competitor.asin} 吗？`,
      '删除确认',
      { confirmButtonText: '删除', cancelButtonText: '取消', type: 'warning' }
    );
    await scDeleteCompetitor(competitor.id);
    ElMessage.success('删除成功');
    await loadCompetitors();
  } catch (error) {
    if (error !== 'cancel') {
      console.error('删除竞品失败:', error);
      ElMessage.error('删除竞品失败');
    }
  }
}

async function handleTypeChange(competitor: ScCompetitor, newType: 'top' | 'direct' | 'rising') {
  try {
    await scUpdateCompetitorType(competitor.id, newType);
    competitor.competitor_type = newType;
  } catch (error) {
    console.error('更新类型失败:', error);
    ElMessage.error('更新类型失败');
  }
}

// 获取竞品 Listing 信息
async function handleFetchListing(competitor: ScCompetitor) {
  if (!currentProject.value) return;
  if (fetchingCompetitorIds.value.has(competitor.id)) return;

  fetchingCompetitorIds.value.add(competitor.id);
  try {
    ElMessage.info(`正在获取 ${competitor.asin} 的 Listing 信息...`);
    const result = await scFetchCompetitorListing(
      competitor.id,
      competitor.asin,
      currentProject.value.marketplace
    );

    if (result.error) {
      ElMessage.error(`获取失败: ${result.error}`);
    } else {
      ElMessage.success(`成功获取 ${competitor.asin} 的信息`);
      // 重新加载竞品列表以确保数据同步
      await loadCompetitors();
    }
  } catch (error) {
    console.error('获取竞品信息失败:', error);
    ElMessage.error('获取竞品信息失败');
  } finally {
    fetchingCompetitorIds.value.delete(competitor.id);
  }
}

// 批量获取所有竞品信息（复用同一个浏览器）
async function handleFetchAllListings() {
  if (!currentProject.value) return;

  const unfetchedCompetitors = competitors.value.filter(c => !c.fetched_at);
  if (unfetchedCompetitors.length === 0) {
    ElMessage.info('所有竞品信息都已获取');
    return;
  }

  // 标记所有竞品为加载中
  unfetchedCompetitors.forEach(c => fetchingCompetitorIds.value.add(c.id));

  try {
    ElMessage.info(`正在批量获取 ${unfetchedCompetitors.length} 个竞品信息...`);

    // 准备批量请求数据: [id, asin, country]
    const items: Array<[number, string, string]> = unfetchedCompetitors.map(c => [
      c.id,
      c.asin,
      currentProject.value!.marketplace
    ]);

    // 调用批量接口（只启动一个浏览器）
    const results = await scFetchCompetitorsBatch(items);

    // 统计结果
    let successCount = 0;
    let errorCount = 0;
    for (const [, result] of results) {
      if (result.error) {
        errorCount++;
      } else {
        successCount++;
      }
    }

    // 重新加载竞品列表
    await loadCompetitors();

    if (errorCount > 0) {
      ElMessage.warning(`批量获取完成: ${successCount} 成功, ${errorCount} 失败`);
    } else {
      ElMessage.success(`成功获取 ${successCount} 个竞品信息`);
    }
  } catch (error) {
    console.error('批量获取失败:', error);
    ElMessage.error(`批量获取失败: ${error}`);
  } finally {
    // 清除所有加载状态
    unfetchedCompetitors.forEach(c => fetchingCompetitorIds.value.delete(c.id));
  }
}

// 一键刷新所有竞品信息（已获取过的也重新获取）
async function handleRefreshAllListings() {
  if (!currentProject.value) return;

  if (competitors.value.length === 0) {
    ElMessage.info('没有竞品需要刷新');
    return;
  }

  // 标记所有竞品为加载中
  competitors.value.forEach(c => fetchingCompetitorIds.value.add(c.id));

  try {
    ElMessage.info(`正在刷新 ${competitors.value.length} 个竞品信息...`);

    // 准备批量请求数据: [id, asin, country]
    const items: Array<[number, string, string]> = competitors.value.map(c => [
      c.id,
      c.asin,
      currentProject.value!.marketplace
    ]);

    // 调用批量接口
    const results = await scFetchCompetitorsBatch(items);

    // 统计结果
    let successCount = 0;
    let errorCount = 0;
    for (const [, result] of results) {
      if (result.error) {
        errorCount++;
      } else {
        successCount++;
      }
    }

    // 重新加载竞品列表
    await loadCompetitors();

    if (errorCount > 0) {
      ElMessage.warning(`刷新完成: ${successCount} 成功, ${errorCount} 失败`);
    } else {
      ElMessage.success(`成功刷新 ${successCount} 个竞品信息`);
    }
  } catch (error) {
    console.error('刷新失败:', error);
    ElMessage.error(`刷新失败: ${error}`);
  } finally {
    // 清除所有加载状态
    competitors.value.forEach(c => fetchingCompetitorIds.value.delete(c.id));
  }
}

// ==================== 评论分析 ====================
const reviewSummaries = ref<Map<number, ScReviewSummary>>(new Map());
const fetchingReviewIds = ref<Set<number>>(new Set());
const showReviewsSection = ref(false);

// 获取单个竞品的评论
async function handleFetchReviews(competitor: ScCompetitor) {
  if (!currentProject.value) return;
  if (fetchingReviewIds.value.has(competitor.id)) return;

  fetchingReviewIds.value.add(competitor.id);
  try {
    ElMessage.info(`正在获取 ${competitor.asin} 的评论...`);
    const result = await scFetchCompetitorReviews(
      competitor.id,
      competitor.asin,
      currentProject.value.marketplace
    );

    if (result.error) {
      ElMessage.error(`获取评论失败: ${result.error}`);
    } else {
      ElMessage.success(`成功获取 ${result.summary.total} 条评论`);
      // 加载评论摘要
      await loadReviewSummary(competitor.id);
    }
  } catch (error) {
    console.error('获取评论失败:', error);
    ElMessage.error('获取评论失败');
  } finally {
    fetchingReviewIds.value.delete(competitor.id);
  }
}

// 加载评论统计摘要
async function loadReviewSummary(competitorId: number) {
  try {
    const summary = await scGetReviewsSummary(competitorId);
    reviewSummaries.value.set(competitorId, summary);
  } catch (error) {
    console.error('加载评论摘要失败:', error);
  }
}

// 批量获取所有评论
async function handleFetchAllReviews() {
  if (!currentProject.value) return;

  const fetchedCompetitors = competitors.value.filter(c => c.fetched_at);
  if (fetchedCompetitors.length === 0) {
    ElMessage.warning('请先获取竞品 Listing 信息');
    return;
  }

  for (const comp of fetchedCompetitors) {
    await handleFetchReviews(comp);
    // 添加延迟避免请求过快
    if (fetchedCompetitors.indexOf(comp) < fetchedCompetitors.length - 1) {
      await new Promise(resolve => setTimeout(resolve, 2000));
    }
  }
}

// 加载竞品后也加载评论摘要
async function loadCompetitorsWithReviews() {
  await loadCompetitors();
  // 并行加载所有评论摘要
  const promises = competitors.value.map(c => loadReviewSummary(c.id));
  await Promise.all(promises);
}

// 获取评论统计显示
function getReviewStats(competitorId: number): string {
  const summary = reviewSummaries.value.get(competitorId);
  if (!summary || summary.total === 0) return '暂无';
  return `${summary.total} 条`;
}

// 获取星级分布
function getStarDistribution(competitorId: number): { star: number; count: number; percent: number }[] {
  const summary = reviewSummaries.value.get(competitorId);
  if (!summary || summary.total === 0) return [];

  const stars = [
    { star: 5, count: summary.star_5 },
    { star: 4, count: summary.star_4 },
    { star: 3, count: summary.star_3 },
    { star: 2, count: summary.star_2 },
    { star: 1, count: summary.star_1 },
  ];

  return stars.map(s => ({
    ...s,
    percent: Math.round((s.count / summary.total) * 100),
  }));
}

// ==================== AI 分析 ====================
const selectedProvider = ref<AIProvider>('deepseek');
const selectedModel = ref(AI_PROVIDERS.deepseek.defaultModel);
const availableModels = computed(() => AI_PROVIDERS[selectedProvider.value].models);

// 分析状态
const analysisStep = ref(0);  // 0=未开始, 1=评论洞察, 2=文案分析, 3=优化建议
const isAnalyzing = ref(false);
const analysisError = ref('');
const streamingContent = ref('');
const abortController = ref<AbortController | null>(null);
// 画布模式并行执行时的多个 AbortController
const parallelAbortControllers = ref<AbortController[]>([]);

// 分析显示模式：classic（经典三步进度条） 或 canvas（可视化画布）
const analysisDisplayMode = ref<'classic' | 'canvas'>(
  (localStorage.getItem('sc_analysis_display_mode') as 'classic' | 'canvas') || 'classic'
);
// 并行执行状态（仅 canvas 模式使用）
const parallelStep1Done = ref(false);
const parallelStep2Done = ref(false);

// 监听模式变化，持久化到 localStorage
watch(analysisDisplayMode, (mode) => {
  localStorage.setItem('sc_analysis_display_mode', mode);
});

// 进度百分比（用于进度条动画）
const progressPercent = computed(() => {
  if (analysisStep.value === 0) return 0;
  if (analysisStep.value === 1) return 33;
  if (analysisStep.value === 2) return 66;
  return 100;
});

// 画布模式的状态对象
const analysisCanvasStatus = computed<AnalysisStatus>(() => {
  // 检查是否有评论数据
  let hasReviews = false;
  for (const comp of competitors.value) {
    if (reviewSummaries.value.get(comp.id)?.total) {
      hasReviews = true;
      break;
    }
  }

  return {
    step: analysisStep.value,
    reviewInsightsCompleted: !!reviewInsights.value,
    listingAnalysisCompleted: !!listingAnalysis.value,
    optimizationCompleted: !!optimizationResult.value,
    hasReviews,
    hasCompetitors: competitors.value.some(c => c.fetched_at),
    hasKeywords: true,  // 关键词数据来自关联的产品
    isParallel: analysisDisplayMode.value === 'canvas',
    parallelStep1Done: parallelStep1Done.value,
    parallelStep2Done: parallelStep2Done.value,
  };
});

// 分析结果
const reviewInsights = ref<ReviewInsights | null>(null);
const listingAnalysis = ref<ListingAnalysis | null>(null);
const optimizationResult = ref<OptimizationResult | null>(null);

// Provider 变化时更新默认模型
watch(selectedProvider, (newProvider) => {
  selectedModel.value = AI_PROVIDERS[newProvider].defaultModel;
});

// 进入项目时加载已有的分析结果
async function loadAnalysisResults() {
  if (!currentProject.value) return;
  try {
    const results = await scGetAllAnalysis(currentProject.value.id);
    for (const r of results) {
      try {
        const parsed = JSON.parse(r.result_json);
        if (r.analysis_type === 'review_insights' && validateReviewInsights(parsed)) {
          reviewInsights.value = parsed;
        } else if (r.analysis_type === 'listing_analysis' && validateListingAnalysis(parsed)) {
          listingAnalysis.value = parsed;
        } else if (r.analysis_type === 'optimization') {
          optimizationResult.value = parsed as OptimizationResult;
        }
      } catch (e) {
        console.error('解析分析结果失败:', e);
      }
    }
    // 如果有所有结果，设置步骤为完成
    if (reviewInsights.value && listingAnalysis.value && optimizationResult.value) {
      analysisStep.value = 3;
    } else if (reviewInsights.value && listingAnalysis.value) {
      analysisStep.value = 2;
    } else if (reviewInsights.value) {
      analysisStep.value = 1;
    }
  } catch (error) {
    console.error('加载分析结果失败:', error);
  }
}

// 开始 AI 分析
async function handleStartAnalysis() {
  if (!currentProject.value) return;

  // 检查 API Key
  const hasKey = await checkApiKeyConfigured(selectedProvider.value);
  if (!hasKey) {
    ElMessage.warning(`请先在设置中配置 ${AI_PROVIDERS[selectedProvider.value].name} API Key`);
    return;
  }

  // 检查是否有评论数据
  const hasReviews = Array.from(reviewSummaries.value.values()).some(s => s.total > 0);
  if (!hasReviews) {
    ElMessage.warning('请先获取竞品评论数据');
    return;
  }

  // 检查是否有文案数据
  const hasListings = competitors.value.some(c => c.fetched_at && c.title);
  if (!hasListings) {
    ElMessage.warning('请先获取竞品 Listing 信息');
    return;
  }

  // 检查新品打造是否填写了产品信息（警告，不阻止）
  if (currentProject.value.scenario_type === 'new' && !currentProject.value.my_product_info) {
    try {
      await ElMessageBox.confirm(
        '您尚未填写"我的产品信息"，AI 将只能生成通用的文案建议。\n\n建议先填写产品信息以获得针对性的文案。',
        '提示',
        {
          confirmButtonText: '继续分析',
          cancelButtonText: '去填写',
          type: 'warning',
        }
      );
    } catch {
      // 用户选择"去填写"，展开产品信息表单
      showMyProductForm.value = true;
      return;
    }
  }

  // 重置状态
  analysisStep.value = 0;
  analysisError.value = '';
  reviewInsights.value = null;
  listingAnalysis.value = null;
  optimizationResult.value = null;
  parallelStep1Done.value = false;
  parallelStep2Done.value = false;
  parallelAbortControllers.value = [];
  isAnalyzing.value = true;

  // 更新项目状态为分析中
  if (currentProject.value && currentProject.value.status !== 'analyzing') {
    await scUpdateProjectStatus(currentProject.value.id, 'analyzing');
    currentProject.value.status = 'analyzing';
  }

  try {
    if (analysisDisplayMode.value === 'canvas') {
      // 画布模式：步骤 1 和 2 并行执行
      analysisStep.value = 1;  // 标记开始分析

      // 并行执行评论洞察和文案分析
      await Promise.all([
        runReviewInsightsAnalysis().then(() => {
          parallelStep1Done.value = true;
        }),
        runListingAnalysis().then(() => {
          parallelStep2Done.value = true;
        }),
      ]);

      // 步骤 3: 优化建议生成（依赖前两步结果）
      await runOptimizationAnalysis();
    } else {
      // 经典模式：顺序执行
      // 步骤 1: 评论洞察分析
      await runReviewInsightsAnalysis();

      // 步骤 2: 文案分析
      await runListingAnalysis();

      // 步骤 3: 优化建议生成
      await runOptimizationAnalysis();
    }

    // 更新项目状态为已完成
    if (currentProject.value) {
      await scUpdateProjectStatus(currentProject.value.id, 'completed');
      currentProject.value.status = 'completed';
    }

    ElMessage.success('AI 分析完成！');
  } catch (error: any) {
    if (error.name === 'AbortError') {
      ElMessage.info('已停止分析');
    } else {
      analysisError.value = error.message || '分析失败';
      ElMessage.error(`分析失败: ${error.message}`);
    }
  } finally {
    isAnalyzing.value = false;
    streamingContent.value = '';
    abortController.value = null;
  }
}

// 步骤 1: 评论洞察分析
async function runReviewInsightsAnalysis() {
  if (!currentProject.value) return;

  // 仅在经典模式下更新步骤和清空流式内容
  if (analysisDisplayMode.value === 'classic') {
    analysisStep.value = 1;
    streamingContent.value = '';
  }

  // 收集评论数据
  const reviewData: Array<{ asin: string; reviews: any[] }> = [];
  for (const comp of competitors.value) {
    if (reviewSummaries.value.get(comp.id)?.total) {
      const reviews = await scGetCompetitorReviews(comp.id);
      reviewData.push({ asin: comp.asin, reviews });
    }
  }

  // 构建 prompt
  const prompt = buildReviewInsightsPrompt(reviewData, currentProject.value.marketplace);

  // 调用 AI - 并行模式使用独立的 AbortController
  const localAbortController = new AbortController();
  if (analysisDisplayMode.value === 'classic') {
    abortController.value = localAbortController;
  } else {
    parallelAbortControllers.value.push(localAbortController);
  }
  let fullResponse = '';

  for await (const chunk of chatStream(
    [{ role: 'user', content: prompt }],
    {
      provider: selectedProvider.value,
      model: selectedModel.value,
      maxTokens: 4096,
      signal: localAbortController.signal,
    }
  )) {
    if (chunk.done) break;
    fullResponse += chunk.content;
    // 仅在经典模式或当前是步骤1时更新流式内容
    if (analysisDisplayMode.value === 'classic') {
      streamingContent.value = fullResponse;
    } else {
      // 画布模式：显示当前正在进行的任务
      streamingContent.value = '[评论分析] ' + fullResponse.slice(-200);
    }
  }

  // 解析结果
  const parsed = parseAIResponse<ReviewInsights>(fullResponse);
  if (!parsed || !validateReviewInsights(parsed)) {
    throw new Error('评论洞察分析结果格式错误');
  }

  reviewInsights.value = parsed;

  // 保存到数据库
  await scSaveAnalysis(
    currentProject.value.id,
    'review_insights',
    JSON.stringify(parsed),
    selectedProvider.value,
    selectedModel.value
  );
}

// 步骤 2: 文案分析
async function runListingAnalysis() {
  if (!currentProject.value) return;

  // 仅在经典模式下更新步骤和清空流式内容
  if (analysisDisplayMode.value === 'classic') {
    analysisStep.value = 2;
    streamingContent.value = '';
  }

  // 构建 prompt
  const prompt = buildListingAnalysisPrompt(
    competitors.value.filter(c => c.fetched_at),
    currentProject.value.marketplace
  );

  // 调用 AI - 并行模式使用独立的 AbortController
  const localAbortController = new AbortController();
  if (analysisDisplayMode.value === 'classic') {
    abortController.value = localAbortController;
  } else {
    parallelAbortControllers.value.push(localAbortController);
  }
  let fullResponse = '';

  for await (const chunk of chatStream(
    [{ role: 'user', content: prompt }],
    {
      provider: selectedProvider.value,
      model: selectedModel.value,
      maxTokens: 4096,
      signal: localAbortController.signal,
    }
  )) {
    if (chunk.done) break;
    fullResponse += chunk.content;
    // 仅在经典模式时更新流式内容
    if (analysisDisplayMode.value === 'classic') {
      streamingContent.value = fullResponse;
    } else {
      // 画布模式：显示当前正在进行的任务
      streamingContent.value = '[文案分析] ' + fullResponse.slice(-200);
    }
  }

  // 解析结果
  const parsed = parseAIResponse<ListingAnalysis>(fullResponse);
  if (!parsed || !validateListingAnalysis(parsed)) {
    throw new Error('文案分析结果格式错误');
  }

  listingAnalysis.value = parsed;

  // 保存到数据库
  await scSaveAnalysis(
    currentProject.value.id,
    'listing_analysis',
    JSON.stringify(parsed),
    selectedProvider.value,
    selectedModel.value
  );
}

// 步骤 3: 优化建议生成
async function runOptimizationAnalysis() {
  if (!currentProject.value || !reviewInsights.value || !listingAnalysis.value) return;
  analysisStep.value = 3;
  streamingContent.value = '';

  // 获取关键词数据
  const keywords = await scGetProjectKeywords(currentProject.value.id, 100);

  // 解析产品信息（新品打造时使用）
  let parsedProductInfo: import('../types').MyProductInfo | null = null;
  if (currentProject.value.scenario_type === 'new' && currentProject.value.my_product_info) {
    try {
      parsedProductInfo = JSON.parse(currentProject.value.my_product_info);
    } catch (e) {
      console.error('解析产品信息失败:', e);
    }
  }

  // 解析用户的 Listing 信息（老品优化时使用）
  let myListing: { title?: string; bullets?: string[]; description?: string } | undefined;
  if (currentProject.value.scenario_type === 'optimize' && currentProject.value.my_title) {
    myListing = {
      title: currentProject.value.my_title || undefined,
      bullets: currentProject.value.my_bullets ? JSON.parse(currentProject.value.my_bullets) : undefined,
      description: currentProject.value.my_description || undefined,
    };
  }

  // 构建 prompt
  const prompt = buildOptimizationPrompt(
    reviewInsights.value,
    listingAnalysis.value,
    keywords,
    currentProject.value.scenario_type as 'new' | 'optimize',
    currentProject.value.marketplace,
    myListing,
    parsedProductInfo
  );

  // 调用 AI
  abortController.value = new AbortController();
  let fullResponse = '';

  for await (const chunk of chatStream(
    [{ role: 'user', content: prompt }],
    {
      provider: selectedProvider.value,
      model: selectedModel.value,
      maxTokens: 8192,
      signal: abortController.value.signal,
    }
  )) {
    if (chunk.done) break;
    fullResponse += chunk.content;
    // 画布模式显示任务标签
    if (analysisDisplayMode.value === 'canvas') {
      streamingContent.value = '[优化建议] ' + fullResponse.slice(-300);
    } else {
      streamingContent.value = fullResponse;
    }
  }

  // 解析结果
  const parsed = parseAIResponse<OptimizationResult>(fullResponse);
  if (!parsed) {
    throw new Error('优化建议结果格式错误');
  }

  optimizationResult.value = parsed;

  // 保存到数据库
  await scSaveAnalysis(
    currentProject.value.id,
    'optimization',
    JSON.stringify(parsed),
    selectedProvider.value,
    selectedModel.value
  );
}

// 停止分析
function handleStopAnalysis() {
  // 经典模式：中断单个控制器
  if (abortController.value) {
    abortController.value.abort();
  }
  // 画布模式：中断所有并行控制器
  for (const controller of parallelAbortControllers.value) {
    controller.abort();
  }
  parallelAbortControllers.value = [];
}

// ==================== 工具函数 ====================

// 复制到剪贴板
async function copyToClipboard(text: string, label?: string) {
  try {
    await navigator.clipboard.writeText(text);
    ElMessage.success(label ? `${label} 已复制` : '已复制到剪贴板');
  } catch (err) {
    console.error('复制失败:', err);
    ElMessage.error('复制失败');
  }
}

// 复制标题
function copyTitle(content: string) {
  copyToClipboard(content, '标题');
}

// 复制单条五点
function copyBullet(content: string, index: number) {
  copyToClipboard(content, `五点${index}`);
}

// 复制全部五点
function copyAllBullets() {
  if (!optimizationResult.value?.bullet_suggestions?.length) return;
  const allBullets = optimizationResult.value.bullet_suggestions
    .map(b => b.content)
    .join('\n\n');
  copyToClipboard(allBullets, '全部五点');
}

// 复制后台关键词
function copyBackendKeywords() {
  if (!optimizationResult.value?.backend_keywords?.length) return;
  const keywords = optimizationResult.value.backend_keywords
    .filter(k => k && k.keyword)
    .map(k => k.keyword)
    .join(', ');
  copyToClipboard(keywords, '后台关键词');
}

function formatDate(dateStr: string): string {
  if (!dateStr) return '';
  // SQLite CURRENT_TIMESTAMP 返回 UTC 时间，格式如 "2026-01-08 08:38:00"
  // 如果没有时区标识，添加 Z 表示 UTC
  let date: Date;
  if (dateStr.includes('T') || dateStr.includes('Z') || dateStr.includes('+')) {
    // 已经是 ISO 格式或包含时区信息
    date = new Date(dateStr);
  } else {
    // SQLite 格式 "YYYY-MM-DD HH:MM:SS"，是 UTC 时间
    date = new Date(dateStr.replace(' ', 'T') + 'Z');
  }
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric', month: '2-digit', day: '2-digit',
    hour: '2-digit', minute: '2-digit',
    timeZone: 'Asia/Shanghai',
  });
}

function parseBullets(bulletsJson: string | null): string[] {
  if (!bulletsJson) return [];
  try {
    const parsed = JSON.parse(bulletsJson);
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

// 格式化上架时间：解析日期并计算距今天数
function formatLaunchDate(dateStr: string): string {
  if (!dateStr) return '-';

  let date: Date | null = null;

  // 尝试解析不同格式的日期
  // 格式1: 2025/8/8 或 2025-8-8 (日语/通用)
  const slashMatch = dateStr.match(/(\d{4})[\/\-年](\d{1,2})[\/\-月](\d{1,2})/);
  if (slashMatch) {
    date = new Date(parseInt(slashMatch[1]), parseInt(slashMatch[2]) - 1, parseInt(slashMatch[3]));
  }

  // 格式2: October 28, 2019 或 January 1, 2020 (英语)
  if (!date) {
    const englishMatch = dateStr.match(/([A-Za-z]+)\.?\s*(\d{1,2}),?\s*(\d{4})/);
    if (englishMatch) {
      const months: Record<string, number> = {
        'january': 0, 'february': 1, 'march': 2, 'april': 3, 'may': 4, 'june': 5,
        'july': 6, 'august': 7, 'september': 8, 'october': 9, 'november': 10, 'december': 11
      };
      const monthNum = months[englishMatch[1].toLowerCase()];
      if (monthNum !== undefined) {
        date = new Date(parseInt(englishMatch[3]), monthNum, parseInt(englishMatch[2]));
      }
    }
  }

  // 格式3: 28 Oct. 2019 或 1 Jan 2020 (英式) 或 22 juin 2020 (法语) 或 17 novembre 2022
  if (!date) {
    const britishMatch = dateStr.match(/(\d{1,2})\s+([A-Za-zéèàùâêîôûäëïöü]+)\.?\s*(\d{4})/);
    if (britishMatch) {
      const months: Record<string, number> = {
        // 英语
        'jan': 0, 'feb': 1, 'mar': 2, 'apr': 3, 'may': 4, 'jun': 5,
        'jul': 6, 'aug': 7, 'sep': 8, 'oct': 9, 'nov': 10, 'dec': 11,
        'january': 0, 'february': 1, 'march': 2, 'april': 3, 'june': 5,
        'july': 6, 'august': 7, 'september': 8, 'october': 9, 'november': 10, 'december': 11,
        // 法语
        'janvier': 0, 'février': 1, 'fevrier': 1, 'mars': 2, 'avril': 3, 'mai': 4, 'juin': 5,
        'juillet': 6, 'août': 7, 'aout': 7, 'septembre': 8, 'octobre': 9, 'novembre': 10, 'décembre': 11, 'decembre': 11,
        // 意大利语
        'gennaio': 0, 'febbraio': 1, 'marzo': 2, 'aprile': 3, 'maggio': 4, 'giugno': 5,
        'luglio': 6, 'agosto': 7, 'settembre': 8, 'ottobre': 9, 'dicembre': 11,
        // 西班牙语 (marzo和agosto与意大利语相同，已在上面定义)
        'enero': 0, 'febrero': 1, 'abril': 3, 'mayo': 4, 'junio': 5,
        'julio': 6, 'septiembre': 8, 'octubre': 9, 'noviembre': 10, 'diciembre': 11,
      };
      const monthNum = months[britishMatch[2].toLowerCase()];
      if (monthNum !== undefined) {
        date = new Date(parseInt(britishMatch[3]), monthNum, parseInt(britishMatch[1]));
      }
    }
  }

  // 格式4: 1. Januar 2020 (德语)
  if (!date) {
    const germanMatch = dateStr.match(/(\d{1,2})\.?\s*([A-Za-zäöüÄÖÜß]+)\.?\s*(\d{4})/);
    if (germanMatch) {
      const months: Record<string, number> = {
        'januar': 0, 'februar': 1, 'märz': 2, 'april': 3, 'mai': 4, 'juni': 5,
        'juli': 6, 'august': 7, 'september': 8, 'oktober': 9, 'november': 10, 'dezember': 11
      };
      const monthNum = months[germanMatch[2].toLowerCase()];
      if (monthNum !== undefined) {
        date = new Date(parseInt(germanMatch[3]), monthNum, parseInt(germanMatch[1]));
      }
    }
  }

  if (!date || isNaN(date.getTime())) {
    return dateStr; // 无法解析，返回原始字符串
  }

  // 计算距今天数
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  date.setHours(0, 0, 0, 0);
  const diffTime = today.getTime() - date.getTime();
  const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24));

  // 格式化为 YYYY-M-D（X天）
  const formatted = `${date.getFullYear()}-${date.getMonth() + 1}-${date.getDate()}`;
  return `${formatted}（${diffDays}天）`;
}

const statusLabels: Record<string, { text: string; type: 'info' | 'warning' | 'primary' | 'success' }> = {
  draft: { text: '草稿', type: 'info' },
  collecting: { text: '采集中', type: 'warning' },
  analyzing: { text: '分析中', type: 'primary' },
  completed: { text: '已完成', type: 'success' },
};

// ==================== 导出 Excel ====================
async function exportToExcel() {
  if (!optimizationResult.value) {
    ElMessage.warning('没有可导出的数据');
    return;
  }

  const result = optimizationResult.value;
  const projectName = currentProject.value?.name || '智能文案';
  const defaultFileName = `${projectName}_优化建议_${new Date().toISOString().slice(0, 10)}.xlsx`;

  // 弹出保存对话框
  const filePath = await save({
    defaultPath: defaultFileName,
    filters: [{ name: 'Excel', extensions: ['xlsx'] }]
  });

  if (!filePath) {
    return; // 用户取消
  }

  const wb = XLSX.utils.book_new();

  // Sheet 1: 标题建议
  if (Array.isArray(result.title_suggestions) && result.title_suggestions.length) {
    const titleData = result.title_suggestions.map((t, i) => ({
      '版本': t.version || (i + 1),
      '标题内容': t.content,
      '关键词理由': Array.isArray(t.reasons) ? t.reasons.map(r => `${r.word}: ${r.reason}`).join('\n') : ''
    }));
    const titleSheet = XLSX.utils.json_to_sheet(titleData);
    XLSX.utils.book_append_sheet(wb, titleSheet, '标题建议');
  }

  // Sheet 2: 五点建议
  if (Array.isArray(result.bullet_suggestions) && result.bullet_suggestions.length) {
    const bulletData = result.bullet_suggestions.map(b => ({
      '序号': b.index,
      '主题': b.focus,
      '内容': b.content,
      '埋入关键词': Array.isArray(b.embedded_keywords) ? b.embedded_keywords.join(', ') : '',
      '理由': b.reason
    }));
    const bulletSheet = XLSX.utils.json_to_sheet(bulletData);
    XLSX.utils.book_append_sheet(wb, bulletSheet, '五点建议');
  }

  // Sheet 3: 后台关键词
  if (Array.isArray(result.backend_keywords) && result.backend_keywords.length) {
    const keywordData = result.backend_keywords.map(k => ({
      '关键词': k.keyword,
      '搜索量': k.search_volume || '-',
      '选择原因': k.reason
    }));
    const keywordSheet = XLSX.utils.json_to_sheet(keywordData);
    XLSX.utils.book_append_sheet(wb, keywordSheet, '后台关键词');
  }

  // Sheet 4: 商品描述
  if (Array.isArray(result.description_suggestions) && result.description_suggestions.length) {
    const descData = result.description_suggestions.map(d => ({
      '版本': d.version,
      '商品描述': d.content,
      '结构': d.structure,
      '埋入关键词': Array.isArray(d.embedded_keywords) ? d.embedded_keywords.join(', ') : '',
      '突出卖点': Array.isArray(d.highlights) ? d.highlights.join(', ') : '',
      '理由': d.reason
    }));
    const descSheet = XLSX.utils.json_to_sheet(descData);
    XLSX.utils.book_append_sheet(wb, descSheet, '商品描述');
  }

  // Sheet 5-7: A+ 内容建议
  if (result.aplus_suggestions) {
    const aplus = result.aplus_suggestions;

    // 主图文案
    if (Array.isArray(aplus.main_image?.key_points) && aplus.main_image.key_points.length) {
      const mainImageData = aplus.main_image.key_points.map((point, i) => ({
        '序号': i + 1,
        '核心卖点文案': point
      }));
      const mainImageSheet = XLSX.utils.json_to_sheet(mainImageData);
      XLSX.utils.book_append_sheet(wb, mainImageSheet, '主图文案');
    }

    // 辅图建议
    if (Array.isArray(aplus.secondary_images) && aplus.secondary_images.length) {
      const secondaryData = aplus.secondary_images.map(img => ({
        '图片序号': img.index,
        '主题': img.theme,
        '文案建议': img.copy_suggestion
      }));
      const secondarySheet = XLSX.utils.json_to_sheet(secondaryData);
      XLSX.utils.book_append_sheet(wb, secondarySheet, '辅图建议');
    }

    // A+ 模块
    if (Array.isArray(aplus.module_recommendations) && aplus.module_recommendations.length) {
      const moduleData = aplus.module_recommendations.map(mod => ({
        '模块名称': mod.module_name,
        '模块类型': mod.module_type,
        '内容要点': Array.isArray(mod.content_points) ? mod.content_points.join('\n') : ''
      }));
      const moduleSheet = XLSX.utils.json_to_sheet(moduleData);
      XLSX.utils.book_append_sheet(wb, moduleSheet, 'A+模块');
    }
  }

  // 写入文件
  try {
    const xlsxData = XLSX.write(wb, { bookType: 'xlsx', type: 'array' });
    await writeFile(filePath, new Uint8Array(xlsxData));
    ElMessage.success(`导出成功: ${filePath}`);
  } catch (err) {
    console.error('导出失败:', err);
    ElMessage.error('导出失败');
  }
}

onMounted(async () => {
  await loadProjects();

  // 检查是否有保存的项目ID，恢复视图状态
  const savedProjectId = localStorage.getItem('sc_current_project_id');
  if (savedProjectId) {
    const projectId = parseInt(savedProjectId);
    const project = projects.value.find(p => p.id === projectId);
    if (project) {
      enterProject(project);
    } else {
      // 项目不存在，清除保存的ID
      localStorage.removeItem('sc_current_project_id');
    }
  }
});
</script>

<template>
  <div class="smart-copy-container">
    <!-- ==================== 项目列表视图 ==================== -->
    <template v-if="viewMode === 'list'">
      <div class="smart-copy-header">
        <div class="header-content">
          <h2>智能文案</h2>
          <p class="subtitle">基于竞品分析，打造符合 A9、COSMO、Rufus 算法的优质 Listing</p>
        </div>
        <el-button circle size="small" class="help-btn" @click="emit('showHelp', 'smartcopy')" title="查看帮助">
          <el-icon><QuestionFilled /></el-icon>
        </el-button>
      </div>

      <!-- 场景选择 -->
      <div class="scenario-selector">
        <div class="scenario-card" :class="{ active: scenarioType === 'new' }" @click="scenarioType = 'new'">
          <div class="card-content">
            <div class="scenario-icon-wrapper blue-theme">
              <img :src="iconRocket" class="scenario-img" alt="New Product" />
            </div>
            <div class="text-content">
              <div class="scenario-header">
                <div class="scenario-title">新品打造</div>
                <div class="scenario-badge" v-if="scenarioType === 'new'"><el-icon><Select /></el-icon></div>
              </div>
              <div class="scenario-desc">从零开始创建全新的 Listing</div>
              <div class="scenario-status">
                 <el-tag size="small" type="info" effect="plain" round>{{ projects.filter(p => p.scenario_type === 'new').length }} 个项目</el-tag>
              </div>
            </div>
          </div>
        </div>

        <div class="scenario-card" :class="{ active: scenarioType === 'optimize' }" @click="scenarioType = 'optimize'">
          <div class="card-content">
             <div class="scenario-icon-wrapper green-theme">
               <img :src="iconChart" class="scenario-img" alt="Optimization" />
            </div>
            <div class="text-content">
              <div class="scenario-header">
                <div class="scenario-title">老品优化</div>
                <div class="scenario-badge" v-if="scenarioType === 'optimize'"><el-icon><Select /></el-icon></div>
              </div>
              <div class="scenario-desc">优化现有 Listing，提升转化和排名</div>
              <div class="scenario-status">
                 <el-tag size="small" type="info" effect="plain" round>{{ projects.filter(p => p.scenario_type === 'optimize').length }} 个项目</el-tag>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 项目列表 -->
      <div class="projects-section">
        <div class="section-header">
          <span class="section-title">{{ scenarioType === 'new' ? '新品打造' : '老品优化' }}项目</span>
          <el-button type="primary" size="small" @click="handleCreateProject">
            <el-icon><Plus /></el-icon>新建项目
          </el-button>
        </div>

        <!-- 搜索和排序工具栏 -->
        <div class="project-toolbar">
          <el-input
            v-model="projectSearch"
            placeholder="搜索项目名称或 ASIN..."
            :prefix-icon="Search"
            clearable
            class="search-input"
          />
          <el-select v-model="projectSort" class="sort-select">
            <el-option value="updated" label="最近更新" />
            <el-option value="created" label="创建时间" />
            <el-option value="name_asc" label="名称 A-Z" />
            <el-option value="name_desc" label="名称 Z-A" />
          </el-select>
          <span class="project-count">{{ filteredProjects.length }} 个项目</span>
        </div>

        <div v-if="loading" class="loading-state"><el-skeleton :rows="3" animated /></div>
        <div v-else-if="filteredProjects.length === 0 && !projectSearch" class="empty-state">
          <el-empty :description="`暂无${scenarioType === 'new' ? '新品打造' : '老品优化'}项目`">
            <el-button type="primary" @click="handleCreateProject"><el-icon><Plus /></el-icon>创建第一个项目</el-button>
          </el-empty>
        </div>
        <div v-else-if="filteredProjects.length === 0 && projectSearch" class="empty-state">
          <el-empty description="未找到匹配的项目">
            <el-button @click="projectSearch = ''">清除搜索</el-button>
          </el-empty>
        </div>
        <div v-else class="project-grid">
          <div v-for="project in filteredProjects" :key="project.id" class="project-card" @click="enterProject(project)">
            <div class="card-header">
              <span class="project-name">{{ project.name }}</span>
              <div class="more-btn" @click.stop>
                <el-dropdown trigger="click">
                  <el-icon class="more-icon"><MoreFilled /></el-icon>
                  <template #dropdown>
                    <el-dropdown-menu>
                      <el-dropdown-item @click="handleEditProject(project)">编辑</el-dropdown-item>
                      <el-dropdown-item @click="handleDeleteProject(project)" divided style="color: var(--el-color-danger)">删除</el-dropdown-item>
                    </el-dropdown-menu>
                  </template>
                </el-dropdown>
              </div>
            </div>
            <div class="card-tags">
              <el-tag size="small" type="info">{{ getCountryLabel(project.marketplace) }}</el-tag>
              <el-tag :type="statusLabels[project.status].type" size="small">{{ statusLabels[project.status].text }}</el-tag>
            </div>
            <div v-if="project.my_asin" class="card-asin">
              ASIN: {{ project.my_asin }}
            </div>
            <div class="card-stats">
              <span class="stat">{{ project.competitor_count || 0 }} 个竞品</span>
              <span class="stat-time">{{ formatDate(project.updated_at) }}</span>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- ==================== 项目详情视图 ==================== -->
    <template v-else-if="viewMode === 'detail' && currentProject">
      <div class="detail-header">
        <el-button text @click="backToList"><el-icon><Back /></el-icon>返回列表</el-button>
        <div class="detail-title">
          <h2>{{ currentProject.name }}</h2>
          <el-tag :type="statusLabels[currentProject.status].type" size="small">{{ statusLabels[currentProject.status].text }}</el-tag>
        </div>
        <div class="detail-meta">
          <span>{{ currentProject.scenario_type === 'new' ? '新品打造' : '老品优化' }}</span>
          <span>·</span>
          <span>{{ getCountryLabel(currentProject.marketplace) }}</span>
          <span v-if="currentProject.my_asin">· 我的 ASIN: {{ currentProject.my_asin }}</span>
        </div>
        <!-- 关联产品信息 -->
        <div v-if="linkedProduct" class="linked-product-info">
          <el-tag type="success" size="small">
            📊 已关联: {{ linkedProduct.name }}
            <span v-if="linkedKeywordCount > 0"> ({{ linkedKeywordCount }} 个关键词)</span>
          </el-tag>
        </div>
        <div v-else-if="currentProject.product_id === null" class="linked-product-info">
          <el-tag type="info" size="small">💡 未关联产品（无法使用关键词数据进行 AI 分析）</el-tag>
        </div>
      </div>

      <!-- ==================== 我的现有文案（仅老品优化显示）==================== -->
      <div v-if="currentProject.scenario_type === 'optimize' && currentProject.my_asin" class="my-listing-section">
        <div class="section-header">
          <span class="section-title">
            📝 我的现有文案
            <el-tag v-if="currentProject.my_title" type="success" size="small" style="margin-left: 8px;">已获取</el-tag>
            <el-tag v-else type="warning" size="small" style="margin-left: 8px;">待获取</el-tag>
          </span>
          <el-button
            type="primary"
            size="small"
            :loading="fetchingMyListing"
            @click="handleFetchMyListing"
          >
            {{ currentProject.my_title ? '重新获取' : '获取文案' }}
          </el-button>
        </div>

        <!-- 已获取的文案展示 -->
        <div v-if="currentProject.my_title" class="my-listing-content">
          <div class="listing-item">
            <div class="listing-label">标题</div>
            <div class="listing-value">{{ currentProject.my_title }}</div>
          </div>
          <div v-if="currentProject.my_bullets" class="listing-item">
            <div class="listing-label">五点描述</div>
            <div class="listing-value">
              <ul class="bullet-list">
                <li v-for="(bullet, idx) in parseBullets(currentProject.my_bullets)" :key="idx">{{ bullet }}</li>
              </ul>
            </div>
          </div>
          <div v-if="currentProject.my_description" class="listing-item">
            <div class="listing-label">商品描述</div>
            <div class="listing-value description-text">{{ currentProject.my_description }}</div>
          </div>
          <div class="fetch-time" v-if="currentProject.my_listing_fetched_at">
            获取时间: {{ formatDate(currentProject.my_listing_fetched_at) }}
          </div>
        </div>
        <div v-else class="my-listing-empty">
          <el-text type="info">点击"获取文案"按钮，自动获取 {{ currentProject.my_asin }} 的现有 Listing 信息</el-text>
        </div>
      </div>

      <!-- ==================== 我的产品信息（仅新品打造显示）==================== -->
      <div v-if="currentProject.scenario_type === 'new'" class="my-product-section">
        <div class="section-header" @click="showMyProductForm = !showMyProductForm">
          <span class="section-title clickable">
            📦 我的产品信息
            <el-tag v-if="currentProject.my_product_info" type="success" size="small" style="margin-left: 8px;">已填写</el-tag>
            <el-tag v-else type="warning" size="small" style="margin-left: 8px;">必填</el-tag>
            <el-icon class="expand-icon" :class="{ expanded: showMyProductForm }"><ArrowRight /></el-icon>
          </span>
        </div>

        <template v-if="showMyProductForm">
          <div class="my-product-form">
            <el-form label-width="100px" label-position="left">
              <!-- 品牌名称 -->
              <el-form-item label="品牌名称" required>
                <el-input v-model="myProductInfo.brand_name" placeholder="如: XYZ Brand" maxlength="50" />
              </el-form-item>

              <!-- 产品名称 -->
              <el-form-item label="产品名称" required>
                <el-input v-model="myProductInfo.product_name" placeholder="如: 便携式榨汁机" maxlength="100" />
              </el-form-item>

              <!-- 核心卖点 -->
              <el-form-item label="核心卖点" required>
                <div class="key-features-input">
                  <el-input
                    v-model="newFeature"
                    placeholder="输入卖点后按回车添加（1-5条）"
                    maxlength="100"
                    @keyup.enter="addKeyFeature"
                  >
                    <template #append>
                      <el-button @click="addKeyFeature" :disabled="myProductInfo.key_features.length >= 5">
                        <el-icon><Plus /></el-icon>
                      </el-button>
                    </template>
                  </el-input>
                  <div v-if="myProductInfo.key_features.length > 0" class="features-list">
                    <el-tag
                      v-for="(feature, index) in myProductInfo.key_features"
                      :key="index"
                      closable
                      type="primary"
                      @close="removeKeyFeature(index)"
                    >
                      {{ feature }}
                    </el-tag>
                  </div>
                  <div class="form-tip">{{ myProductInfo.key_features.length }}/5 条卖点</div>
                </div>
              </el-form-item>

              <!-- 差异化特点 -->
              <el-form-item label="差异化特点">
                <el-input
                  v-model="myProductInfo.differentiators"
                  type="textarea"
                  :rows="2"
                  placeholder="与竞品的主要区别，如: 比竞品多50ml容量，续航更久"
                  maxlength="500"
                />
              </el-form-item>

              <!-- 规格参数 -->
              <el-form-item label="规格参数">
                <el-input
                  v-model="myProductInfo.specifications"
                  type="textarea"
                  :rows="2"
                  placeholder="产品尺寸、容量、功率等，如: 350ml容量, 350W, 重量280g"
                  maxlength="500"
                />
              </el-form-item>

              <!-- 目标人群 -->
              <el-form-item label="目标人群">
                <el-input
                  v-model="myProductInfo.target_audience"
                  placeholder="如: 户外运动爱好者、上班族"
                  maxlength="200"
                />
              </el-form-item>

              <!-- 包装配件 -->
              <el-form-item label="包装配件">
                <el-input
                  v-model="myProductInfo.package_contents"
                  placeholder="如: 榨汁杯x1, USB-C充电线x1, 清洁刷x1"
                  maxlength="300"
                />
              </el-form-item>

              <!-- 补充说明 -->
              <el-form-item label="补充说明">
                <el-input
                  v-model="myProductInfo.additional_notes"
                  type="textarea"
                  :rows="2"
                  placeholder="其他需要强调的信息，如: FDA认证，食品级材质"
                  maxlength="500"
                />
              </el-form-item>

              <!-- 保存按钮 -->
              <el-form-item>
                <el-button type="primary" :loading="savingMyProduct" @click="saveMyProductInfo">
                  保存产品信息
                </el-button>
              </el-form-item>
            </el-form>
          </div>
        </template>

        <!-- 未展开时的简要显示 -->
        <div v-if="!showMyProductForm && currentProject.my_product_info" class="my-product-summary">
          <span class="summary-label">品牌:</span> {{ myProductInfo.brand_name }}
          <span class="summary-divider">|</span>
          <span class="summary-label">产品:</span> {{ myProductInfo.product_name }}
          <span class="summary-divider">|</span>
          <span class="summary-label">卖点:</span> {{ myProductInfo.key_features.length }}条
        </div>
      </div>

      <!-- 竞品管理 -->
      <div class="competitors-section">
        <div class="section-header">
          <span class="section-title">竞品列表 ({{ competitors.length }}/5)</span>
          <div class="header-actions">
            <el-button
              v-if="competitors.length > 0 && competitors.some(c => !c.fetched_at)"
              type="success"
              size="small"
              :loading="fetchingCompetitorIds.size > 0"
              @click="handleFetchAllListings"
            >
              <el-icon><Refresh /></el-icon>批量获取
            </el-button>
            <el-button
              v-if="competitors.length > 0 && competitors.every(c => c.fetched_at)"
              type="warning"
              size="small"
              :loading="fetchingCompetitorIds.size > 0"
              @click="handleRefreshAllListings"
            >
              <el-icon><Refresh /></el-icon>一键刷新
            </el-button>
            <el-button type="primary" size="small" :disabled="competitors.length >= 5" @click="handleAddCompetitor">
              <el-icon><Plus /></el-icon>添加竞品
            </el-button>
          </div>
        </div>

        <div v-if="loadingCompetitors" class="loading-state"><el-skeleton :rows="2" animated /></div>
        <div v-else-if="competitors.length === 0" class="empty-state">
          <el-empty description="暂无竞品，点击上方按钮添加">
            <el-button type="primary" @click="handleAddCompetitor"><el-icon><Plus /></el-icon>添加第一个竞品</el-button>
          </el-empty>
        </div>
        <!-- 横向对比表格 -->
        <div v-else class="compare-table-wrapper">
          <table class="compare-table">
            <!-- 表头：ASIN + 操作 -->
            <thead>
              <tr>
                <th class="label-col">属性</th>
                <th v-for="comp in competitors" :key="comp.id" class="data-col">
                  <div class="asin-header">
                    <span class="asin-text">{{ comp.asin }}</span>
                    <div class="asin-actions">
                      <el-button
                        size="small"
                        :type="comp.fetched_at ? 'default' : 'primary'"
                        :loading="fetchingCompetitorIds.has(comp.id)"
                        :title="comp.fetched_at ? '获取最新竞品信息' : '获取竞品信息'"
                        @click="handleFetchListing(comp)"
                      >
                        {{ fetchingCompetitorIds.has(comp.id) ? '...' : (comp.fetched_at ? '刷新' : '获取') }}
                      </el-button>
                      <el-button size="small" type="danger" text title="删除竞品" @click="handleDeleteCompetitor(comp)">
                        <el-icon><Delete /></el-icon>
                      </el-button>
                    </div>
                  </div>
                </th>
              </tr>
            </thead>
            <tbody>
              <!-- 图片行 -->
              <tr class="image-row">
                <td class="label-col">图片</td>
                <td v-for="comp in competitors" :key="comp.id" class="data-col image-cell">
                  <template v-if="comp.image_url">
                    <img :src="comp.image_url" :alt="comp.asin" class="product-image" />
                  </template>
                  <span v-else class="pending-text">-</span>
                </td>
              </tr>
              <!-- 标题行（重点对比） -->
              <tr class="title-row">
                <td class="label-col">标题</td>
                <td v-for="comp in competitors" :key="comp.id" class="data-col title-cell">
                  <template v-if="comp.fetched_at">
                    {{ comp.title || '(无标题)' }}
                  </template>
                  <span v-else class="pending-text">待获取</span>
                </td>
              </tr>
              <!-- 类型 -->
              <tr>
                <td class="label-col">类型</td>
                <td v-for="comp in competitors" :key="comp.id" class="data-col">
                  <el-select
                    :model-value="comp.competitor_type"
                    size="small"
                    style="width: 120px;"
                    @change="(val: 'top' | 'direct' | 'rising') => handleTypeChange(comp, val)"
                  >
                    <el-option v-for="opt in COMPETITOR_TYPE_OPTIONS" :key="opt.value" :value="opt.value" :label="opt.label">
                      <span :style="{ color: opt.color }">{{ opt.label }}</span>
                    </el-option>
                  </el-select>
                </td>
              </tr>
              <!-- 价格 -->
              <tr>
                <td class="label-col">价格</td>
                <td v-for="comp in competitors" :key="comp.id" class="data-col">
                  <span v-if="comp.price" class="price-value">{{ comp.price }}</span>
                  <span v-else class="pending-text">-</span>
                </td>
              </tr>
              <!-- 评论/星级 -->
              <tr>
                <td class="label-col">评论/星级</td>
                <td v-for="comp in competitors" :key="comp.id" class="data-col">
                  <div v-if="comp.rating || comp.review_count" class="rating-cell-inline">
                    <template v-if="comp.rating">
                      <span class="stars-container">
                        <span class="star-filled">{{ '★'.repeat(Math.floor(Number(comp.rating))) }}</span><span v-if="Number(comp.rating) % 1 >= 0.3" class="star-half">★</span><span class="star-empty">{{ '★'.repeat(5 - Math.floor(Number(comp.rating)) - (Number(comp.rating) % 1 >= 0.3 ? 1 : 0)) }}</span>
                      </span>
                      <span class="rating-num">{{ Number(comp.rating).toFixed(1) }}</span>
                    </template>
                    <span v-if="comp.review_count" class="reviews-count">({{ comp.review_count.toLocaleString() }})</span>
                  </div>
                  <span v-else class="pending-text">-</span>
                </td>
              </tr>
              <!-- BSR -->
              <tr>
                <td class="label-col">BSR</td>
                <td v-for="comp in competitors" :key="comp.id" class="data-col">
                  <div v-if="comp.bsr_rank" class="bsr-cell">
                    <span class="bsr-rank">{{ comp.bsr_rank.split(' in ')[0] }}</span>
                    <span v-if="comp.bsr_rank.includes(' in ')" class="bsr-category">
                      in {{ comp.bsr_rank.split(' in ').slice(1).join(' in ') }}
                    </span>
                  </div>
                  <span v-else class="pending-text">-</span>
                </td>
              </tr>
              <!-- 上架时间 -->
              <tr>
                <td class="label-col">上架时间</td>
                <td v-for="comp in competitors" :key="comp.id" class="data-col">
                  <span v-if="comp.date_first_available" class="date-cell">{{ formatLaunchDate(comp.date_first_available) }}</span>
                  <span v-else class="pending-text">-</span>
                </td>
              </tr>
              <!-- 五点描述（可折叠） -->
              <tr class="bullets-header-row" @click="showBullets = !showBullets">
                <td class="label-col clickable">
                  <span>五点描述</span>
                  <el-icon class="expand-icon" :class="{ expanded: showBullets }"><ArrowRight /></el-icon>
                </td>
                <td v-for="comp in competitors" :key="comp.id" class="data-col">
                  <span v-if="comp.bullets" class="bullet-count">{{ parseBullets(comp.bullets).length }} 条</span>
                  <span v-else class="pending-text">-</span>
                </td>
              </tr>
              <!-- 展开的五点描述 -->
              <template v-if="showBullets">
                <tr v-for="idx in maxBulletCount" :key="'bullet-' + idx" class="bullet-row">
                  <td class="label-col bullet-label">五点{{ idx }}</td>
                  <td v-for="comp in competitors" :key="comp.id" class="data-col bullet-cell">
                    <template v-if="parseBullets(comp.bullets)[idx - 1]">
                      {{ parseBullets(comp.bullets)[idx - 1] }}
                    </template>
                    <span v-else class="pending-text">-</span>
                  </td>
                </tr>
              </template>
            </tbody>
          </table>
        </div>

        <!-- 下一步提示 -->
        <div v-if="competitors.length > 0" class="next-step-hint">
          <el-alert
            v-if="competitors.some(c => !c.fetched_at)"
            type="info"
            :closable="false"
            show-icon
          >
            <template #title>
              下一步：点击"获取信息"或"批量获取"按钮，爬取竞品的标题、五点、描述等数据
            </template>
          </el-alert>
        </div>
      </div>

      <!-- ==================== 评论分析区域 ==================== -->
      <div v-if="competitors.length > 0 && competitors.some(c => c.fetched_at)" class="reviews-section">
        <div class="section-header" @click="showReviewsSection = !showReviewsSection">
          <span class="section-title clickable">
            💬 评论分析
            <el-icon class="expand-icon" :class="{ expanded: showReviewsSection }"><ArrowRight /></el-icon>
          </span>
          <div class="header-actions" @click.stop>
            <el-button
              type="primary"
              size="small"
              :loading="fetchingReviewIds.size > 0"
              @click="handleFetchAllReviews"
            >
              <el-icon><Refresh /></el-icon>获取所有评论
            </el-button>
          </div>
        </div>

        <!-- 评论统计表格 -->
        <template v-if="showReviewsSection">
          <div class="reviews-summary-table">
            <table class="compare-table">
              <thead>
                <tr>
                  <th class="label-col">评论统计</th>
                  <th v-for="comp in competitors" :key="comp.id" class="data-col">
                    <span class="asin-text">{{ comp.asin }}</span>
                  </th>
                </tr>
              </thead>
              <tbody>
                <!-- 已获取评论数 -->
                <tr>
                  <td class="label-col">已获取评论</td>
                  <td v-for="comp in competitors" :key="comp.id" class="data-col">
                    <div class="review-stats-cell">
                      <span>{{ getReviewStats(comp.id) }}</span>
                      <el-button
                        size="small"
                        type="primary"
                        text
                        :loading="fetchingReviewIds.has(comp.id)"
                        @click="handleFetchReviews(comp)"
                      >
                        {{ fetchingReviewIds.has(comp.id) ? '...' : '获取' }}
                      </el-button>
                    </div>
                  </td>
                </tr>
                <!-- 星级分布 -->
                <tr v-for="star in [5, 4, 3, 2, 1]" :key="star" class="star-row">
                  <td class="label-col">{{ star }}星</td>
                  <td v-for="comp in competitors" :key="comp.id" class="data-col">
                    <template v-if="reviewSummaries.get(comp.id)?.total">
                      <div class="star-bar">
                        <div
                          class="star-fill"
                          :class="'star-' + star"
                          :style="{ width: getStarDistribution(comp.id).find(s => s.star === star)?.percent + '%' }"
                        ></div>
                      </div>
                      <span class="star-count">
                        {{ getStarDistribution(comp.id).find(s => s.star === star)?.count || 0 }}
                        ({{ getStarDistribution(comp.id).find(s => s.star === star)?.percent || 0 }}%)
                      </span>
                    </template>
                    <span v-else class="pending-text">-</span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- ==================== AI 分析区域 ==================== -->
          <div class="ai-analysis-section">
            <div class="section-header">
              <span class="section-title">AI 分析</span>
              <div class="ai-settings">
                <!-- 显示模式切换 -->
                <el-tooltip :content="analysisDisplayMode === 'classic' ? '切换到画布模式（并行执行，更快）' : '切换到经典模式'" placement="top">
                  <el-button
                    :icon="analysisDisplayMode === 'classic' ? DataLine : Select"
                    size="small"
                    :disabled="isAnalyzing"
                    @click="analysisDisplayMode = analysisDisplayMode === 'classic' ? 'canvas' : 'classic'"
                  >
                    {{ analysisDisplayMode === 'classic' ? '经典' : '画布' }}
                  </el-button>
                </el-tooltip>
                <span class="setting-label">AI 服务:</span>
                <el-select v-model="selectedProvider" size="small" style="width: 110px" :disabled="isAnalyzing">
                  <el-option v-for="(config, key) in AI_PROVIDERS" :key="key" :label="config.name" :value="key" />
                </el-select>
                <span class="setting-label">模型:</span>
                <el-select v-model="selectedModel" size="small" style="width: 150px" :disabled="isAnalyzing">
                  <el-option v-for="model in availableModels" :key="model" :label="model" :value="model" />
                </el-select>
                <el-button
                  v-if="!isAnalyzing"
                  type="primary"
                  size="small"
                  @click="handleStartAnalysis"
                >
                  {{ analysisStep > 0 ? '重新分析' : '开始分析' }}
                </el-button>
                <el-button v-else type="danger" size="small" @click="handleStopAnalysis">停止</el-button>
              </div>
            </div>

            <!-- 画布模式进度显示 -->
            <AnalysisCanvas
              v-if="isAnalyzing && analysisDisplayMode === 'canvas'"
              :status="analysisCanvasStatus"
              :streaming-content="streamingContent"
            />

            <!-- 经典模式进度显示 -->
            <div v-if="isAnalyzing && analysisDisplayMode === 'classic'" class="analysis-progress">
              <div class="progress-steps">
                <div :class="['step', { active: analysisStep === 1, done: analysisStep > 1 }]">
                  <span class="step-icon">
                    <span v-if="analysisStep > 1" class="icon-done">✓</span>
                    <span v-else-if="analysisStep === 1" class="loading-spinner"></span>
                    <span v-else class="icon-pending">○</span>
                  </span>
                  <span class="step-text">1. 评论洞察</span>
                </div>
                <div :class="['step', { active: analysisStep === 2, done: analysisStep > 2 }]">
                  <span class="step-icon">
                    <span v-if="analysisStep > 2" class="icon-done">✓</span>
                    <span v-else-if="analysisStep === 2" class="loading-spinner"></span>
                    <span v-else class="icon-pending">○</span>
                  </span>
                  <span class="step-text">2. 文案分析</span>
                </div>
                <div :class="['step', { active: analysisStep === 3 }]">
                  <span class="step-icon">
                    <span v-if="optimizationResult" class="icon-done">✓</span>
                    <span v-else-if="analysisStep === 3" class="loading-spinner"></span>
                    <span v-else class="icon-pending">○</span>
                  </span>
                  <span class="step-text">3. 优化建议</span>
                </div>
              </div>

              <!-- 进度条 -->
              <div class="progress-bar">
                <div class="progress-fill" :style="{ width: progressPercent + '%' }"></div>
              </div>

              <div v-if="streamingContent" class="streaming-output">
                <pre>{{ streamingContent.slice(-500) }}</pre>
                <span class="typing-cursor">▋</span>
              </div>
            </div>

            <!-- 错误提示 -->
            <el-alert v-if="analysisError" type="error" :title="analysisError" :closable="false" style="margin-bottom: 16px;" />

            <!-- 分析结果展示 -->
            <div v-if="reviewInsights || listingAnalysis || optimizationResult" class="analysis-results">
              <!-- 评论洞察 -->
              <div v-if="reviewInsights" class="result-card">
                <h4>评论洞察</h4>
                <div class="insights-grid">
                  <div class="insight-group">
                    <div class="insight-label">使用场景</div>
                    <div class="insight-tags">
                      <el-tag v-for="(s, i) in reviewInsights.usage_scenarios" :key="i" type="success" size="small">
                        {{ s.scenario }} ({{ s.source_count }})
                      </el-tag>
                    </div>
                  </div>
                  <div class="insight-group">
                    <div class="insight-label">卖点/爽点</div>
                    <div class="insight-tags">
                      <el-tag v-for="(p, i) in reviewInsights.praise_points" :key="i" type="primary" size="small">
                        {{ p.point }} ({{ p.frequency }})
                      </el-tag>
                    </div>
                  </div>
                  <div class="insight-group">
                    <div class="insight-label">痛点/问题</div>
                    <div class="insight-tags">
                      <el-tag v-for="(p, i) in reviewInsights.pain_points" :key="i" type="danger" size="small">
                        {{ p.point }} ({{ p.frequency }})
                      </el-tag>
                    </div>
                  </div>
                </div>
                <div class="insight-summary">{{ reviewInsights.summary }}</div>
              </div>

              <!-- 优化建议 -->
              <div v-if="optimizationResult" class="result-card">
                <h4>优化建议</h4>

                <!-- 标题建议 -->
                <div v-if="optimizationResult.title_suggestions?.length" class="suggestion-group">
                  <h5>标题建议</h5>
                  <div v-for="(t, i) in optimizationResult.title_suggestions" :key="i" class="suggestion-item">
                    <div class="suggestion-content-wrapper">
                      <div class="suggestion-content">{{ t.content }}</div>
                      <el-button class="copy-btn" size="small" text @click="copyTitle(t.content)">
                        <el-icon><CopyDocument /></el-icon>
                      </el-button>
                    </div>
                    <div class="suggestion-reasons">
                      <span v-for="(r, j) in t.reasons" :key="j" class="reason-tag">
                        <strong>{{ r.word }}</strong>: {{ r.reason }}
                      </span>
                    </div>
                  </div>
                </div>

                <!-- 五点建议 -->
                <div v-if="optimizationResult.bullet_suggestions?.length" class="suggestion-group">
                  <div class="suggestion-group-header">
                    <h5>五点描述建议</h5>
                    <el-button size="small" text @click="copyAllBullets">
                      <el-icon><CopyDocument /></el-icon>复制全部
                    </el-button>
                  </div>
                  <div v-for="(b, i) in optimizationResult.bullet_suggestions" :key="i" class="suggestion-item">
                    <div class="bullet-header">
                      <span class="bullet-index">{{ b.index || (i + 1) }}</span>
                      <el-tag size="small" type="info">{{ b.focus }}</el-tag>
                    </div>
                    <div class="suggestion-content-wrapper">
                      <div class="suggestion-content">{{ b.content }}</div>
                      <el-button class="copy-btn" size="small" text @click="copyBullet(b.content, b.index || (i + 1))">
                        <el-icon><CopyDocument /></el-icon>
                      </el-button>
                    </div>
                    <div v-if="b.embedded_keywords?.length" class="embedded-keywords">
                      <span class="keywords-label">埋入关键词：</span>
                      <el-tag v-for="(kw, ki) in b.embedded_keywords" :key="ki" size="small" type="success">
                        {{ kw }}
                      </el-tag>
                    </div>
                    <div class="suggestion-reason">{{ b.reason }}</div>
                  </div>
                  <!-- 关键词分布总结 -->
                  <div v-if="optimizationResult.keyword_distribution_summary" class="keyword-summary">
                    <strong>关键词分布：</strong>{{ optimizationResult.keyword_distribution_summary }}
                  </div>
                </div>

                <!-- 后台关键词 -->
                <div v-if="optimizationResult.backend_keywords?.length" class="suggestion-group">
                  <div class="suggestion-group-header">
                    <h5>后台关键词建议</h5>
                    <el-button size="small" text @click="copyBackendKeywords">
                      <el-icon><CopyDocument /></el-icon>复制全部
                    </el-button>
                  </div>
                  <div class="keyword-list">
                    <template v-for="(k, i) in optimizationResult.backend_keywords" :key="i">
                      <el-tag v-if="k && k.keyword && String(k.keyword).trim()" size="small">
                        {{ k.keyword }}
                        <span v-if="k.search_volume" class="keyword-volume">({{ k.search_volume }})</span>
                      </el-tag>
                    </template>
                  </div>
                </div>

                <!-- 商品描述建议 -->
                <div v-if="optimizationResult.description_suggestions?.length" class="suggestion-group">
                  <div class="suggestion-header">
                    <h5>商品描述建议</h5>
                  </div>
                  <div v-for="(desc, i) in optimizationResult.description_suggestions" :key="i" class="description-item">
                    <div class="description-header">
                      <el-tag size="small" type="primary">版本 {{ desc.version }}</el-tag>
                      <el-button size="small" text @click="copyToClipboard(desc.content)">
                        <el-icon><CopyDocument /></el-icon>复制
                      </el-button>
                    </div>
                    <div class="description-content">
                      <pre class="description-text">{{ desc.content }}</pre>
                    </div>
                    <div class="description-meta">
                      <div class="meta-row">
                        <span class="meta-label">结构：</span>
                        <span>{{ desc.structure }}</span>
                      </div>
                      <div v-if="desc.embedded_keywords?.length" class="meta-row">
                        <span class="meta-label">埋入关键词：</span>
                        <el-tag v-for="kw in desc.embedded_keywords" :key="kw" size="small" type="info" class="meta-tag">{{ kw }}</el-tag>
                      </div>
                      <div v-if="desc.highlights?.length" class="meta-row">
                        <span class="meta-label">突出卖点：</span>
                        <el-tag v-for="hl in desc.highlights" :key="hl" size="small" type="success" class="meta-tag">{{ hl }}</el-tag>
                      </div>
                      <div class="meta-row reason-row">
                        <span class="meta-label">理由：</span>
                        <span class="reason-text">{{ desc.reason }}</span>
                      </div>
                    </div>
                  </div>
                </div>

                <!-- A+ 内容建议 -->
                <div v-if="optimizationResult.aplus_suggestions" class="suggestion-group aplus-section">
                  <h5>A+ 内容建议</h5>

                  <!-- 主图文案 -->
                  <div v-if="optimizationResult.aplus_suggestions.main_image?.key_points?.length" class="aplus-subsection">
                    <h6>主图核心卖点文案</h6>
                    <ul class="main-image-points">
                      <li v-for="(point, i) in optimizationResult.aplus_suggestions.main_image.key_points" :key="i">
                        {{ point }}
                      </li>
                    </ul>
                  </div>

                  <!-- 辅图建议 -->
                  <div v-if="optimizationResult.aplus_suggestions.secondary_images?.length" class="aplus-subsection">
                    <h6>辅图建议</h6>
                    <div class="secondary-images-list">
                      <div v-for="img in optimizationResult.aplus_suggestions.secondary_images" :key="img.index" class="secondary-image-item">
                        <div class="image-header">
                          <span class="image-index">图{{ img.index }}</span>
                          <el-tag size="small" type="info">{{ img.theme }}</el-tag>
                        </div>
                        <div class="image-copy">{{ img.copy_suggestion }}</div>
                      </div>
                    </div>
                  </div>

                  <!-- A+ 模块推荐 -->
                  <div v-if="optimizationResult.aplus_suggestions.module_recommendations?.length" class="aplus-subsection">
                    <h6>A+ 模块推荐</h6>
                    <div class="module-recommendations">
                      <div v-for="(mod, i) in optimizationResult.aplus_suggestions.module_recommendations" :key="i" class="module-item">
                        <div class="module-header">
                          <el-tag size="small" type="warning">{{ mod.module_name }}</el-tag>
                        </div>
                        <ul class="module-points">
                          <li v-for="(point, j) in mod.content_points" :key="j">{{ point }}</li>
                        </ul>
                      </div>
                    </div>
                  </div>
                </div>

                <!-- 导出按钮 -->
                <div class="export-section">
                  <el-button type="primary" @click="exportToExcel" :disabled="!optimizationResult">
                    导出 Excel
                  </el-button>
                </div>
              </div>
            </div>

            <!-- 未分析提示 -->
            <div v-else-if="!isAnalyzing" class="no-analysis">
              <el-empty description="尚未进行 AI 分析">
                <div class="analysis-hint">
                  确保已获取竞品 Listing 和评论数据后，点击「开始分析」按钮
                </div>
              </el-empty>
            </div>
          </div>
        </template>
      </div>
    </template>

    <!-- ==================== 新建项目弹窗 ==================== -->
    <el-dialog v-model="showCreateDialog" :title="`新建${scenarioType === 'new' ? '新品打造' : '老品优化'}项目`" width="480px" :close-on-click-modal="false">
      <el-form :model="createForm" label-width="100px">
        <el-form-item label="项目名称" required>
          <el-input v-model="createForm.name" placeholder="如：XX产品竞品分析" maxlength="50" show-word-limit />
        </el-form-item>
        <el-form-item label="目标站点" required>
          <el-select v-model="createForm.marketplace" style="width: 100%;">
            <el-option v-for="country in COUNTRY_OPTIONS" :key="country.value" :label="country.label" :value="country.value">
              <div style="display: flex; align-items: center; gap: 8px;">
                <span class="country-flag" v-html="country.flag"></span>
                <span>{{ country.label }}</span>
              </div>
            </el-option>
          </el-select>
        </el-form-item>
        <el-form-item v-if="scenarioType === 'optimize'" label="您的ASIN" required>
          <el-input v-model="createForm.myAsin" placeholder="输入需要优化的产品 ASIN" maxlength="20" />
          <div class="form-tip">输入您需要优化的现有产品 ASIN</div>
        </el-form-item>
        <el-form-item label="关联产品">
          <el-select v-model="createForm.productId" placeholder="选择已导入关键词的产品（可选）" clearable style="width: 100%;">
            <el-option v-for="p in products" :key="p.id" :label="p.name" :value="p.id">
              <span>{{ p.name }}</span>
              <span v-if="p.country" style="color: #999; margin-left: 8px;">({{ p.country }})</span>
            </el-option>
          </el-select>
          <div class="form-tip">关联产品可获取关键词数据供 AI 分析使用</div>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showCreateDialog = false">取消</el-button>
        <el-button type="primary" :loading="creating" @click="confirmCreate">创建</el-button>
      </template>
    </el-dialog>

    <!-- ==================== 编辑项目弹窗 ==================== -->
    <el-dialog v-model="showEditDialog" title="编辑项目" width="480px" :close-on-click-modal="false">
      <el-form :model="editForm" label-width="100px">
        <el-form-item label="项目名称" required>
          <el-input v-model="editForm.name" placeholder="项目名称" maxlength="50" show-word-limit />
        </el-form-item>
        <el-form-item label="目标站点" required>
          <el-select v-model="editForm.marketplace" style="width: 100%;">
            <el-option v-for="country in COUNTRY_OPTIONS" :key="country.value" :label="country.label" :value="country.value">
              <div style="display: flex; align-items: center; gap: 8px;">
                <span class="country-flag" v-html="country.flag"></span>
                <span>{{ country.label }}</span>
              </div>
            </el-option>
          </el-select>
        </el-form-item>
        <el-form-item v-if="editingProject?.scenario_type === 'optimize'" label="您的ASIN">
          <el-input v-model="editForm.myAsin" placeholder="输入需要优化的产品 ASIN" maxlength="20" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showEditDialog = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="confirmEdit">保存</el-button>
      </template>
    </el-dialog>

    <!-- ==================== 添加竞品弹窗 ==================== -->
    <el-dialog v-model="showAddCompetitorDialog" title="添加竞品" width="420px" :close-on-click-modal="false">
      <el-form :model="addCompetitorForm" label-width="80px">
        <el-form-item label="ASIN" required>
          <el-input v-model="addCompetitorForm.asin" placeholder="如：B0XXXXXXXXX" maxlength="10" style="text-transform: uppercase;" />
          <div class="form-tip">输入竞品的 ASIN（B0 开头的 10 位字符）</div>
        </el-form-item>
        <el-form-item label="竞品类型">
          <el-radio-group v-model="addCompetitorForm.competitorType">
            <el-radio v-for="opt in COMPETITOR_TYPE_OPTIONS" :key="opt.value" :value="opt.value">
              <span :style="{ color: opt.color }">{{ opt.label }}</span>
              <span class="type-desc">{{ opt.description }}</span>
            </el-radio>
          </el-radio-group>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showAddCompetitorDialog = false">取消</el-button>
        <el-button type="primary" :loading="addingCompetitor" @click="confirmAddCompetitor">添加</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
/* 国旗样式 */
.country-flag {
  width: 24px;
  height: 16px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 2px;
  overflow: hidden;
  flex-shrink: 0;
}

.country-flag :deep(svg) {
  width: 100%;
  height: 100%;
}

.smart-copy-container {
  height: 100%;
  padding: 24px;
  overflow-y: auto;
  background: var(--el-bg-color);
}

.smart-copy-header, .detail-header {
  margin-bottom: 24px;
}

.smart-copy-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.smart-copy-header .help-btn {
  color: var(--el-text-color-secondary);
  border-color: var(--el-border-color-light);
}

.smart-copy-header .help-btn:hover {
  color: var(--el-color-primary);
  border-color: var(--el-color-primary-light-5);
}

.smart-copy-header h2, .detail-title h2 {
  margin: 0 0 8px 0;
  font-size: 24px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.smart-copy-header .subtitle {
  margin: 0;
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

/* 详情头部 */
.detail-header {
  padding-bottom: 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.detail-title {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 8px;
}

.detail-meta {
  margin-top: 8px;
  font-size: 14px;
  color: var(--el-text-color-secondary);
  display: flex;
  gap: 8px;
}

.linked-product-info {
  margin-top: 12px;
}

/* 场景选择器 */
.scenario-selector {
  display: flex;
  gap: 20px;
  margin-bottom: 32px;
}

.scenario-card {
  flex: 1;
  max-width: 320px;
  padding: 0;
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
  position: relative;
}

.scenario-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.08);
  border-color: var(--el-color-primary-light-5);
}

.scenario-card.active {
  border-color: var(--el-color-primary);
  background: var(--el-color-primary-light-9);
  box-shadow: 0 4px 12px rgba(64, 158, 255, 0.15);
}

.card-content {
  display: flex;
  align-items: flex-start;
  padding: 24px;
  gap: 16px;
}

.scenario-icon-wrapper {
  width: 56px;
  height: 56px;
  border-radius: 16px; /* Squircle looks more modern for 3D icons */
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  overflow: hidden;
}

.scenario-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  mix-blend-mode: multiply; /* Make white bg transparent on colored wrapper */
}

.scenario-icon-wrapper.blue-theme {
  background: linear-gradient(135deg, #e6f7ff 0%, #bae7ff 100%);
}

.scenario-icon-wrapper.green-theme {
  background: linear-gradient(135deg, #f6ffed 0%, #d9f7be 100%);
}

.text-content {
  flex: 1;
}

.scenario-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.scenario-title {
  font-size: 16px;
  font-weight: 700;
  color: var(--el-text-color-primary);
}

.scenario-badge {
  color: var(--el-color-primary);
  font-weight: bold;
}

.scenario-desc {
  font-size: 13px;
  color: var(--el-text-color-regular);
  margin-bottom: 12px;
  line-height: 1.5;
}

.scenario-status {
  display: flex;
  align-items: center;
}

/* 项目列表和竞品列表 */
.projects-section, .competitors-section {
  background: var(--el-bg-color-page);
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 20px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.section-title { font-size: 16px; font-weight: 600; color: var(--el-text-color-primary); }
.header-actions { display: flex; gap: 8px; }
.loading-state { padding: 20px 0; }
.empty-state { padding: 40px 0; }

/* 搜索和排序工具栏 */
.project-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
  padding: 12px 16px;
  background: var(--el-fill-color-lighter);
  border-radius: 8px;
}

.project-toolbar .search-input {
  width: 280px;
}

.project-toolbar .sort-select {
  width: 140px;
}

.project-toolbar .project-count {
  margin-left: auto;
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

/* 项目网格 */
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
  font-size: 15px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  margin-right: 8px;
}

.more-btn {
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
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

.card-tags {
  display: flex;
  gap: 8px;
  margin-bottom: 10px;
  flex-wrap: wrap;
}

.card-asin {
  font-size: 13px;
  color: var(--el-text-color-secondary);
  margin-bottom: 10px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.card-stats {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: var(--el-text-color-placeholder);
  padding-top: 10px;
  border-top: 1px solid var(--el-border-color-lighter);
}

.card-stats .stat {
  font-weight: 500;
}

.card-stats .stat-time {
  color: var(--el-text-color-disabled);
}

/* 横向对比表格 */
.compare-table-wrapper {
  overflow-x: auto;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 8px;
  background: var(--el-bg-color);
}

.compare-table {
  width: 100%;
  border-collapse: collapse;
  table-layout: fixed;
  min-width: 600px;
}

.compare-table th,
.compare-table td {
  padding: 12px 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  vertical-align: top;
  text-align: left;
}

.compare-table thead th {
  background: var(--el-fill-color-light);
  font-weight: 600;
  position: sticky;
  top: 0;
  z-index: 1;
}

.compare-table .label-col {
  width: 100px;
  min-width: 100px;
  background: var(--el-fill-color-lighter);
  font-weight: 500;
  color: var(--el-text-color-secondary);
  font-size: 13px;
  position: sticky;
  left: 0;
  z-index: 2;
}

.compare-table thead .label-col {
  z-index: 3;
}

.compare-table .data-col {
  min-width: 200px;
}

/* 图片行 */
.image-row {
  background: var(--el-fill-color-lighter);
}

.image-cell {
  text-align: center;
}

.product-image {
  width: 80px;
  height: 80px;
  object-fit: contain;
  border-radius: 4px;
  border: 1px solid var(--el-border-color-lighter);
  background: white;
}

/* 表头 ASIN */
.asin-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.asin-header .asin-text {
  font-size: 14px;
  font-weight: 600;
  font-family: monospace;
  color: var(--el-color-primary);
}

.asin-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

/* 标题行样式 */
.title-row {
  background: var(--el-color-primary-light-9);
}

.title-cell {
  font-size: 13px;
  line-height: 1.6;
  color: var(--el-text-color-primary);
}

/* 数据样式 */
.price-value {
  font-weight: 600;
  color: var(--el-text-color-primary);
  font-size: 15px;
}

/* 评论星级样式 */
.rating-cell-inline {
  display: flex;
  align-items: center;
  gap: 4px;
}

.stars-container {
  font-size: 13px;
  letter-spacing: -2px;
}

.star-filled {
  color: #f5a623;
}

.star-half {
  background: linear-gradient(90deg, #f5a623 50%, #ddd 50%);
  -webkit-background-clip: text;
  background-clip: text;
  -webkit-text-fill-color: transparent;
}

.star-empty {
  color: #ddd;
}

.rating-num {
  font-size: 13px;
  color: var(--el-text-color-primary);
}

.reviews-count {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

/* BSR 显示样式 */
.bsr-cell {
  display: flex;
  align-items: baseline;
  gap: 4px;
  flex-wrap: wrap;
}

.bsr-rank {
  font-weight: 600;
  color: var(--el-text-color-primary);
  font-size: 14px;
}

.bsr-category {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.date-cell {
  font-size: 13px;
  color: var(--el-text-color-regular);
}

.pending-text {
  color: var(--el-text-color-placeholder);
  font-style: italic;
}

/* 五点描述折叠 */
.bullets-header-row {
  cursor: pointer;
}

.bullets-header-row:hover {
  background: var(--el-fill-color-lighter);
}

.label-col.clickable {
  display: flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
}

.expand-icon {
  font-size: 12px;
  transition: transform 0.2s;
}

.expand-icon.expanded {
  transform: rotate(90deg);
}

.bullet-count {
  color: var(--el-text-color-secondary);
  font-size: 12px;
}

.bullet-row {
  background: var(--el-fill-color-lighter);
}

.bullet-label {
  font-size: 12px !important;
  color: var(--el-text-color-placeholder) !important;
}

.bullet-cell {
  font-size: 12px;
  line-height: 1.6;
  color: var(--el-text-color-regular);
}

/* 下一步提示 */
.next-step-hint { margin-top: 16px; }

/* 表单提示 */
.form-tip { font-size: 12px; color: var(--el-text-color-placeholder); margin-top: 4px; }
.type-desc { font-size: 12px; color: var(--el-text-color-placeholder); margin-left: 8px; }

/* 单选框组样式 */
:deep(.el-radio-group) {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* ==================== 评论分析区域 ==================== */
.reviews-section {
  background: var(--el-bg-color-page);
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 20px;
}

.section-title.clickable {
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
}

.reviews-summary-table {
  overflow-x: auto;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 8px;
  background: var(--el-bg-color);
  margin-bottom: 16px;
}

.review-stats-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 星级分布条 */
.star-row {
  background: var(--el-fill-color-lighter);
}

.star-bar {
  height: 8px;
  background: var(--el-fill-color);
  border-radius: 4px;
  overflow: hidden;
  width: 100%;
  max-width: 120px;
  margin-bottom: 4px;
}

.star-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.3s ease;
}

.star-fill.star-5 { background: #67c23a; }
.star-fill.star-4 { background: #95d475; }
.star-fill.star-3 { background: #e6a23c; }
.star-fill.star-2 { background: #f89898; }
.star-fill.star-1 { background: #f56c6c; }

.star-count {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

/* ==================== AI 分析区域 ==================== */
.ai-analysis-section {
  background: var(--el-bg-color-page);
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 20px;
}

.ai-settings {
  display: flex;
  align-items: center;
  gap: 24px;
  margin-bottom: 20px;
  padding: 16px;
  background: var(--el-bg-color);
  border-radius: 8px;
  border: 1px solid var(--el-border-color-lighter);
}

.setting-label {
  font-size: 14px;
  color: var(--el-text-color-secondary);
  white-space: nowrap;
}

/* 分析进度 */
.analysis-progress {
  margin-bottom: 20px;
  padding: 16px;
  background: var(--el-bg-color);
  border-radius: 8px;
  border: 1px solid var(--el-border-color-lighter);
}

.progress-steps {
  display: flex;
  gap: 24px;
  margin-bottom: 12px;
}

.step {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: var(--el-text-color-secondary);
  transition: all 0.3s ease;
}

.step.active {
  color: var(--el-color-primary);
  font-weight: 500;
  animation: pulse 2s ease-in-out infinite;
}

.step.done {
  color: var(--el-color-success);
}

.step-icon {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-pending {
  color: var(--el-text-color-placeholder);
  font-size: 14px;
}

.icon-done {
  display: inline-block;
  font-size: 14px;
  font-weight: bold;
  color: var(--el-color-success);
  animation: bounce 0.4s ease;
}

/* 旋转加载圈 */
.loading-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--el-color-primary-light-7);
  border-top-color: var(--el-color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}

@keyframes bounce {
  0% { transform: scale(0); opacity: 0; }
  50% { transform: scale(1.3); }
  100% { transform: scale(1); opacity: 1; }
}

/* 进度条 */
.progress-bar {
  height: 4px;
  background: var(--el-fill-color-light);
  border-radius: 2px;
  margin-bottom: 16px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--el-color-primary), var(--el-color-success));
  border-radius: 2px;
  transition: width 0.5s ease;
}

/* 流式输出 */
.streaming-output {
  background: var(--el-fill-color-lighter);
  border-radius: 8px;
  padding: 12px 16px;
  font-family: 'SF Mono', Monaco, 'Courier New', monospace;
  font-size: 13px;
  color: var(--el-text-color-regular);
  max-height: 200px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-all;
  position: relative;
}

.streaming-output pre {
  margin: 0;
  display: inline;
}

/* 打字光标 */
.typing-cursor {
  display: inline-block;
  animation: blink 0.8s step-end infinite;
  color: var(--el-color-primary);
  font-weight: bold;
}

@keyframes blink {
  0%, 50% { opacity: 1; }
  51%, 100% { opacity: 0; }
}

/* 分析结果 */
.analysis-results {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.result-card {
  background: var(--el-bg-color);
  border-radius: 8px;
  padding: 16px;
  border: 1px solid var(--el-border-color-lighter);
}

.result-card h4 {
  margin: 0 0 12px 0;
  font-size: 15px;
  color: var(--el-text-color-primary);
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 洞察网格 */
.insights-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-bottom: 16px;
}

.insight-group {
  background: var(--el-fill-color-lighter);
  border-radius: 8px;
  padding: 12px;
}

.insight-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--el-text-color-secondary);
  margin-bottom: 8px;
  display: flex;
  align-items: center;
  gap: 4px;
}

.insight-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.insight-summary {
  font-size: 14px;
  color: var(--el-text-color-regular);
  line-height: 1.6;
  padding: 12px;
  background: var(--el-fill-color-lighter);
  border-radius: 8px;
  border-left: 3px solid var(--el-color-primary);
}

/* 建议组 */
.suggestion-group {
  margin-bottom: 16px;
}

.suggestion-group:last-child {
  margin-bottom: 0;
}

.suggestion-group h5 {
  margin: 0 0 12px 0;
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

/* 建议组头部（标题 + 复制全部按钮） */
.suggestion-group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.suggestion-group-header h5 {
  margin: 0;
}

/* 内容 + 复制按钮包装 */
.suggestion-content-wrapper {
  display: flex;
  align-items: flex-start;
  gap: 8px;
}

.suggestion-content-wrapper .suggestion-content {
  flex: 1;
  margin-bottom: 0;
}

/* 复制按钮 */
.copy-btn {
  flex-shrink: 0;
  opacity: 0.6;
  transition: opacity 0.2s ease;
}

.suggestion-item:hover .copy-btn,
.suggestion-content-wrapper:hover .copy-btn {
  opacity: 1;
}

.copy-btn:hover {
  opacity: 1;
  color: var(--el-color-primary);
}

.suggestion-item {
  background: var(--el-fill-color-lighter);
  border-radius: 8px;
  padding: 12px;
  margin-bottom: 12px;
}

.suggestion-item:last-child {
  margin-bottom: 0;
}

.suggestion-content {
  font-size: 14px;
  color: var(--el-text-color-primary);
  line-height: 1.6;
  margin-bottom: 8px;
  padding: 8px 12px;
  background: var(--el-bg-color);
  border-radius: 6px;
  border: 1px solid var(--el-border-color-lighter);
}

.suggestion-reasons {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.reason-tag {
  font-size: 12px;
  padding: 4px 8px;
  background: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
  border-radius: 4px;
}

/* 商品描述建议 */
.description-item {
  background: var(--el-fill-color-lighter);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
}

.description-item:last-child {
  margin-bottom: 0;
}

.description-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.description-content {
  margin: 12px 0;
  padding: 12px;
  background: var(--el-bg-color);
  border-radius: 6px;
  border: 1px solid var(--el-border-color-lighter);
}

.description-text {
  white-space: pre-wrap;
  word-break: break-word;
  font-family: inherit;
  margin: 0;
  line-height: 1.6;
  font-size: 14px;
  color: var(--el-text-color-primary);
}

.description-meta {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.description-meta .meta-row {
  margin-bottom: 8px;
  display: flex;
  align-items: flex-start;
  flex-wrap: wrap;
  gap: 4px;
}

.description-meta .meta-row:last-child {
  margin-bottom: 0;
}

.description-meta .meta-label {
  font-weight: 500;
  color: var(--el-text-color-regular);
  flex-shrink: 0;
}

.description-meta .meta-tag {
  margin-right: 4px;
}

.description-meta .reason-row {
  flex-direction: column;
  gap: 4px;
}

.description-meta .reason-text {
  color: var(--el-text-color-secondary);
  line-height: 1.5;
}

/* 五点建议 */
.bullet-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.bullet-index {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: var(--el-color-primary);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
  flex-shrink: 0;
}

.bullet-header .el-tag {
  flex-shrink: 0;
}

.embedded-keywords {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
  margin: 8px 0;
  padding: 8px 12px;
  background: var(--el-color-success-light-9);
  border-radius: 6px;
}

.keywords-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  font-weight: 500;
}

.keyword-summary {
  margin-top: 16px;
  padding: 12px;
  background: var(--el-fill-color-lighter);
  border-radius: 8px;
  font-size: 13px;
  color: var(--el-text-color-regular);
  line-height: 1.6;
}

.suggestion-reason {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px dashed var(--el-border-color-lighter);
}

/* 关键词列表 */
.keyword-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.keyword-list .el-tag {
  display: flex;
  align-items: center;
  gap: 4px;
}

.keyword-volume {
  font-size: 11px;
  color: var(--el-text-color-placeholder);
  margin-left: 4px;
}

/* 无分析结果 */
.no-analysis {
  text-align: center;
  padding: 40px 20px;
  color: var(--el-text-color-secondary);
}

.no-analysis .el-icon {
  font-size: 48px;
  margin-bottom: 16px;
  color: var(--el-text-color-placeholder);
}

.no-analysis p {
  margin: 0 0 8px 0;
  font-size: 14px;
}

.analysis-hint {
  font-size: 12px;
  color: var(--el-text-color-placeholder);
}

/* ==================== 我的产品信息 ==================== */
.my-product-section {
  background: var(--el-bg-color-page);
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 20px;
  border: 2px dashed var(--el-color-warning-light-5);
}

.my-product-form {
  background: var(--el-bg-color);
  border-radius: 8px;
  padding: 20px;
  margin-top: 16px;
}

.key-features-input {
  width: 100%;
}

.features-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 12px;
}

.features-list .el-tag {
  padding: 6px 12px;
}

.my-product-summary {
  margin-top: 12px;
  padding: 12px 16px;
  background: var(--el-fill-color-lighter);
  border-radius: 8px;
  font-size: 14px;
  color: var(--el-text-color-regular);
}

.summary-label {
  color: var(--el-text-color-secondary);
  font-weight: 500;
}

.summary-divider {
  margin: 0 12px;
  color: var(--el-border-color);
}

/* ==================== 我的现有文案 ==================== */
.my-listing-section {
  background: var(--el-bg-color-page);
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 20px;
  border: 2px dashed var(--el-color-primary-light-5);
}

.my-listing-section .section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.my-listing-content {
  margin-top: 16px;
  background: var(--el-bg-color);
  border-radius: 8px;
  padding: 16px;
}

.my-listing-content .listing-item {
  margin-bottom: 16px;
}

.my-listing-content .listing-item:last-child {
  margin-bottom: 0;
}

.my-listing-content .listing-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 6px;
  font-weight: 500;
}

.my-listing-content .listing-value {
  font-size: 14px;
  color: var(--el-text-color-primary);
  line-height: 1.6;
}

.my-listing-content .bullet-list {
  margin: 0;
  padding-left: 20px;
}

.my-listing-content .bullet-list li {
  margin-bottom: 6px;
}

.my-listing-content .description-text {
  white-space: pre-wrap;
  max-height: 200px;
  overflow-y: auto;
}

.my-listing-content .fetch-time {
  margin-top: 12px;
  font-size: 12px;
  color: var(--el-text-color-placeholder);
  text-align: right;
}

.my-listing-empty {
  margin-top: 16px;
  padding: 20px;
  text-align: center;
  background: var(--el-fill-color-light);
  border-radius: 8px;
}

/* A+ 内容建议样式 */
.aplus-section {
  margin-top: 20px;
}

.aplus-subsection {
  margin-bottom: 16px;
  padding: 12px;
  background: var(--el-fill-color-lighter);
  border-radius: 6px;
}

.aplus-subsection h6 {
  margin: 0 0 10px 0;
  font-size: 13px;
  color: var(--el-text-color-primary);
  font-weight: 600;
}

.main-image-points {
  margin: 0;
  padding-left: 20px;
}

.main-image-points li {
  margin-bottom: 6px;
  font-size: 13px;
  color: var(--el-text-color-regular);
}

.secondary-images-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.secondary-image-item {
  padding: 10px;
  background: var(--el-bg-color);
  border-radius: 4px;
  border: 1px solid var(--el-border-color-lighter);
}

.image-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.image-index {
  font-size: 12px;
  font-weight: 600;
  color: var(--el-color-primary);
}

.image-copy {
  font-size: 13px;
  color: var(--el-text-color-regular);
  line-height: 1.5;
}

.module-recommendations {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.module-item {
  padding: 10px;
  background: var(--el-bg-color);
  border-radius: 4px;
  border: 1px solid var(--el-border-color-lighter);
}

.module-header {
  margin-bottom: 8px;
}

.module-points {
  margin: 0;
  padding-left: 20px;
}

.module-points li {
  margin-bottom: 4px;
  font-size: 13px;
  color: var(--el-text-color-regular);
}

/* 导出按钮区域 */
.export-section {
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid var(--el-border-color-lighter);
  text-align: right;
}

/* 响应式 */
@media (max-width: 768px) {
  .insights-grid {
    grid-template-columns: 1fr;
  }

  .ai-settings {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
  }

  .progress-steps {
    flex-direction: column;
    gap: 8px;
  }
}
</style>

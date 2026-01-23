<script setup lang="ts">
import { ref, onMounted, computed, nextTick, watch } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { open } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';
import { convertFileSrc } from '@tauri-apps/api/core';
import draggable from 'vuedraggable';
import * as api from '../api';
import { chat, chatStream, buildStrictModePrompt, buildAnalysisModePrompt, buildDirectChatPrompt, parseSourceReferences, checkApiKeyConfigured, recognizeImage, getTextEmbedding, getTextEmbeddingsBatchParallel } from '../ai-service';
import type { ChatMessage } from '../ai-service';
import type { KbDocument, KbConversation, KbMessage, KbSearchResult, KbChunk, AIProvider, DependencyStatus, InstallProgress, KbDocumentLink, KbDocumentCategory } from '../types';
import { AI_PROVIDERS } from '../types';
import { marked } from 'marked';
import { QuestionFilled } from '@element-plus/icons-vue';

// Emits
const emit = defineEmits<{
  (e: 'showHelp', tab: string): void;
}>();

// ==================== 状态 ====================

// 文档管理
const documents = ref<KbDocument[]>([]);
const loadingDocuments = ref(false);
const processingDocuments = ref<Set<number>>(new Set());

// 分类管理
interface KbCategory {
  id: number;
  name: string;
  parent_id: number | null;
  sort_order: number;
  color: string;
  created_at: string;
  document_count?: number;
}
const categories = ref<KbCategory[]>([]);

// 预设颜色
const CATEGORY_COLORS = [
  { name: '蓝色', value: '#409EFF' },
  { name: '绿色', value: '#67C23A' },
  { name: '橙色', value: '#E6A23C' },
  { name: '红色', value: '#F56C6C' },
  { name: '紫色', value: '#9C27B0' },
  { name: '青色', value: '#00BCD4' },
  { name: '粉色', value: '#E91E63' },
  { name: '灰色', value: '#909399' },
];

// 选中状态（筛选用）
const selectedCategory = ref<number | null>(null);
const categoryFilteredDocIds = ref<Set<number>>(new Set()); // 按分类筛选的文档 ID 集合
const categoryDocCounts = ref<Map<number, number>>(new Map()); // 每个分类的文档数量

// 文档所属分类（在预览抽屉中使用，多对多）
const previewDocCategories = ref<KbDocumentCategory[]>([]);

// 文档链接（双向链接）
const previewDocLinks = ref<KbDocumentLink[]>([]);
const previewDocBacklinks = ref<KbDocumentLink[]>([]);

// 知识图谱
const showGraphView = ref(false);
const graphDocuments = ref<KbDocument[]>([]);
const graphLinks = ref<KbDocumentLink[]>([]);

// 大纲导航
interface OutlineItem {
  level: number;
  text: string;
  anchor: string;
}
const previewOutline = ref<OutlineItem[]>([]);

// 搜索和排序
const searchText = ref('');
const sortBy = ref<'date_desc' | 'date_asc' | 'name_asc' | 'name_desc'>('date_desc');

// AI 对话
const conversations = ref<KbConversation[]>([]);
const currentConversation = ref<KbConversation | null>(null);
const messages = ref<KbMessage[]>([]);
const inputMessage = ref('');
const MAX_CONTEXT_MESSAGES = 30;  // 最多保留最近 15 轮对话作为上下文
const isGenerating = ref(false);
const streamingContent = ref('');

// AI 设置
const selectedProvider = ref<AIProvider>('deepseek');
const selectedModel = ref(AI_PROVIDERS.deepseek.defaultModel);

// 对话模式
type ChatMode = 'strict' | 'analysis' | 'direct';
const chatMode = ref<ChatMode>(
  (localStorage.getItem('kb_chat_mode') as ChatMode) || 'analysis'
);

// 监听模式变化，持久化到 localStorage
watch(chatMode, (newMode) => {
  localStorage.setItem('kb_chat_mode', newMode);
});

// API Key 状态（用于功能状态提示）
const apiKeyStatusChecked = ref(false);  // 是否已完成检查
const apiKeyStatus = ref({
  deepseek: false,
  qwen: false,
  openai: false,
  gemini: false,
});

// 检查 API Key 配置状态
async function refreshApiKeyStatus() {
  try {
    apiKeyStatus.value.deepseek = await checkApiKeyConfigured('deepseek');
    apiKeyStatus.value.qwen = await checkApiKeyConfigured('qwen');
    apiKeyStatus.value.openai = await checkApiKeyConfigured('openai');
    apiKeyStatus.value.gemini = await checkApiKeyConfigured('gemini');
  } catch (e) {
    console.error('检查 API Key 状态失败:', e);
  } finally {
    apiKeyStatusChecked.value = true;
  }
}

// 是否有任意 AI 服务配置（未检查完成前返回 true 避免闪烁）
const hasAnyAiKey = computed(() => {
  if (!apiKeyStatusChecked.value) return true;  // 检查中，暂不显示警告
  return apiKeyStatus.value.deepseek || apiKeyStatus.value.qwen ||
         apiKeyStatus.value.openai || apiKeyStatus.value.gemini;
});

// 是否可以进行向量化（需要千问或DeepSeek，未检查完成前返回 true）
const canEmbed = computed(() => {
  if (!apiKeyStatusChecked.value) return true;
  return apiKeyStatus.value.qwen || apiKeyStatus.value.deepseek;
});

// UI 状态
const activeSection = ref<'documents' | 'chat'>('chat');
const messagesContainer = ref<HTMLElement | null>(null);
const abortController = ref<AbortController | null>(null);
const expandedSources = ref<Set<number>>(new Set()); // 展开的参考来源索引

function toggleSources(index: number) {
  if (expandedSources.value.has(index)) {
    expandedSources.value.delete(index);
  } else {
    expandedSources.value.add(index);
  }
}

// 图片预览状态
const imagePreviewVisible = ref(false);
const previewImagePath = ref<string | null>(null);

function previewImage(imagePath: string) {
  previewImagePath.value = imagePath;
  imagePreviewVisible.value = true;
}

function handleImageError(event: Event) {
  const img = event.target as HTMLImageElement;
  img.style.display = 'none';
}

// 获取消息关联的图片列表（去重）
function getMessageImages(msg: KbMessage | { role: string; content: string; isStreaming?: boolean }): Array<{path: string, name: string}> {
  if (!('sources' in msg) || !msg.sources) return [];
  try {
    const sources = JSON.parse(msg.sources as string);
    const seen = new Set<string>();
    return sources
      .filter((s: any) => {
        if (!s.image_path || seen.has(s.image_path)) return false;
        seen.add(s.image_path);
        return true;
      })
      .map((s: any) => ({
        path: s.image_path,
        name: s.document_title || '图片'
      }));
  } catch {
    return [];
  }
}

// 获取消息所有图片路径（用于预览列表）
function getAllImagePaths(msg: KbMessage | { role: string; content: string; isStreaming?: boolean }): string[] {
  return getMessageImages(msg).map(img => convertFileSrc(img.path));
}

// 文档预览状态
const showDocPreview = ref(false);
const previewDoc = ref<KbDocument | null>(null);
const previewChunks = ref<KbChunk[]>([]);
const loadingPreview = ref(false);

async function handlePreviewDocument(doc: KbDocument) {
  previewDoc.value = doc;
  showDocPreview.value = true;
  loadingPreview.value = true;

  try {
    // 并行加载分块、分类、链接
    const [chunks] = await Promise.all([
      api.kbGetChunks(doc.id),
      loadDocumentCategories(doc.id),
      loadDocumentLinks(doc.id),
    ]);
    previewChunks.value = chunks;

    // 从分块内容中提取大纲
    if (chunks.length > 0) {
      const fullContent = chunks.map(c => c.content).join('\n\n');
      previewOutline.value = extractOutline(fullContent);
    } else {
      previewOutline.value = [];
    }
  } catch (e) {
    console.error('加载分块失败:', e);
    previewChunks.value = [];
    previewOutline.value = [];
  } finally {
    loadingPreview.value = false;
  }
}

function formatFileSize(bytes: number | null): string {
  if (!bytes) return '-';
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
}

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}

// PDF 依赖安装状态
const showPdfDepsDialog = ref(false);
const pdfDepsInstalling = ref(false);
const pdfDepsProgress = ref<InstallProgress | null>(null);
const pdfDepsStatus = ref<DependencyStatus | null>(null);

// Embedding 处理进度
const embeddingProgress = ref({
  show: false,
  current: 0,
  total: 0,
  percentage: 0
});

// 批量上传队列
interface UploadTask {
  id: string;
  fileName: string;
  filePath: string;
  status: 'pending' | 'uploading' | 'processing' | 'embedding' | 'completed' | 'failed' | 'cancelled';
  progress: number;
  message: string;
  docId?: number;
}

const uploadQueue = ref<UploadTask[]>([]);
const showUploadDialog = ref(false);
const isUploading = ref(false);
const uploadAbortController = ref<Map<string, boolean>>(new Map());

// ==================== 计算属性 ====================

const availableModels = computed(() => {
  return AI_PROVIDERS[selectedProvider.value].models;
});

const displayMessages = computed(() => {
  const result: (KbMessage | { role: 'assistant'; content: string; isStreaming: true })[] = [...messages.value];
  if (isGenerating.value && streamingContent.value) {
    result.push({
      role: 'assistant',
      content: streamingContent.value,
      isStreaming: true,
    });
  }
  return result;
});

// 文档列表（分类筛选 + 搜索 + 排序）
const displayDocuments = computed(() => {
  let result = [...documents.value];

  // 1. 按分类筛选（多对多）
  if (selectedCategory.value !== null) {
    if (categoryFilteredDocIds.value.size > 0) {
      result = result.filter(doc => categoryFilteredDocIds.value.has(doc.id));
    } else {
      // 分类已选择但没有匹配的文档
      result = [];
    }
  }

  // 2. 按关键词搜索
  if (searchText.value.trim()) {
    const keyword = searchText.value.toLowerCase();
    result = result.filter(doc =>
      doc.title.toLowerCase().includes(keyword) ||
      doc.file_name.toLowerCase().includes(keyword)
    );
  }

  // 3. 排序
  result.sort((a, b) => {
    switch (sortBy.value) {
      case 'date_desc':
        return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
      case 'date_asc':
        return new Date(a.created_at).getTime() - new Date(b.created_at).getTime();
      case 'name_asc':
        return a.title.localeCompare(b.title, 'zh-CN');
      case 'name_desc':
        return b.title.localeCompare(a.title, 'zh-CN');
      default:
        return 0;
    }
  });

  return result;
});

// 计算每个分类的文档数（多对多）
const getCategoryDocCount = (categoryId: number | null) => {
  if (categoryId === null) {
    return documents.value.length;
  }
  return categoryDocCounts.value.get(categoryId) || 0;
};

// 加载所有分类的文档数量
async function loadCategoryDocCounts() {
  const counts = new Map<number, number>();
  for (const cat of categories.value) {
    try {
      const docs = await api.kbGetDocumentsByCategories(cat.id);
      counts.set(cat.id, docs.length);
    } catch (e) {
      console.error(`加载分类 ${cat.id} 文档数量失败:`, e);
      counts.set(cat.id, 0);
    }
  }
  categoryDocCounts.value = counts;
}

// ==================== 文档管理 ====================

async function loadDocuments() {
  loadingDocuments.value = true;
  try {
    const docs = await api.kbGetDocuments();

    // 并行获取每个文档的向量化状态
    const docsWithStats = await Promise.all(
      docs.map(async (doc) => {
        try {
          const [total, count] = await api.kbGetDocumentEmbeddingStats(doc.id);
          return { ...doc, embedding_total: total, embedding_count: count };
        } catch {
          return { ...doc, embedding_total: 0, embedding_count: 0 };
        }
      })
    );

    documents.value = docsWithStats;
  } catch (e) {
    console.error('加载文档失败:', e);
    ElMessage.error('加载文档失败');
  } finally {
    loadingDocuments.value = false;
  }
}

// ==================== 分类管理 ====================

async function loadCategories() {
  try {
    categories.value = await api.kbGetCategories();
    // 加载分类文档数量
    await loadCategoryDocCounts();
  } catch (e) {
    console.error('加载分类失败:', e);
  }
}

async function handleAddCategory() {
  try {
    const { value: name } = await ElMessageBox.prompt('请输入分类名称', '新建分类', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      inputPattern: /\S+/,
      inputErrorMessage: '分类名称不能为空',
    });
    if (name) {
      await api.kbCreateCategory(name.trim());
      await loadCategories();
      ElMessage.success('分类创建成功');
    }
  } catch {
    // 用户取消
  }
}

async function handleRenameCategory(cat: KbCategory) {
  try {
    const { value: newName } = await ElMessageBox.prompt('请输入新的分类名称', '重命名分类', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      inputValue: cat.name,
      inputPattern: /\S+/,
      inputErrorMessage: '分类名称不能为空',
    });
    if (newName && newName.trim() !== cat.name) {
      await api.kbUpdateCategory(cat.id, newName.trim());
      await loadCategories();
      ElMessage.success('分类已重命名');
    }
  } catch {
    // 用户取消
  }
}

async function handleDeleteCategory(cat: KbCategory) {
  try {
    await ElMessageBox.confirm(
      `确定删除分类 "${cat.name}"？该分类下的文档将变为未分类。`,
      '删除分类',
      { type: 'warning' }
    );
    await api.kbDeleteCategory(cat.id);
    await loadCategories();
    if (selectedCategory.value === cat.id) {
      selectedCategory.value = null;
    }
    ElMessage.success('分类已删除');
  } catch {
    // 用户取消
  }
}

function handleCategoryAction(action: string, cat: KbCategory) {
  if (action === 'rename') {
    handleRenameCategory(cat);
  } else if (action === 'delete') {
    handleDeleteCategory(cat);
  }
}

// 更新分类颜色
async function handleCategoryColor(id: number, color: string) {
  try {
    await api.kbUpdateCategoryColor(id, color);
    await loadCategories();
  } catch (e) {
    console.error('更新颜色失败:', e);
    ElMessage.error('更新颜色失败');
  }
}

// 拖拽排序变化（使用 vuedraggable @change 事件）
async function handleCategoryDragChange(evt: any) {
  // @change 事件提供 { moved: { element, oldIndex, newIndex } } 结构
  if (!evt.moved) return;

  const { oldIndex, newIndex } = evt.moved;
  if (oldIndex === newIndex) return;

  // vuedraggable 已经更新了 categories 数组，直接获取新顺序
  const newOrder = categories.value.map(c => c.id);

  try {
    await api.kbUpdateCategoriesOrder(newOrder);
    ElMessage.success('排序已更新');
  } catch (e) {
    console.error('更新排序失败:', e);
    ElMessage.error('更新排序失败');
    await loadCategories();
  }
}

// ==================== 文档分类关联（多对多）====================

// 加载文档的分类（多对多）
async function loadDocumentCategories(docId: number) {
  try {
    previewDocCategories.value = await api.kbGetDocumentCategories(docId);
  } catch (e) {
    console.error('加载文档分类失败:', e);
    previewDocCategories.value = [];
  }
}

// 给文档添加分类
async function handleAddDocumentCategory(docId: number, categoryId: number) {
  try {
    await api.kbAddDocumentCategory(docId, categoryId);
    await loadDocumentCategories(docId);
    // 更新分类文档计数
    const currentCount = categoryDocCounts.value.get(categoryId) || 0;
    categoryDocCounts.value.set(categoryId, currentCount + 1);
    // 如果当前正在按此分类筛选，需要刷新筛选结果
    if (selectedCategory.value === categoryId) {
      await handleCategoryFilter(categoryId);
    }
    ElMessage.success('分类已添加');
  } catch (e) {
    console.error('添加分类失败:', e);
    ElMessage.error('添加分类失败');
  }
}

// 移除文档分类
async function handleRemoveDocumentCategory(docId: number, categoryId: number) {
  try {
    await api.kbRemoveDocumentCategory(docId, categoryId);
    await loadDocumentCategories(docId);
    // 更新分类文档计数
    const currentCount = categoryDocCounts.value.get(categoryId) || 0;
    if (currentCount > 0) {
      categoryDocCounts.value.set(categoryId, currentCount - 1);
    }
    // 如果当前正在按此分类筛选，需要刷新筛选结果
    if (selectedCategory.value === categoryId) {
      await handleCategoryFilter(categoryId);
    }
  } catch (e) {
    console.error('移除分类失败:', e);
  }
}

// 按分类筛选文档（多对多版本）
async function handleCategoryFilter(categoryId: number | null) {
  selectedCategory.value = categoryId;

  if (categoryId !== null) {
    try {
      const docs = await api.kbGetDocumentsByCategories(categoryId);
      categoryFilteredDocIds.value = new Set(docs.map(d => d.id));
    } catch (e) {
      console.error('按分类筛选文档失败:', e);
      categoryFilteredDocIds.value = new Set();
    }
  } else {
    categoryFilteredDocIds.value = new Set();
  }
}

// ==================== 文档链接（双向链接）====================

async function loadDocumentLinks(docId: number) {
  try {
    previewDocLinks.value = await api.kbGetDocumentLinks(docId);
    previewDocBacklinks.value = await api.kbGetDocumentBacklinks(docId);
  } catch (e) {
    console.error('加载文档链接失败:', e);
    previewDocLinks.value = [];
    previewDocBacklinks.value = [];
  }
}

async function handleAddDocumentLink(sourceId: number, targetId: number) {
  try {
    await api.kbAddDocumentLink(sourceId, targetId);
    await loadDocumentLinks(sourceId);
    ElMessage.success('链接已创建');
  } catch (e) {
    console.error('创建链接失败:', e);
    ElMessage.error('创建链接失败');
  }
}

async function handleRemoveDocumentLink(sourceId: number, targetId: number) {
  try {
    await api.kbRemoveDocumentLink(sourceId, targetId);
    await loadDocumentLinks(sourceId);
    ElMessage.success('链接已移除');
  } catch (e) {
    console.error('移除链接失败:', e);
  }
}

function handleNavigateToDocument(docId: number) {
  const doc = documents.value.find(d => d.id === docId);
  if (doc) {
    handlePreviewDocument(doc);
  }
}

// ==================== 知识图谱 ====================

async function openGraphView() {
  try {
    graphDocuments.value = documents.value.filter(d => d.status === 'completed');
    graphLinks.value = await api.kbGetAllLinks();
    showGraphView.value = true;
  } catch (e) {
    console.error('加载图谱数据失败:', e);
    ElMessage.error('加载图谱数据失败');
  }
}

function getNodePosition(docId: number, axis: 'x' | 'y'): number {
  const index = graphDocuments.value.findIndex(d => d.id === docId);
  if (index === -1) return 0;
  if (axis === 'x') {
    return 25 + (index % 5) * 18;
  } else {
    return 20 + Math.floor(index / 5) * 20;
  }
}

// ==================== 大纲导航 ====================

function extractOutline(content: string): OutlineItem[] {
  const lines = content.split('\n');
  const outline: OutlineItem[] = [];
  let anchorIndex = 0;

  for (const line of lines) {
    const match = line.match(/^(#{1,6})\s+(.+)$/);
    if (match) {
      const level = match[1].length;
      const text = match[2].trim();
      const anchor = `outline-${anchorIndex++}`;
      outline.push({ level, text, anchor });
    }
  }

  return outline;
}

function scrollToOutlineItem(anchor: string) {
  const element = document.getElementById(anchor);
  if (element) {
    element.scrollIntoView({ behavior: 'smooth', block: 'start' });
  }
}

// ==================== AI 回答保存为笔记 ====================

async function handleSaveAsNote(msg: KbMessage) {
  try {
    const { value: title } = await ElMessageBox.prompt('请输入笔记标题', '保存为笔记', {
      confirmButtonText: '保存',
      cancelButtonText: '取消',
      inputValue: '来自 AI 的回答',
      inputPattern: /\S+/,
      inputErrorMessage: '标题不能为空',
    });

    if (!title) return;

    // 创建 Markdown 内容
    let content = `# ${title}\n\n`;
    content += `> 创建时间：${new Date().toLocaleString('zh-CN')}\n\n`;
    content += msg.content;

    // 如果有来源引用，添加引用链接
    if (msg.sources) {
      const sources = JSON.parse(msg.sources);
      if (sources.length > 0) {
        content += '\n\n---\n\n## 参考来源\n\n';
        for (const source of sources) {
          content += `- [[${source.document_title}]]\n`;
        }
      }
    }

    // 创建临时文件并添加文档
    const docId = await api.kbAddDocument(
      selectedCategory.value,
      title.trim(),
      `${title.trim()}.md`,
      '', // 将在后端生成
      'md',
      content.length
    );

    // 添加分块
    await api.kbAddChunk(docId, 0, content);
    await api.kbUpdateDocumentStatus(docId, 'completed', 1);

    // 建立与引用文档的链接
    if (msg.sources) {
      const sources = JSON.parse(msg.sources);
      for (const source of sources) {
        await api.kbAddDocumentLink(docId, source.document_id);
      }
    }

    await loadDocuments();
    ElMessage.success('已保存为笔记');
  } catch (e) {
    if (e !== 'cancel') {
      console.error('保存笔记失败:', e);
      ElMessage.error('保存失败');
    }
  }
}

async function handleUploadDocument() {
  const selected = await open({
    multiple: true,
    filters: [
      {
        name: '支持的文档',
        extensions: ['pdf', 'docx', 'xlsx', 'xls', 'pptx', 'md', 'txt'],
      },
    ],
  });

  if (!selected) return;

  const files = Array.isArray(selected) ? selected : [selected];
  if (files.length === 0) return;

  // 创建上传任务队列
  uploadQueue.value = files.map((filePath, index) => {
    const parts = filePath.split(/[/\\]/);
    const fileName = parts[parts.length - 1];
    return {
      id: `upload-${Date.now()}-${index}`,
      fileName,
      filePath,
      status: 'pending' as const,
      progress: 0,
      message: '等待上传...',
    };
  });

  // 显示上传对话框
  showUploadDialog.value = true;
  isUploading.value = true;

  // 开始处理队列
  await processUploadQueue();
}

// 处理上传队列
async function processUploadQueue() {
  for (const task of uploadQueue.value) {
    // 检查是否被取消
    if (uploadAbortController.value.get(task.id)) {
      task.status = 'cancelled';
      task.message = '已取消';
      continue;
    }

    if (task.status !== 'pending') continue;

    try {
      await processUploadTask(task);
    } catch (e) {
      console.error('上传任务失败:', e);
      task.status = 'failed';
      task.message = `失败: ${e}`;
    }
  }

  isUploading.value = false;

  // 刷新文档列表
  await loadDocuments();

  // 统计结果
  const completed = uploadQueue.value.filter(t => t.status === 'completed').length;
  const failed = uploadQueue.value.filter(t => t.status === 'failed').length;

  if (completed > 0) {
    ElMessage.success(`成功上传 ${completed} 个文档${failed > 0 ? `，${failed} 个失败` : ''}`);
  } else if (failed > 0) {
    ElMessage.error(`${failed} 个文档上传失败`);
  }
}

// 处理单个上传任务
async function processUploadTask(task: UploadTask) {
  const filePath = task.filePath;
  const parts = filePath.split(/[/\\]/);
  const fileName = parts[parts.length - 1];
  const ext = fileName.split('.').pop()?.toLowerCase() || 'txt';
  const title = fileName.replace(/\.[^.]+$/, '');

  // 1. 上传阶段
  task.status = 'uploading';
  task.progress = 10;
  task.message = '检查文件...';

  // 检测重复文件
  const existingDoc = documents.value.find(
    doc => doc.file_name === fileName || doc.title === title
  );

  if (existingDoc) {
    task.message = '检测到重复，覆盖中...';
    await api.kbDeleteDocument(existingDoc.id);
  }

  // 添加文档记录
  task.progress = 20;
  task.message = '创建文档记录...';
  const docId = await api.kbAddDocument(null, title, fileName, filePath, ext);
  task.docId = docId;

  // 检查是否被取消
  if (uploadAbortController.value.get(task.id)) {
    // 删除已创建的文档
    await api.kbDeleteDocument(docId);
    task.status = 'cancelled';
    task.message = '已取消';
    return;
  }

  // 2. 处理阶段（解析 + 分块）
  task.status = 'processing';
  task.progress = 30;
  task.message = '解析文档内容...';
  processingDocuments.value.add(docId);

  let chunkCount = 0;

  try {
    chunkCount = await api.kbProcessDocument(docId, filePath);
    task.progress = 50;
  } catch (processError: any) {
    // PDF 文本提取失败，尝试使用 OCR
    if (ext === 'pdf' && processError?.toString().includes('未提取到文本')) {
      task.message = '使用 AI OCR 识别 PDF...';
      const ocrChunks = await processPdfWithOcr(docId, filePath);
      if (ocrChunks > 0) {
        chunkCount = ocrChunks;
      } else {
        throw new Error('PDF OCR 识别失败');
      }
    } else {
      throw processError;
    }
  }

  // 检查是否被取消
  if (uploadAbortController.value.get(task.id)) {
    await api.kbDeleteDocument(docId);
    processingDocuments.value.delete(docId);
    task.status = 'cancelled';
    task.message = '已取消';
    return;
  }

  // 对于支持图片的文档格式，提取并识别图片
  task.progress = 60;
  const imageTypes = ['xlsx', 'xls', 'pptx', 'docx'];
  if (imageTypes.includes(ext)) {
    task.message = '识别图片内容...';
    const imageChunks = await processDocumentImages(docId, filePath, task.id);
    if (imageChunks > 0) {
      chunkCount += imageChunks;
      await api.kbUpdateDocumentStatus(docId, 'completed', chunkCount);
    }
  }

  // 3. 向量化阶段
  task.status = 'embedding';
  task.progress = 70;
  task.message = '生成向量索引...';

  // 检查是否被取消
  if (uploadAbortController.value.get(task.id)) {
    processingDocuments.value.delete(docId);
    task.status = 'cancelled';
    task.message = '已取消（文档已保存但未向量化）';
    return;
  }

  await generateDocumentEmbeddingsForTask(docId, task);

  // 4. 完成
  processingDocuments.value.delete(docId);
  task.status = 'completed';
  task.progress = 100;
  task.message = `完成，${chunkCount} 个分块`;
}

// 为单个任务生成向量（带进度更新）
async function generateDocumentEmbeddingsForTask(documentId: number, task: UploadTask): Promise<number> {
  try {
    const hasDeepSeekKey = await checkApiKeyConfigured('deepseek');
    const hasQwenKey = await checkApiKeyConfigured('qwen');
    if (!hasDeepSeekKey && !hasQwenKey) {
      task.message = '向量化跳过（未配置 API Key）';
      return 0;
    }

    const chunks = await api.kbGetChunksWithoutEmbedding(documentId);
    if (chunks.length === 0) {
      return 0;
    }

    // 检查是否被取消
    if (uploadAbortController.value.get(task.id)) {
      return 0;
    }

    const embeddings = await getTextEmbeddingsBatchParallel(
      chunks.map(c => ({ id: c.id, content: c.content })),
      (current, total) => {
        const embeddingPercent = Math.round((current / total) * 30);  // 70-100%
        task.progress = 70 + embeddingPercent;
        task.message = `向量化 ${current}/${total}`;
      },
      5
    );

    let savedCount = 0;
    for (const [chunkId, embedding] of embeddings) {
      // 每保存一个就检查是否被取消
      if (uploadAbortController.value.get(task.id)) {
        task.message = `已取消（已保存 ${savedCount} 个向量）`;
        return savedCount;
      }
      await api.kbUpdateChunkEmbedding(chunkId, embedding);
      savedCount++;
    }

    return savedCount;
  } catch (e) {
    console.error('向量化失败:', e);
    task.message = '向量化失败';
    return 0;
  }
}

// 取消单个上传任务
function cancelUploadTask(taskId: string) {
  uploadAbortController.value.set(taskId, true);
  const task = uploadQueue.value.find(t => t.id === taskId);
  if (task && task.status === 'pending') {
    task.status = 'cancelled';
    task.message = '已取消';
  }
}

// 取消所有上传任务
function cancelAllUploads() {
  for (const task of uploadQueue.value) {
    if (task.status === 'pending') {
      // pending 任务直接标记为已取消
      uploadAbortController.value.set(task.id, true);
      task.status = 'cancelled';
      task.message = '已取消';
    } else if (task.status === 'uploading' || task.status === 'processing' || task.status === 'embedding') {
      // 正在处理的任务设置取消标志，会在下一个检查点中断
      uploadAbortController.value.set(task.id, true);
    }
  }
  ElMessage.info('已发送取消请求');
}

// 关闭上传对话框
function closeUploadDialog() {
  if (isUploading.value) {
    ElMessageBox.confirm(
      '还有文件正在上传，确定要关闭吗？\n正在处理的文件会继续完成。',
      '确认关闭',
      {
        confirmButtonText: '关闭',
        cancelButtonText: '取消',
        type: 'warning',
      }
    ).then(() => {
      showUploadDialog.value = false;
    }).catch(() => {});
  } else {
    showUploadDialog.value = false;
    uploadQueue.value = [];
    uploadAbortController.value.clear();
  }
}

// 获取任务状态图标类型
function getTaskStatusType(status: UploadTask['status']): 'success' | 'warning' | 'danger' | 'info' | 'primary' {
  const types: Record<UploadTask['status'], 'success' | 'warning' | 'danger' | 'info' | 'primary'> = {
    pending: 'info',
    uploading: 'primary',
    processing: 'primary',
    embedding: 'primary',
    completed: 'success',
    failed: 'danger',
    cancelled: 'warning',
  };
  return types[status];
}

// 获取任务状态标签
function getTaskStatusLabel(status: UploadTask['status']): string {
  const labels: Record<UploadTask['status'], string> = {
    pending: '等待中',
    uploading: '上传中',
    processing: '处理中',
    embedding: '向量化',
    completed: '完成',
    failed: '失败',
    cancelled: '已取消',
  };
  return labels[status];
}

/**
 * 处理文档中的图片：提取图片 -> AI 识别（通义千问/Gemini） -> 存储为 chunk
 */
async function processDocumentImages(documentId: number, filePath: string, taskId?: string): Promise<number> {
  try {
    // 检查是否有图片识别服务可用（通义千问或 Gemini）
    const hasQwenKey = await checkApiKeyConfigured('qwen');
    const hasGeminiKey = await checkApiKeyConfigured('gemini');
    if (!hasQwenKey && !hasGeminiKey) {
      console.log('通义千问/Gemini API Key 均未配置，跳过图片识别');
      return 0;
    }

    // 提取文档中的图片
    const images = await api.kbExtractImages(filePath);
    if (images.length === 0) {
      return 0;
    }

    console.log(`发现 ${images.length} 张图片，开始识别...`);

    let processedCount = 0;
    for (let i = 0; i < images.length; i++) {
      // 检查是否被取消
      if (taskId && uploadAbortController.value.get(taskId)) {
        console.log('图片处理被取消');
        return processedCount;
      }

      const image = images[i];

      // 跳过 EMF/WMF 等 Gemini 不支持的格式
      if (image.mime_type.includes('emf') || image.mime_type.includes('wmf')) {
        console.log(`跳过不支持的图片格式: ${image.name} (${image.mime_type})`);
        continue;
      }

      // 多张图片之间添加延迟，避免触发速率限制
      if (i > 0) {
        console.log('等待 3 秒后处理下一张图片...');
        await new Promise(resolve => setTimeout(resolve, 3000));
      }

      // 调用 AI Vision API 识别图片 (优先通义千问，备选 Gemini)
      const result = await recognizeImage(
        image.base64_data,
        image.mime_type,
        "请详细描述这张图片中的所有内容。如果图片中包含文字、表格、数字、地址、联系方式等信息，请完整提取出来。"
      );

      if (result.success && result.description) {
        // 将识别结果作为 chunk 存储（同时保存图片用于图文问答）
        await api.kbAddImageChunkWithFile(documentId, image.name, result.description, image.base64_data);
        processedCount++;
        console.log(`图片 ${image.name} 识别成功，已保存图片`);
      } else {
        console.warn(`图片 ${image.name} 识别失败:`, result.error);
      }
    }

    return processedCount;
  } catch (e) {
    console.error('处理文档图片失败:', e);
    return 0;
  }
}

/**
 * 检查 PDF 依赖状态
 */
async function checkPdfDependencies(): Promise<boolean> {
  try {
    const status = await api.checkDependencies();
    pdfDepsStatus.value = status;
    return status.pdf2image_installed && status.poppler_installed;
  } catch (e) {
    console.error('检查 PDF 依赖失败:', e);
    return false;
  }
}

/**
 * 安装 PDF 依赖
 */
async function installPdfDependencies(): Promise<boolean> {
  pdfDepsInstalling.value = true;
  pdfDepsProgress.value = null;

  try {
    // 监听安装进度
    const unlisten = await listen<InstallProgress>('install-progress', (event) => {
      pdfDepsProgress.value = event.payload;
    });

    const result = await api.installPdfDependencies();
    unlisten();

    if (result.success) {
      ElMessage.success('PDF 依赖安装成功！');
      showPdfDepsDialog.value = false;
      // 刷新状态
      await checkPdfDependencies();
      return true;
    } else {
      ElMessage.error(`安装失败: ${result.message}`);
      return false;
    }
  } catch (e: any) {
    console.error('安装 PDF 依赖失败:', e);
    ElMessage.error(`安装失败: ${e.toString()}`);
    return false;
  } finally {
    pdfDepsInstalling.value = false;
  }
}

/**
 * 使用 AI OCR 处理 PDF（当文本提取失败时的后备方案）
 * 流程：PDF -> 转换为图片 -> 通义千问/Gemini OCR -> 存储
 */
async function processPdfWithOcr(documentId: number, filePath: string): Promise<number> {
  try {
    // 检查是否有 OCR 服务可用
    const hasQwenKey = await checkApiKeyConfigured('qwen');
    const hasGeminiKey = await checkApiKeyConfigured('gemini');
    if (!hasQwenKey && !hasGeminiKey) {
      console.log('通义千问/Gemini API Key 均未配置，无法进行 PDF OCR');
      return 0;
    }

    // 检查 PDF 依赖
    const hasPdfDeps = await checkPdfDependencies();

    console.log('[PDF OCR] 将 PDF 转换为图片...');
    let pageImages;
    try {
      pageImages = await api.kbPdfToImages(filePath);
    } catch (e: any) {
      // 如果 PDF 转图片失败，检查是否缺少依赖
      if (!hasPdfDeps) {
        console.log('[PDF OCR] 缺少 PDF 处理依赖，显示安装对话框...');
        showPdfDepsDialog.value = true;
        // 同时尝试用 Gemini 作为备选
      }

      // 如果 Gemini 可用，直接用它（支持 PDF）
      console.log('[PDF OCR] PDF 转图片失败，尝试直接使用 Gemini...');
      if (hasGeminiKey) {
        const base64Data = await api.kbReadFileBase64(filePath);
        const result = await recognizeImage(
          base64Data,
          'application/pdf',
          '请完整提取这份 PDF 文档中的所有文字内容。如果有表格请用 Markdown 表格格式输出。'
        );
        if (result.success && result.description) {
          await api.kbAddImageChunk(documentId, 'PDF OCR', result.description);
          await api.kbUpdateDocumentStatus(documentId, 'completed', 1);
          return 1;
        }
      }
      throw new Error(`PDF 转图片失败: ${e}。缺少必要的 PDF 处理依赖。`);
    }

    console.log(`[PDF OCR] PDF 共 ${pageImages.length} 页，开始 OCR 识别...`);

    let processedCount = 0;
    const allTexts: string[] = [];

    for (let i = 0; i < pageImages.length; i++) {
      const page = pageImages[i];
      console.log(`[PDF OCR] 识别第 ${page.page_number}/${pageImages.length} 页...`);

      const result = await recognizeImage(
        page.base64_data,
        page.mime_type,
        '请完整提取图中的文字内容，保留表格、公式（转为 LaTeX）、代码块和段落结构，输出为 Markdown。'
      );

      if (result.success && result.description) {
        allTexts.push(`## 第 ${page.page_number} 页\n\n${result.description}`);
        processedCount++;
      } else {
        console.warn(`[PDF OCR] 第 ${page.page_number} 页识别失败:`, result.error);
      }

      // 页面之间稍微延迟，避免 API 限流
      if (i < pageImages.length - 1) {
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
    }

    if (allTexts.length > 0) {
      // 合并所有页面的文本
      const fullText = allTexts.join('\n\n---\n\n');
      await api.kbAddImageChunk(documentId, 'PDF OCR', fullText);
      await api.kbUpdateDocumentStatus(documentId, 'completed', 1);
      console.log(`[PDF OCR] 完成，成功识别 ${processedCount}/${pageImages.length} 页`);
      return 1;
    }

    return 0;
  } catch (e) {
    console.error('[PDF OCR] 处理失败:', e);
    return 0;
  }
}

/**
 * 为文档的所有分块生成 embedding 向量（并行处理 + 进度显示）
 */
async function generateDocumentEmbeddings(documentId: number): Promise<number> {
  try {
    // 检查 DeepSeek 或千问 API Key 是否配置
    const hasDeepSeekKey = await checkApiKeyConfigured('deepseek');
    const hasQwenKey = await checkApiKeyConfigured('qwen');
    if (!hasDeepSeekKey && !hasQwenKey) {
      console.log('DeepSeek/千问 API Key 未配置，跳过向量化');
      ElMessage.warning('未配置千问 API Key，无法生成向量索引（向量搜索将不可用）');
      return 0;
    }

    // 获取所有需要向量化的分块
    const chunks = await api.kbGetChunksWithoutEmbedding(documentId);
    if (chunks.length === 0) {
      return 0;
    }

    console.log(`开始为 ${chunks.length} 个分块生成向量...`);

    // 显示进度条
    embeddingProgress.value = {
      show: true,
      current: 0,
      total: chunks.length,
      percentage: 0
    };

    try {
      // 使用并行处理（5个并发）
      const embeddings = await getTextEmbeddingsBatchParallel(
        chunks.map(c => ({ id: c.id, content: c.content })),
        (current, total) => {
          embeddingProgress.value.current = current;
          embeddingProgress.value.total = total;
          embeddingProgress.value.percentage = Math.round((current / total) * 100);
          console.log(`向量化进度: ${current}/${total} (${embeddingProgress.value.percentage}%)`);
        },
        5  // 并发数
      );

      // 保存向量到数据库
      let savedCount = 0;
      for (const [chunkId, embedding] of embeddings) {
        await api.kbUpdateChunkEmbedding(chunkId, embedding);
        savedCount++;
      }

      console.log(`完成 ${savedCount} 个分块的向量化`);

      // 检查是否有失败的
      const failedCount = chunks.length - savedCount;
      if (failedCount > 0) {
        ElMessage.warning(`向量化部分失败：${savedCount}/${chunks.length} 成功`);
      }

      return savedCount;
    } finally {
      // 隐藏进度条
      embeddingProgress.value.show = false;
    }
  } catch (e) {
    console.error('生成向量失败:', e);
    ElMessage.error(`向量化失败: ${e}`);
    embeddingProgress.value.show = false;
    return 0;
  }
}

async function handleDeleteDocument(doc: KbDocument) {
  try {
    await ElMessageBox.confirm(`确定要删除文档 "${doc.title}" 吗？`, '确认删除', {
      confirmButtonText: '删除',
      cancelButtonText: '取消',
      type: 'warning',
    });

    await api.kbDeleteDocument(doc.id);
    await loadDocuments();
    // 刷新分类文档计数
    await loadCategoryDocCounts();
    ElMessage.success('文档已删除');
  } catch (e) {
    if (e !== 'cancel') {
      console.error('删除文档失败:', e);
      ElMessage.error('删除失败');
    }
  }
}

// 保留用于未来扩展
async function _handleMoveToCategory(doc: KbDocument, categoryId: number | null) {
  try {
    // 使用多对多 API：设置文档的分类（会替换所有现有分类）
    const categoryIds = categoryId === null ? [] : [categoryId];
    await api.kbSetDocumentCategories(doc.id, categoryIds);
    // 刷新分类文档计数
    await loadCategoryDocCounts();
    const catName = categoryId === null ? '未分类' : categories.value.find(c => c.id === categoryId)?.name || '未知';
    ElMessage.success(`已移动到 "${catName}"`);
  } catch (e) {
    console.error('移动文档失败:', e);
    ElMessage.error('移动失败');
  }
}
// 避免 TS 未使用警告
void _handleMoveToCategory;

function getFileTypeIcon(fileType: string): string {
  const icons: Record<string, string> = {
    pdf: 'DocumentCopy',
    docx: 'Document',
    xlsx: 'Grid',
    xls: 'Grid',
    md: 'EditPen',
    txt: 'Tickets',
  };
  return icons[fileType] || 'Document';
}

function getStatusTag(status: string): { type: 'success' | 'warning' | 'danger' | 'info'; label: string } {
  const statusMap: Record<string, { type: 'success' | 'warning' | 'danger' | 'info'; label: string }> = {
    pending: { type: 'info', label: '待处理' },
    processing: { type: 'warning', label: '处理中' },
    completed: { type: 'success', label: '已完成' },
    failed: { type: 'danger', label: '失败' },
  };
  return statusMap[status] || { type: 'info', label: status };
}

// 获取向量化状态类型
function getEmbeddingStatusType(doc: KbDocument): 'success' | 'warning' | 'danger' | 'info' {
  if (!doc.embedding_total || doc.embedding_total === 0) return 'info';
  if (doc.embedding_count === doc.embedding_total) return 'success';
  if ((doc.embedding_count ?? 0) > 0) return 'warning';
  return 'danger';
}

// 获取向量化状态提示
function getEmbeddingTooltip(doc: KbDocument): string {
  if (!doc.embedding_total || doc.embedding_total === 0) {
    return '暂无分块';
  }
  if (doc.embedding_count === doc.embedding_total) {
    return '向量化完成';
  }
  return `${doc.embedding_count ?? 0}/${doc.embedding_total} 块已向量化`;
}

// 重新向量化文档
async function handleReEmbed(doc: KbDocument) {
  try {
    await generateDocumentEmbeddings(doc.id);
    await loadDocuments();
    ElMessage.success('向量化完成');
  } catch (e) {
    console.error('向量化失败:', e);
    ElMessage.error('向量化失败');
  }
}

// ==================== 对话管理 ====================

async function loadConversations() {
  try {
    conversations.value = await api.kbGetConversations();
  } catch (e) {
    console.error('加载对话失败:', e);
  }
}

async function createNewConversation() {
  try {
    const id = await api.kbCreateConversation(
      selectedProvider.value,
      selectedModel.value,
      '新对话'
    );
    await loadConversations();

    // 选中新对话
    currentConversation.value = conversations.value.find(c => c.id === id) || null;
    messages.value = [];
    inputMessage.value = '';
  } catch (e) {
    console.error('创建对话失败:', e);
    ElMessage.error('创建对话失败');
  }
}

async function selectConversation(conv: KbConversation) {
  currentConversation.value = conv;
  selectedProvider.value = (conv.ai_provider as AIProvider) || 'deepseek';
  selectedModel.value = conv.ai_model || AI_PROVIDERS[selectedProvider.value].defaultModel;

  try {
    messages.value = await api.kbGetMessages(conv.id);
    await nextTick();
    scrollToBottom();
  } catch (e) {
    console.error('加载消息失败:', e);
    messages.value = [];
  }
}

async function deleteConversation(conv: KbConversation) {
  try {
    await ElMessageBox.confirm('确定要删除这个对话吗？', '确认删除', {
      confirmButtonText: '删除',
      cancelButtonText: '取消',
      type: 'warning',
    });

    await api.kbDeleteConversation(conv.id);
    await loadConversations();

    if (currentConversation.value?.id === conv.id) {
      currentConversation.value = null;
      messages.value = [];
    }
  } catch (e) {
    if (e !== 'cancel') {
      console.error('删除对话失败:', e);
    }
  }
}

// ==================== AI 对话 ====================

/**
 * 使用 AI 生成对话标题（异步执行，不阻塞主流程）
 */
async function generateConversationTitle(conversationId: number, firstMessage: string) {
  try {
    const prompt = `用10个字以内总结这个问题的主题，只返回标题文字，不要标点符号和引号：\n\n"${firstMessage}"`;

    let title = '';
    for await (const chunk of chatStream(
      [{ role: 'user', content: prompt }],
      {
        provider: selectedProvider.value,
        model: selectedModel.value,
        maxTokens: 50,
      }
    )) {
      if (chunk.done) break;
      title += chunk.content;
    }

    // 清理标题（去除引号、多余空格、换行）
    title = title.trim().replace(/^["'""'']+|["'""'']+$/g, '').replace(/\n/g, '').slice(0, 20);

    if (title) {
      await api.kbUpdateConversationTitle(conversationId, title);
      // 更新当前对话的标题
      if (currentConversation.value?.id === conversationId) {
        currentConversation.value.title = title;
      }
      // 更新列表中的标题
      const conv = conversations.value.find(c => c.id === conversationId);
      if (conv) conv.title = title;
    }
  } catch (e) {
    console.warn('生成标题失败:', e);
    // 降级：使用简单截断
    const fallbackTitle = firstMessage.substring(0, 20) + (firstMessage.length > 20 ? '...' : '');
    await api.kbUpdateConversationTitle(conversationId, fallbackTitle);
  }
}

/**
 * 混合检索：向量搜索 + 关键词搜索，合并去重后返回最相关的结果
 */
async function hybridSearch(query: string): Promise<KbSearchResult[]> {
  const results: KbSearchResult[] = [];
  const seenChunkIds = new Set<number>();

  // 1. 向量搜索（语义匹配）
  try {
    const hasDeepSeekKey = await checkApiKeyConfigured('deepseek');
    const hasQwenKey = await checkApiKeyConfigured('qwen');
    if (hasDeepSeekKey || hasQwenKey) {
      const embeddingResult = await getTextEmbedding(query);
      if (embeddingResult.success && embeddingResult.embedding.length > 0) {
        const vectorResults = await api.kbVectorSearch(embeddingResult.embedding, 30, 0.4);
        console.log(`向量搜索找到 ${vectorResults.length} 个结果`);
        for (const r of vectorResults) {
          if (!seenChunkIds.has(r.chunk_id)) {
            seenChunkIds.add(r.chunk_id);
            results.push(r);
          }
        }
      }
    }
  } catch (e) {
    console.warn('向量搜索失败:', e);
  }

  // 2. 关键词搜索（精确匹配）
  try {
    const keywordResults = await api.kbSearch(query, 20);
    console.log(`关键词搜索找到 ${keywordResults.length} 个结果`);
    for (const r of keywordResults) {
      if (!seenChunkIds.has(r.chunk_id)) {
        seenChunkIds.add(r.chunk_id);
        // 关键词匹配的结果给一个基础分数
        results.push({ ...r, score: r.score || 0.6 });
      }
    }
  } catch (e) {
    console.warn('关键词搜索失败:', e);
  }

  // 3. 按相关度排序，取 Top 50（留给 Re-ranking 筛选）
  const finalResults = results
    .sort((a, b) => (b.score || 0) - (a.score || 0))
    .slice(0, 50);

  console.log(`混合检索最终返回 ${finalResults.length} 个结果`);
  return finalResults;
}

/**
 * Re-ranking：使用 LLM 对检索结果重新排序
 * 评估每个候选文档与用户问题的相关性，返回最相关的结果
 */
async function reRankResults(
  query: string,
  candidates: KbSearchResult[],
  topK: number = 8
): Promise<KbSearchResult[]> {
  if (candidates.length === 0) return [];
  if (candidates.length <= topK) return candidates;

  console.log(`[Re-ranking] 对 ${candidates.length} 个候选结果重排序...`);

  try {
    // 构建候选文档列表（限制每个文档长度避免 token 过多）
    const candidateList = candidates.map((c, i) => {
      const content = c.content.length > 300 ? c.content.substring(0, 300) + '...' : c.content;
      return `[${i + 1}] ${c.document_title}\n${content}`;
    }).join('\n\n---\n\n');

    const prompt = `你是一个文档相关性评估专家。请评估以下候选文档与用户问题的相关性。

用户问题：${query}

候选文档：
${candidateList}

请按相关性从高到低排序，返回最相关的 ${topK} 个文档编号。
只返回编号列表，用逗号分隔，不要有其他内容。
例如：3,1,7,5,2,8,4,6

排序结果：`;

    const response = await chat(
      [{ role: 'user', content: prompt }],
      {
        provider: selectedProvider.value,
        model: selectedModel.value,
        temperature: 0,  // 使用 0 确保结果稳定
        maxTokens: 100,
      }
    );

    // 解析返回的编号列表
    const rankingStr = response.trim();
    const rankingMatch = rankingStr.match(/[\d,\s]+/);
    if (!rankingMatch) {
      console.warn('[Re-ranking] 无法解析排序结果，使用原始顺序');
      return candidates.slice(0, topK);
    }

    const ranking = rankingMatch[0]
      .split(/[,\s]+/)
      .map(s => parseInt(s.trim()))
      .filter(n => !isNaN(n) && n >= 1 && n <= candidates.length);

    // 根据排序结果重新排列
    const rerankedResults: KbSearchResult[] = [];
    const usedIndices = new Set<number>();

    for (const rank of ranking) {
      const idx = rank - 1;
      if (!usedIndices.has(idx) && idx < candidates.length) {
        usedIndices.add(idx);
        rerankedResults.push(candidates[idx]);
        if (rerankedResults.length >= topK) break;
      }
    }

    // 如果结果不足，用剩余的候选补充
    if (rerankedResults.length < topK) {
      for (let i = 0; i < candidates.length && rerankedResults.length < topK; i++) {
        if (!usedIndices.has(i)) {
          rerankedResults.push(candidates[i]);
        }
      }
    }

    console.log(`[Re-ranking] 重排序完成，返回 ${rerankedResults.length} 个结果`);
    return rerankedResults;

  } catch (e) {
    console.warn('[Re-ranking] 重排序失败，使用原始顺序:', e);
    return candidates.slice(0, topK);
  }
}

async function sendMessage() {
  const content = inputMessage.value.trim();
  if (!content || isGenerating.value) return;

  // 检查 API Key
  const hasKey = await checkApiKeyConfigured(selectedProvider.value);
  if (!hasKey) {
    ElMessage.warning(`请先在设置中配置 ${AI_PROVIDERS[selectedProvider.value].name} API Key`);
    return;
  }

  // 如果没有对话，创建新对话
  if (!currentConversation.value) {
    await createNewConversation();
  }

  const convId = currentConversation.value!.id;

  // 保存用户消息
  await api.kbAddMessage(convId, 'user', content);

  // 添加到本地显示
  messages.value.push({
    id: Date.now(),
    conversation_id: convId,
    role: 'user',
    content,
    sources: null,
    created_at: new Date().toISOString(),
  });

  inputMessage.value = '';
  await nextTick();
  scrollToBottom();

  // 根据对话模式决定是否检索知识库
  let searchResults: KbSearchResult[] = [];
  let systemPrompt: string;

  if (chatMode.value === 'direct') {
    // 直接对话模式：不检索知识库
    systemPrompt = buildDirectChatPrompt();
  } else {
    // 严格/分析模式：检索知识库
    const rawResults = await hybridSearch(content);
    searchResults = await reRankResults(content, rawResults, 8);

    if (chatMode.value === 'strict') {
      systemPrompt = buildStrictModePrompt(searchResults);
    } else {
      systemPrompt = buildAnalysisModePrompt(searchResults);
    }
  }

  // 构建消息（只保留最近 N 条历史消息作为上下文）
  const recentMessages = messages.value.slice(-MAX_CONTEXT_MESSAGES);
  const chatMessages: ChatMessage[] = [
    { role: 'system', content: systemPrompt },
    ...recentMessages.map(m => ({
      role: m.role as 'user' | 'assistant',
      content: m.content,
    })),
  ];

  // 开始生成
  isGenerating.value = true;
  streamingContent.value = '';
  abortController.value = new AbortController();

  try {
    // 使用流式响应
    for await (const chunk of chatStream(chatMessages, {
      provider: selectedProvider.value,
      model: selectedModel.value,
      maxTokens: 8192,
      signal: abortController.value.signal,
    })) {
      if (chunk.done) break;
      streamingContent.value += chunk.content;
      await nextTick();
      scrollToBottom();
    }

    // 解析来源引用（直接对话模式下没有来源）
    const sources = chatMode.value === 'direct' ? [] : parseSourceReferences(streamingContent.value, searchResults);

    // 保存助手消息
    await api.kbAddMessage(
      convId,
      'assistant',
      streamingContent.value,
      sources.length > 0 ? JSON.stringify(sources) : undefined
    );

    // 更新对话标题（如果是第一条消息，用 AI 生成标题）
    if (messages.value.length === 1) {
      generateConversationTitle(convId, content);
    }

    // 添加到本地显示
    messages.value.push({
      id: Date.now() + 1,
      conversation_id: convId,
      role: 'assistant',
      content: streamingContent.value,
      sources: sources.length > 0 ? JSON.stringify(sources) : null,
      created_at: new Date().toISOString(),
    });

  } catch (e: any) {
    if (e.name === 'AbortError') {
      ElMessage.info('已停止生成');
    } else {
      console.error('生成失败:', e);
      ElMessage.error(`生成失败: ${e.message}`);
    }
  } finally {
    isGenerating.value = false;
    streamingContent.value = '';
    abortController.value = null;
  }
}

function stopGeneration() {
  if (abortController.value) {
    abortController.value.abort();
  }
}

// Markdown 渲染
// Callout 类型配置
const CALLOUT_TYPES: Record<string, { icon: string; color: string; label: string }> = {
  note: { icon: '📝', color: '#409EFF', label: '笔记' },
  tip: { icon: '💡', color: '#67C23A', label: '提示' },
  warning: { icon: '⚠️', color: '#E6A23C', label: '警告' },
  danger: { icon: '❌', color: '#F56C6C', label: '危险' },
  info: { icon: 'ℹ️', color: '#409EFF', label: '信息' },
  quote: { icon: '💬', color: '#909399', label: '引用' },
  example: { icon: '📋', color: '#00BCD4', label: '示例' },
  question: { icon: '❓', color: '#9C27B0', label: '问题' },
  success: { icon: '✅', color: '#67C23A', label: '成功' },
  failure: { icon: '❎', color: '#F56C6C', label: '失败' },
};

function renderMarkdown(content: string): string {
  // 预处理 Callouts: > [!type] 标题
  let processedContent = content;

  // 分割内容为块（按 Obsidian Callout 语法分割）
  const blocks = processedContent.split(/\n(?=>\s*\[!)/);
  const processedBlocks = blocks.map(block => {
    const calloutMatch = block.match(/^>\s*\[!(\w+)\]\s*(.*?)(?:\n|$)/);
    if (calloutMatch) {
      const type = calloutMatch[1].toLowerCase();
      const title = calloutMatch[2] || '';
      const config = CALLOUT_TYPES[type] || CALLOUT_TYPES['note'];

      // 提取 callout 内容（去掉首行和 > 前缀）
      const lines = block.split('\n');
      const contentLines = lines.slice(1).map(line => {
        return line.replace(/^>\s?/, '');
      }).join('\n');

      return `<div class="callout callout-${type}" style="--callout-color: ${config.color}">
        <div class="callout-header">
          <span class="callout-icon">${config.icon}</span>
          <span class="callout-title">${title || config.label}</span>
        </div>
        <div class="callout-content">${marked(contentLines, { breaks: true })}</div>
      </div>`;
    }
    return block;
  });

  processedContent = processedBlocks.join('\n');

  let html = marked(processedContent, { breaks: true }) as string;

  // 将 [来源X] 包裹在 span 中以便样式化
  html = html.replace(/\[来源(\d+)\]/g, '<span class="source-ref">[来源$1]</span>');

  // 处理 [[文档标题]] 双向链接语法
  html = html.replace(/\[\[([^\]]+)\]\]/g, '<span class="wiki-link">$1</span>');

  return html;
}

function scrollToBottom() {
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
  }
}

function handleKeyDown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    sendMessage();
  }
}

// 当切换 provider 时，更新默认 model
watch(selectedProvider, (newProvider) => {
  selectedModel.value = AI_PROVIDERS[newProvider].defaultModel;
});

// ==================== 生命周期 ====================

onMounted(async () => {
  await Promise.all([
    loadDocuments(),
    loadConversations(),
    loadCategories(),
    refreshApiKeyStatus(),
  ]);
});
</script>

<template>
  <div class="knowledge-base-container">
    <!-- 左侧边栏 -->
    <div class="sidebar">
      <!-- 切换标签 -->
      <div class="sidebar-tabs">
        <el-button-group class="section-toggle">
          <el-button
            :type="activeSection === 'chat' ? 'primary' : 'default'"
            size="small"
            @click="activeSection = 'chat'"
          >
            <el-icon><ChatDotRound /></el-icon>
            AI 问答
          </el-button>
          <el-button
            :type="activeSection === 'documents' ? 'primary' : 'default'"
            size="small"
            @click="activeSection = 'documents'"
          >
            <el-icon><FolderOpened /></el-icon>
            知识库
          </el-button>
        </el-button-group>
      </div>

      <!-- 对话列表 -->
      <div v-if="activeSection === 'chat'" class="conversation-list">
        <div class="list-header">
          <span>对话历史</span>
          <el-button type="primary" size="small" @click="createNewConversation">
            <el-icon><Plus /></el-icon>
            新对话
          </el-button>
        </div>
        <div class="list-content">
          <div
            v-for="conv in conversations"
            :key="conv.id"
            class="conversation-item"
            :class="{ active: currentConversation?.id === conv.id }"
            @click="selectConversation(conv)"
          >
            <div class="conv-title">{{ conv.title || '新对话' }}</div>
            <div class="conv-meta">
              <span class="conv-provider">{{ AI_PROVIDERS[conv.ai_provider as AIProvider]?.name || conv.ai_provider }}</span>
              <el-button
                type="danger"
                text
                size="small"
                class="delete-btn"
                @click.stop="deleteConversation(conv)"
              >
                <el-icon><Delete /></el-icon>
              </el-button>
            </div>
          </div>
          <div v-if="conversations.length === 0" class="empty-state">
            暂无对话历史
          </div>
        </div>
      </div>

      <!-- 知识库模式：分类侧边栏 -->
      <div v-else class="category-sidebar">
        <div class="category-header">分类</div>
        <div
          class="category-item"
          :class="{ active: selectedCategory === null }"
          @click="handleCategoryFilter(null)"
        >
          <el-icon><Folder /></el-icon>
          <span class="category-name">全部文档</span>
          <span class="category-count">{{ documents.length }}</span>
        </div>
        <draggable
          v-model="categories"
          item-key="id"
          ghost-class="category-ghost"
          animation="200"
          :force-fallback="true"
          :fallback-class="'category-fallback'"
          @change="handleCategoryDragChange"
        >
          <template #item="{ element: cat }">
            <div
              class="category-item"
              :class="{ active: selectedCategory === cat.id }"
              @click="handleCategoryFilter(cat.id)"
            >
              <el-icon class="drag-handle"><Rank /></el-icon>
              <el-icon class="folder-filled" :style="{ '--folder-color': cat.color }"><FolderOpened /></el-icon>
              <span class="category-name">{{ cat.name }}</span>
              <span class="category-count">{{ getCategoryDocCount(cat.id) }}</span>
              <el-dropdown trigger="click" @command="(cmd: string) => handleCategoryAction(cmd, cat)">
                <el-icon class="category-more" @click.stop><MoreFilled /></el-icon>
                <template #dropdown>
                  <el-dropdown-menu>
                    <el-dropdown-item command="rename">重命名</el-dropdown-item>
                    <el-dropdown-item divided>
                      <div class="color-picker-wrapper" @click.stop>
                        <span style="margin-right: 8px;">设置颜色</span>
                        <el-color-picker
                          :model-value="cat.color"
                          :predefine="CATEGORY_COLORS.map(c => c.value)"
                          size="small"
                          @change="(color: string | null) => color && handleCategoryColor(cat.id, color)"
                        />
                      </div>
                    </el-dropdown-item>
                    <el-dropdown-item command="delete" divided>删除</el-dropdown-item>
                  </el-dropdown-menu>
                </template>
              </el-dropdown>
            </div>
          </template>
        </draggable>
        <div class="add-category" @click="handleAddCategory">
          <el-icon><Plus /></el-icon>
          <span>新建分类</span>
        </div>

        <!-- 知识图谱按钮 -->
        <div class="graph-btn-container">
          <el-button type="primary" plain size="small" @click="openGraphView" class="graph-btn">
            <el-icon><Share /></el-icon>
            知识图谱
          </el-button>
        </div>
      </div>
    </div>

    <!-- 中间区域：文档列表（仅知识库模式显示） -->
    <div v-if="activeSection === 'documents'" class="document-list-area">
      <!-- 头部标题 -->
      <div class="doc-area-header">
        <span class="header-title">文档列表</span>
        <span class="doc-count-badge">{{ displayDocuments.length }} 个文档</span>
      </div>
      <!-- 工具栏：搜索 + 排序 + 上传 -->
      <div class="doc-toolbar">
        <el-input
          v-model="searchText"
          placeholder="搜索文档..."
          clearable
          size="small"
          style="width: 180px"
        >
          <template #prefix>
            <el-icon><Search /></el-icon>
          </template>
        </el-input>
        <el-select v-model="sortBy" size="small" style="width: 120px">
          <el-option value="date_desc" label="最新上传" />
          <el-option value="date_asc" label="最早上传" />
          <el-option value="name_asc" label="名称 A-Z" />
          <el-option value="name_desc" label="名称 Z-A" />
        </el-select>
        <el-button type="primary" size="small" @click="handleUploadDocument">
          <el-icon><Upload /></el-icon>
          上传
        </el-button>
      </div>

      <!-- Embedding 处理进度条 -->
      <div v-if="embeddingProgress.show" class="embedding-progress">
        <div class="progress-header">
          <span>正在生成向量索引...</span>
          <span>{{ embeddingProgress.current }}/{{ embeddingProgress.total }}</span>
        </div>
        <el-progress
          :percentage="embeddingProgress.percentage"
          :stroke-width="8"
          :show-text="true"
        />
      </div>

      <!-- 文档列表内容 -->
      <div class="doc-list-content" v-loading="loadingDocuments">
        <div
          v-for="doc in displayDocuments"
          :key="doc.id"
          class="document-item"
          @click="handlePreviewDocument(doc)"
        >
          <div class="doc-info">
            <el-icon class="doc-icon">
              <component :is="getFileTypeIcon(doc.file_type)" />
            </el-icon>
            <div class="doc-details">
              <div class="doc-title">{{ doc.title }}</div>
              <div class="doc-meta">
                <el-tag
                  :type="getStatusTag(doc.status).type"
                  size="small"
                >
                  {{ getStatusTag(doc.status).label }}
                </el-tag>
                <span v-if="doc.status === 'completed'">
                  {{ doc.chunk_count }} 个分块
                </span>
                <!-- 向量化状态 -->
                <el-tooltip v-if="doc.status === 'completed' && doc.embedding_total" :content="getEmbeddingTooltip(doc)">
                  <el-tag
                    :type="getEmbeddingStatusType(doc)"
                    size="small"
                    class="embedding-tag"
                  >
                    <el-icon v-if="getEmbeddingStatusType(doc) === 'success'"><CircleCheckFilled /></el-icon>
                    <el-icon v-else-if="getEmbeddingStatusType(doc) === 'warning'"><WarningFilled /></el-icon>
                    <el-icon v-else><CircleCloseFilled /></el-icon>
                    <span>{{ doc.embedding_count }}/{{ doc.embedding_total }}</span>
                  </el-tag>
                </el-tooltip>
                <span v-if="processingDocuments.has(doc.id)" class="processing-hint">
                  处理中...
                </span>
              </div>
            </div>
          </div>
          <!-- 添加分类（多对多，可添加多个分类） -->
          <el-dropdown v-if="categories.length > 0" trigger="click" @command="(catId: number) => handleAddDocumentCategory(doc.id, catId)">
            <el-button type="primary" text size="small" @click.stop title="添加分类">
              <el-icon><FolderOpened /></el-icon>
            </el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item
                  v-for="cat in categories"
                  :key="cat.id"
                  :command="cat.id"
                >
                  <span class="category-dropdown-item">
                    <span class="category-color-dot" :style="{ backgroundColor: cat.color }"></span>
                    {{ cat.name }}
                  </span>
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
          <!-- 重新向量化按钮 -->
          <el-tooltip
            v-if="doc.status === 'completed' && doc.embedding_total && doc.embedding_count !== doc.embedding_total"
            :disabled="canEmbed"
            content="请先配置千问或 DeepSeek API Key"
            placement="top"
          >
            <el-button
              type="warning"
              text
              size="small"
              :disabled="!canEmbed"
              @click.stop="handleReEmbed(doc)"
              title="重新向量化"
            >
              <el-icon><Refresh /></el-icon>
            </el-button>
          </el-tooltip>
          <el-button
            type="danger"
            text
            size="small"
            @click.stop="handleDeleteDocument(doc)"
          >
            <el-icon><Delete /></el-icon>
          </el-button>
        </div>
        <div v-if="displayDocuments.length === 0" class="empty-state">
          <p v-if="searchText || selectedCategory !== null">未找到匹配的文档</p>
          <template v-else>
            <p>暂无文档</p>
            <p class="empty-hint">上传 PDF、Word、Excel、Markdown 或文本文件</p>
          </template>
        </div>
      </div>
    </div>

    <!-- 右侧主内容区 -->
    <div class="main-content">
      <!-- AI 设置栏 -->
      <div class="ai-settings">
        <div class="setting-item">
          <span class="label">AI 服务：</span>
          <el-select v-model="selectedProvider" size="small" style="width: 120px">
            <el-option
              v-for="(config, key) in AI_PROVIDERS"
              :key="key"
              :label="config.name"
              :value="key"
            />
          </el-select>
        </div>
        <div class="setting-item">
          <span class="label">模型：</span>
          <el-select v-model="selectedModel" size="small" style="width: 180px">
            <el-option
              v-for="model in availableModels"
              :key="model"
              :label="model"
              :value="model"
            />
          </el-select>
        </div>
        <div class="doc-count">
          <el-icon><FolderOpened /></el-icon>
          {{ documents.filter(d => d.status === 'completed').length }} 个文档已就绪
        </div>
      </div>

      <!-- 消息区域 -->
      <div class="messages-area" ref="messagesContainer">
        <div v-if="displayMessages.length === 0" class="welcome-message">
          <div class="welcome-icon">
            <el-icon :size="48"><ChatDotRound /></el-icon>
          </div>
          <h3>企业知识库助手</h3>
          <p>基于您上传的文档回答问题，支持 PDF、Word、Excel、Markdown 等格式</p>
          <div class="quick-tips">
            <div class="tip">
              <el-icon><Document /></el-icon>
              上传文档到知识库
            </div>
            <div class="tip">
              <el-icon><Search /></el-icon>
              AI 自动检索相关内容
            </div>
            <div class="tip">
              <el-icon><ChatLineRound /></el-icon>
              获取带来源引用的回答
            </div>
          </div>
        </div>

        <div
          v-for="(msg, index) in displayMessages"
          :key="index"
          class="message"
          :class="[msg.role]"
        >
          <div class="message-content">
            <div class="message-text" v-html="renderMarkdown(msg.content)"></div>
            <div
              v-if="'sources' in msg && msg.sources"
              class="message-sources"
            >
              <div
                class="sources-title"
                @click="toggleSources(index)"
              >
                <el-icon><Link /></el-icon>
                参考来源 ({{ JSON.parse(msg.sources as string).length }})
                <el-icon class="expand-icon" :class="{ expanded: expandedSources.has(index) }">
                  <ArrowRight />
                </el-icon>
              </div>
              <div v-show="expandedSources.has(index)" class="source-list">
                <div
                  v-for="(source, sIndex) in JSON.parse(msg.sources as string)"
                  :key="sIndex"
                  class="source-item"
                  :class="{ 'has-image': source.image_path }"
                >
                  <!-- 图片缩略图（如果有） -->
                  <img
                    v-if="source.image_path"
                    :src="convertFileSrc(source.image_path)"
                    class="source-image"
                    @click.stop="previewImage(source.image_path)"
                    @error="handleImageError($event)"
                  />
                  <div class="source-info">
                    <span class="source-title">{{ source.document_title }}</span>
                    <span class="source-snippet">{{ source.snippet }}</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- 关联图片展示（直接显示，不需要展开） -->
            <div v-if="getMessageImages(msg).length > 0" class="message-images">
              <div class="images-label">📷 相关图片:</div>
              <div class="images-grid">
                <div
                  v-for="(img, imgIndex) in getMessageImages(msg)"
                  :key="imgIndex"
                  class="image-item"
                >
                  <el-image
                    :src="convertFileSrc(img.path)"
                    :preview-src-list="getAllImagePaths(msg)"
                    :initial-index="imgIndex"
                    fit="cover"
                    :hide-on-click-modal="true"
                  />
                  <div class="image-name">{{ img.name }}</div>
                </div>
              </div>
            </div>

            <!-- 保存为笔记按钮（仅 AI 回复显示，不包括流式输出） -->
            <div v-if="msg.role === 'assistant' && !('isStreaming' in msg)" class="message-actions">
              <el-button
                type="primary"
                text
                size="small"
                @click="handleSaveAsNote(msg as KbMessage)"
                title="保存为知识库笔记"
              >
                <el-icon><DocumentAdd /></el-icon>
                保存为笔记
              </el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 输入区域 -->
      <div class="input-area">
        <!-- API Key 未配置提示 -->
        <el-alert
          v-if="!hasAnyAiKey"
          type="warning"
          :closable="false"
          show-icon
          class="no-api-key-alert"
        >
          请先在设置中配置 AI 服务的 API Key（DeepSeek / 通义千问 / OpenAI / Gemini）
        </el-alert>
        <el-input
          v-model="inputMessage"
          type="textarea"
          :rows="2"
          :placeholder="hasAnyAiKey ? '输入问题，按 Enter 发送...' : '请先配置 API Key'"
          :disabled="isGenerating || !hasAnyAiKey"
          @keydown="handleKeyDown"
        />
        <div class="input-actions">
          <!-- 对话模式切换 -->
          <div class="custom-mode-selector">
            <div
              v-for="mode in [
                { value: 'strict', label: '严格模式', desc: '仅基于知识库内容回答' },
                { value: 'analysis', label: '分析模式', desc: '知识库 + AI 分析建议' },
                { value: 'direct', label: '对话模式', desc: '不检索知识库，直接对话' }
              ]"
              :key="mode.value"
              class="mode-item"
              :class="{ active: chatMode === mode.value }"
              @click="chatMode = mode.value as ChatMode"
            >
              <el-tooltip :content="mode.desc" placement="top" :show-after="500">
                <div class="mode-content">
                  <span>{{ mode.label }}</span>
                </div>
              </el-tooltip>
            </div>
          </div>
          <el-button
            v-if="!isGenerating"
            type="primary"
            :disabled="!inputMessage.trim() || !hasAnyAiKey"
            @click="sendMessage"
          >
            <el-icon><Promotion /></el-icon>
            发送
          </el-button>
          <el-button
            v-else
            type="danger"
            @click="stopGeneration"
          >
            <el-icon><VideoPause /></el-icon>
            停止
          </el-button>
        </div>
      </div>
    </div>

    <!-- PDF 依赖安装对话框 -->
    <el-dialog
      v-model="showPdfDepsDialog"
      title="安装 PDF 处理依赖"
      width="450px"
      :close-on-click-modal="!pdfDepsInstalling"
      :close-on-press-escape="!pdfDepsInstalling"
      :show-close="!pdfDepsInstalling"
    >
      <div class="pdf-deps-dialog">
        <div v-if="!pdfDepsInstalling" class="deps-status">
          <p>PDF 图片识别需要安装以下依赖：</p>
          <div class="deps-list">
            <div class="dep-item">
              <el-icon v-if="pdfDepsStatus?.poppler_installed" class="installed"><CircleCheck /></el-icon>
              <el-icon v-else class="not-installed"><CircleClose /></el-icon>
              <span>Poppler（PDF 转图片工具）</span>
            </div>
            <div class="dep-item">
              <el-icon v-if="pdfDepsStatus?.pdf2image_installed" class="installed"><CircleCheck /></el-icon>
              <el-icon v-else class="not-installed"><CircleClose /></el-icon>
              <span>pdf2image（Python 库）</span>
            </div>
          </div>
          <p class="tip">点击"自动安装"将尝试自动安装所需依赖。</p>
          <p class="tip">如果自动安装失败，可使用 Gemini API 直接处理 PDF（需配置 Gemini API Key）。</p>
        </div>

        <div v-else class="install-progress">
          <el-progress
            :percentage="pdfDepsProgress?.progress || 0"
            :status="pdfDepsProgress?.is_error ? 'exception' : undefined"
          />
          <p class="progress-step">{{ pdfDepsProgress?.step_name || '准备中...' }}</p>
          <p class="progress-message">{{ pdfDepsProgress?.message || '' }}</p>
        </div>
      </div>

      <template #footer>
        <span class="dialog-footer">
          <el-button
            v-if="!pdfDepsInstalling"
            @click="showPdfDepsDialog = false"
          >
            稍后处理
          </el-button>
          <el-button
            type="primary"
            :loading="pdfDepsInstalling"
            @click="installPdfDependencies"
          >
            {{ pdfDepsInstalling ? '安装中...' : '自动安装' }}
          </el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 图片预览对话框 -->
    <el-dialog
      v-model="imagePreviewVisible"
      title="图片预览"
      width="80%"
      :append-to-body="true"
      class="image-preview-dialog"
    >
      <div class="preview-image-container">
        <img
          v-if="previewImagePath"
          :src="convertFileSrc(previewImagePath)"
          class="preview-image"
        />
      </div>
    </el-dialog>

    <!-- 文档预览抽屉 -->
    <el-drawer
      v-model="showDocPreview"
      :title="previewDoc?.title || '文档预览'"
      direction="rtl"
      size="500px"
    >
      <template v-if="previewDoc">
        <div class="preview-drawer-content">
          <!-- 左侧：主要内容 -->
          <div class="preview-main">
            <!-- 文档信息 -->
            <div class="preview-info">
              <p><strong>文件名：</strong>{{ previewDoc.file_name }}</p>
              <p><strong>类型：</strong>{{ previewDoc.file_type.toUpperCase() }}</p>
              <p><strong>大小：</strong>{{ formatFileSize(previewDoc.file_size) }}</p>
              <p><strong>上传时间：</strong>{{ formatDate(previewDoc.created_at) }}</p>
              <p><strong>分块数：</strong>{{ previewDoc.chunk_count }}</p>
              <p v-if="previewDoc.embedding_total">
                <strong>向量化：</strong>
                <el-tag :type="previewDoc.embedding_count === previewDoc.embedding_total ? 'success' : 'warning'" size="small">
                  {{ previewDoc.embedding_count }}/{{ previewDoc.embedding_total }}
                </el-tag>
              </p>
            </div>

            <!-- 所属分类（多对多） -->
            <el-divider content-position="left">所属分类</el-divider>
            <div class="preview-categories">
              <el-tag
                v-for="cat in previewDocCategories"
                :key="cat.category_id"
                :color="cat.category_color"
                closable
                @close="handleRemoveDocumentCategory(previewDoc!.id, cat.category_id)"
                class="doc-category-tag"
              >
                {{ cat.category_name }}
              </el-tag>
              <el-dropdown trigger="click" v-if="categories.filter(c => !previewDocCategories.some(pc => pc.category_id === c.id)).length > 0" @command="(catId: number) => handleAddDocumentCategory(previewDoc!.id, catId)">
                <el-button type="primary" text size="small">
                  <el-icon><Plus /></el-icon> 添加分类
                </el-button>
                <template #dropdown>
                  <el-dropdown-menu>
                    <el-dropdown-item
                      v-for="cat in categories.filter(c => !previewDocCategories.some(pc => pc.category_id === c.id))"
                      :key="cat.id"
                      :command="cat.id"
                    >
                      <span class="category-color-dot" :style="{ backgroundColor: cat.color }"></span>
                      {{ cat.name }}
                    </el-dropdown-item>
                  </el-dropdown-menu>
                </template>
              </el-dropdown>
              <span v-if="previewDocCategories.length === 0 && categories.length === 0" class="no-categories-hint">
                暂无分类，请先在侧边栏创建
              </span>
            </div>

            <!-- 反向链接 -->
            <el-divider content-position="left">
              反向链接 ({{ previewDocBacklinks.length }})
            </el-divider>
            <div class="preview-backlinks">
              <div v-if="previewDocBacklinks.length === 0" class="empty-hint">
                暂无反向链接
              </div>
              <div
                v-for="link in previewDocBacklinks"
                :key="link.id"
                class="backlink-item"
                @click="handleNavigateToDocument(link.source_doc_id)"
              >
                <el-icon><Link /></el-icon>
                <span>{{ link.source_title }}</span>
              </div>
            </div>

            <!-- 出链 -->
            <el-divider content-position="left">
              出链 ({{ previewDocLinks.length }})
            </el-divider>
            <div class="preview-outlinks">
              <div v-if="previewDocLinks.length === 0" class="empty-hint">
                暂无出链
              </div>
              <div
                v-for="link in previewDocLinks"
                :key="link.id"
                class="outlink-item"
              >
                <span class="link-title" @click="handleNavigateToDocument(link.target_doc_id)">
                  <el-icon><Link /></el-icon>
                  {{ link.target_title }}
                </span>
                <el-button type="danger" text size="small" @click="handleRemoveDocumentLink(previewDoc!.id, link.target_doc_id)">
                  <el-icon><Delete /></el-icon>
                </el-button>
              </div>
              <!-- 添加链接 -->
              <el-dropdown trigger="click" v-if="documents.filter(d => d.id !== previewDoc?.id && !previewDocLinks.some(l => l.target_doc_id === d.id)).length > 0">
                <el-button type="primary" text size="small">
                  <el-icon><Plus /></el-icon> 添加链接
                </el-button>
                <template #dropdown>
                  <el-dropdown-menu class="link-dropdown">
                    <el-dropdown-item
                      v-for="doc in documents.filter(d => d.id !== previewDoc?.id && !previewDocLinks.some(l => l.target_doc_id === d.id)).slice(0, 20)"
                      :key="doc.id"
                      @click="handleAddDocumentLink(previewDoc!.id, doc.id)"
                    >
                      {{ doc.title }}
                    </el-dropdown-item>
                  </el-dropdown-menu>
                </template>
              </el-dropdown>
            </div>

            <el-divider content-position="left">分块内容</el-divider>

            <!-- 分块列表 -->
            <div v-loading="loadingPreview" class="chunk-list">
              <el-empty v-if="!loadingPreview && previewChunks.length === 0" description="暂无分块" />
              <el-collapse v-else>
                <el-collapse-item
                  v-for="chunk in previewChunks"
                  :key="chunk.id"
                  :name="chunk.id"
                >
                  <template #title>
                    <span class="chunk-title">
                      分块 {{ chunk.chunk_index + 1 }}
                      <el-tag v-if="chunk.page_number" size="small" type="info">第{{ chunk.page_number }}页</el-tag>
                      <el-tag v-if="chunk.image_path" size="small" type="warning">含图片</el-tag>
                    </span>
                  </template>
                  <div class="chunk-content">{{ chunk.content }}</div>
                </el-collapse-item>
              </el-collapse>
            </div>
          </div>

          <!-- 右侧：大纲导航 -->
          <div v-if="previewOutline.length > 0" class="preview-outline">
            <div class="outline-header">大纲</div>
            <div class="outline-items">
              <div
                v-for="item in previewOutline"
                :key="item.anchor"
                class="outline-item"
                :style="{ paddingLeft: (item.level - 1) * 12 + 'px' }"
                @click="scrollToOutlineItem(item.anchor)"
              >
                {{ item.text }}
              </div>
            </div>
          </div>
        </div>
      </template>
    </el-drawer>

    <!-- 批量上传对话框 -->
    <el-dialog
      v-model="showUploadDialog"
      title="文档上传"
      width="600px"
      :close-on-click-modal="!isUploading"
      :close-on-press-escape="!isUploading"
      :show-close="true"
      @close="closeUploadDialog"
    >
      <div class="upload-dialog">
        <!-- 总体进度 -->
        <div class="upload-summary">
          <div class="summary-stats">
            <span class="stat">
              <el-icon class="stat-icon completed"><CircleCheckFilled /></el-icon>
              {{ uploadQueue.filter(t => t.status === 'completed').length }} 完成
            </span>
            <span v-if="uploadQueue.filter(t => ['uploading', 'processing', 'embedding'].includes(t.status)).length > 0" class="stat">
              <el-icon class="stat-icon processing"><Loading /></el-icon>
              {{ uploadQueue.filter(t => ['uploading', 'processing', 'embedding'].includes(t.status)).length }} 处理中
            </span>
            <span v-if="uploadQueue.filter(t => t.status === 'pending').length > 0" class="stat">
              <el-icon class="stat-icon pending"><Clock /></el-icon>
              {{ uploadQueue.filter(t => t.status === 'pending').length }} 等待
            </span>
            <span v-if="uploadQueue.filter(t => t.status === 'failed').length > 0" class="stat">
              <el-icon class="stat-icon failed"><CircleCloseFilled /></el-icon>
              {{ uploadQueue.filter(t => t.status === 'failed').length }} 失败
            </span>
          </div>
          <el-progress
            :percentage="Math.round((uploadQueue.filter(t => t.status === 'completed' || t.status === 'failed' || t.status === 'cancelled').length / uploadQueue.length) * 100)"
            :status="uploadQueue.every(t => t.status === 'completed') ? 'success' : undefined"
          />
        </div>

        <!-- 上传队列列表 -->
        <div class="upload-queue">
          <div
            v-for="task in uploadQueue"
            :key="task.id"
            class="upload-task"
            :class="task.status"
          >
            <div class="task-info">
              <div class="task-name">
                <el-icon class="file-icon"><Document /></el-icon>
                <span class="file-name" :title="task.fileName">{{ task.fileName }}</span>
              </div>
              <div class="task-status">
                <el-tag :type="getTaskStatusType(task.status)" size="small">
                  {{ getTaskStatusLabel(task.status) }}
                </el-tag>
                <span class="task-message">{{ task.message }}</span>
              </div>
            </div>
            <div class="task-actions">
              <!-- 进度条（处理中） -->
              <el-progress
                v-if="['uploading', 'processing', 'embedding'].includes(task.status)"
                type="circle"
                :percentage="task.progress"
                :width="50"
                :stroke-width="4"
              />
              <!-- 完成图标 -->
              <el-icon v-else-if="task.status === 'completed'" class="status-icon success">
                <CircleCheckFilled />
              </el-icon>
              <!-- 失败图标 -->
              <el-icon v-else-if="task.status === 'failed'" class="status-icon failed">
                <CircleCloseFilled />
              </el-icon>
              <!-- 取消按钮（等待中） -->
              <el-button
                v-else-if="task.status === 'pending'"
                type="danger"
                text
                size="small"
                @click="cancelUploadTask(task.id)"
                title="取消上传"
              >
                <el-icon><Close /></el-icon>
              </el-button>
              <!-- 已取消 -->
              <el-icon v-else-if="task.status === 'cancelled'" class="status-icon cancelled">
                <RemoveFilled />
              </el-icon>
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <div class="upload-dialog-footer">
          <el-button
            v-if="isUploading"
            type="warning"
            @click="cancelAllUploads"
          >
            取消全部
          </el-button>
          <el-button
            :type="isUploading ? 'default' : 'primary'"
            @click="closeUploadDialog"
          >
            {{ isUploading ? '后台运行' : '关闭' }}
          </el-button>
        </div>
      </template>
    </el-dialog>

    <!-- 知识图谱对话框 -->
    <el-dialog
      v-model="showGraphView"
      title="知识图谱"
      width="90%"
      class="graph-dialog"
    >
      <div class="knowledge-graph">
        <div class="graph-stats">
          <span>{{ graphDocuments.length }} 个文档</span>
          <span>{{ graphLinks.length }} 条链接</span>
        </div>
        <div class="graph-container">
          <!-- 简易图谱视图（使用 CSS 力导向布局模拟） -->
          <div class="graph-nodes">
            <div
              v-for="(doc, index) in graphDocuments"
              :key="doc.id"
              class="graph-node"
              :style="{
                left: (20 + (index % 5) * 18) + '%',
                top: (15 + Math.floor(index / 5) * 20) + '%'
              }"
              @click="handleNavigateToDocument(doc.id); showGraphView = false"
              :title="doc.title"
            >
              <div class="node-circle" :class="{ 'has-links': graphLinks.some(l => l.source_doc_id === doc.id || l.target_doc_id === doc.id) }"></div>
              <div class="node-label">{{ doc.title.length > 15 ? doc.title.slice(0, 15) + '...' : doc.title }}</div>
            </div>
          </div>
          <!-- SVG 连线 -->
          <svg class="graph-edges" v-if="graphDocuments.length > 0">
            <line
              v-for="link in graphLinks"
              :key="link.id"
              :x1="getNodePosition(link.source_doc_id, 'x') + '%'"
              :y1="getNodePosition(link.source_doc_id, 'y') + '%'"
              :x2="getNodePosition(link.target_doc_id, 'x') + '%'"
              :y2="getNodePosition(link.target_doc_id, 'y') + '%'"
              class="graph-edge"
            />
          </svg>
        </div>
        <div class="graph-legend">
          <div class="legend-item">
            <span class="legend-dot has-links"></span>
            <span>有链接的文档</span>
          </div>
          <div class="legend-item">
            <span class="legend-dot"></span>
            <span>独立文档</span>
          </div>
        </div>
      </div>
    </el-dialog>
  </div>
</template>

<style scoped>
.knowledge-base-container {
  display: flex;
  height: 100%;
  background: var(--el-bg-color);
}

/* 侧边栏 */
.sidebar {
  width: 280px;
  border-right: 1px solid var(--el-border-color);
  display: flex;
  flex-direction: column;
  background: var(--el-bg-color-page);
}

.sidebar-tabs {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  border-bottom: 1px solid var(--el-border-color);
}

.section-toggle {
  flex: 1;
}

.sidebar-tabs .help-btn {
  color: var(--el-text-color-secondary);
  border-color: var(--el-border-color-light);
}

.sidebar-tabs .help-btn:hover {
  color: var(--el-color-primary);
  border-color: var(--el-color-primary-light-5);
}

.section-toggle .el-button {
  flex: 1;
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  border-bottom: 1px solid var(--el-border-color);
  font-weight: 500;
}

.list-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

/* 对话列表 */
.conversation-item {
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  margin-bottom: 4px;
  transition: background 0.2s;
}

.conversation-item:hover {
  background: var(--el-fill-color);
}

.conversation-item.active {
  background: var(--el-color-primary-light-9);
}

.conv-title {
  font-size: 14px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.conv-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 4px;
}

.conv-provider {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.delete-btn {
  opacity: 0;
  transition: opacity 0.2s;
}

.conversation-item:hover .delete-btn {
  opacity: 1;
}

/* 分类侧边栏（知识库模式下占满整个左侧边栏） */
.category-sidebar {
  flex: 1;
  padding: 12px;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}

.category-header {
  font-size: 12px;
  font-weight: 500;
  color: var(--el-text-color-secondary);
  padding: 4px 8px;
  margin-bottom: 4px;
}

.category-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  transition: background 0.2s;
}

.category-item:hover {
  background: var(--el-fill-color-light);
}

.category-item.active {
  background: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
}

.category-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.category-count {
  font-size: 11px;
  color: var(--el-text-color-secondary);
  background: var(--el-fill-color);
  padding: 0 6px;
  border-radius: 10px;
}

.category-more {
  opacity: 0;
  cursor: pointer;
  transition: opacity 0.2s;
}

.category-item:hover .category-more {
  opacity: 1;
}

.add-category {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  margin-top: 8px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  color: var(--el-text-color-secondary);
  border: 1px dashed var(--el-border-color);
}

.add-category:hover {
  color: var(--el-color-primary);
  border-color: var(--el-color-primary);
  background: var(--el-color-primary-light-9);
}

/* 颜色选择器 */
.color-picker-wrapper {
  display: flex;
  align-items: center;
  padding: 4px 0;
}

/* 拖拽手柄 */
.drag-handle {
  cursor: grab;
  color: var(--el-text-color-placeholder);
  opacity: 0;
  transition: opacity 0.2s;
  margin-right: 4px;
}

.category-item:hover .drag-handle {
  opacity: 1;
}

.drag-handle:active {
  cursor: grabbing;
}

/* vuedraggable 拖拽中的幽灵效果 */
.category-ghost {
  opacity: 0.5;
  background: var(--el-color-primary-light-9);
  border: 2px dashed var(--el-color-primary);
}

/* 实心文件夹图标 */
.folder-filled {
  color: var(--folder-color, #409EFF);
}

.folder-filled svg,
.folder-filled svg path {
  fill: var(--folder-color, #409EFF) !important;
  stroke: none !important;
}

/* 分类下拉菜单项 */
.category-dropdown-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.category-color-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}

/* 文档列表区（中间区域） */
.document-list-area {
  width: 360px;
  border-right: 1px solid var(--el-border-color);
  display: flex;
  flex-direction: column;
  background: var(--el-bg-color-page);
  flex-shrink: 0;
}

.doc-area-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--el-border-color);
  background: var(--el-bg-color);
}

.header-title {
  font-weight: 500;
  font-size: 14px;
}

.doc-count-badge {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

/* Embedding 进度条 */
.embedding-progress {
  padding: 16px;
  margin: 12px;
  background: var(--el-color-primary-light-9);
  border-radius: 8px;
  border: 1px solid var(--el-color-primary-light-7);
}

.progress-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 10px;
  font-size: 13px;
  color: var(--el-text-color-regular);
}

.doc-list-content {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
}

.doc-toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px;
  border-bottom: 1px solid var(--el-border-color);
  background: var(--el-bg-color);
}

/* 文档列表 */
.document-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  border-radius: 6px;
  margin-bottom: 4px;
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-light);
}

.doc-info {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
  min-width: 0;
}

.doc-icon {
  font-size: 24px;
  color: var(--el-color-primary);
}

.doc-details {
  flex: 1;
  min-width: 0;
}

.doc-title {
  font-size: 14px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.doc-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.embedding-tag {
  display: inline-flex !important;
  align-items: center !important;
  white-space: nowrap !important;
  gap: 2px !important;
  flex-wrap: nowrap !important;
}

.embedding-tag :deep(.el-tag__content) {
  display: inline-flex !important;
  align-items: center !important;
  gap: 2px !important;
  flex-wrap: nowrap !important;
}

.processing-hint {
  color: var(--el-color-warning);
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

/* 文档预览抽屉 */
.preview-info {
  font-size: 14px;
  line-height: 1.8;
  padding: 0 4px;
}

.preview-info p {
  margin: 6px 0;
}

.chunk-list {
  max-height: calc(100vh - 320px);
  overflow-y: auto;
}

.chunk-title {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.chunk-content {
  white-space: pre-wrap;
  font-size: 13px;
  color: var(--el-text-color-regular);
  max-height: 300px;
  overflow-y: auto;
  padding: 8px;
  background: var(--el-fill-color-light);
  border-radius: 4px;
}

.document-item {
  cursor: pointer;
  transition: background-color 0.2s;
}

.document-item:hover {
  background-color: var(--el-fill-color-light);
}

.empty-state {
  text-align: center;
  padding: 40px 20px;
  color: var(--el-text-color-secondary);
}

.empty-hint {
  font-size: 12px;
  margin-top: 8px;
}

/* 主内容区 */
.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
}

/* AI 设置栏 */
.ai-settings {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--el-border-color);
  background: var(--el-bg-color-page);
}

.setting-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.setting-item .label {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.doc-count {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

/* 消息区域 */
.messages-area {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.welcome-message {
  text-align: center;
  padding: 60px 20px;
  color: var(--el-text-color-secondary);
}

.welcome-icon {
  margin-bottom: 16px;
  color: var(--el-color-primary);
}

.welcome-message h3 {
  margin: 0 0 8px;
  font-size: 18px;
  color: var(--el-text-color-primary);
}

.welcome-message p {
  margin: 0;
  font-size: 14px;
}

.quick-tips {
  display: flex;
  justify-content: center;
  gap: 24px;
  margin-top: 32px;
}

.tip {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
}

/* 消息样式 */
.message {
  margin-bottom: 16px;
}

.message.user {
  display: flex;
  justify-content: flex-end;
}

.message.user .message-content {
  background: var(--el-color-primary);
  color: white;
  border-radius: 12px 12px 4px 12px;
}

.message.assistant .message-content {
  background: var(--el-fill-color);
  border-radius: 12px 12px 12px 4px;
}

.message-content {
  max-width: 80%;
  padding: 12px 16px;
}

.message-text {
  word-break: break-word;
  line-height: 1.6;
}

.message-text :deep(h1),
.message-text :deep(h2),
.message-text :deep(h3) {
  margin: 0.5em 0;
  font-weight: 600;
}

.message-text :deep(h1) { font-size: 1.3em; }
.message-text :deep(h2) { font-size: 1.2em; }
.message-text :deep(h3) { font-size: 1.1em; }

.message-text :deep(p) {
  margin: 0.5em 0;
}

.message-text :deep(ul),
.message-text :deep(ol) {
  padding-left: 1.5em;
  margin: 0.5em 0;
}

.message-text :deep(li) {
  margin: 0.25em 0;
}

.message-text :deep(strong) {
  font-weight: 600;
}

.message-text :deep(code) {
  background: var(--el-fill-color);
  padding: 0.2em 0.4em;
  border-radius: 3px;
  font-family: monospace;
  font-size: 0.9em;
}

.message-text :deep(pre) {
  background: var(--el-fill-color);
  padding: 1em;
  border-radius: 6px;
  overflow-x: auto;
  margin: 0.5em 0;
}

.message-text :deep(pre code) {
  background: transparent;
  padding: 0;
}

.message-text :deep(blockquote) {
  border-left: 3px solid var(--el-border-color);
  padding-left: 1em;
  margin: 0.5em 0;
  color: var(--el-text-color-secondary);
}

.message-text :deep(.source-ref) {
  font-size: 0.75em;
  color: var(--el-color-primary);
  vertical-align: super;
}

.message-sources {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--el-border-color);
}

.sources-title {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  cursor: pointer;
  user-select: none;
}

.sources-title:hover {
  color: var(--el-color-primary);
}

.expand-icon {
  margin-left: auto;
  transition: transform 0.2s;
}

.expand-icon.expanded {
  transform: rotate(90deg);
}

.source-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 8px;
}

.source-item {
  font-size: 12px;
  padding: 6px 10px;
  background: var(--el-bg-color);
  border-radius: 4px;
  display: flex;
  gap: 10px;
  align-items: flex-start;
}

.source-item.has-image {
  flex-direction: row;
}

.source-image {
  width: 60px;
  height: 60px;
  object-fit: cover;
  border-radius: 4px;
  cursor: pointer;
  flex-shrink: 0;
  border: 1px solid var(--el-border-color);
  transition: transform 0.2s, box-shadow 0.2s;
}

.source-image:hover {
  transform: scale(1.05);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.source-info {
  flex: 1;
  min-width: 0;
}

.source-title {
  font-weight: 500;
  color: var(--el-color-primary);
}

.source-snippet {
  display: block;
  margin-top: 4px;
  color: var(--el-text-color-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 消息关联图片展示 */
.message-images {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--el-border-color-lighter);
}

.images-label {
  font-size: 13px;
  color: var(--el-text-color-secondary);
  margin-bottom: 8px;
}

.images-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.image-item {
  width: 120px;
}

.image-item .el-image {
  width: 120px;
  height: 90px;
  border-radius: 6px;
  cursor: pointer;
  border: 1px solid var(--el-border-color-lighter);
  transition: transform 0.2s, box-shadow 0.2s;
}

.image-item .el-image:hover {
  transform: scale(1.02);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
}

.image-name {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  text-align: center;
  margin-top: 4px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 图片预览对话框 */
.preview-image-container {
  display: flex;
  justify-content: center;
  align-items: center;
  max-height: 70vh;
  overflow: auto;
}

.preview-image {
  max-width: 100%;
  max-height: 70vh;
  object-fit: contain;
}

/* 输入区域 */
.input-area {
  padding: 16px;
  border-top: 1px solid var(--el-border-color);
  background: var(--el-bg-color-page);
}


.custom-mode-selector {
  display: flex;
  align-items: center;
  background: var(--el-fill-color);
  padding: 4px;
  border-radius: 8px;
  gap: 4px;
}

.mode-item {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 6px 12px;
  border-radius: 6px;
  cursor: pointer;
  color: var(--el-text-color-regular);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  font-size: 13px;
  user-select: none;
}

.mode-item:hover:not(.active) {
  background: var(--el-fill-color-darker);
  color: var(--el-text-color-primary);
}

.mode-item.active {
  background: var(--el-color-primary);
  color: white;
  box-shadow: 0 2px 4px rgba(64, 158, 255, 0.3);
  font-weight: 500;
}

.mode-content {
  display: flex;
  align-items: center;
  gap: 6px;
}

.mode-content .el-icon {
  font-size: 14px;
}

.no-api-key-alert {
  margin-bottom: 10px;
}

.input-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 10px;
}

/* PDF 依赖安装对话框 */
.pdf-deps-dialog .deps-status p {
  margin: 0 0 12px;
  color: var(--el-text-color-regular);
}

.pdf-deps-dialog .deps-list {
  background: var(--el-fill-color-light);
  border-radius: 8px;
  padding: 12px 16px;
  margin: 16px 0;
}

.pdf-deps-dialog .dep-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 0;
}

.pdf-deps-dialog .dep-item:not(:last-child) {
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.pdf-deps-dialog .dep-item .installed {
  color: var(--el-color-success);
}

.pdf-deps-dialog .dep-item .not-installed {
  color: var(--el-color-danger);
}

.pdf-deps-dialog .dep-item span {
  font-size: 14px;
}

.pdf-deps-dialog .tip {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin: 8px 0;
}

.pdf-deps-dialog .install-progress {
  text-align: center;
  padding: 20px 0;
}

.pdf-deps-dialog .install-progress .progress-step {
  margin-top: 16px;
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.pdf-deps-dialog .install-progress .progress-message {
  margin-top: 8px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  word-break: break-all;
}

/* 批量上传对话框 */
.upload-dialog {
  max-height: 60vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.upload-summary {
  margin-bottom: 16px;
  padding: 16px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
}

.summary-stats {
  display: flex;
  gap: 20px;
  margin-bottom: 12px;
}

.summary-stats .stat {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--el-text-color-regular);
}

.stat-icon {
  font-size: 16px;
}

.stat-icon.completed {
  color: var(--el-color-success);
}

.stat-icon.processing {
  color: var(--el-color-primary);
  animation: spin 1s linear infinite;
}

.stat-icon.pending {
  color: var(--el-text-color-secondary);
}

.stat-icon.failed {
  color: var(--el-color-danger);
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.upload-queue {
  flex: 1;
  overflow-y: auto;
  max-height: 400px;
}

.upload-task {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  margin-bottom: 8px;
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-light);
  border-radius: 8px;
  transition: all 0.2s;
}

.upload-task.completed {
  background: var(--el-color-success-light-9);
  border-color: var(--el-color-success-light-5);
}

.upload-task.failed {
  background: var(--el-color-danger-light-9);
  border-color: var(--el-color-danger-light-5);
}

.upload-task.cancelled {
  background: var(--el-color-warning-light-9);
  border-color: var(--el-color-warning-light-5);
  opacity: 0.7;
}

.upload-task.uploading,
.upload-task.processing,
.upload-task.embedding {
  background: var(--el-color-primary-light-9);
  border-color: var(--el-color-primary-light-5);
}

.task-info {
  flex: 1;
  min-width: 0;
}

.task-name {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.task-name .file-icon {
  font-size: 18px;
  color: var(--el-color-primary);
  flex-shrink: 0;
}

.task-name .file-name {
  font-size: 14px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-status {
  display: flex;
  align-items: center;
  gap: 8px;
}

.task-message {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-actions {
  flex-shrink: 0;
  margin-left: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
}

.status-icon {
  font-size: 24px;
}

.status-icon.success {
  color: var(--el-color-success);
}

.status-icon.failed {
  color: var(--el-color-danger);
}

.status-icon.cancelled {
  color: var(--el-color-warning);
}

.upload-dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

/* ==================== 知识图谱按钮 ==================== */
.graph-btn-container {
  padding: 12px;
  border-top: 1px solid var(--el-border-color-lighter);
  margin-top: auto;
}

.graph-btn {
  width: 100%;
}

/* ==================== 文档预览抽屉增强 ==================== */
.preview-drawer-content {
  display: flex;
  height: 100%;
}

.preview-main {
  flex: 1;
  overflow-y: auto;
  padding-right: 12px;
}

/* 文档所属分类（多对多） */
.preview-categories {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  padding: 8px 0;
}

.doc-category-tag {
  color: white !important;
}

.category-color-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
  display: inline-block;
  margin-right: 6px;
}

.no-categories-hint {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.preview-backlinks,
.preview-outlinks {
  padding: 8px 0;
}

.backlink-item,
.outlink-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  transition: background 0.2s;
}

.backlink-item:hover,
.outlink-item:hover {
  background: var(--el-fill-color-light);
}

.outlink-item {
  justify-content: space-between;
}

.link-title {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
}

.link-title:hover {
  color: var(--el-color-primary);
}

.link-dropdown {
  max-height: 300px;
  overflow-y: auto;
}

/* 大纲导航 */
.preview-outline {
  width: 150px;
  border-left: 1px solid var(--el-border-color-lighter);
  padding-left: 12px;
  overflow-y: auto;
}

.outline-header {
  font-size: 12px;
  font-weight: 500;
  color: var(--el-text-color-secondary);
  padding: 4px 0 8px;
}

.outline-items {
  font-size: 12px;
}

.outline-item {
  padding: 4px 8px;
  cursor: pointer;
  border-radius: 4px;
  transition: background 0.2s;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.outline-item:hover {
  background: var(--el-fill-color-light);
  color: var(--el-color-primary);
}

/* ==================== 知识图谱 ==================== */
.knowledge-graph {
  height: 60vh;
  display: flex;
  flex-direction: column;
}

.graph-stats {
  display: flex;
  gap: 16px;
  padding: 8px 0;
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.graph-container {
  flex: 1;
  position: relative;
  background: var(--el-fill-color-lighter);
  border-radius: 8px;
  overflow: hidden;
}

.graph-nodes {
  position: absolute;
  inset: 0;
}

.graph-node {
  position: absolute;
  transform: translate(-50%, -50%);
  cursor: pointer;
  text-align: center;
  transition: transform 0.2s;
}

.graph-node:hover {
  transform: translate(-50%, -50%) scale(1.1);
  z-index: 10;
}

.node-circle {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: var(--el-color-info-light-3);
  margin: 0 auto 4px;
  transition: all 0.2s;
}

.node-circle.has-links {
  background: var(--el-color-primary);
  box-shadow: 0 0 8px var(--el-color-primary-light-5);
}

.node-label {
  font-size: 11px;
  max-width: 100px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  background: rgba(255, 255, 255, 0.8);
  padding: 2px 6px;
  border-radius: 4px;
}

.graph-edges {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}

.graph-edge {
  stroke: var(--el-color-primary-light-5);
  stroke-width: 1.5;
  stroke-linecap: round;
}

.graph-legend {
  display: flex;
  gap: 16px;
  padding: 12px 0;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.legend-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--el-color-info-light-3);
}

.legend-dot.has-links {
  background: var(--el-color-primary);
}

/* ==================== Callouts 样式 ==================== */
:deep(.callout) {
  border-radius: 8px;
  padding: 12px 16px;
  margin: 12px 0;
  border-left: 4px solid var(--callout-color, #409EFF);
  background: color-mix(in srgb, var(--callout-color, #409EFF) 10%, transparent);
}

:deep(.callout-header) {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
  margin-bottom: 8px;
}

:deep(.callout-icon) {
  font-size: 16px;
}

:deep(.callout-title) {
  color: var(--callout-color, #409EFF);
}

:deep(.callout-content) {
  font-size: 14px;
  line-height: 1.6;
}

:deep(.callout-content p:last-child) {
  margin-bottom: 0;
}

/* Wiki 链接样式 */
:deep(.wiki-link) {
  color: var(--el-color-primary);
  background: var(--el-color-primary-light-9);
  padding: 2px 6px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.2s;
}

:deep(.wiki-link:hover) {
  background: var(--el-color-primary-light-7);
}

/* ==================== 消息操作按钮 ==================== */
.message-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid var(--el-border-color-lighter);
}

.message-actions .el-button {
  opacity: 0.7;
  transition: opacity 0.2s;
}

.message-actions .el-button:hover {
  opacity: 1;
}
</style>

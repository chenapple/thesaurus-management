<script setup lang="ts">
import { ref, onMounted, computed, nextTick, watch } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { open } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';
import { convertFileSrc } from '@tauri-apps/api/core';
import * as api from '../api';
import { chatStream, buildRAGSystemPrompt, parseSourceReferences, checkApiKeyConfigured, recognizeImage, getTextEmbedding, getTextEmbeddingsBatchParallel } from '../ai-service';
import type { ChatMessage } from '../ai-service';
import type { KbDocument, KbConversation, KbMessage, KbSearchResult, KbChunk, AIProvider, DependencyStatus, InstallProgress } from '../types';
import { AI_PROVIDERS } from '../types';
import { marked } from 'marked';

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
  document_count?: number;
}
const categories = ref<KbCategory[]>([]);
const selectedCategory = ref<number | null>(null);

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
    previewChunks.value = await api.kbGetChunks(doc.id);
  } catch (e) {
    console.error('加载分块失败:', e);
    previewChunks.value = [];
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

  // 1. 按分类筛选
  if (selectedCategory.value !== null) {
    result = result.filter(doc => doc.category_id === selectedCategory.value);
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

// 计算每个分类的文档数
const getCategoryDocCount = (categoryId: number | null) => {
  if (categoryId === null) {
    return documents.value.length;
  }
  return documents.value.filter(doc => doc.category_id === categoryId).length;
};

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

  for (const filePath of files) {
    try {
      // 提取文件信息
      const parts = filePath.split(/[/\\]/);
      const fileName = parts[parts.length - 1];
      const ext = fileName.split('.').pop()?.toLowerCase() || 'txt';
      const title = fileName.replace(/\.[^.]+$/, '');

      // 检测重复文件
      const existingDoc = documents.value.find(
        doc => doc.file_name === fileName || doc.title === title
      );

      if (existingDoc) {
        try {
          await ElMessageBox.confirm(
            `文档 "${title}" 已存在，是否覆盖？\n\n覆盖将删除原文档及其所有分块数据。`,
            '检测到重复文件',
            {
              confirmButtonText: '覆盖',
              cancelButtonText: '跳过',
              type: 'warning',
            }
          );
          // 用户选择覆盖，删除旧文档
          await api.kbDeleteDocument(existingDoc.id);
          await loadDocuments();
        } catch {
          // 用户选择跳过
          ElMessage.info(`已跳过 "${title}"`);
          continue;
        }
      }

      // 添加文档记录
      const docId = await api.kbAddDocument(null, title, fileName, filePath, ext);

      // 处理文档（解析 + 分块）
      processingDocuments.value.add(docId);
      await loadDocuments();

      let chunkCount = 0;

      try {
        chunkCount = await api.kbProcessDocument(docId, filePath);
      } catch (processError: any) {
        // PDF 文本提取失败，尝试使用 OCR
        if (ext === 'pdf' && processError?.toString().includes('未提取到文本')) {
          console.log('[PDF] 文本提取失败，尝试使用 AI OCR...');
          const ocrChunks = await processPdfWithOcr(docId, filePath);
          if (ocrChunks > 0) {
            chunkCount = ocrChunks;
            ElMessage.info('PDF 使用 AI OCR 识别成功');
          } else {
            throw new Error('PDF OCR 识别失败，请确保已配置通义千问 API Key');
          }
        } else {
          throw processError;
        }
      }

      // 对于支持图片的文档格式，提取并识别图片
      const imageTypes = ['xlsx', 'xls', 'pptx', 'docx'];
      if (imageTypes.includes(ext)) {
        const imageChunks = await processDocumentImages(docId, filePath);
        if (imageChunks > 0) {
          chunkCount += imageChunks;
          // 更新数据库中的分块数
          await api.kbUpdateDocumentStatus(docId, 'completed', chunkCount);
          ElMessage.info(`识别了 ${imageChunks} 张图片内容`);
        }
      }

      // 生成 embedding 向量（用于语义搜索）
      await generateDocumentEmbeddings(docId);

      processingDocuments.value.delete(docId);
      await loadDocuments();

      ElMessage.success(`文档 "${title}" 处理完成，生成 ${chunkCount} 个分块`);
    } catch (e) {
      console.error('上传文档失败:', e);
      ElMessage.error(`上传失败: ${e}`);
    }
  }
}

/**
 * 处理文档中的图片：提取图片 -> AI 识别（通义千问/Gemini） -> 存储为 chunk
 */
async function processDocumentImages(documentId: number, filePath: string): Promise<number> {
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
    ElMessage.success('文档已删除');
  } catch (e) {
    if (e !== 'cancel') {
      console.error('删除文档失败:', e);
      ElMessage.error('删除失败');
    }
  }
}

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

  // 3. 按相关度排序，取 Top 10
  const finalResults = results
    .sort((a, b) => (b.score || 0) - (a.score || 0))
    .slice(0, 10);

  console.log(`混合检索最终返回 ${finalResults.length} 个结果`);
  return finalResults;
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

  // 混合检索：向量搜索 + 关键词搜索
  const searchResults = await hybridSearch(content);

  // 构建消息（只保留最近 N 条历史消息作为上下文）
  const systemPrompt = buildRAGSystemPrompt(searchResults);
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

    // 解析来源引用
    const sources = parseSourceReferences(streamingContent.value, searchResults);

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
function renderMarkdown(content: string): string {
  let html = marked(content, { breaks: true }) as string;
  // 将 [来源X] 包裹在 span 中以便样式化
  html = html.replace(/\[来源(\d+)\]/g, '<span class="source-ref">[来源$1]</span>');
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
  await Promise.all([loadDocuments(), loadConversations(), loadCategories()]);
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
          @click="selectedCategory = null"
        >
          <el-icon><Folder /></el-icon>
          <span class="category-name">全部文档</span>
          <span class="category-count">{{ getCategoryDocCount(null) }}</span>
        </div>
        <div
          v-for="cat in categories"
          :key="cat.id"
          class="category-item"
          :class="{ active: selectedCategory === cat.id }"
          @click="selectedCategory = cat.id"
        >
          <el-icon><FolderOpened /></el-icon>
          <span class="category-name">{{ cat.name }}</span>
          <span class="category-count">{{ getCategoryDocCount(cat.id) }}</span>
          <el-dropdown trigger="click" @command="(cmd: string) => handleCategoryAction(cmd, cat)">
            <el-icon class="category-more" @click.stop><MoreFilled /></el-icon>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="rename">重命名</el-dropdown-item>
                <el-dropdown-item command="delete" divided>删除</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
        <div class="add-category" @click="handleAddCategory">
          <el-icon><Plus /></el-icon>
          <span>新建分类</span>
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
          <!-- 重新向量化按钮 -->
          <el-button
            v-if="doc.status === 'completed' && doc.embedding_total && doc.embedding_count !== doc.embedding_total"
            type="warning"
            text
            size="small"
            @click.stop="handleReEmbed(doc)"
            title="重新向量化"
          >
            <el-icon><Refresh /></el-icon>
          </el-button>
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
          </div>
        </div>
      </div>

      <!-- 输入区域 -->
      <div class="input-area">
        <el-input
          v-model="inputMessage"
          type="textarea"
          :rows="2"
          placeholder="输入问题，按 Enter 发送..."
          :disabled="isGenerating"
          @keydown="handleKeyDown"
        />
        <div class="input-actions">
          <el-button
            v-if="!isGenerating"
            type="primary"
            :disabled="!inputMessage.trim()"
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
      size="420px"
    >
      <template v-if="previewDoc">
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
      </template>
    </el-drawer>
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
  padding: 12px;
  border-bottom: 1px solid var(--el-border-color);
}

.section-toggle {
  width: 100%;
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

.input-actions {
  display: flex;
  justify-content: flex-end;
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
</style>

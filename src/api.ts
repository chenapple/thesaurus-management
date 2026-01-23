import { invoke } from "@tauri-apps/api/core";
import type { BackupInfo, Category, KeywordData, KeywordMonitoring, MonitoringSparkline, MonitoringStats, Product, RankingHistory, RankingResult, RankingSnapshot, Root, ScAnalysis, TrafficLevelStats, WorkflowStatus } from "./types";

// ==================== 产品管理 ====================

export async function getProducts(): Promise<Product[]> {
  return await invoke("get_products");
}

export async function createProduct(
  name: string,
  country?: string
): Promise<number> {
  return await invoke("create_product", {
    name,
    country: country || null,
  });
}

export async function updateProduct(
  id: number,
  name: string,
  country?: string
): Promise<void> {
  return await invoke("update_product", {
    id,
    name,
    country: country || null,
  });
}

export async function deleteProduct(id: number): Promise<void> {
  return await invoke("delete_product", { id });
}

export async function updateProductHeaders(
  id: number,
  cpcHeader?: string,
  bidRangeHeader?: string
): Promise<void> {
  return await invoke("update_product_headers", {
    id,
    cpcHeader: cpcHeader || null,
    bidRangeHeader: bidRangeHeader || null,
  });
}

// ==================== 分类 ====================

export async function getCategories(): Promise<Category[]> {
  return await invoke("get_categories");
}

// ==================== 关键词和词根 ====================

export async function importKeywords(
  productId: number,
  keywords: string[]
): Promise<void> {
  return await invoke("import_keywords", { productId, keywords });
}

export async function getRoots(params: {
  productId?: number;
  search?: string;
  categoryIds?: number[];
  sortBy?: string;
  sortOrder?: string;
  page: number;
  pageSize: number;
}): Promise<[Root[], number]> {
  return await invoke("get_roots", {
    productId: params.productId || null,
    search: params.search || null,
    categoryIds: params.categoryIds?.length ? params.categoryIds : null,
    sortBy: params.sortBy || null,
    sortOrder: params.sortOrder || null,
    page: params.page,
    pageSize: params.pageSize,
  });
}

export async function updateRootTranslation(
  id: number,
  translation: string
): Promise<void> {
  return await invoke("update_root_translation", { id, translation });
}

export async function addRootCategory(
  rootId: number,
  categoryId: number
): Promise<void> {
  return await invoke("add_root_category", { rootId, categoryId });
}

export async function removeRootCategory(
  rootId: number,
  categoryId: number
): Promise<void> {
  return await invoke("remove_root_category", { rootId, categoryId });
}

export async function getStats(productId?: number): Promise<[number, number]> {
  return await invoke("get_stats", { productId: productId || null });
}

export async function getCategoryCounts(
  productId: number
): Promise<[number, number][]> {
  return await invoke("get_category_counts", { productId });
}

export async function clearProductData(productId: number): Promise<void> {
  return await invoke("clear_product_data", { productId });
}

export async function getUntranslatedRoots(productId: number): Promise<string[]> {
  return await invoke("get_untranslated_roots", { productId });
}

export async function batchUpdateRootAnalysis(
  productId: number,
  updates: [string, string, string[]][]
): Promise<void> {
  return await invoke("batch_update_root_analysis", { productId, updates });
}

// ==================== 关键词完整数据 ====================

export async function importKeywordData(
  productId: number,
  dataList: KeywordData[]
): Promise<void> {
  return await invoke("import_keyword_data", { productId, dataList });
}

export async function getKeywordData(params: {
  productId: number;
  search?: string;
  trafficLevels?: string[];
  relevanceLevels?: string[];
  primaryCategories?: string[];
  orderlinessValues?: string[];
  sortBy?: string;
  sortOrder?: string;
  page: number;
  pageSize: number;
}): Promise<[KeywordData[], number]> {
  return await invoke("get_keyword_data", {
    productId: params.productId,
    search: params.search || null,
    trafficLevels: params.trafficLevels?.length ? params.trafficLevels : null,
    relevanceLevels: params.relevanceLevels?.length ? params.relevanceLevels : null,
    primaryCategories: params.primaryCategories?.length ? params.primaryCategories : null,
    orderlinessValues: params.orderlinessValues?.length ? params.orderlinessValues : null,
    sortBy: params.sortBy || null,
    sortOrder: params.sortOrder || null,
    page: params.page,
    pageSize: params.pageSize,
  });
}

export async function updateKeywordField(
  id: number,
  field: string,
  value: string
): Promise<void> {
  return await invoke("update_keyword_field", { id, field, value });
}

export async function clearKeywordData(productId: number): Promise<void> {
  return await invoke("clear_keyword_data", { productId });
}

export async function getKeywordDataStats(productId: number): Promise<number> {
  return await invoke("get_keyword_data_stats", { productId });
}

// ==================== 流量级别管理 ====================

export async function updateProductThresholds(
  id: number,
  bigWordThreshold: number,
  mediumWordThreshold: number
): Promise<void> {
  return await invoke("update_product_thresholds", {
    id,
    bigWordThreshold,
    mediumWordThreshold,
  });
}

export async function calculateTrafficLevels(
  productId: number,
  bigThreshold: number,
  mediumThreshold: number
): Promise<void> {
  return await invoke("calculate_traffic_levels", {
    productId,
    bigThreshold,
    mediumThreshold,
  });
}

export async function getTrafficLevelStats(
  productId: number
): Promise<TrafficLevelStats> {
  return await invoke("get_traffic_level_stats", { productId });
}

export async function recommendThreshold(
  productId: number,
  targetBigCount: number = 20
): Promise<number> {
  return await invoke("recommend_threshold", { productId, targetBigCount });
}

// ==================== 流量占比计算 ====================

export async function calculateTrafficShare(productId: number): Promise<void> {
  return await invoke("calculate_traffic_share", { productId });
}

// ==================== 关键词分类管理 ====================

export interface UncategorizedKeyword {
  id: number;
  keyword: string;
  translation: string | null;
}

export async function getUncategorizedKeywords(
  productId: number
): Promise<UncategorizedKeyword[]> {
  return await invoke("get_uncategorized_keywords", { productId });
}

export async function batchUpdateKeywordCategories(
  productId: number,
  updates: [string, string, string, string][]
): Promise<void> {
  return await invoke("batch_update_keyword_categories", { productId, updates });
}

// ==================== 词组打标 ====================

export async function calculatePhraseTags(productId: number): Promise<void> {
  return await invoke("calculate_phrase_tags", { productId });
}

// ==================== 有序性计算 ====================

export async function calculateOrderliness(productId: number): Promise<void> {
  return await invoke("calculate_orderliness", { productId });
}

// ==================== 流程状态 ====================

export async function getWorkflowStatus(productId: number): Promise<WorkflowStatus> {
  return await invoke("get_workflow_status", { productId });
}

// ==================== 备份管理 ====================

export async function createBackup(
  productId: number,
  backupName?: string
): Promise<number> {
  return await invoke("create_backup", {
    productId,
    backupName: backupName || null,
  });
}

export async function getBackups(productId: number): Promise<BackupInfo[]> {
  return await invoke("get_backups", { productId });
}

export async function restoreBackup(backupId: number): Promise<void> {
  return await invoke("restore_backup", { backupId });
}

export async function deleteBackup(backupId: number): Promise<void> {
  return await invoke("delete_backup", { backupId });
}

// ==================== API Key 安全存储 ====================

/**
 * 保存 API Key 到系统密钥链
 * @param keyName 密钥名称，如 "deepseek"
 * @param apiKey API Key 值
 */
export async function setApiKey(keyName: string, apiKey: string): Promise<void> {
  return await invoke("set_api_key", { keyName, apiKey });
}

/**
 * 从系统密钥链获取 API Key
 * @param keyName 密钥名称
 * @returns API Key 值，如果不存在返回 null
 */
export async function getApiKey(keyName: string): Promise<string | null> {
  return await invoke("get_api_key", { keyName });
}

/**
 * 从系统密钥链删除 API Key
 * @param keyName 密钥名称
 */
export async function deleteApiKey(keyName: string): Promise<void> {
  return await invoke("delete_api_key", { keyName });
}

/**
 * 检查 API Key 是否存在
 * @param keyName 密钥名称
 * @returns 是否存在
 */
export async function hasApiKey(keyName: string): Promise<boolean> {
  return await invoke("has_api_key", { keyName });
}

// ==================== 关键词排名监控 ====================

/**
 * 添加关键词监控
 */
export async function addKeywordMonitoring(
  productId: number,
  keyword: string,
  asin: string,
  country: string,
  priority?: string
): Promise<number> {
  return await invoke("add_keyword_monitoring", {
    productId,
    keyword,
    asin,
    country,
    priority: priority || null,
  });
}

/**
 * 批量添加关键词监控
 */
export async function batchAddKeywordMonitoring(
  productId: number,
  items: { keyword: string; asin: string; country: string; priority?: string }[]
): Promise<number[]> {
  const tuples = items.map(item => [
    item.keyword,
    item.asin,
    item.country,
    item.priority || null,
  ] as [string, string, string, string | null]);
  return await invoke("batch_add_keyword_monitoring", { productId, items: tuples });
}

/**
 * 获取关键词监控列表
 */
export async function getKeywordMonitoringList(params: {
  productId: number;
  country?: string;
  priority?: string;
  isActive?: boolean;
  search?: string;
  sortBy?: string;
  sortOrder?: string;
  page: number;
  pageSize: number;
}): Promise<[KeywordMonitoring[], number]> {
  return await invoke("get_keyword_monitoring_list", {
    productId: params.productId,
    country: params.country || null,
    priority: params.priority || null,
    isActive: params.isActive ?? null,
    search: params.search || null,
    sortBy: params.sortBy || null,
    sortOrder: params.sortOrder || null,
    page: params.page,
    pageSize: params.pageSize,
  });
}

/**
 * 更新关键词监控
 */
export async function updateKeywordMonitoring(
  id: number,
  priority?: string,
  isActive?: boolean
): Promise<void> {
  return await invoke("update_keyword_monitoring", {
    id,
    priority: priority || null,
    isActive: isActive ?? null,
  });
}

/**
 * 更新关键词监控标签
 */
export async function updateKeywordMonitoringTags(
  id: number,
  tags: string[] | null
): Promise<void> {
  return await invoke("update_keyword_monitoring_tags", {
    id,
    tags: tags ? JSON.stringify(tags) : null,
  });
}

/**
 * 删除关键词监控
 */
export async function deleteKeywordMonitoring(id: number): Promise<void> {
  return await invoke("delete_keyword_monitoring", { id });
}

/**
 * 批量删除关键词监控
 */
export async function batchDeleteKeywordMonitoring(ids: number[]): Promise<void> {
  return await invoke("batch_delete_keyword_monitoring", { ids });
}

/**
 * 获取监控统计
 */
export async function getMonitoringStats(productId: number): Promise<MonitoringStats> {
  return await invoke("get_monitoring_stats", { productId });
}

/**
 * 获取排名历史
 */
export async function getRankingHistory(
  monitoringId: number,
  days: number = 30
): Promise<RankingHistory[]> {
  return await invoke("get_ranking_history", { monitoringId, days });
}

/**
 * 获取竞品快照
 */
export async function getRankingSnapshots(
  keyword: string,
  country: string,
  days: number = 30
): Promise<RankingSnapshot[]> {
  return await invoke("get_ranking_snapshots", { keyword, country, days });
}

/**
 * 获取监控迷你图数据
 */
export async function getMonitoringSparklines(
  productId: number,
  days: number = 7
): Promise<MonitoringSparkline[]> {
  return await invoke("get_monitoring_sparklines", { productId, days });
}

/**
 * 检测单个关键词排名
 */
export async function checkSingleRanking(
  monitoringId: number,
  maxPages?: number
): Promise<RankingResult> {
  return await invoke("check_single_ranking", {
    monitoringId,
    maxPages: maxPages || null,
  });
}

/**
 * 批量检测排名
 */
export async function checkAllRankings(
  productId: number,
  maxPages?: number,
  hoursSinceLastCheck?: number
): Promise<[number, RankingResult][]> {
  return await invoke("check_all_rankings", {
    productId,
    maxPages: maxPages || null,
    // 用 undefined 判断而非 || null，因为 0 是有效值（表示无时间限制）
    hoursSinceLastCheck: hoursSinceLastCheck === undefined ? null : hoursSinceLastCheck,
  });
}

/**
 * 检测选中的关键词排名
 */
export async function checkSelectedRankings(
  ids: number[],
  maxPages?: number
): Promise<[number, RankingResult][]> {
  return await invoke("check_selected_rankings", {
    ids,
    maxPages: maxPages || null,
  });
}

// ==================== 调度器管理 ====================

import type { SchedulerSettings, SchedulerStatus, TaskLog } from "./types";

/**
 * 获取调度器设置
 */
export async function getSchedulerSettings(): Promise<SchedulerSettings> {
  return await invoke("get_scheduler_settings");
}

/**
 * 更新调度器设置
 */
export async function updateSchedulerSettings(settings: SchedulerSettings): Promise<void> {
  return await invoke("update_scheduler_settings", { settings });
}

/**
 * 启动调度器
 */
export async function startScheduler(): Promise<void> {
  return await invoke("start_scheduler");
}

/**
 * 停止调度器
 */
export async function stopScheduler(): Promise<void> {
  return await invoke("stop_scheduler");
}

/**
 * 获取调度器状态
 */
export async function getSchedulerStatus(): Promise<SchedulerStatus> {
  return await invoke("get_scheduler_status");
}

/**
 * 获取任务记录列表
 */
export async function getTaskLogs(limit?: number): Promise<TaskLog[]> {
  return await invoke("get_task_logs", { limit });
}

/**
 * 获取正在运行的任务
 */
export async function getRunningTask(): Promise<TaskLog | null> {
  return await invoke("get_running_task");
}

/**
 * 清空任务记录
 */
export async function clearTaskLogs(): Promise<void> {
  return await invoke("clear_task_logs");
}

// ==================== 依赖安装 ====================

import type { DependencyStatus, InstallResult } from "./types";

/**
 * 检查依赖状态
 */
export async function checkDependencies(): Promise<DependencyStatus> {
  return await invoke("check_dependencies");
}

/**
 * 安装所有依赖 (Python + Playwright + Chromium)
 */
export async function installAllDependencies(): Promise<InstallResult> {
  return await invoke("install_all_dependencies");
}

/**
 * 仅安装 Playwright 依赖 (需要 Python 已安装)
 */
export async function installPlaywrightOnly(): Promise<InstallResult> {
  return await invoke("install_playwright_only");
}

/**
 * 安装 PDF 处理依赖 (pdf2image + poppler)
 */
export async function installPdfDependencies(): Promise<InstallResult> {
  return await invoke("install_pdf_dependencies");
}

// ==================== 优化事件管理 ====================

import type { OptimizationEvent, EventMainType } from "./types";

/**
 * 添加优化事件
 */
export async function addOptimizationEvent(
  productId: number,
  eventDate: string,
  eventType: EventMainType,
  eventSubType: string,  // JSON 字符串，支持多选
  title: string,
  description?: string,
  targetAsin?: string,
  affectedKeywords?: string
): Promise<number> {
  return await invoke("add_optimization_event", {
    productId,
    eventDate,
    eventType,
    eventSubType,
    title,
    description,
    targetAsin,
    affectedKeywords,
  });
}

/**
 * 获取优化事件列表
 */
export async function getOptimizationEvents(
  productId: number,
  startDate?: string,
  endDate?: string
): Promise<OptimizationEvent[]> {
  return await invoke("get_optimization_events", {
    productId,
    startDate,
    endDate,
  });
}

/**
 * 更新优化事件
 */
export async function updateOptimizationEvent(
  id: number,
  eventDate: string,
  eventType: EventMainType,
  eventSubType: string,  // JSON 字符串，支持多选
  title: string,
  description?: string,
  targetAsin?: string,
  affectedKeywords?: string
): Promise<void> {
  return await invoke("update_optimization_event", {
    id,
    eventDate,
    eventType,
    eventSubType,
    title,
    description,
    targetAsin,
    affectedKeywords,
  });
}

/**
 * 删除优化事件
 */
export async function deleteOptimizationEvent(id: number): Promise<void> {
  return await invoke("delete_optimization_event", { id });
}

// ==================== 知识库管理 ====================

import type { KbCategory, KbDocument, KbChunk, KbSearchResult, KbConversation, KbMessage, ExtractedImage, PdfPageImage } from "./types";

/**
 * 创建知识库分类
 */
export async function kbCreateCategory(name: string, parentId?: number): Promise<number> {
  return await invoke("kb_create_category", { name, parentId: parentId || null });
}

/**
 * 获取所有知识库分类
 */
export async function kbGetCategories(): Promise<KbCategory[]> {
  return await invoke("kb_get_categories");
}

/**
 * 删除知识库分类
 */
export async function kbDeleteCategory(id: number): Promise<void> {
  return await invoke("kb_delete_category", { id });
}

/**
 * 更新知识库分类名称
 */
export async function kbUpdateCategory(id: number, name: string): Promise<void> {
  return await invoke("kb_update_category", { id, name });
}

/**
 * 更新知识库分类颜色
 */
export async function kbUpdateCategoryColor(id: number, color: string): Promise<void> {
  return await invoke("kb_update_category_color", { id, color });
}

/**
 * 批量更新知识库分类排序
 */
export async function kbUpdateCategoriesOrder(ids: number[]): Promise<void> {
  return await invoke("kb_update_categories_order", { ids });
}

/**
 * 添加文档
 */
export async function kbAddDocument(
  categoryId: number | null,
  title: string,
  fileName: string,
  filePath: string,
  fileType: string,
  fileSize?: number
): Promise<number> {
  return await invoke("kb_add_document", {
    categoryId,
    title,
    fileName,
    filePath,
    fileType,
    fileSize: fileSize || null,
  });
}

/**
 * 更新文档状态
 */
export async function kbUpdateDocumentStatus(
  id: number,
  status: string,
  chunkCount: number
): Promise<void> {
  return await invoke("kb_update_document_status", { id, status, chunkCount });
}

/**
 * 更新文档分类
 */
export async function kbUpdateDocumentCategory(
  id: number,
  categoryId: number | null
): Promise<void> {
  return await invoke("kb_update_document_category", { id, categoryId });
}

/**
 * 获取文档列表
 */
export async function kbGetDocuments(categoryId?: number): Promise<KbDocument[]> {
  return await invoke("kb_get_documents", { categoryId: categoryId || null });
}

/**
 * 删除文档
 */
export async function kbDeleteDocument(id: number): Promise<void> {
  return await invoke("kb_delete_document", { id });
}

/**
 * 处理文档（解析 + 分块）
 */
export async function kbProcessDocument(documentId: number, filePath: string): Promise<number> {
  return await invoke("kb_process_document", { documentId, filePath });
}

/**
 * 从文档中提取嵌入的图片
 */
export async function kbExtractImages(filePath: string): Promise<ExtractedImage[]> {
  return await invoke("kb_extract_images", { filePath });
}

/**
 * 读取文件并返回 base64 编码（用于 PDF OCR）
 */
export async function kbReadFileBase64(filePath: string): Promise<string> {
  return await invoke("kb_read_file_base64", { filePath });
}

/**
 * 将 PDF 转换为图片（用于 OCR）
 */
export async function kbPdfToImages(filePath: string): Promise<PdfPageImage[]> {
  return await invoke("kb_pdf_to_images", { filePath });
}

/**
 * 将图片识别结果作为 chunk 添加到文档（不保存图片）
 */
export async function kbAddImageChunk(documentId: number, imageName: string, description: string): Promise<number> {
  return await invoke("kb_add_image_chunk", { documentId, imageName, description });
}

/**
 * 将图片识别结果作为 chunk 添加到文档（同时保存图片用于图文问答）
 */
export async function kbAddImageChunkWithFile(
  documentId: number,
  imageName: string,
  description: string,
  base64Data: string
): Promise<number> {
  return await invoke("kb_add_image_chunk_with_file", { documentId, imageName, description, base64Data });
}

/**
 * 更新分块的 embedding 向量
 */
export async function kbUpdateChunkEmbedding(chunkId: number, embedding: number[]): Promise<void> {
  return await invoke("kb_update_chunk_embedding", { chunkId, embedding });
}

/**
 * 清除所有 embedding（用于迁移到新的 embedding 模型）
 * @returns 清除的 embedding 数量
 */
export async function kbClearAllEmbeddings(): Promise<number> {
  return await invoke("kb_clear_all_embeddings");
}

/**
 * 获取没有 embedding 的分块
 */
export async function kbGetChunksWithoutEmbedding(documentId: number): Promise<KbChunk[]> {
  return await invoke("kb_get_chunks_without_embedding", { documentId });
}

/**
 * 获取文档的向量化统计（总分块数，已向量化数）
 */
export async function kbGetDocumentEmbeddingStats(documentId: number): Promise<[number, number]> {
  return await invoke("kb_get_document_embedding_stats", { documentId });
}

/**
 * 向量相似度搜索（支持相关度阈值过滤）
 * @param queryEmbedding 查询向量
 * @param limit 最大返回数量
 * @param minScore 最低相关度阈值 (0.0-1.0)，低于此值的结果会被过滤
 */
export async function kbVectorSearch(
  queryEmbedding: number[],
  limit: number = 50,
  minScore: number = 0.5
): Promise<KbSearchResult[]> {
  return await invoke("kb_vector_search", { queryEmbedding, limit, minScore });
}

/**
 * 添加单个分块
 */
export async function kbAddChunk(
  documentId: number,
  chunkIndex: number,
  content: string,
  pageNumber?: number
): Promise<number> {
  return await invoke("kb_add_chunk", {
    documentId,
    chunkIndex,
    content,
    pageNumber: pageNumber || null,
  });
}

/**
 * 批量添加分块
 */
export async function kbAddChunksBatch(
  documentId: number,
  chunks: { index: number; content: string; page_number?: number }[]
): Promise<number> {
  return await invoke("kb_add_chunks_batch", { documentId, chunks });
}

/**
 * 获取文档分块
 */
export async function kbGetChunks(documentId: number): Promise<KbChunk[]> {
  return await invoke("kb_get_chunks", { documentId });
}

/**
 * 搜索知识库
 */
export async function kbSearch(query: string, limit: number = 5): Promise<KbSearchResult[]> {
  return await invoke("kb_search", { query, limit });
}

/**
 * 创建对话
 */
export async function kbCreateConversation(
  aiProvider: string,
  aiModel?: string,
  title?: string
): Promise<number> {
  return await invoke("kb_create_conversation", {
    aiProvider,
    aiModel: aiModel || null,
    title: title || null,
  });
}

/**
 * 获取对话列表
 */
export async function kbGetConversations(): Promise<KbConversation[]> {
  return await invoke("kb_get_conversations");
}

/**
 * 更新对话标题
 */
export async function kbUpdateConversationTitle(id: number, title: string): Promise<void> {
  return await invoke("kb_update_conversation_title", { id, title });
}

/**
 * 删除对话
 */
export async function kbDeleteConversation(id: number): Promise<void> {
  return await invoke("kb_delete_conversation", { id });
}

/**
 * 添加消息
 */
export async function kbAddMessage(
  conversationId: number,
  role: 'user' | 'assistant',
  content: string,
  sources?: string
): Promise<number> {
  return await invoke("kb_add_message", {
    conversationId,
    role,
    content,
    sources: sources || null,
  });
}

/**
 * 获取对话消息
 */
export async function kbGetMessages(conversationId: number): Promise<KbMessage[]> {
  return await invoke("kb_get_messages", { conversationId });
}

// ==================== 文档链接 ====================

import type { KbDocumentLink, KbDocumentCategory } from "./types";

/**
 * 添加文档链接
 */
export async function kbAddDocumentLink(
  sourceId: number,
  targetId: number,
  linkText?: string
): Promise<number> {
  return await invoke("kb_add_document_link", {
    sourceId,
    targetId,
    linkText: linkText || null,
  });
}

/**
 * 移除文档链接
 */
export async function kbRemoveDocumentLink(
  sourceId: number,
  targetId: number
): Promise<void> {
  return await invoke("kb_remove_document_link", { sourceId, targetId });
}

/**
 * 获取文档的出链
 */
export async function kbGetDocumentLinks(docId: number): Promise<KbDocumentLink[]> {
  return await invoke("kb_get_document_links", { docId });
}

/**
 * 获取文档的反向链接
 */
export async function kbGetDocumentBacklinks(docId: number): Promise<KbDocumentLink[]> {
  return await invoke("kb_get_document_backlinks", { docId });
}

/**
 * 获取所有链接（用于知识图谱）
 */
export async function kbGetAllLinks(): Promise<KbDocumentLink[]> {
  return await invoke("kb_get_all_links");
}

// ==================== 文档分类关联（多对多）====================

/**
 * 给文档添加分类
 */
export async function kbAddDocumentCategory(docId: number, categoryId: number): Promise<void> {
  return await invoke("kb_add_document_category", { docId, categoryId });
}

/**
 * 移除文档分类
 */
export async function kbRemoveDocumentCategory(docId: number, categoryId: number): Promise<void> {
  return await invoke("kb_remove_document_category", { docId, categoryId });
}

/**
 * 获取文档的所有分类
 */
export async function kbGetDocumentCategories(docId: number): Promise<KbDocumentCategory[]> {
  return await invoke("kb_get_document_categories", { docId });
}

/**
 * 按分类筛选文档（多对多版本）
 */
export async function kbGetDocumentsByCategories(categoryId: number): Promise<KbDocument[]> {
  return await invoke("kb_get_documents_by_categories", { categoryId });
}

/**
 * 设置文档的分类（替换所有现有分类）
 */
export async function kbSetDocumentCategories(docId: number, categoryIds: number[]): Promise<void> {
  return await invoke("kb_set_document_categories", { docId, categoryIds });
}

// ==================== 智能文案 ====================

import type { ScProject } from "./types";

/**
 * 创建智能文案项目
 */
export async function scCreateProject(
  name: string,
  scenarioType: 'new' | 'optimize',
  marketplace: string,
  myAsin?: string,
  productId?: number
): Promise<number> {
  return await invoke("sc_create_project", {
    name,
    scenarioType,
    marketplace,
    myAsin: myAsin || null,
    productId: productId || null,
  });
}

/**
 * 获取智能文案项目列表
 */
export async function scGetProjects(scenarioType?: 'new' | 'optimize'): Promise<ScProject[]> {
  return await invoke("sc_get_projects", {
    scenarioType: scenarioType || null,
  });
}

/**
 * 获取单个智能文案项目
 */
export async function scGetProject(id: number): Promise<ScProject | null> {
  return await invoke("sc_get_project", { id });
}

/**
 * 更新智能文案项目基本信息
 */
export async function scUpdateProject(
  id: number,
  name: string,
  marketplace: string,
  myAsin?: string
): Promise<void> {
  return await invoke("sc_update_project", {
    id,
    name,
    marketplace,
    myAsin: myAsin || null,
  });
}

/**
 * 更新智能文案项目状态
 */
export async function scUpdateProjectStatus(
  id: number,
  status: 'draft' | 'collecting' | 'analyzing' | 'completed'
): Promise<void> {
  return await invoke("sc_update_project_status", { id, status });
}

/**
 * 更新项目的"我的产品信息"
 */
export async function scUpdateMyProductInfo(
  projectId: number,
  info: import("./types").MyProductInfo | null
): Promise<void> {
  return await invoke("sc_update_my_product_info", {
    id: projectId,
    myProductInfo: info ? JSON.stringify(info) : null,
  });
}

/**
 * 更新用户的 Listing 信息（老品优化时使用）
 */
export async function scUpdateMyListing(
  projectId: number,
  myTitle: string | null,
  myBullets: string[] | null,
  myDescription: string | null
): Promise<void> {
  return await invoke("sc_update_my_listing", {
    id: projectId,
    myTitle,
    myBullets: myBullets ? JSON.stringify(myBullets) : null,
    myDescription,
  });
}

/**
 * 删除智能文案项目
 */
export async function scDeleteProject(id: number): Promise<void> {
  return await invoke("sc_delete_project", { id });
}

// ==================== 竞品管理 ====================

import type { ScCompetitor } from "./types";

/**
 * 添加竞品（仅 ASIN）
 */
export async function scAddCompetitor(
  projectId: number,
  asin: string,
  competitorType: 'top' | 'direct' | 'rising' = 'direct'
): Promise<number> {
  return await invoke("sc_add_competitor", {
    projectId,
    asin,
    competitorType,
  });
}

/**
 * 获取项目的竞品列表
 */
export async function scGetCompetitors(projectId: number): Promise<ScCompetitor[]> {
  return await invoke("sc_get_competitors", { projectId });
}

/**
 * 更新竞品信息（爬取后）
 */
export async function scUpdateCompetitorInfo(
  id: number,
  info: {
    title?: string;
    price?: string;
    rating?: string;
    reviewCount?: number;
    bsrRank?: string;
    bullets?: string;
    description?: string;
  }
): Promise<void> {
  return await invoke("sc_update_competitor_info", {
    id,
    title: info.title || null,
    price: info.price || null,
    rating: info.rating || null,
    reviewCount: info.reviewCount || null,
    bsrRank: info.bsrRank || null,
    bullets: info.bullets || null,
    description: info.description || null,
  });
}

/**
 * 删除竞品
 */
export async function scDeleteCompetitor(id: number): Promise<void> {
  return await invoke("sc_delete_competitor", { id });
}

/**
 * 更新竞品类型
 */
export async function scUpdateCompetitorType(
  id: number,
  competitorType: 'top' | 'direct' | 'rising'
): Promise<void> {
  return await invoke("sc_update_competitor_type", { id, competitorType });
}

// Listing 爬取结果
export interface ListingResult {
  asin: string;
  country: string;
  title: string | null;
  price: string | null;
  rating: string | null;
  review_count: number | null;
  bsr_rank: string | null;
  bullets: string[];
  description: string | null;
  fetched_at: string;
  error: string | null;
}

/**
 * 爬取竞品 Listing 信息
 */
export async function scFetchCompetitorListing(
  id: number,
  asin: string,
  country: string
): Promise<ListingResult> {
  return await invoke("sc_fetch_competitor_listing", { id, asin, country });
}

/**
 * 批量爬取竞品 Listing 信息（复用同一个浏览器，效率更高）
 */
export async function scFetchCompetitorsBatch(
  items: Array<[number, string, string]>  // [id, asin, country]
): Promise<Array<[number, ListingResult]>> {
  return await invoke("sc_fetch_competitors_batch", { items });
}

// ==================== 评论管理 ====================

import type { ScReview, ScReviewSummary, ReviewResult } from "./types";

/**
 * 爬取竞品评论
 */
export async function scFetchCompetitorReviews(
  id: number,
  asin: string,
  country: string
): Promise<ReviewResult> {
  return await invoke("sc_fetch_competitor_reviews", { id, asin, country });
}

/**
 * 获取竞品的评论列表
 */
export async function scGetCompetitorReviews(competitorId: number): Promise<ScReview[]> {
  return await invoke("sc_get_competitor_reviews", { competitorId });
}

/**
 * 获取评论统计摘要
 */
export async function scGetReviewsSummary(competitorId: number): Promise<ScReviewSummary> {
  return await invoke("sc_get_reviews_summary", { competitorId });
}

// ==================== AI 分析 ====================

/**
 * 保存分析结果
 */
export async function scSaveAnalysis(
  projectId: number,
  analysisType: 'review_insights' | 'listing_analysis' | 'optimization',
  resultJson: string,
  modelProvider?: string,
  modelName?: string
): Promise<number> {
  return await invoke("sc_save_analysis", {
    projectId,
    analysisType,
    resultJson,
    modelProvider: modelProvider || null,
    modelName: modelName || null,
  });
}

/**
 * 获取指定类型的分析结果
 */
export async function scGetAnalysis(
  projectId: number,
  analysisType: 'review_insights' | 'listing_analysis' | 'optimization'
): Promise<ScAnalysis | null> {
  return await invoke("sc_get_analysis", { projectId, analysisType });
}

/**
 * 获取项目的所有分析结果
 */
export async function scGetAllAnalysis(projectId: number): Promise<ScAnalysis[]> {
  return await invoke("sc_get_all_analysis", { projectId });
}

/**
 * 删除项目的所有分析结果
 */
export async function scDeleteAllAnalysis(projectId: number): Promise<void> {
  return await invoke("sc_delete_all_analysis", { projectId });
}

/**
 * 获取项目关联的关键词数据（Top N 高搜索量）
 */
export async function scGetProjectKeywords(projectId: number, limit: number = 100): Promise<KeywordData[]> {
  return await invoke("sc_get_project_keywords", { projectId, limit });
}

// ==================== 智能广告（Smart Ads）====================

import type { AdProject, AdSearchTerm, AdAnalysisRecord } from "./types";

/**
 * 创建广告项目
 */
export async function adCreateProject(
  productId: number | null,
  name: string,
  marketplace: string,
  targetAcos: number
): Promise<number> {
  return await invoke("ad_create_project", {
    productId,
    name,
    marketplace,
    targetAcos,
  });
}

/**
 * 获取所有广告项目
 */
export async function adGetProjects(): Promise<AdProject[]> {
  return await invoke("ad_get_projects");
}

/**
 * 获取单个广告项目
 */
export async function adGetProject(id: number): Promise<AdProject | null> {
  return await invoke("ad_get_project", { id });
}

/**
 * 更新广告项目
 */
export async function adUpdateProject(
  id: number,
  name: string,
  marketplace: string,
  targetAcos: number
): Promise<void> {
  return await invoke("ad_update_project", { id, name, marketplace, targetAcos });
}

/**
 * 删除广告项目
 */
export async function adDeleteProject(id: number): Promise<void> {
  return await invoke("ad_delete_project", { id });
}

/**
 * 导入搜索词数据
 */
export async function adImportSearchTerms(
  projectId: number,
  searchTerms: AdSearchTerm[]
): Promise<number> {
  return await invoke("ad_import_search_terms", { projectId, searchTerms });
}

/**
 * 获取搜索词数据
 */
export async function adGetSearchTerms(projectId: number): Promise<AdSearchTerm[]> {
  return await invoke("ad_get_search_terms", { projectId });
}

/**
 * 获取搜索词统计（总花费, 总销售, 平均ACOS, 数量，按国家分组）
 */
export async function adGetSearchTermsStats(
  projectId: number
): Promise<import("./types").SearchTermsStatsResult> {
  return await invoke("ad_get_search_terms_stats", { projectId });
}

/**
 * 保存分析结果
 */
export async function adSaveAnalysis(
  projectId: number,
  analysisType: string,
  resultJson: string,
  aiProvider: string,
  aiModel: string
): Promise<number> {
  return await invoke("ad_save_analysis", {
    projectId,
    analysisType,
    resultJson,
    aiProvider,
    aiModel,
  });
}

/**
 * 获取指定类型的分析结果
 */
export async function adGetAnalysis(
  projectId: number,
  analysisType: string
): Promise<AdAnalysisRecord | null> {
  return await invoke("ad_get_analysis", { projectId, analysisType });
}

/**
 * 获取项目的所有分析结果
 */
export async function adGetAllAnalysis(projectId: number): Promise<AdAnalysisRecord[]> {
  return await invoke("ad_get_all_analysis", { projectId });
}

// ==================== 快捷备忘录 ====================

import type { QuickNote, ExchangeRateCache } from './types';

/**
 * 添加快捷备忘
 */
export async function addQuickNote(content: string): Promise<number> {
  return await invoke("add_quick_note", { content });
}

/**
 * 获取快捷备忘列表
 * @param filter - 筛选条件: 'pending' | 'completed' | undefined (全部)
 */
export async function getQuickNotes(filter?: string): Promise<QuickNote[]> {
  return await invoke("get_quick_notes", { filter: filter || null });
}

/**
 * 更新快捷备忘内容
 */
export async function updateQuickNote(id: number, content: string): Promise<void> {
  return await invoke("update_quick_note", { id, content });
}

/**
 * 切换快捷备忘完成状态
 * @returns 新的完成状态
 */
export async function toggleQuickNote(id: number): Promise<boolean> {
  return await invoke("toggle_quick_note", { id });
}

/**
 * 删除快捷备忘
 */
export async function deleteQuickNote(id: number): Promise<void> {
  return await invoke("delete_quick_note", { id });
}

/**
 * 获取快捷备忘统计
 * @returns [总数, 待完成数]
 */
export async function getQuickNotesCount(): Promise<[number, number]> {
  return await invoke("get_quick_notes_count");
}

/**
 * 更新快捷备忘截止日期
 */
export async function updateQuickNoteDueDate(id: number, dueDate: string | null): Promise<void> {
  return await invoke("update_quick_note_due_date", { id, dueDate });
}

/**
 * 更新快捷备忘重复设置
 * @param id 备忘ID
 * @param repeatType 重复类型: 'daily' | 'weekly' | 'monthly' | null
 * @param repeatInterval 重复间隔，默认1
 */
export async function updateQuickNoteRepeat(
  id: number,
  repeatType: string | null,
  repeatInterval: number = 1
): Promise<void> {
  return await invoke("update_quick_note_repeat", { id, repeatType, repeatInterval });
}

/**
 * 批量更新快捷备忘排序
 * @param ids 按顺序排列的备忘ID数组
 */
export async function reorderQuickNotes(ids: number[]): Promise<void> {
  return await invoke("reorder_quick_notes", { ids });
}

// ==================== 汇率 ====================

/**
 * 保存汇率到缓存
 */
export async function saveExchangeRates(rates: [string, number][]): Promise<void> {
  return await invoke("save_exchange_rates", { rates });
}

/**
 * 获取缓存的汇率
 */
export async function getExchangeRates(): Promise<ExchangeRateCache[]> {
  return await invoke("get_exchange_rates");
}

/**
 * 从网络获取最新汇率（通过后端代理）
 * @param currencies 需要获取的货币代码列表，如 ['USD', 'EUR', 'GBP', 'JPY']
 * @returns 更新后的汇率缓存
 */
export async function fetchExchangeRates(currencies: string[]): Promise<ExchangeRateCache[]> {
  return await invoke("fetch_exchange_rates", { currencies });
}

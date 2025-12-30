import { invoke } from "@tauri-apps/api/core";
import type { BackupInfo, Category, KeywordData, KeywordMonitoring, MonitoringSparkline, MonitoringStats, Product, RankingHistory, RankingResult, RankingSnapshot, Root, TrafficLevelStats, WorkflowStatus } from "./types";

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

// ==================== 优化事件管理 ====================

import type { OptimizationEvent, EventMainType, EventSubType } from "./types";

/**
 * 添加优化事件
 */
export async function addOptimizationEvent(
  productId: number,
  eventDate: string,
  eventType: EventMainType,
  eventSubType: EventSubType,
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
  eventSubType: EventSubType,
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

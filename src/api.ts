import { invoke } from "@tauri-apps/api/core";
import type { BackupInfo, Category, KeywordData, Product, Root, TrafficLevelStats, WorkflowStatus } from "./types";

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

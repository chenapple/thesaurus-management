import { invoke } from "@tauri-apps/api/core";
import type { Category, Product, Root } from "./types";

// ==================== 产品管理 ====================

export async function getProducts(): Promise<Product[]> {
  return await invoke("get_products");
}

export async function createProduct(
  name: string,
  sku?: string,
  asin?: string
): Promise<number> {
  return await invoke("create_product", {
    name,
    sku: sku || null,
    asin: asin || null,
  });
}

export async function updateProduct(
  id: number,
  name: string,
  sku?: string,
  asin?: string
): Promise<void> {
  return await invoke("update_product", {
    id,
    name,
    sku: sku || null,
    asin: asin || null,
  });
}

export async function deleteProduct(id: number): Promise<void> {
  return await invoke("delete_product", { id });
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

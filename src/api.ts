import { invoke } from "@tauri-apps/api/core";
import type { Category, Root } from "./types";

export async function getCategories(): Promise<Category[]> {
  return await invoke("get_categories");
}

export async function importKeywords(keywords: string[]): Promise<void> {
  return await invoke("import_keywords", { keywords });
}

export async function getRoots(params: {
  search?: string;
  categoryIds?: number[];
  sortBy?: string;
  sortOrder?: string;
  page: number;
  pageSize: number;
}): Promise<[Root[], number]> {
  return await invoke("get_roots", {
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

export async function getStats(): Promise<[number, number]> {
  return await invoke("get_stats");
}

export async function clearAllData(): Promise<void> {
  return await invoke("clear_all_data");
}

export async function getUntranslatedRoots(): Promise<string[]> {
  return await invoke("get_untranslated_roots");
}

export async function batchUpdateRootAnalysis(
  updates: [string, string, string[]][]
): Promise<void> {
  return await invoke("batch_update_root_analysis", { updates });
}

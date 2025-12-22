import { defineStore } from "pinia";
import { ref } from "vue";
import type { KeywordData, Root, TrafficLevelStats, WorkflowStatus } from "../types";

/**
 * 缓存条目接口
 */
interface CacheEntry<T> {
  data: T;
  timestamp: number;
  key: string;
}

/**
 * 缓存配置
 */
const CACHE_TTL = 5 * 60 * 1000; // 5 分钟过期
const MAX_CACHE_SIZE = 10; // 最多缓存 10 个产品的数据

/**
 * 数据缓存 Store
 * 缓存已加载的数据，避免重复请求
 */
export const useCacheStore = defineStore("cache", () => {
  // ==================== 缓存存储 ====================

  // 关键词数据缓存 - key: `${productId}-${page}-${pageSize}-${filters}`
  const keywordCache = ref<Map<string, CacheEntry<{ data: KeywordData[]; total: number }>>>(new Map());

  // 词根数据缓存 - key: `${productId}-${page}-${pageSize}-${filters}`
  const rootCache = ref<Map<string, CacheEntry<{ data: Root[]; total: number }>>>(new Map());

  // 流量统计缓存 - key: productId
  const trafficStatsCache = ref<Map<number, CacheEntry<TrafficLevelStats>>>(new Map());

  // 工作流状态缓存 - key: productId
  const workflowCache = ref<Map<number, CacheEntry<WorkflowStatus>>>(new Map());

  // ==================== 工具函数 ====================

  /**
   * 检查缓存是否有效
   */
  function isValid<T>(entry: CacheEntry<T> | undefined): entry is CacheEntry<T> {
    if (!entry) return false;
    return Date.now() - entry.timestamp < CACHE_TTL;
  }

  /**
   * 清理过期缓存
   */
  function cleanExpired<T>(cache: Map<string | number, CacheEntry<T>>) {
    const now = Date.now();
    for (const [key, entry] of cache.entries()) {
      if (now - entry.timestamp >= CACHE_TTL) {
        cache.delete(key);
      }
    }
  }

  /**
   * 限制缓存大小
   */
  function limitSize<T>(cache: Map<string | number, CacheEntry<T>>, maxSize: number) {
    if (cache.size <= maxSize) return;

    // 按时间戳排序，删除最旧的
    const entries = Array.from(cache.entries())
      .sort((a, b) => a[1].timestamp - b[1].timestamp);

    const deleteCount = cache.size - maxSize;
    for (let i = 0; i < deleteCount; i++) {
      cache.delete(entries[i][0]);
    }
  }

  // ==================== 关键词缓存方法 ====================

  /**
   * 生成关键词缓存键
   */
  function getKeywordCacheKey(
    productId: number,
    page: number,
    pageSize: number,
    search: string,
    filters: Record<string, string[]>
  ): string {
    return `${productId}-${page}-${pageSize}-${search}-${JSON.stringify(filters)}`;
  }

  /**
   * 获取关键词缓存
   */
  function getKeywordCache(
    productId: number,
    page: number,
    pageSize: number,
    search: string,
    filters: Record<string, string[]>
  ): { data: KeywordData[]; total: number } | null {
    const key = getKeywordCacheKey(productId, page, pageSize, search, filters);
    const entry = keywordCache.value.get(key);
    if (isValid(entry)) {
      return entry.data;
    }
    return null;
  }

  /**
   * 设置关键词缓存
   */
  function setKeywordCache(
    productId: number,
    page: number,
    pageSize: number,
    search: string,
    filters: Record<string, string[]>,
    data: KeywordData[],
    total: number
  ) {
    const key = getKeywordCacheKey(productId, page, pageSize, search, filters);
    keywordCache.value.set(key, {
      data: { data, total },
      timestamp: Date.now(),
      key,
    });
    cleanExpired(keywordCache.value);
    limitSize(keywordCache.value, MAX_CACHE_SIZE * 10); // 每个产品可能有多页
  }

  // ==================== 词根缓存方法 ====================

  /**
   * 生成词根缓存键
   */
  function getRootCacheKey(
    productId: number,
    page: number,
    pageSize: number,
    search: string,
    categoryIds: number[]
  ): string {
    return `${productId}-${page}-${pageSize}-${search}-${JSON.stringify(categoryIds)}`;
  }

  /**
   * 获取词根缓存
   */
  function getRootCache(
    productId: number,
    page: number,
    pageSize: number,
    search: string,
    categoryIds: number[]
  ): { data: Root[]; total: number } | null {
    const key = getRootCacheKey(productId, page, pageSize, search, categoryIds);
    const entry = rootCache.value.get(key);
    if (isValid(entry)) {
      return entry.data;
    }
    return null;
  }

  /**
   * 设置词根缓存
   */
  function setRootCache(
    productId: number,
    page: number,
    pageSize: number,
    search: string,
    categoryIds: number[],
    data: Root[],
    total: number
  ) {
    const key = getRootCacheKey(productId, page, pageSize, search, categoryIds);
    rootCache.value.set(key, {
      data: { data, total },
      timestamp: Date.now(),
      key,
    });
    cleanExpired(rootCache.value);
    limitSize(rootCache.value, MAX_CACHE_SIZE * 10);
  }

  // ==================== 流量统计缓存方法 ====================

  function getTrafficStatsCache(productId: number): TrafficLevelStats | null {
    const entry = trafficStatsCache.value.get(productId);
    if (isValid(entry)) {
      return entry.data;
    }
    return null;
  }

  function setTrafficStatsCache(productId: number, data: TrafficLevelStats) {
    trafficStatsCache.value.set(productId, {
      data,
      timestamp: Date.now(),
      key: String(productId),
    });
    cleanExpired(trafficStatsCache.value);
    limitSize(trafficStatsCache.value, MAX_CACHE_SIZE);
  }

  // ==================== 工作流状态缓存方法 ====================

  function getWorkflowCache(productId: number): WorkflowStatus | null {
    const entry = workflowCache.value.get(productId);
    if (isValid(entry)) {
      return entry.data;
    }
    return null;
  }

  function setWorkflowCache(productId: number, data: WorkflowStatus) {
    workflowCache.value.set(productId, {
      data,
      timestamp: Date.now(),
      key: String(productId),
    });
    cleanExpired(workflowCache.value);
    limitSize(workflowCache.value, MAX_CACHE_SIZE);
  }

  // ==================== 缓存管理方法 ====================

  /**
   * 清除指定产品的所有缓存
   */
  function invalidateProduct(productId: number) {
    // 清除关键词缓存
    for (const [key] of keywordCache.value.entries()) {
      if (key.startsWith(`${productId}-`)) {
        keywordCache.value.delete(key);
      }
    }

    // 清除词根缓存
    for (const [key] of rootCache.value.entries()) {
      if (key.startsWith(`${productId}-`)) {
        rootCache.value.delete(key);
      }
    }

    // 清除流量统计缓存
    trafficStatsCache.value.delete(productId);

    // 清除工作流状态缓存
    workflowCache.value.delete(productId);
  }

  /**
   * 清除所有缓存
   */
  function clearAll() {
    keywordCache.value.clear();
    rootCache.value.clear();
    trafficStatsCache.value.clear();
    workflowCache.value.clear();
  }

  return {
    // 关键词缓存
    getKeywordCache,
    setKeywordCache,

    // 词根缓存
    getRootCache,
    setRootCache,

    // 流量统计缓存
    getTrafficStatsCache,
    setTrafficStatsCache,

    // 工作流状态缓存
    getWorkflowCache,
    setWorkflowCache,

    // 缓存管理
    invalidateProduct,
    clearAll,
  };
});

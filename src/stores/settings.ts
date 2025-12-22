import { defineStore } from "pinia";
import { ref, watch } from "vue";

// 列定义
export const columnDefinitions = [
  { key: "keyword", label: "关键词", required: true },
  { key: "translation", label: "翻译", default: true },
  { key: "traffic_level", label: "流量级别", default: true },
  { key: "negative_word", label: "否词", default: false },
  { key: "orderliness", label: "有序性", default: true },
  { key: "phrase_tag", label: "词组标签", default: true },
  { key: "primary_category", label: "一级分类", default: true },
  { key: "secondary_category", label: "二级分类", default: true },
  { key: "search_intent", label: "搜索意图", default: true },
  { key: "traffic_share", label: "流量占比", default: true },
  { key: "relevance_score", label: "相关性得分", default: false },
  { key: "relevance_level", label: "相关性档位", default: true },
  { key: "traffic_total", label: "流量总和", default: true },
  { key: "avg_keyword_rank", label: "周平均排名", default: true },
  { key: "avg_search_volume", label: "周平均搜索量", default: true },
  { key: "cpc_bid", label: "CPC建议竞价", default: false },
  { key: "bid_range", label: "建议竞价范围", default: false },
  { key: "click_rate", label: "点击转化率", default: false },
  { key: "conversion_competition", label: "周转化竞争", default: false },
  { key: "competition_level", label: "竞争度档位", default: false },
  { key: "natural_position_flow", label: "自然位流动率", default: false },
  { key: "top3_click_share", label: "Top3点击份额", default: false },
  { key: "avg_conversion_share", label: "Top3转化份额", default: false },
  { key: "asin_count", label: "ASIN数量", default: false },
];

// 获取默认列配置
function getDefaultColumnConfig(): Record<string, boolean> {
  const config: Record<string, boolean> = {};
  for (const col of columnDefinitions) {
    config[col.key] = col.required || col.default || false;
  }
  return config;
}

// 从 localStorage 加载列配置
function loadColumnConfig(): Record<string, boolean> {
  try {
    const saved = localStorage.getItem("columnConfig");
    if (saved) {
      return JSON.parse(saved);
    }
  } catch (e) {
    console.error("Failed to load column config:", e);
  }
  return getDefaultColumnConfig();
}

// 从 localStorage 加载主题设置
function loadDarkMode(): boolean {
  try {
    const saved = localStorage.getItem("darkMode");
    if (saved !== null) {
      return JSON.parse(saved);
    }
    // 检查系统偏好
    return window.matchMedia("(prefers-color-scheme: dark)").matches;
  } catch (e) {
    return false;
  }
}

export const useSettingsStore = defineStore("settings", () => {
  // ==================== 状态 ====================

  // 主题
  const isDarkMode = ref(loadDarkMode());

  // 列配置
  const columnConfig = ref<Record<string, boolean>>(loadColumnConfig());

  // 视图模式
  const viewMode = ref<"keywords" | "roots" | "wordcloud">("keywords");

  // 侧边栏宽度
  const sidebarWidth = ref(240);
  const MIN_SIDEBAR_WIDTH = 180;
  const MAX_SIDEBAR_WIDTH = 400;

  // ==================== 方法 ====================

  // 切换主题
  function toggleDarkMode() {
    isDarkMode.value = !isDarkMode.value;
  }

  // 设置主题
  function setDarkMode(value: boolean) {
    isDarkMode.value = value;
  }

  // 应用主题到 DOM
  function applyTheme() {
    if (isDarkMode.value) {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  }

  // 更新列配置
  function updateColumnConfig(key: string, value: boolean) {
    columnConfig.value[key] = value;
  }

  // 全选/取消全选列
  function setAllColumns(value: boolean) {
    for (const col of columnDefinitions) {
      if (!col.required) {
        columnConfig.value[col.key] = value;
      }
    }
  }

  // 重置列配置为默认值
  function resetColumnConfig() {
    columnConfig.value = getDefaultColumnConfig();
  }

  // 设置视图模式
  function setViewMode(mode: "keywords" | "roots" | "wordcloud") {
    viewMode.value = mode;
  }

  // 设置侧边栏宽度
  function setSidebarWidth(width: number) {
    sidebarWidth.value = Math.max(MIN_SIDEBAR_WIDTH, Math.min(MAX_SIDEBAR_WIDTH, width));
  }

  // 监听主题变化，保存到 localStorage 并应用
  watch(isDarkMode, (newValue) => {
    localStorage.setItem("darkMode", JSON.stringify(newValue));
    applyTheme();
  }, { immediate: true });

  // 监听列配置变化，保存到 localStorage
  watch(columnConfig, (newValue) => {
    localStorage.setItem("columnConfig", JSON.stringify(newValue));
  }, { deep: true });

  return {
    // 状态
    isDarkMode,
    columnConfig,
    viewMode,
    sidebarWidth,
    MIN_SIDEBAR_WIDTH,
    MAX_SIDEBAR_WIDTH,

    // 方法
    toggleDarkMode,
    setDarkMode,
    applyTheme,
    updateColumnConfig,
    setAllColumns,
    resetColumnConfig,
    setViewMode,
    setSidebarWidth,
  };
});

export interface Product {
  id: number;
  name: string;
  country: string | null;            // 国家代码: US, UK, DE, FR, IT, ES
  cpc_header: string | null;         // Excel中CPC列的表头（包含货币符号）
  bid_range_header: string | null;   // Excel中竞价范围列的表头（包含货币符号）
  big_word_threshold: number | null;    // 大词阈值（默认20000）
  medium_word_threshold: number | null; // 中词阈值（默认100000）
}

export interface TrafficLevelStats {
  big_count: number;
  medium_count: number;
  small_count: number;
}

export interface WorkflowStatus {
  has_data: boolean;           // 是否有关键词数据
  has_traffic_level: boolean;  // 是否有流量级别
  has_category: boolean;       // 是否有AI分类
  has_phrase_tag: boolean;     // 是否有词组打标
  has_orderliness: boolean;    // 是否有有序性
}

export interface Category {
  id: number;
  name: string;
  name_en: string | null;
  parent_id: number | null;
}

export interface Root {
  id: number;
  word: string;
  translation: string | null;
  contains_count: number;
  percentage: number;
  categories: number[];
}

export interface Stats {
  keywordCount: number;
  rootCount: number;
}

// 关键词完整数据
export interface KeywordData {
  id: number;
  product_id: number;
  // 原始Excel列 (A-P)
  keyword: string;                       // A: 关键词
  translation: string | null;            // B: 翻译
  relevance_score: string | null;        // C: 相关性得分
  relevance_level: string | null;        // D: 相关性档位
  traffic_total: number | null;          // E: 流量总和
  avg_keyword_rank: string | null;       // F: 周平均关键词排名
  avg_search_volume: number | null;      // G: 周平均搜索量
  cpc_bid: string | null;                // H: CPC建议竞价(元)
  bid_range: string | null;              // I: 建议竞价范围(元)
  click_rate: string | null;             // J: 点击转化率/周
  conversion_competition: string | null; // K: 周转化竞争
  competition_level: string | null;      // L: 竞争度档位
  natural_position_flow: string | null;  // M: 自然位流动率%
  top3_click_share: string | null;       // N: Top3周平均点击份额
  avg_conversion_share: string | null;   // O: 周平均转化份额
  asin_count: number | null;             // P: asin数量
  // 新增计算列
  traffic_level: string | null;          // 流量级别 (大词/中词/小词)
  negative_word: string | null;          // 否词
  orderliness: string | null;            // 有序性
  phrase_tag: string | null;             // 词组标签
  primary_category: string | null;       // 一级分类
  secondary_category: string | null;     // 二级分类
  search_intent: string | null;          // 搜索意图
  traffic_share: number | null;          // 流量占比
  // ASIN动态列
  asin_data: string | null;
}

// 备份信息
export interface BackupInfo {
  id: number;
  product_id: number;
  backup_name: string | null;
  created_at: string;
  keyword_data_count: number;
}

// ==================== 关键词排名监控 ====================

// 关键词监控
export interface KeywordMonitoring {
  id: number;
  product_id: number;
  keyword: string;
  asin: string;
  country: string;           // US/UK/DE/FR/IT/ES
  priority: string;          // high/medium/low
  is_active: boolean;

  // 最新排名
  latest_organic_rank: number | null;
  latest_organic_page: number | null;
  latest_sponsored_rank: number | null;
  latest_sponsored_page: number | null;

  // 产品信息
  image_url: string | null;
  price: string | null;
  reviews_count: number | null;
  rating: number | null;

  last_checked: string | null;
  created_at: string;
}

// 排名历史
export interface RankingHistory {
  id: number;
  monitoring_id: number;
  check_date: string;

  organic_rank: number | null;
  organic_page: number | null;
  sponsored_rank: number | null;
  sponsored_page: number | null;

  checked_at: string;
}

// 竞品快照
export interface RankingSnapshot {
  id: number;
  keyword: string;
  country: string;
  snapshot_date: string;

  organic_top_50: string | null;    // JSON: ["ASIN1", "ASIN2", ...]
  sponsored_top_20: string | null;  // JSON

  created_at: string;
}

// 监控统计
export interface MonitoringStats {
  total: number;
  active: number;
  top10_organic: number;
  top30_organic: number;
  with_sponsored: number;
}

// 监控迷你图数据
export interface MonitoringSparkline {
  monitoring_id: number;
  organic_ranks: (number | null)[];
  sponsored_ranks: (number | null)[];
}

// 排名检测结果
export interface RankingResult {
  keyword: string;
  target_asin: string;
  country: string;

  organic_rank: number | null;
  organic_page: number | null;
  sponsored_rank: number | null;
  sponsored_page: number | null;

  product_info: {
    asin: string;
    title: string | null;
    price: string | null;
    rating: number | null;
    reviews_count: number | null;
    image_url: string | null;
    availability: string | null;  // 商品可用性信息
  } | null;

  organic_top_50: string[];
  sponsored_top_20: string[];

  checked_at: string;
  error: string | null;
  warning: string | null;  // 警告信息（如地理限制）
}

// 国家选项（使用SVG国旗）
export const COUNTRY_OPTIONS = [
  { value: 'US', label: '美国', flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="20" fill="#B22234"/><rect y="1.54" width="30" height="1.54" fill="white"/><rect y="4.62" width="30" height="1.54" fill="white"/><rect y="7.69" width="30" height="1.54" fill="white"/><rect y="10.77" width="30" height="1.54" fill="white"/><rect y="13.85" width="30" height="1.54" fill="white"/><rect y="16.92" width="30" height="1.54" fill="white"/><rect width="12" height="10.77" fill="#3C3B6E"/></svg>` },
  { value: 'UK', label: '英国', flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="20" fill="#012169"/><path d="M0,0 L30,20 M30,0 L0,20" stroke="white" stroke-width="4"/><path d="M0,0 L30,20 M30,0 L0,20" stroke="#C8102E" stroke-width="2.5"/><path d="M15,0 V20 M0,10 H30" stroke="white" stroke-width="6"/><path d="M15,0 V20 M0,10 H30" stroke="#C8102E" stroke-width="3.5"/></svg>` },
  { value: 'DE', label: '德国', flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="6.67" fill="#000"/><rect y="6.67" width="30" height="6.67" fill="#DD0000"/><rect y="13.33" width="30" height="6.67" fill="#FFCE00"/></svg>` },
  { value: 'FR', label: '法国', flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="10" height="20" fill="#002395"/><rect x="10" width="10" height="20" fill="white"/><rect x="20" width="10" height="20" fill="#ED2939"/></svg>` },
  { value: 'IT', label: '意大利', flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="10" height="20" fill="#009246"/><rect x="10" width="10" height="20" fill="white"/><rect x="20" width="10" height="20" fill="#CE2B37"/></svg>` },
  { value: 'ES', label: '西班牙', flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="5" fill="#AA151B"/><rect y="5" width="30" height="10" fill="#F1BF00"/><rect y="15" width="30" height="5" fill="#AA151B"/></svg>` },
];

// 获取国旗SVG
export function getCountryFlag(code: string): string {
  const country = COUNTRY_OPTIONS.find(c => c.value === code);
  return country?.flag || '';
}

// 获取国家名称
export function getCountryLabel(code: string): string {
  const country = COUNTRY_OPTIONS.find(c => c.value === code);
  return country?.label || code;
}

// 优先级选项
export const PRIORITY_OPTIONS = [
  { value: 'high', label: '高', color: '#f56c6c' },
  { value: 'medium', label: '中', color: '#e6a23c' },
  { value: 'low', label: '低', color: '#909399' },
];

// ==================== 调度器设置 ====================

// 调度器设置
export interface SchedulerSettings {
  enabled: boolean;
  morning_start: number;     // 8
  morning_end: number;       // 10
  evening_start: number;     // 18
  evening_end: number;       // 21
  rank_change_threshold: number;  // 排名变化阈值，默认10
  notify_on_enter_top10: boolean;
  notify_on_exit_top10: boolean;
  notify_on_new_rank: boolean;
  notify_on_lost_rank: boolean;
}

// 调度器状态
export interface SchedulerStatus {
  is_running: boolean;
  last_check_time: string | null;
  next_check_time: string | null;
  current_task: string | null;
}

// 任务记录
export interface TaskLog {
  id: number;
  started_at: string;
  ended_at: string | null;
  status: 'running' | 'completed' | 'failed';
  total_keywords: number;
  success_count: number;
  failed_count: number;
  trigger_type: 'auto' | 'manual';
  error_message: string | null;
}

// 默认调度器设置
export const DEFAULT_SCHEDULER_SETTINGS: SchedulerSettings = {
  enabled: false,
  morning_start: 8,
  morning_end: 10,
  evening_start: 18,
  evening_end: 21,
  rank_change_threshold: 10,
  notify_on_enter_top10: true,
  notify_on_exit_top10: true,
  notify_on_new_rank: true,
  notify_on_lost_rank: true,
};

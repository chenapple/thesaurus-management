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

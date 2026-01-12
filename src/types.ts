export interface Product {
  id: number;
  name: string;
  country: string | null;            // 国家代码: US, UK, DE, FR, IT, ES, JP
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
  country: string;           // US/UK/DE/FR/IT/ES/JP
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
  tags: string | null;  // JSON array: ["high_traffic", "high_conversion"]
}

// 关键词标签定义
export const KEYWORD_TAGS = [
  { key: 'high_traffic', label: '高流量', color: '#409eff', description: '搜索量大，曝光机会多的关键词' },
  { key: 'high_click', label: '高点击', color: '#67c23a', description: '点击率高于平均水平的关键词' },
  { key: 'high_conversion', label: '高转化', color: '#e6a23c', description: '转化率表现优秀的关键词' },
  { key: 'low_roi', label: '低投产', color: '#f56c6c', description: 'ACOS低的优质关键词' },
  { key: 'high_return', label: '高回报', color: '#9b59b6', description: 'ROAS高的优质关键词' },
] as const;

export type KeywordTagKey = typeof KEYWORD_TAGS[number]['key'];

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
  { value: 'JP', label: '日本', flag: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect width="30" height="20" fill="white"/><circle cx="15" cy="10" r="6" fill="#BC002D"/></svg>` },
];

// 获取国旗SVG（支持代码或名称查找）
export function getCountryFlag(codeOrName: string): string {
  const country = COUNTRY_OPTIONS.find(c => c.value === codeOrName || c.label === codeOrName);
  return country?.flag || '';
}

// 获取国家名称（支持代码或名称查找）
export function getCountryLabel(codeOrName: string): string {
  const country = COUNTRY_OPTIONS.find(c => c.value === codeOrName || c.label === codeOrName);
  return country?.label || codeOrName;
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
  max_pages: number;         // 监控页数: 1/3/5
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

// ==================== 优化事件 ====================

// 主类型
export type EventMainType = 'listing' | 'ad';

// 子类型
export type ListingSubType = 'image' | 'price' | 'title' | 'bullets' | 'description' | 'aplus' | 'video' | 'brand_story' | 'brand_store';
export type AdSubType = 'toggle' | 'strategy' | 'bid' | 'budget' | 'keywords';
export type EventSubType = ListingSubType | AdSubType;

// 主类型配置
export const EVENT_MAIN_TYPES: Record<EventMainType, { label: string; color: string }> = {
  listing: { label: '文案优化', color: '#67C23A' },
  ad: { label: '广告优化', color: '#409EFF' },
};

// 子类型配置
export const EVENT_SUB_TYPES: Record<EventMainType, Record<string, { label: string }>> = {
  listing: {
    image: { label: '图片' },
    price: { label: '价格' },
    title: { label: '标题' },
    bullets: { label: '五点' },
    description: { label: '描述' },
    aplus: { label: 'A+页面' },
    video: { label: '视频' },
    brand_story: { label: '品牌故事' },
    brand_store: { label: '品牌旗舰店' },
  },
  ad: {
    toggle: { label: '启用/暂停广告' },
    strategy: { label: '调整广告策略' },
    bid: { label: '调整竞价' },
    budget: { label: '调整预算' },
    keywords: { label: '增词/否词' },
  },
};

// 优化事件
export interface OptimizationEvent {
  id: number;
  product_id: number;
  event_date: string;
  event_type: EventMainType;       // 主类型
  event_sub_type: EventSubType;    // 子类型
  title: string;
  description?: string;
  target_asin?: string;            // 目标 ASIN（可选）
  affected_keywords?: string;      // JSON字符串，存储关键词数组（可选）
  created_at: string;
}

// 兼容旧代码的 EVENT_TYPE_LABELS（基于主类型）
export const EVENT_TYPE_LABELS = EVENT_MAIN_TYPES;

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
  max_pages: 5,              // 默认监控前5页
};

// ==================== 依赖安装 ====================

// 依赖状态
export interface DependencyStatus {
  python_installed: boolean;
  python_version: string | null;
  python_path: string | null;
  playwright_installed: boolean;
  chromium_installed: boolean;
  // PDF 处理依赖
  pdf2image_installed: boolean;
  poppler_installed: boolean;
  error_message: string | null;
}

// 安装进度
export interface InstallProgress {
  step: 'python' | 'playwright' | 'chromium' | 'pdf2image' | 'poppler';
  step_name: string;
  progress: number;
  message: string;
  is_error: boolean;
}

// 安装结果
export interface InstallResult {
  success: boolean;
  message: string;
  python_path: string | null;
}

// ==================== 知识库 ====================

// 知识库分类
export interface KbCategory {
  id: number;
  name: string;
  parent_id: number | null;
  sort_order: number;
  color: string;
  created_at: string;
}

// 知识库文档
export interface KbDocument {
  id: number;
  category_id: number | null;
  title: string;
  file_name: string;
  file_path: string;
  file_type: string;  // pdf/docx/xlsx/txt/md
  file_size: number | null;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  chunk_count: number;
  created_at: string;
  // 向量化统计（前端加载时填充）
  embedding_total?: number;    // 总分块数
  embedding_count?: number;    // 已向量化数
}

// 从文档中提取的图片
export interface ExtractedImage {
  name: string;           // 图片文件名
  mime_type: string;      // MIME 类型 (image/png, image/jpeg, etc.)
  base64_data: string;    // Base64 编码的图片数据
  source_location: string; // 来源位置（如 "Sheet1", "Slide 3"）
}

// 文档分块
export interface KbChunk {
  id: number;
  document_id: number;
  chunk_index: number;
  content: string;
  page_number: number | null;
  image_path: string | null;  // 关联的图片路径（用于图文问答）
}

// 搜索结果
export interface KbSearchResult {
  chunk_id: number;
  document_id: number;
  document_title: string;
  content: string;
  page_number: number | null;
  score: number;
  image_path: string | null;  // 关联的图片路径（用于图文问答）
}

// AI 对话
export interface KbConversation {
  id: number;
  title: string | null;
  ai_provider: string;
  ai_model: string | null;
  created_at: string;
}

// AI 消息
export interface KbMessage {
  id: number;
  conversation_id: number;
  role: 'user' | 'assistant';
  content: string;
  sources: string | null;  // JSON: 引用的文档来源
  created_at: string;
}

// 消息来源（解析后的）
export interface MessageSource {
  document_id: number;
  document_title: string;
  chunk_id: number;
  page_number: number | null;
  snippet: string;
  image_path?: string | null;  // 关联的图片路径（用于图文问答）
}

// PDF 页面图片（用于 OCR）
export interface PdfPageImage {
  page_number: number;
  mime_type: string;
  base64_data: string;
}

// ==================== 智能文案 ====================

// 我的产品信息（新品打造时填写）
export interface MyProductInfo {
  brand_name: string;           // 品牌名称（必填）
  product_name: string;         // 产品名称（必填）
  key_features: string[];       // 核心卖点（必填，1-5条）
  differentiators?: string;     // 差异化特点（选填）
  specifications?: string;      // 规格参数（选填）
  target_audience?: string;     // 目标人群（选填）
  package_contents?: string;    // 包装配件（选填）
  additional_notes?: string;    // 补充说明（选填）
}

// 智能文案项目
export interface ScProject {
  id: number;
  name: string;
  scenario_type: 'new' | 'optimize';  // new: 新品打造, optimize: 老品优化
  marketplace: string;                 // US, UK, DE, FR, IT, ES, JP, etc.
  my_asin: string | null;             // 仅优化场景需要
  product_id: number | null;          // 关联的产品ID（用于获取关键词数据）
  my_product_info: string | null;     // 我的产品信息（JSON）
  // 用户的 Listing 信息（老品优化时使用）
  my_title: string | null;
  my_bullets: string | null;          // JSON 数组
  my_description: string | null;
  my_listing_fetched_at: string | null;
  status: 'draft' | 'collecting' | 'analyzing' | 'completed';
  created_at: string;
  updated_at: string;
  competitor_count: number;           // 竞品数量
}

// 竞品信息
export interface ScCompetitor {
  id: number;
  project_id: number;
  asin: string;
  competitor_type: 'top' | 'direct' | 'rising';  // 头部/直接/新锐竞品
  title: string | null;
  price: string | null;
  rating: string | null;
  review_count: number | null;
  bsr_rank: string | null;
  date_first_available: string | null;  // 上架时间
  image_url: string | null;
  bullets: string | null;         // JSON array
  description: string | null;
  fetched_at: string | null;
}

// 竞品类型选项
export const COMPETITOR_TYPE_OPTIONS = [
  { value: 'top', label: '头部竞品', color: '#f56c6c', description: 'BSR排名最高，市场标杆' },
  { value: 'direct', label: '直接竞品', color: '#409eff', description: '价格、功能相近的对手' },
  { value: 'rising', label: '新锐竞品', color: '#67c23a', description: '近期上升快，可能有创新打法' },
] as const;

// 竞品评论
export interface ScReview {
  id: number;
  competitor_id: number;
  star_rating: number;
  review_text: string | null;
  review_title: string | null;
  review_date: string | null;
  helpful_votes: number;
}

// 评论统计摘要
export interface ScReviewSummary {
  total: number;
  star_1: number;
  star_2: number;
  star_3: number;
  star_4: number;
  star_5: number;
}

// 评论爬取结果（从爬虫返回）
export interface ReviewResult {
  asin: string;
  country: string;
  reviews: Array<{
    star_rating: number;
    review_text: string;
    review_title: string | null;
    review_date: string | null;
    helpful_votes: number;
  }>;
  summary: {
    total: number;
    by_star: Record<string, number>;
  };
  fetched_at: string;
  error: string | null;
}

// 竞品问答
export interface ScQA {
  id: number;
  competitor_id: number;
  question: string;
  answer: string | null;
  votes: number | null;
  asked_date: string | null;
}

// AI 分析结果（数据库存储格式）
export interface ScAnalysis {
  id: number;
  project_id: number;
  analysis_type: 'review_insights' | 'listing_analysis' | 'optimization';
  result_json: string;             // JSON 格式的分析结果
  model_provider: string | null;   // 使用的 AI 服务商
  model_name: string | null;       // 使用的模型
  created_at: string;
}

// 评论洞察结果
export interface ReviewInsights {
  usage_scenarios: Array<{
    scenario: string;
    source_count: number;
    example_review: string;
  }>;
  praise_points: Array<{
    point: string;
    frequency: number;
    example_review: string;
  }>;
  pain_points: Array<{
    point: string;
    frequency: number;
    star_distribution: string;
    example_review: string;
  }>;
  summary: string;
}

// 文案分析结果
export interface ListingAnalysis {
  title_analysis: {
    common_structure: string;
    high_frequency_words: string[];
    competitors: Array<{
      asin: string;
      title: string;
      structure_breakdown: Record<string, string>;
    }>;
  };
  bullet_analysis: {
    common_themes: string[];
    best_practices: string[];
  };
  keyword_coverage: {
    covered: string[];
    missing: string[];
  };
}

// A+ 内容建议
export interface AplusSuggestions {
  main_image: {
    key_points: string[];  // 主图核心卖点文案
  };
  secondary_images: Array<{
    index: number;
    theme: string;           // 图片主题
    copy_suggestion: string; // 文案建议
  }>;
  module_recommendations: Array<{
    module_type: string;     // 模块类型标识
    module_name: string;     // 模块名称
    content_points: string[]; // 内容要点
  }>;
}

// 优化建议结果
export interface OptimizationResult {
  title_suggestions: Array<{
    version: number;
    content: string;
    reasons: Array<{
      word: string;
      reason: string;
      source: string;
    }>;
  }>;
  bullet_suggestions: Array<{
    index: number;
    focus: string;
    content: string;
    embedded_keywords?: string[];  // 埋入的关键词
    reason: string;
    source: string;
  }>;
  backend_keywords: Array<{
    keyword: string;
    reason: string;
    search_volume: number | null;
  }>;
  keyword_distribution_summary?: string;  // 关键词分布总结
  description_suggestions?: Array<{       // 商品描述建议
    version: number;           // 版本号
    content: string;           // 商品描述内容
    structure: string;         // 结构说明
    embedded_keywords: string[]; // 埋入的关键词
    highlights: string[];      // 突出的卖点
    reason: string;            // 为什么这样写
  }>;
  aplus_suggestions?: AplusSuggestions;   // A+ 内容建议
}

// AI 服务提供商
export type AIProvider = 'deepseek' | 'openai' | 'gemini' | 'qwen';

// AI 服务配置
export interface AIProviderConfig {
  provider: AIProvider;
  name: string;
  models: string[];
  defaultModel: string;
  apiKeyName: string;  // 用于密钥链存储的名称
}

// 已支持的 AI 服务
export const AI_PROVIDERS: Record<AIProvider, AIProviderConfig> = {
  deepseek: {
    provider: 'deepseek',
    name: 'DeepSeek',
    models: ['deepseek-chat', 'deepseek-reasoner'],
    defaultModel: 'deepseek-chat',
    apiKeyName: 'deepseek',
  },
  openai: {
    provider: 'openai',
    name: 'OpenAI',
    models: ['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo', 'gpt-3.5-turbo'],
    defaultModel: 'gpt-4o-mini',
    apiKeyName: 'openai',
  },
  gemini: {
    provider: 'gemini',
    name: 'Gemini',
    models: ['gemini-3-pro-preview', 'gemini-3-flash-preview', 'gemini-2.5-flash', 'gemini-2.5-pro', 'gemini-2.0-flash'],
    defaultModel: 'gemini-2.5-flash',
    apiKeyName: 'gemini',
  },
  qwen: {
    provider: 'qwen',
    name: '通义千问',
    models: ['qwen-turbo', 'qwen-plus', 'qwen-max', 'qwen-vl-plus', 'qwen-vl-max'],
    defaultModel: 'qwen-plus',
    apiKeyName: 'qwen',
  },
};

// ==================== 智能广告（Smart Ads）====================

// 国家到货币的映射（支持英文代码和中文名称）
export const COUNTRY_CURRENCY_MAP: Record<string, { symbol: string; code: string }> = {
  // 英文代码
  'UK': { symbol: '£', code: 'GBP' },
  'DE': { symbol: '€', code: 'EUR' },
  'FR': { symbol: '€', code: 'EUR' },
  'IT': { symbol: '€', code: 'EUR' },
  'ES': { symbol: '€', code: 'EUR' },
  'US': { symbol: '$', code: 'USD' },
  'CA': { symbol: 'C$', code: 'CAD' },
  'JP': { symbol: '¥', code: 'JPY' },
  'AU': { symbol: 'A$', code: 'AUD' },
  'MX': { symbol: 'MX$', code: 'MXN' },
  'IN': { symbol: '₹', code: 'INR' },
  'BR': { symbol: 'R$', code: 'BRL' },
  // 中文名称
  '英国': { symbol: '£', code: 'GBP' },
  '德国': { symbol: '€', code: 'EUR' },
  '法国': { symbol: '€', code: 'EUR' },
  '意大利': { symbol: '€', code: 'EUR' },
  '西班牙': { symbol: '€', code: 'EUR' },
  '美国': { symbol: '$', code: 'USD' },
  '加拿大': { symbol: 'C$', code: 'CAD' },
  '日本': { symbol: '¥', code: 'JPY' },
  '澳大利亚': { symbol: 'A$', code: 'AUD' },
  '墨西哥': { symbol: 'MX$', code: 'MXN' },
  '印度': { symbol: '₹', code: 'INR' },
  '巴西': { symbol: 'R$', code: 'BRL' },
  // 默认
  'Unknown': { symbol: '', code: '' },
  '未知市场': { symbol: '', code: '' },
};

// 按国家分组的统计数据
export interface CountryStats {
  country: string;
  total_spend: number;
  total_sales: number;
  avg_acos: number;
  term_count: number;
}

// 搜索词统计结果（包含总计和按国家分组）
export interface SearchTermsStatsResult {
  total_spend: number;
  total_sales: number;
  avg_acos: number;
  count: number;
  by_country: CountryStats[];
}

// 广告项目
export interface AdProject {
  id: number;
  product_id: number | null;
  name: string;
  marketplace: string;
  target_acos: number;
  created_at: string;
  updated_at: string;
  search_term_count: number;
}

// 搜索词数据
export interface AdSearchTerm {
  id: number;
  project_id: number;
  portfolio_name: string | null;  // 广告组合名称
  campaign_name: string | null;   // 广告活动名称
  ad_group_name: string | null;   // 广告组名称
  country: string | null;         // 国家/地区
  targeting: string | null;       // 投放词
  match_type: 'broad' | 'phrase' | 'exact' | 'auto' | null;  // 匹配类型
  customer_search_term: string | null;  // 客户搜索词
  impressions: number;            // 展示量
  clicks: number;                 // 点击量
  ctr: number;                    // 点击率
  spend: number;                  // 花费
  sales: number;                  // 销售额
  orders: number;                 // 订单数
  acos: number;                   // ACOS
  roas: number;                   // ROAS
  conversion_rate: number;        // 转化率
  cpc: number;                    // CPC
  report_date: string | null;     // 报告日期
  sku: string | null;             // SKU
  imported_at?: string;           // 导入时间
}

// 否定词建议
export interface NegativeWordSuggestion {
  search_term: string;
  reason: string;
  risk_level: 'high' | 'medium' | 'low';
  spend_wasted: number;
  match_type_suggestion: 'exact' | 'phrase';
  campaigns_affected: string[];
  // 精准定位字段
  campaign_name?: string;         // 广告活动名称
  ad_group_name?: string;         // 广告组名称
  targeting?: string;             // 投放词
  sku?: string;                   // SKU
  // 否定原因分类
  reason_category?: 'wrong_category' | 'wrong_scenario' | 'non_target_customer' | 'low_intent' | 'competitor' | 'other';
  // 否定层级建议
  negation_level?: 'ad_group' | 'campaign' | 'account';
  negation_level_reason?: string; // 为什么建议这个层级
}

// 竞价调整建议
export interface BidAdjustment {
  targeting: string;
  campaign_name: string;
  ad_group_name?: string;         // 广告组名称
  current_performance: {
    acos: number;
    conversion_rate: number;
    impressions: number;
    clicks: number;
  };
  suggestion: 'increase' | 'decrease' | 'pause' | 'maintain';
  adjustment_percent: number;
  reason: string;
  priority: 'high' | 'medium' | 'low';
  // 调整等级和信心值
  adjustment_level?: 'L1' | 'L2' | 'L3';  // L1轻微试探 L2明确调整 L3激进策略
  confidence?: number;                     // 信心值 0-1
  confidence_factors?: string[];           // 信心来源说明
}

// 关键词机会
export interface KeywordOpportunity {
  search_term: string;
  campaign_name?: string;         // 广告活动名称
  ad_group_name?: string;         // 广告组名称
  targeting?: string;             // 投放词
  performance: {
    orders: number;
    conversion_rate: number;
    acos: number;
  };
  suggestion: string;
  match_type: string;
  estimated_potential: string;
  // 机会类型分类
  opportunity_type?: 'expansion' | 'testing' | 'structure';  // 扩量词/测试词/结构词
  recommended_match_type?: 'exact' | 'phrase' | 'broad';     // 推荐匹配方式
  match_type_reason?: string;                                 // 匹配建议原因
}

// 单个国家的分析结果
export interface CountryAnalysisResult {
  country: string;
  currency: { symbol: string; code: string };
  negative_words: NegativeWordSuggestion[];
  bid_adjustments: BidAdjustment[];
  keyword_opportunities: KeywordOpportunity[];
  summary: {
    total_spend_analyzed: number;
    potential_savings: number;
    optimization_score: number;
    key_insights: string[];
  };
}

// 广告分析结果汇总
export interface AdAnalysisResult {
  negative_words: NegativeWordSuggestion[];
  bid_adjustments: BidAdjustment[];
  keyword_opportunities: KeywordOpportunity[];
  summary: {
    total_spend_analyzed: number;
    potential_savings: number;
    optimization_score: number;
    key_insights: string[];
  };
  // 按国家的详细结果（多国家数据时使用）
  by_country?: CountryAnalysisResult[];
}

// 广告分析记录（数据库存储）
export interface AdAnalysisRecord {
  id: number;
  project_id: number;
  analysis_type: string;
  result_json: string;
  ai_provider: string | null;
  ai_model: string | null;
  created_at: string;
}

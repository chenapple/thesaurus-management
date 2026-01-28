/**
 * 广告数据图表工具函数
 */
import type { AdSearchTerm } from '../types';

// 四象限数据点
export interface QuadrantPoint {
  searchTerm: string;
  conversionRate: number;
  acos: number;
  spend: number;
  sales: number;
  quadrant: 'high_potential' | 'needs_optimization' | 'stable' | 'eliminate';
}

// 四象限统计
export interface QuadrantData {
  points: QuadrantPoint[];
  avgConversionRate: number;
  targetAcos: number;
  quadrantCounts: {
    high_potential: number;
    needs_optimization: number;
    stable: number;
    eliminate: number;
  };
}

// 花费效率数据点
export interface SpendEfficiencyPoint {
  searchTerm: string;
  spend: number;
  sales: number;
  acos: number;
  orders: number;
  isEfficient: boolean;
}

// 花费效率数据
export interface SpendEfficiencyData {
  points: SpendEfficiencyPoint[];
  targetAcos: number;
  totalSpend: number;
  totalSales: number;
}

// ACOS 分布区间
export interface AcosDistribution {
  range: string;
  count: number;
  color: string;
}

// 花费占比项
export interface SpendShareItem {
  name: string;
  value: number;
  percent: number;
}

// Top 排行项
export interface TopRankItem {
  searchTerm: string;
  value: number;
  acos?: number;
  spend?: number;
  sales?: number;
}

// 仪表盘数据
export interface DashboardData {
  acosDistribution: AcosDistribution[];
  spendByCountry: SpendShareItem[];
  spendByCampaign: SpendShareItem[];
  topSpend: TopRankItem[];
  topSales: TopRankItem[];
  topWaste: TopRankItem[];
}

// 匹配类型对比数据
export interface MatchTypeStats {
  matchType: string;
  avgAcos: number;
  avgConversionRate: number;
  totalSpend: number;
  totalSales: number;
  spendPercent: number;
  salesPercent: number;
  count: number;
}

export interface MatchTypeComparisonData {
  stats: MatchTypeStats[];
  totalSpend: number;
  totalSales: number;
}

// 时间趋势数据点
export interface TrendDataPoint {
  date: string;
  spend: number;
  sales: number;
  orders: number;
  acos: number;
}

// 时间趋势数据
export interface TrendData {
  points: TrendDataPoint[];
  summary: {
    totalSpend: number;
    totalSales: number;
    totalOrders: number;
    avgAcos: number;
  };
  dateRange: { start: string; end: string };
}

/**
 * 计算四象限分布数据
 */
export function calculateQuadrantData(
  terms: AdSearchTerm[],
  targetAcos: number
): QuadrantData {
  // 过滤有效数据（至少有花费的）
  const validTerms = terms.filter(t => t.spend > 0);

  // 计算平均转化率
  const totalClicks = validTerms.reduce((sum, t) => sum + t.clicks, 0);
  const totalOrders = validTerms.reduce((sum, t) => sum + t.orders, 0);
  const avgConversionRate = totalClicks > 0 ? (totalOrders / totalClicks) * 100 : 0;

  const quadrantCounts = {
    high_potential: 0,
    needs_optimization: 0,
    stable: 0,
    eliminate: 0,
  };

  const points: QuadrantPoint[] = validTerms.map(term => {
    const conversionRate = term.conversion_rate;
    const acos = term.acos;

    // 确定象限
    // X轴 = 转化率，Y轴 = ACOS
    // 高潜力（右下）: 高转化率 + 低ACOS
    // 待优化（右上）: 高转化率 + 高ACOS
    // 稳定（左下）: 低转化率 + 低ACOS
    // 淘汰（左上）: 低转化率 + 高ACOS
    let quadrant: QuadrantPoint['quadrant'];
    if (conversionRate >= avgConversionRate && acos <= targetAcos) {
      quadrant = 'high_potential';
    } else if (conversionRate >= avgConversionRate && acos > targetAcos) {
      quadrant = 'needs_optimization';
    } else if (conversionRate < avgConversionRate && acos <= targetAcos) {
      quadrant = 'stable';
    } else {
      quadrant = 'eliminate';
    }

    quadrantCounts[quadrant]++;

    return {
      searchTerm: term.customer_search_term || term.targeting || '',
      conversionRate,
      acos,
      spend: term.spend,
      sales: term.sales,
      quadrant,
    };
  });

  return {
    points,
    avgConversionRate,
    targetAcos,
    quadrantCounts,
  };
}

/**
 * 计算花费效率数据
 */
export function calculateSpendEfficiencyData(
  terms: AdSearchTerm[],
  targetAcos: number
): SpendEfficiencyData {
  const validTerms = terms.filter(t => t.spend > 0);

  const totalSpend = validTerms.reduce((sum, t) => sum + t.spend, 0);
  const totalSales = validTerms.reduce((sum, t) => sum + t.sales, 0);

  const points: SpendEfficiencyPoint[] = validTerms.map(term => ({
    searchTerm: term.customer_search_term || term.targeting || '',
    spend: term.spend,
    sales: term.sales,
    acos: term.acos,
    orders: term.orders,
    isEfficient: term.sales > 0 && (term.spend / term.sales) * 100 <= targetAcos,
  }));

  return {
    points,
    targetAcos,
    totalSpend,
    totalSales,
  };
}

/**
 * 计算仪表盘数据
 */
export function calculateDashboardData(terms: AdSearchTerm[]): DashboardData {
  const validTerms = terms.filter(t => t.spend > 0);

  // ACOS 分布
  const acosRanges = [
    { range: '0-10%', min: 0, max: 10, color: '#67c23a' },
    { range: '10-20%', min: 10, max: 20, color: '#95d475' },
    { range: '20-30%', min: 20, max: 30, color: '#e6a23c' },
    { range: '30-50%', min: 30, max: 50, color: '#f56c6c' },
    { range: '50-100%', min: 50, max: 100, color: '#c45656' },
    { range: '>100%', min: 100, max: Infinity, color: '#8b0000' },
  ];

  const acosDistribution: AcosDistribution[] = acosRanges.map(range => ({
    range: range.range,
    count: validTerms.filter(t => t.acos >= range.min && t.acos < range.max).length,
    color: range.color,
  }));

  // 按国家分组花费
  const spendByCountryMap = new Map<string, number>();
  validTerms.forEach(t => {
    const country = t.country || 'Unknown';
    spendByCountryMap.set(country, (spendByCountryMap.get(country) || 0) + t.spend);
  });

  const totalSpend = validTerms.reduce((sum, t) => sum + t.spend, 0);
  let spendByCountry = Array.from(spendByCountryMap.entries())
    .map(([name, value]) => ({
      name,
      value,
      percent: totalSpend > 0 ? (value / totalSpend) * 100 : 0,
    }))
    .sort((a, b) => b.value - a.value);

  // Top5 + 其他
  if (spendByCountry.length > 5) {
    const top5 = spendByCountry.slice(0, 5);
    const others = spendByCountry.slice(5);
    const othersValue = others.reduce((sum, item) => sum + item.value, 0);
    spendByCountry = [
      ...top5,
      {
        name: '其他',
        value: othersValue,
        percent: totalSpend > 0 ? (othersValue / totalSpend) * 100 : 0,
      },
    ];
  }

  // 按广告活动分组花费
  const spendByCampaignMap = new Map<string, number>();
  validTerms.forEach(t => {
    const campaign = t.campaign_name || 'Unknown';
    spendByCampaignMap.set(campaign, (spendByCampaignMap.get(campaign) || 0) + t.spend);
  });

  let spendByCampaign = Array.from(spendByCampaignMap.entries())
    .map(([name, value]) => ({
      name,
      value,
      percent: totalSpend > 0 ? (value / totalSpend) * 100 : 0,
    }))
    .sort((a, b) => b.value - a.value);

  // Top5 + 其他
  if (spendByCampaign.length > 5) {
    const top5 = spendByCampaign.slice(0, 5);
    const others = spendByCampaign.slice(5);
    const othersValue = others.reduce((sum, item) => sum + item.value, 0);
    spendByCampaign = [
      ...top5,
      {
        name: '其他',
        value: othersValue,
        percent: totalSpend > 0 ? (othersValue / totalSpend) * 100 : 0,
      },
    ];
  }

  // Top 花费
  const topSpend: TopRankItem[] = [...validTerms]
    .sort((a, b) => b.spend - a.spend)
    .slice(0, 10)
    .map(t => ({
      searchTerm: t.customer_search_term || t.targeting || '',
      value: t.spend,
      acos: t.acos,
      sales: t.sales,
    }));

  // Top 销售
  const topSales: TopRankItem[] = [...validTerms]
    .filter(t => t.sales > 0)
    .sort((a, b) => b.sales - a.sales)
    .slice(0, 10)
    .map(t => ({
      searchTerm: t.customer_search_term || t.targeting || '',
      value: t.sales,
      acos: t.acos,
      spend: t.spend,
    }));

  // Top 浪费（有花费无销售）
  const topWaste: TopRankItem[] = [...validTerms]
    .filter(t => t.spend > 0 && t.sales === 0)
    .sort((a, b) => b.spend - a.spend)
    .slice(0, 10)
    .map(t => ({
      searchTerm: t.customer_search_term || t.targeting || '',
      value: t.spend,
      spend: t.spend,
    }));

  return {
    acosDistribution,
    spendByCountry,
    spendByCampaign,
    topSpend,
    topSales,
    topWaste,
  };
}

/**
 * 计算匹配类型对比数据
 */
export function calculateMatchTypeComparison(
  terms: AdSearchTerm[]
): MatchTypeComparisonData {
  const validTerms = terms.filter(t => t.spend > 0 && t.match_type);

  const totalSpend = validTerms.reduce((sum, t) => sum + t.spend, 0);
  const totalSales = validTerms.reduce((sum, t) => sum + t.sales, 0);

  // 按匹配类型分组
  const matchTypeMap = new Map<string, {
    spend: number;
    sales: number;
    clicks: number;
    orders: number;
    count: number;
  }>();

  const matchTypes = ['broad', 'phrase', 'exact', 'auto'];
  matchTypes.forEach(type => {
    matchTypeMap.set(type, { spend: 0, sales: 0, clicks: 0, orders: 0, count: 0 });
  });

  validTerms.forEach(t => {
    const type = t.match_type || 'auto';
    const data = matchTypeMap.get(type);
    if (data) {
      data.spend += t.spend;
      data.sales += t.sales;
      data.clicks += t.clicks;
      data.orders += t.orders;
      data.count++;
    }
  });

  const matchTypeLabels: Record<string, string> = {
    broad: '广泛匹配',
    phrase: '词组匹配',
    exact: '精确匹配',
    auto: '自动投放',
  };

  const stats: MatchTypeStats[] = matchTypes.map(type => {
    const data = matchTypeMap.get(type)!;
    const avgAcos = data.sales > 0 ? (data.spend / data.sales) * 100 : 0;
    const avgConversionRate = data.clicks > 0 ? (data.orders / data.clicks) * 100 : 0;

    return {
      matchType: matchTypeLabels[type],
      avgAcos,
      avgConversionRate,
      totalSpend: data.spend,
      totalSales: data.sales,
      spendPercent: totalSpend > 0 ? (data.spend / totalSpend) * 100 : 0,
      salesPercent: totalSales > 0 ? (data.sales / totalSales) * 100 : 0,
      count: data.count,
    };
  });

  return {
    stats,
    totalSpend,
    totalSales,
  };
}

/**
 * 获取象限颜色
 */
export function getQuadrantColor(quadrant: QuadrantPoint['quadrant']): string {
  const colors: Record<QuadrantPoint['quadrant'], string> = {
    high_potential: '#67c23a',    // 绿色 - 高潜力
    needs_optimization: '#e6a23c', // 橙色 - 待优化
    stable: '#409eff',            // 蓝色 - 稳定
    eliminate: '#f56c6c',         // 红色 - 淘汰
  };
  return colors[quadrant];
}

/**
 * 获取象限名称
 */
export function getQuadrantName(quadrant: QuadrantPoint['quadrant']): string {
  const names: Record<QuadrantPoint['quadrant'], string> = {
    high_potential: '高潜力',
    needs_optimization: '待优化',
    stable: '稳定',
    eliminate: '淘汰',
  };
  return names[quadrant];
}

/**
 * 计算时间趋势数据
 * 按日期聚合搜索词数据
 */
export function calculateTrendData(terms: AdSearchTerm[]): TrendData {
  // 过滤有日期的数据
  const validTerms = terms.filter(t => t.report_date);

  // 按日期分组聚合
  const dateMap = new Map<string, {
    spend: number;
    sales: number;
    orders: number;
  }>();

  validTerms.forEach(term => {
    const date = term.report_date!;
    const existing = dateMap.get(date) || { spend: 0, sales: 0, orders: 0 };
    dateMap.set(date, {
      spend: existing.spend + term.spend,
      sales: existing.sales + term.sales,
      orders: existing.orders + term.orders,
    });
  });

  // 转换为数组并按日期排序
  const points: TrendDataPoint[] = Array.from(dateMap.entries())
    .map(([date, data]) => ({
      date,
      spend: data.spend,
      sales: data.sales,
      orders: data.orders,
      acos: data.sales > 0 ? (data.spend / data.sales) * 100 : 0,
    }))
    .sort((a, b) => a.date.localeCompare(b.date));

  // 计算汇总
  const totalSpend = points.reduce((sum, p) => sum + p.spend, 0);
  const totalSales = points.reduce((sum, p) => sum + p.sales, 0);
  const totalOrders = points.reduce((sum, p) => sum + p.orders, 0);
  const avgAcos = totalSales > 0 ? (totalSpend / totalSales) * 100 : 0;

  // 计算日期范围
  const dates = points.map(p => p.date);
  const dateRange = {
    start: dates[0] || '',
    end: dates[dates.length - 1] || '',
  };

  return {
    points,
    summary: {
      totalSpend,
      totalSales,
      totalOrders,
      avgAcos,
    },
    dateRange,
  };
}

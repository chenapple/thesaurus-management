// 市场调研周报模板生成器
// 固定模板 + 数据填充，确保输出一致性

export interface BsrChangeItem {
  asin: string;
  title?: string;
  imageUrl?: string;
  oldRank: number;
  newRank: number;
  change: number;
}

export interface NewProductItem {
  asin: string;
  rank: number;
  title?: string;
  imageUrl?: string;
  price?: string;
  rating?: number;
  reviews: number;
}

export interface PriceTrendData {
  avgPrice: string | null;
  minPrice: string | null;
  maxPrice: string | null;
  priceChange: string | null;
  priceChangePercent: string | null;
  trend: 'increasing' | 'decreasing' | 'stable' | 'unknown';
}

export interface ReportData {
  marketplace: string;
  categoryName: string;
  categoryId: string;
  reportDate: string;
  periodFrom: string;
  periodTo: string;
  // BSR 数据
  totalProducts: number;
  significantChanges: number;
  newEntries: number;
  droppedOut: number;
  topRankImprovements: BsrChangeItem[];
  topRankDeclines: BsrChangeItem[];
  // 新品数据
  newProducts: NewProductItem[];
  // 价格数据
  priceTrend: PriceTrendData;
  // 是否有足够数据
  hasEnoughData: boolean;
}

// ==================== 辅助函数 ====================

function truncateTitle(title: string | undefined, maxLen: number = 35): string {
  if (!title) return '-';
  return title.length > maxLen ? title.slice(0, maxLen) + '...' : title;
}

// 根据站点生成 Amazon 产品链接
function getAmazonProductUrl(marketplace: string, asin: string): string {
  const domains: Record<string, string> = {
    US: 'amazon.com',
    CA: 'amazon.ca',
    MX: 'amazon.com.mx',
    BR: 'amazon.com.br',
    UK: 'amazon.co.uk',
    DE: 'amazon.de',
    FR: 'amazon.fr',
    IT: 'amazon.it',
    ES: 'amazon.es',
    NL: 'amazon.nl',
    SE: 'amazon.se',
    PL: 'amazon.pl',
    JP: 'amazon.co.jp',
    AU: 'amazon.com.au',
  };
  const domain = domains[marketplace] || 'amazon.com';
  return `https://www.${domain}/dp/${asin}`;
}

function formatPrice(price: string | null | undefined): string {
  if (!price) return '-';
  return price;
}

function formatNumber(num: number | null | undefined): string {
  if (num === null || num === undefined) return '-';
  return num.toLocaleString();
}

function formatRating(rating: number | null | undefined): string {
  if (rating === null || rating === undefined) return '-';
  return rating.toFixed(1);
}

// 保留用于未来扩展
function _getTrendIcon(trend: string): string {
  switch (trend) {
    case 'increasing': return '📈';
    case 'decreasing': return '📉';
    case 'stable': return '➡️';
    default: return '❓';
  }
}

function _getTrendText(trend: string): string {
  switch (trend) {
    case 'increasing': return '上涨趋势';
    case 'decreasing': return '下跌趋势';
    case 'stable': return '保持稳定';
    default: return '数据不足';
  }
}

// 导出以避免 TS 未使用警告
export const _unused = { _getTrendIcon, _getTrendText };

// ==================== 行动建议生成规则 ====================

interface ActionRule {
  condition: (data: ReportData) => boolean;
  icon: string;
  generate: (data: ReportData) => string;
  priority: number;
}

const ACTION_RULES: ActionRule[] = [
  // 规则1: 数据不足警告
  {
    condition: (data) => !data.hasEnoughData,
    icon: '⚠️',
    generate: () => '历史数据不足，建议持续监控以积累更多数据进行分析',
    priority: 0,
  },
  // 规则2: 快速上升产品
  {
    condition: (data) => data.topRankImprovements.some(p => p.change >= 30),
    icon: '⚡',
    generate: (data) => {
      const top = data.topRankImprovements.find(p => p.change >= 30);
      return `关注快速上升产品 ${top?.asin}（上升 ${top?.change} 位），分析其 Listing 优化策略`;
    },
    priority: 1,
  },
  // 规则3: 新品进入 Top 30
  {
    condition: (data) => data.newProducts.some(p => p.rank <= 30),
    icon: '🆕',
    generate: (data) => {
      const top = data.newProducts.find(p => p.rank <= 30);
      return `警惕新进 Top 30 的竞品 ${top?.asin}（排名 #${top?.rank}），建议分析其定价和卖点`;
    },
    priority: 2,
  },
  // 规则4: 价格下跌超过 5%
  {
    condition: (data) => {
      const percent = parseFloat(data.priceTrend.priceChangePercent || '0');
      return percent < -5;
    },
    icon: '💰',
    generate: (data) => `市场整体降价 ${data.priceTrend.priceChangePercent}%，建议关注成本优化或差异化竞争`,
    priority: 3,
  },
  // 规则5: 价格上涨超过 5%
  {
    condition: (data) => {
      const percent = parseFloat(data.priceTrend.priceChangePercent || '0');
      return percent > 5;
    },
    icon: '📈',
    generate: (data) => `市场溢价空间增加 ${data.priceTrend.priceChangePercent}%，可考虑适当提价`,
    priority: 4,
  },
  // 规则6: 大量产品跌出榜单
  {
    condition: (data) => data.droppedOut >= 10,
    icon: '🔄',
    generate: (data) => `本周有 ${data.droppedOut} 个产品跌出 Top 100，市场竞争加剧，需关注`,
    priority: 5,
  },
  // 规则7: 排名快速下降
  {
    condition: (data) => data.topRankDeclines.some(p => Math.abs(p.change) >= 30),
    icon: '📊',
    generate: (data) => {
      const top = data.topRankDeclines.find(p => Math.abs(p.change) >= 30);
      return `产品 ${top?.asin} 排名大幅下滑（下降 ${Math.abs(top?.change || 0)} 位），可分析其失败原因`;
    },
    priority: 6,
  },
  // 规则8: 市场稳定
  {
    condition: (data) => data.hasEnoughData && data.significantChanges < 5 && data.newEntries < 3,
    icon: '✅',
    generate: () => '市场相对稳定，可保持当前运营策略',
    priority: 99,
  },
];

function generateActionItems(data: ReportData): string[] {
  const actions: { text: string; priority: number }[] = [];

  for (const rule of ACTION_RULES) {
    if (rule.condition(data)) {
      actions.push({
        text: `${rule.icon} ${rule.generate(data)}`,
        priority: rule.priority,
      });
    }
  }

  // 按优先级排序，取前5条
  actions.sort((a, b) => a.priority - b.priority);
  return actions.slice(0, 5).map(a => a.text);
}

// ==================== 模板生成 ====================

export function generateWeeklyReport(data: ReportData): string {
  const actionItems = generateActionItems(data);

  // 生成图片单元格
  const imgCell = (url?: string) => url
    ? `<td style="padding:6px;border-bottom:1px solid #e5e7eb;width:50px;"><img src="${url}" style="width:40px;height:40px;object-fit:contain;border-radius:4px;background:#f8fafc;" /></td>`
    : `<td style="padding:6px;border-bottom:1px solid #e5e7eb;width:50px;"><div style="width:40px;height:40px;background:#f1f5f9;border-radius:4px;"></div></td>`;

  // 生成 ASIN 链接单元格
  const asinCell = (asin: string) => {
    const url = getAmazonProductUrl(data.marketplace, asin);
    return `<td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;font-family:monospace;font-size:12px;"><a href="${url}" target="_blank" style="color:#0369a1;text-decoration:none;" title="在 Amazon 打开">${asin}</a></td>`;
  };

  // 排名上升行
  const improvementRows = data.topRankImprovements.slice(0, 5).map((item) =>
    `<tr>
      ${asinCell(item.asin)}
      ${imgCell(item.imageUrl)}
      <td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;color:#374151;">${truncateTitle(item.title, 26)}</td>
      <td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;text-align:center;color:#6b7280;">#${item.oldRank}</td>
      <td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;text-align:center;font-weight:600;color:#374151;">#${item.newRank}</td>
      <td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;text-align:center;color:#059669;font-weight:600;">↑${item.change}</td>
    </tr>`
  ).join('');

  // 排名下降行
  const declineRows = data.topRankDeclines.slice(0, 5).map((item) =>
    `<tr>
      ${asinCell(item.asin)}
      ${imgCell(item.imageUrl)}
      <td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;color:#374151;">${truncateTitle(item.title, 26)}</td>
      <td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;text-align:center;color:#6b7280;">#${item.oldRank}</td>
      <td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;text-align:center;font-weight:600;color:#374151;">#${item.newRank}</td>
      <td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;text-align:center;color:#dc2626;font-weight:600;">↓${Math.abs(item.change)}</td>
    </tr>`
  ).join('');

  // 新品行
  const newProductRows = data.newProducts.slice(0, 6).map((item) =>
    `<tr>
      <td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;text-align:center;font-weight:600;color:#374151;">#${item.rank}</td>
      ${asinCell(item.asin)}
      ${imgCell(item.imageUrl)}
      <td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;color:#374151;">${truncateTitle(item.title, 24)}</td>
      <td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;text-align:right;color:#374151;">${formatPrice(item.price)}</td>
      <td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;text-align:center;color:#6b7280;">${formatRating(item.rating)}</td>
      <td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;text-align:right;color:#6b7280;">${formatNumber(item.reviews)}</td>
    </tr>`
  ).join('');

  // 行动建议
  const actionList = actionItems.map((item) =>
    `<div style="padding:10px 14px;background:#f8fafc;border-left:3px solid #0369a1;margin-bottom:8px;color:#374151;font-size:14px;">${item}</div>`
  ).join('');

  // 趋势颜色
  const trendColor = data.priceTrend.trend === 'increasing' ? '#059669' :
                     data.priceTrend.trend === 'decreasing' ? '#dc2626' : '#6b7280';

  return `<div style="font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;max-width:800px;margin:0 auto;color:#1f2937;line-height:1.5;">

  <!-- 标题 -->
  <div style="border-bottom:2px solid #e5e7eb;padding-bottom:16px;margin-bottom:24px;">
    <h1 style="margin:0 0 4px 0;font-size:22px;font-weight:600;color:#111827;">市场调研周报</h1>
    <div style="font-size:15px;color:#374151;font-weight:500;">${data.categoryName} · ${data.marketplace}</div>
    <div style="font-size:13px;color:#6b7280;margin-top:4px;">报告日期: ${data.reportDate} | 数据周期: ${data.periodFrom} ~ ${data.periodTo}</div>
  </div>

  <!-- 数据概览 -->
  <div style="display:flex;gap:12px;margin-bottom:28px;">
    <div style="flex:1;background:#f8fafc;border:1px solid #e5e7eb;border-radius:6px;padding:14px;text-align:center;">
      <div style="font-size:24px;font-weight:600;color:#111827;">${formatNumber(data.totalProducts)}</div>
      <div style="font-size:12px;color:#6b7280;margin-top:2px;">监控产品</div>
    </div>
    <div style="flex:1;background:#f8fafc;border:1px solid #e5e7eb;border-radius:6px;padding:14px;text-align:center;">
      <div style="font-size:24px;font-weight:600;color:#111827;">${formatNumber(data.significantChanges)}</div>
      <div style="font-size:12px;color:#6b7280;margin-top:2px;">显著变化</div>
    </div>
    <div style="flex:1;background:#f8fafc;border:1px solid #e5e7eb;border-radius:6px;padding:14px;text-align:center;">
      <div style="font-size:24px;font-weight:600;color:#059669;">${formatNumber(data.newEntries)}</div>
      <div style="font-size:12px;color:#6b7280;margin-top:2px;">新进榜单</div>
    </div>
    <div style="flex:1;background:#f8fafc;border:1px solid #e5e7eb;border-radius:6px;padding:14px;text-align:center;">
      <div style="font-size:24px;font-weight:600;color:#dc2626;">${formatNumber(data.droppedOut)}</div>
      <div style="font-size:12px;color:#6b7280;margin-top:2px;">跌出榜单</div>
    </div>
  </div>

  <!-- 排名上升 -->
  <div style="margin-bottom:28px;">
    <h2 style="font-size:15px;font-weight:600;color:#111827;margin:0 0 12px 0;display:flex;align-items:center;gap:6px;">
      <span style="color:#059669;">▲</span> 排名上升 TOP 5
    </h2>
    ${data.topRankImprovements.length > 0 ? `
    <table style="width:100%;border-collapse:collapse;font-size:13px;">
      <thead>
        <tr style="background:#f8fafc;">
          <th style="padding:8px 12px;text-align:left;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">ASIN</th>
          <th style="padding:6px;width:50px;text-align:center;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">图片</th>
          <th style="padding:8px 12px;text-align:left;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">产品标题</th>
          <th style="padding:8px 12px;text-align:center;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">原排名</th>
          <th style="padding:8px 12px;text-align:center;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">现排名</th>
          <th style="padding:8px 12px;text-align:center;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">变化</th>
        </tr>
      </thead>
      <tbody>${improvementRows}</tbody>
    </table>` : '<div style="padding:16px;background:#f8fafc;border-radius:6px;color:#6b7280;font-size:13px;">暂无显著排名上升的产品</div>'}
  </div>

  <!-- 排名下降 -->
  <div style="margin-bottom:28px;">
    <h2 style="font-size:15px;font-weight:600;color:#111827;margin:0 0 12px 0;display:flex;align-items:center;gap:6px;">
      <span style="color:#dc2626;">▼</span> 排名下降 TOP 5
    </h2>
    ${data.topRankDeclines.length > 0 ? `
    <table style="width:100%;border-collapse:collapse;font-size:13px;">
      <thead>
        <tr style="background:#f8fafc;">
          <th style="padding:8px 12px;text-align:left;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">ASIN</th>
          <th style="padding:6px;width:50px;text-align:center;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">图片</th>
          <th style="padding:8px 12px;text-align:left;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">产品标题</th>
          <th style="padding:8px 12px;text-align:center;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">原排名</th>
          <th style="padding:8px 12px;text-align:center;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">现排名</th>
          <th style="padding:8px 12px;text-align:center;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">变化</th>
        </tr>
      </thead>
      <tbody>${declineRows}</tbody>
    </table>` : '<div style="padding:16px;background:#f8fafc;border-radius:6px;color:#6b7280;font-size:13px;">暂无显著排名下降的产品</div>'}
  </div>

  <!-- 本周新品 -->
  <div style="margin-bottom:28px;">
    <h2 style="font-size:15px;font-weight:600;color:#111827;margin:0 0 12px 0;">本周新品</h2>
    ${data.newProducts.length > 0 ? `
    <table style="width:100%;border-collapse:collapse;font-size:13px;">
      <thead>
        <tr style="background:#f8fafc;">
          <th style="padding:8px 12px;text-align:center;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">排名</th>
          <th style="padding:8px 12px;text-align:left;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">ASIN</th>
          <th style="padding:6px;width:50px;text-align:center;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">图片</th>
          <th style="padding:8px 12px;text-align:left;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">产品标题</th>
          <th style="padding:8px 12px;text-align:right;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">价格</th>
          <th style="padding:8px 12px;text-align:center;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">评分</th>
          <th style="padding:8px 12px;text-align:right;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">评论数</th>
        </tr>
      </thead>
      <tbody>${newProductRows}</tbody>
    </table>` : '<div style="padding:16px;background:#f8fafc;border-radius:6px;color:#6b7280;font-size:13px;">本周暂无新品进入榜单</div>'}
  </div>

  <!-- 价格趋势 -->
  <div style="margin-bottom:28px;">
    <h2 style="font-size:15px;font-weight:600;color:#111827;margin:0 0 12px 0;">价格趋势</h2>
    <div style="display:flex;gap:12px;">
      <div style="flex:1;background:#f8fafc;border:1px solid #e5e7eb;border-radius:6px;padding:14px;">
        <div style="font-size:12px;color:#6b7280;margin-bottom:4px;">平均价格</div>
        <div style="font-size:18px;font-weight:600;color:#111827;">${formatPrice(data.priceTrend.avgPrice)}</div>
      </div>
      <div style="flex:1;background:#f8fafc;border:1px solid #e5e7eb;border-radius:6px;padding:14px;">
        <div style="font-size:12px;color:#6b7280;margin-bottom:4px;">价格区间</div>
        <div style="font-size:18px;font-weight:600;color:#111827;">${formatPrice(data.priceTrend.minPrice)} ~ ${formatPrice(data.priceTrend.maxPrice)}</div>
      </div>
      <div style="flex:1;background:#f8fafc;border:1px solid #e5e7eb;border-radius:6px;padding:14px;">
        <div style="font-size:12px;color:#6b7280;margin-bottom:4px;">周变化</div>
        <div style="font-size:18px;font-weight:600;color:${trendColor};">${data.priceTrend.priceChange || '-'} (${data.priceTrend.priceChangePercent || '-'}%)</div>
      </div>
    </div>
  </div>

  <!-- 行动建议 -->
  <div style="margin-bottom:20px;">
    <h2 style="font-size:15px;font-weight:600;color:#111827;margin:0 0 12px 0;">行动建议</h2>
    ${actionList || '<div style="padding:16px;background:#f8fafc;border-radius:6px;color:#6b7280;font-size:13px;">暂无特别建议，请持续关注市场动态</div>'}
  </div>

  <!-- 页脚 -->
  <div style="border-top:1px solid #e5e7eb;padding-top:12px;text-align:center;font-size:12px;color:#9ca3af;">
    报告由系统自动生成 · ${new Date().toISOString().split('T')[0]}
  </div>

</div>`;
}

// ==================== 从工具数据构建报告数据 ====================

export interface ToolResults {
  bsrComparison?: {
    status: string;
    comparison_period?: { from: string; to: string };
    summary?: {
      total_products: number;
      significant_rank_changes: number;
      new_entries: number;
      dropped_out: number;
    };
    top_rank_improvements?: any[];
    top_rank_declines?: any[];
  };
  newProducts?: {
    status: string;
    period?: { from: string; to: string };
    new_products?: any[];
  };
  priceTrends?: {
    status: string;
    summary?: {
      current_avg_price?: string;
      current_min_price?: string;
      current_max_price?: string;
      price_change?: string;
      price_change_percent?: string;
      trend?: string;
    };
  };
}

export function buildReportDataFromTools(
  marketplace: string,
  categoryId: string,
  categoryName: string,
  toolResults: ToolResults
): ReportData {
  const today = new Date().toISOString().split('T')[0];
  const bsr = toolResults.bsrComparison;
  const newProd = toolResults.newProducts;
  const price = toolResults.priceTrends;

  const hasEnoughData = bsr?.status === 'success';

  return {
    marketplace,
    categoryId,
    categoryName,
    reportDate: today,
    periodFrom: bsr?.comparison_period?.from || today,
    periodTo: bsr?.comparison_period?.to || today,
    totalProducts: bsr?.summary?.total_products || 0,
    significantChanges: bsr?.summary?.significant_rank_changes || 0,
    newEntries: bsr?.summary?.new_entries || 0,
    droppedOut: bsr?.summary?.dropped_out || 0,
    topRankImprovements: (bsr?.top_rank_improvements || []).map(item => ({
      asin: item.asin,
      title: item.title,
      imageUrl: item.image_url,
      oldRank: item.oldRank,
      newRank: item.newRank,
      change: item.change,
    })),
    topRankDeclines: (bsr?.top_rank_declines || []).map(item => ({
      asin: item.asin,
      title: item.title,
      imageUrl: item.image_url,
      oldRank: item.oldRank,
      newRank: item.newRank,
      change: item.change,
    })),
    newProducts: (newProd?.new_products || []).map(item => ({
      asin: item.asin,
      rank: item.rank,
      title: item.title,
      imageUrl: item.image_url,
      price: item.price,
      rating: item.rating,
      reviews: item.reviews,
    })),
    priceTrend: {
      avgPrice: price?.summary?.current_avg_price || null,
      minPrice: price?.summary?.current_min_price || null,
      maxPrice: price?.summary?.current_max_price || null,
      priceChange: price?.summary?.price_change || null,
      priceChangePercent: price?.summary?.price_change_percent || null,
      trend: (price?.summary?.trend as any) || 'unknown',
    },
    hasEnoughData,
  };
}

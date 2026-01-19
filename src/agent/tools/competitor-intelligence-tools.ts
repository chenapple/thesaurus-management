// 竞品情报 Agent 工具集

import { createTool } from '../Tool';
import type { Tool } from '../types';
import { invoke } from '@tauri-apps/api/core';

// ==================== 类型定义 ====================

interface ListingResult {
  asin: string;
  country: string;
  title?: string;
  price?: string;
  rating?: string;
  review_count?: number;
  bsr_rank?: string;
  date_first_available?: string;
  image_url?: string;
  bullets: string[];
  description?: string;
  fetched_at: string;
  error?: string;
}

interface CompetitorSnapshot {
  id: number;
  asin_id: number;
  snapshot_date: string;
  price?: number;
  bsr_rank?: number;
  rating?: number;
  review_count?: number;
  availability?: string;
  created_at: string;
}

interface CompetitorAsin {
  id: number;
  task_id: number;
  asin: string;
  title?: string;
  tags?: string;
  created_at: string;
}

// ==================== 竞品数据抓取工具 ====================

export const fetchCompetitorListingTool: Tool = createTool(
  'fetch_competitor_listing',
  '抓取单个竞品 ASIN 的详细信息（价格、评分、评论数、BSR排名等）',
  {
    type: 'object',
    properties: {
      asin: {
        type: 'string',
        description: '竞品 ASIN（10位字母数字）',
      },
      marketplace: {
        type: 'string',
        description: 'Amazon 站点代码（如 US, UK, DE, JP 等）',
        enum: ['US', 'CA', 'MX', 'BR', 'UK', 'DE', 'FR', 'IT', 'ES', 'NL', 'SE', 'PL', 'JP', 'AU'],
      },
    },
    required: ['asin', 'marketplace'],
  },
  async (params) => {
    try {
      const result = await invoke<ListingResult>('fetch_listing_info', {
        asin: params.asin,
        country: params.marketplace,
      });
      return result;
    } catch (error) {
      console.error('竞品数据抓取失败:', error);
      return {
        asin: params.asin,
        country: params.marketplace,
        error: `抓取失败: ${error}`,
        fetched_at: new Date().toISOString(),
      };
    }
  }
);

// ==================== 批量抓取工具 ====================

export const fetchCompetitorsBatchTool: Tool = createTool(
  'fetch_competitors_batch',
  '批量抓取多个竞品 ASIN 的信息（一次浏览器获取全部），返回所有竞品的当前数据',
  {
    type: 'object',
    properties: {
      task_id: {
        type: 'number',
        description: '竞品监控任务 ID',
      },
      marketplace: {
        type: 'string',
        description: 'Amazon 站点代码',
      },
    },
    required: ['task_id', 'marketplace'],
  },
  async (params) => {
    try {
      // 使用批量 API（一次浏览器获取全部 ASIN）
      const results = await invoke<ListingResult[]>('fetch_competitor_listings_batch', {
        taskId: params.task_id,
        marketplace: params.marketplace,
      });

      if (results.length === 0) {
        return {
          task_id: params.task_id,
          results: [],
          message: '该任务下没有配置竞品 ASIN',
        };
      }

      return {
        task_id: params.task_id,
        marketplace: params.marketplace,
        total: results.length,
        success: results.filter(r => !r.error).length,
        failed: results.filter(r => r.error).length,
        results,
      };
    } catch (error) {
      console.error('批量抓取失败:', error);
      return {
        task_id: params.task_id,
        error: `批量抓取失败: ${error}`,
      };
    }
  }
);

// ==================== 历史数据对比工具 ====================

export const compareCompetitorHistoryTool: Tool = createTool(
  'compare_competitor_history',
  '对比竞品的历史数据变化（价格、BSR、评论等）',
  {
    type: 'object',
    properties: {
      task_id: {
        type: 'number',
        description: '竞品监控任务 ID',
      },
      days: {
        type: 'number',
        description: '对比多少天内的数据（默认 7 天）',
      },
    },
    required: ['task_id'],
  },
  async (params) => {
    try {
      const days = params.days || 7;

      // 获取任务下的所有 ASIN
      const asins = await invoke<CompetitorAsin[]>('get_competitor_asins', {
        taskId: params.task_id,
      });

      if (asins.length === 0) {
        return {
          task_id: params.task_id,
          changes: [],
          message: '该任务下没有配置竞品 ASIN',
        };
      }

      const changes: {
        asin: string;
        title?: string;
        price_change?: { old: number; new: number; direction: string };
        bsr_change?: { old: number; new: number; direction: string };
        review_change?: { old: number; new: number; diff: number };
        rating_change?: { old: number; new: number };
      }[] = [];

      // 对每个 ASIN 获取历史数据并对比
      for (const asin of asins) {
        try {
          const snapshots = await invoke<CompetitorSnapshot[]>('get_competitor_snapshots', {
            asinId: asin.id,
            days,
          });

          if (snapshots.length >= 2) {
            const latest = snapshots[0];
            const oldest = snapshots[snapshots.length - 1];

            const change: typeof changes[0] = {
              asin: asin.asin,
              title: asin.title,
            };

            // 价格变化
            if (latest.price !== undefined && oldest.price !== undefined && latest.price !== oldest.price) {
              change.price_change = {
                old: oldest.price,
                new: latest.price,
                direction: latest.price > oldest.price ? 'up' : 'down',
              };
            }

            // BSR 变化
            if (latest.bsr_rank !== undefined && oldest.bsr_rank !== undefined && latest.bsr_rank !== oldest.bsr_rank) {
              change.bsr_change = {
                old: oldest.bsr_rank,
                new: latest.bsr_rank,
                direction: latest.bsr_rank < oldest.bsr_rank ? 'up' : 'down', // BSR越小越好
              };
            }

            // 评论数变化
            if (latest.review_count !== undefined && oldest.review_count !== undefined) {
              const diff = (latest.review_count || 0) - (oldest.review_count || 0);
              if (diff !== 0) {
                change.review_change = {
                  old: oldest.review_count || 0,
                  new: latest.review_count || 0,
                  diff,
                };
              }
            }

            // 评分变化
            if (latest.rating !== undefined && oldest.rating !== undefined && latest.rating !== oldest.rating) {
              change.rating_change = {
                old: oldest.rating,
                new: latest.rating,
              };
            }

            // 只有有变化的才加入结果
            if (change.price_change || change.bsr_change || change.review_change || change.rating_change) {
              changes.push(change);
            }
          }
        } catch (e) {
          console.error(`获取 ${asin.asin} 历史数据失败:`, e);
        }
      }

      return {
        task_id: params.task_id,
        days,
        total_asins: asins.length,
        changed_count: changes.length,
        changes,
      };
    } catch (error) {
      console.error('历史对比失败:', error);
      return {
        task_id: params.task_id,
        error: `历史对比失败: ${error}`,
      };
    }
  }
);

// ==================== 生成竞品分析报告工具 ====================

export const generateCompetitorReportTool: Tool = createTool(
  'generate_competitor_report',
  '根据当前数据和历史变化生成竞品情报分析报告',
  {
    type: 'object',
    properties: {
      task_id: {
        type: 'number',
        description: '竞品监控任务 ID',
      },
      task_name: {
        type: 'string',
        description: '任务名称',
      },
      marketplace: {
        type: 'string',
        description: 'Amazon 站点代码',
      },
      my_asin: {
        type: 'string',
        description: '我的 ASIN（可选，用于对比）',
      },
      current_data: {
        type: 'object',
        description: '当前竞品数据（来自 fetch_competitors_batch 的结果）',
      },
      history_changes: {
        type: 'object',
        description: '历史变化数据（来自 compare_competitor_history 的结果）',
      },
    },
    required: ['task_id', 'task_name', 'marketplace', 'current_data'],
  },
  async (params) => {
    try {
      const reportDate = new Date().toISOString().split('T')[0];
      const currentData = params.current_data as {
        results: ListingResult[];
        total: number;
        success: number;
      };
      const historyChanges = params.history_changes as {
        changes: any[];
        days?: number;
      } | undefined;

      // 辅助函数
      const truncateTitle = (title: string | undefined, maxLen: number = 28): string => {
        if (!title) return '-';
        return title.length > maxLen ? title.slice(0, maxLen) + '...' : title;
      };

      const formatNumber = (num: number | null | undefined): string => {
        if (num === null || num === undefined) return '-';
        return num.toLocaleString();
      };

      const getAmazonUrl = (asin: string): string => {
        const domains: Record<string, string> = {
          US: 'amazon.com', UK: 'amazon.co.uk', DE: 'amazon.de',
          FR: 'amazon.fr', IT: 'amazon.it', ES: 'amazon.es',
          JP: 'amazon.co.jp', CA: 'amazon.ca', AU: 'amazon.com.au',
        };
        const domain = domains[params.marketplace] || 'amazon.com';
        return `https://www.${domain}/dp/${asin}`;
      };

      // 生成 ASIN 链接
      const asinCell = (asin: string) =>
        `<td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;font-family:monospace;font-size:12px;"><a href="${getAmazonUrl(asin)}" target="_blank" style="color:#0369a1;text-decoration:none;">${asin}</a></td>`;

      // 生成图片单元格
      const imgCell = (url?: string) => url
        ? `<td style="padding:6px;border-bottom:1px solid #e5e7eb;width:50px;"><img src="${url}" style="width:40px;height:40px;object-fit:contain;border-radius:4px;background:#f8fafc;" /></td>`
        : `<td style="padding:6px;border-bottom:1px solid #e5e7eb;width:50px;"><div style="width:40px;height:40px;background:#f1f5f9;border-radius:4px;"></div></td>`;

      // 竞品列表行
      const validResults = currentData.results?.filter(r => !r.error) || [];
      const competitorRows = validResults.map((item) =>
        `<tr>${asinCell(item.asin)}${imgCell(item.image_url)}<td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;color:#374151;">${truncateTitle(item.title)}</td><td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;text-align:right;color:#374151;font-weight:500;">${item.price || '-'}</td><td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;text-align:center;color:#6b7280;">${item.rating || '-'}</td><td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;text-align:right;color:#6b7280;">${formatNumber(item.review_count)}</td><td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;text-align:right;color:#374151;">${item.bsr_rank || '-'}</td></tr>`
      ).join('');

      // 变化行
      const changes = historyChanges?.changes || [];
      const changeRows = changes.map((change: any) => {
        const changeTags: string[] = [];

        if (change.price_change) {
          const color = change.price_change.direction === 'up' ? '#dc2626' : '#059669';
          const icon = change.price_change.direction === 'up' ? '↑' : '↓';
          changeTags.push(`<span style="display:inline-block;padding:2px 8px;background:${change.price_change.direction === 'up' ? '#fef2f2' : '#f0fdf4'};color:${color};border-radius:4px;font-size:12px;margin-right:6px;">价格 ${icon} $${change.price_change.old?.toFixed(2)} → $${change.price_change.new?.toFixed(2)}</span>`);
        }
        if (change.bsr_change) {
          const color = change.bsr_change.direction === 'up' ? '#059669' : '#dc2626';
          const icon = change.bsr_change.direction === 'up' ? '↑' : '↓';
          changeTags.push(`<span style="display:inline-block;padding:2px 8px;background:${change.bsr_change.direction === 'up' ? '#f0fdf4' : '#fef2f2'};color:${color};border-radius:4px;font-size:12px;margin-right:6px;">BSR ${icon} #${formatNumber(change.bsr_change.old)} → #${formatNumber(change.bsr_change.new)}</span>`);
        }
        if (change.review_change && change.review_change.diff > 0) {
          changeTags.push(`<span style="display:inline-block;padding:2px 8px;background:#eff6ff;color:#1d4ed8;border-radius:4px;font-size:12px;margin-right:6px;">评论 +${change.review_change.diff}</span>`);
        }
        if (change.rating_change) {
          const ratingColor = change.rating_change.new > change.rating_change.old ? '#059669' : '#dc2626';
          changeTags.push(`<span style="display:inline-block;padding:2px 8px;background:#fefce8;color:${ratingColor};border-radius:4px;font-size:12px;">评分 ${change.rating_change.old?.toFixed(1)} → ${change.rating_change.new?.toFixed(1)}</span>`);
        }

        return `<tr>${asinCell(change.asin)}<td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;color:#374151;">${truncateTitle(change.title, 24)}</td><td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;">${changeTags.join('')}</td></tr>`;
      }).join('');

      // 生成行动建议
      const actionItems: string[] = [];

      // 价格下降的竞品
      const priceDrops = changes.filter((c: any) => c.price_change?.direction === 'down');
      if (priceDrops.length > 0) {
        actionItems.push(`⚠️ ${priceDrops.length} 个竞品降价，需关注是否跟进调整定价策略`);
      }

      // BSR 快速上升
      const bsrRising = changes.filter((c: any) => c.bsr_change?.direction === 'up');
      if (bsrRising.length > 0) {
        actionItems.push(`📈 ${bsrRising.length} 个竞品 BSR 排名上升，建议分析其运营动作`);
      }

      // 评论快速增长
      const reviewGrowth = changes.filter((c: any) => c.review_change?.diff > 10);
      if (reviewGrowth.length > 0) {
        actionItems.push(`💬 ${reviewGrowth.length} 个竞品评论增长明显，可能在进行促销活动`);
      }

      if (actionItems.length === 0) {
        actionItems.push(`✅ 竞品数据平稳，暂无需要特别关注的变化`);
      }

      const actionList = actionItems.map(item =>
        `<div style="padding:10px 14px;background:#f8fafc;border-left:3px solid #0369a1;margin-bottom:8px;color:#374151;font-size:14px;">${item}</div>`
      ).join('');

      const html = `
<div style="font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;max-width:800px;margin:0 auto;color:#1f2937;line-height:1.5;">
  <div style="border-bottom:2px solid #e5e7eb;padding-bottom:16px;margin-bottom:24px;">
    <h1 style="margin:0 0 4px 0;font-size:22px;font-weight:600;color:#111827;">竞品情报报告</h1>
    <div style="font-size:15px;color:#374151;font-weight:500;">${params.task_name} · ${params.marketplace}</div>
    <div style="font-size:13px;color:#6b7280;margin-top:4px;">报告日期: ${reportDate}${params.my_asin ? ` | 我的 ASIN: ${params.my_asin}` : ''}</div>
  </div>
  <div style="display:flex;gap:12px;margin-bottom:28px;">
    <div style="flex:1;background:#f8fafc;border:1px solid #e5e7eb;border-radius:6px;padding:14px;text-align:center;">
      <div style="font-size:24px;font-weight:600;color:#111827;">${formatNumber(currentData.total || 0)}</div>
      <div style="font-size:12px;color:#6b7280;margin-top:2px;">监控竞品</div>
    </div>
    <div style="flex:1;background:#f8fafc;border:1px solid #e5e7eb;border-radius:6px;padding:14px;text-align:center;">
      <div style="font-size:24px;font-weight:600;color:#059669;">${formatNumber(currentData.success || 0)}</div>
      <div style="font-size:12px;color:#6b7280;margin-top:2px;">成功获取</div>
    </div>
    <div style="flex:1;background:#f8fafc;border:1px solid #e5e7eb;border-radius:6px;padding:14px;text-align:center;">
      <div style="font-size:24px;font-weight:600;color:#f59e0b;">${formatNumber(changes.length)}</div>
      <div style="font-size:12px;color:#6b7280;margin-top:2px;">检测到变化</div>
    </div>
    <div style="flex:1;background:#f8fafc;border:1px solid #e5e7eb;border-radius:6px;padding:14px;text-align:center;">
      <div style="font-size:24px;font-weight:600;color:#dc2626;">${formatNumber((currentData.total || 0) - (currentData.success || 0))}</div>
      <div style="font-size:12px;color:#6b7280;margin-top:2px;">获取失败</div>
    </div>
  </div>
  <div style="margin-bottom:28px;">
    <h2 style="font-size:15px;font-weight:600;color:#111827;margin:0 0 12px 0;">竞品数据一览</h2>
    ${validResults.length > 0 ? `<table style="width:100%;border-collapse:collapse;font-size:13px;"><thead><tr style="background:#f8fafc;"><th style="padding:8px 12px;text-align:left;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">ASIN</th><th style="padding:6px;width:50px;text-align:center;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">图片</th><th style="padding:8px 12px;text-align:left;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">产品标题</th><th style="padding:8px 12px;text-align:right;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">价格</th><th style="padding:8px 12px;text-align:center;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">评分</th><th style="padding:8px 12px;text-align:right;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">评论</th><th style="padding:8px 12px;text-align:right;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">BSR</th></tr></thead><tbody>${competitorRows}</tbody></table>` : '<div style="padding:16px;background:#f8fafc;border-radius:6px;color:#6b7280;font-size:13px;">暂无竞品数据</div>'}
  </div>
  <div style="margin-bottom:28px;">
    <h2 style="font-size:15px;font-weight:600;color:#111827;margin:0 0 12px 0;display:flex;align-items:center;gap:6px;"><span style="color:#f59e0b;">⚡</span> 变化追踪 (近 ${historyChanges?.days || 7} 天)</h2>
    ${changes.length > 0 ? `<table style="width:100%;border-collapse:collapse;font-size:13px;"><thead><tr style="background:#f8fafc;"><th style="padding:8px 12px;text-align:left;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">ASIN</th><th style="padding:8px 12px;text-align:left;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">产品</th><th style="padding:8px 12px;text-align:left;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">变化详情</th></tr></thead><tbody>${changeRows}</tbody></table>` : '<div style="padding:16px;background:#f8fafc;border-radius:6px;color:#6b7280;font-size:13px;">近期暂无显著变化</div>'}
  </div>
  <div style="margin-bottom:20px;">
    <h2 style="font-size:15px;font-weight:600;color:#111827;margin:0 0 12px 0;">行动建议</h2>
    ${actionList}
  </div>
  <div style="border-top:1px solid #e5e7eb;padding-top:12px;text-align:center;font-size:12px;color:#9ca3af;">
    报告由系统自动生成 · ${reportDate}
  </div>
</div>`.trim();

      return {
        success: true,
        task_id: params.task_id,
        report_date: reportDate,
        html,
        summary: {
          total_competitors: currentData.total || 0,
          success_fetched: currentData.success || 0,
          changes_detected: changes.length,
        },
      };
    } catch (error) {
      console.error('生成报告失败:', error);
      return {
        success: false,
        task_id: params.task_id,
        error: `生成报告失败: ${error}`,
      };
    }
  }
);

// ==================== 快速报告生成工具（不依赖数据库） ====================

export const generateQuickReportTool: Tool = createTool(
  'generate_quick_report',
  '根据直接传入的产品数据生成快速分析报告（不依赖数据库任务）',
  {
    type: 'object',
    properties: {
      marketplace: {
        type: 'string',
        description: 'Amazon 站点代码',
      },
      my_asin: {
        type: 'string',
        description: '我的 ASIN（可选，用于对比）',
      },
      products: {
        type: 'array',
        description: '产品数据数组（来自 fetch_competitor_listing 的结果）',
        items: {
          type: 'object',
          description: '单个产品数据对象',
        },
      },
    },
    required: ['marketplace', 'products'],
  },
  async (params) => {
    try {
      const reportDate = new Date().toISOString().split('T')[0];
      const products = params.products as ListingResult[];

      // 辅助函数
      const truncateTitle = (title: string | undefined, maxLen: number = 28): string => {
        if (!title) return '-';
        return title.length > maxLen ? title.slice(0, maxLen) + '...' : title;
      };

      const formatNumber = (num: number | null | undefined): string => {
        if (num === null || num === undefined) return '-';
        return num.toLocaleString();
      };

      const getAmazonUrl = (asin: string): string => {
        const domains: Record<string, string> = {
          US: 'amazon.com', UK: 'amazon.co.uk', DE: 'amazon.de',
          FR: 'amazon.fr', IT: 'amazon.it', ES: 'amazon.es',
          JP: 'amazon.co.jp', CA: 'amazon.ca', AU: 'amazon.com.au',
        };
        const domain = domains[params.marketplace] || 'amazon.com';
        return `https://www.${domain}/dp/${asin}`;
      };

      // 生成 ASIN 链接
      const asinCell = (asin: string, isMyAsin: boolean = false) => {
        const style = isMyAsin
          ? 'color:#059669;text-decoration:none;font-weight:600;'
          : 'color:#0369a1;text-decoration:none;';
        return `<td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;font-family:monospace;font-size:12px;"><a href="${getAmazonUrl(asin)}" target="_blank" style="${style}">${asin}</a>${isMyAsin ? ' <span style="font-size:10px;background:#dcfce7;color:#166534;padding:1px 4px;border-radius:2px;">我的</span>' : ''}</td>`;
      };

      // 生成图片单元格
      const imgCell = (url?: string) => url
        ? `<td style="padding:6px;border-bottom:1px solid #e5e7eb;width:50px;"><img src="${url}" style="width:40px;height:40px;object-fit:contain;border-radius:4px;background:#f8fafc;" /></td>`
        : `<td style="padding:6px;border-bottom:1px solid #e5e7eb;width:50px;"><div style="width:40px;height:40px;background:#f1f5f9;border-radius:4px;"></div></td>`;

      // 过滤有效产品
      const validProducts = products.filter(r => !r.error);
      const failedProducts = products.filter(r => r.error);

      // 竞品列表行
      const productRows = validProducts.map((item) => {
        const isMyAsin = params.my_asin && item.asin === params.my_asin;
        return `<tr style="${isMyAsin ? 'background:#f0fdf4;' : ''}">${asinCell(item.asin, isMyAsin)}${imgCell(item.image_url)}<td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;color:#374151;">${truncateTitle(item.title)}</td><td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;text-align:right;color:#374151;font-weight:500;">${item.price || '-'}</td><td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;text-align:center;color:#6b7280;">${item.rating || '-'}</td><td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;text-align:right;color:#6b7280;">${formatNumber(item.review_count)}</td><td style="padding:8px 12px;border-bottom:1px solid #e5e7eb;text-align:right;color:#374151;">${item.bsr_rank || '-'}</td></tr>`;
      }).join('');

      // 数据统计
      const prices = validProducts
        .map(p => parseFloat(p.price?.replace(/[^0-9.]/g, '') || '0'))
        .filter(p => p > 0);
      const avgPrice = prices.length > 0 ? (prices.reduce((a, b) => a + b, 0) / prices.length).toFixed(2) : '-';
      const minPrice = prices.length > 0 ? Math.min(...prices).toFixed(2) : '-';
      const maxPrice = prices.length > 0 ? Math.max(...prices).toFixed(2) : '-';

      // 我的产品对比分析
      let comparisonSection = '';
      if (params.my_asin) {
        const myProduct = validProducts.find(p => p.asin === params.my_asin);
        const competitors = validProducts.filter(p => p.asin !== params.my_asin);

        if (myProduct && competitors.length > 0) {
          const myPrice = parseFloat(myProduct.price?.replace(/[^0-9.]/g, '') || '0');
          const avgCompPrice = competitors
            .map(p => parseFloat(p.price?.replace(/[^0-9.]/g, '') || '0'))
            .filter(p => p > 0);
          const avgComp = avgCompPrice.length > 0
            ? avgCompPrice.reduce((a, b) => a + b, 0) / avgCompPrice.length
            : 0;

          const priceDiff = avgComp > 0 ? ((myPrice - avgComp) / avgComp * 100).toFixed(1) : '0';
          const priceStatus = parseFloat(priceDiff) > 5 ? '偏高' : parseFloat(priceDiff) < -5 ? '偏低' : '适中';
          const priceColor = parseFloat(priceDiff) > 5 ? '#dc2626' : parseFloat(priceDiff) < -5 ? '#059669' : '#6b7280';

          comparisonSection = `
  <div style="margin-bottom:28px;">
    <h2 style="font-size:15px;font-weight:600;color:#111827;margin:0 0 12px 0;display:flex;align-items:center;gap:6px;"><span style="color:#059669;">📊</span> 我的产品对比</h2>
    <div style="display:grid;grid-template-columns:repeat(3,1fr);gap:12px;">
      <div style="background:#f8fafc;border:1px solid #e5e7eb;border-radius:6px;padding:14px;">
        <div style="font-size:12px;color:#6b7280;margin-bottom:4px;">我的价格</div>
        <div style="font-size:18px;font-weight:600;color:#111827;">${myProduct.price || '-'}</div>
      </div>
      <div style="background:#f8fafc;border:1px solid #e5e7eb;border-radius:6px;padding:14px;">
        <div style="font-size:12px;color:#6b7280;margin-bottom:4px;">竞品均价</div>
        <div style="font-size:18px;font-weight:600;color:#111827;">$${avgComp.toFixed(2)}</div>
      </div>
      <div style="background:#f8fafc;border:1px solid #e5e7eb;border-radius:6px;padding:14px;">
        <div style="font-size:12px;color:#6b7280;margin-bottom:4px;">价格定位</div>
        <div style="font-size:18px;font-weight:600;color:${priceColor};">${priceStatus} (${priceDiff}%)</div>
      </div>
    </div>
  </div>`;
        }
      }

      const html = `
<div style="font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;max-width:800px;margin:0 auto;color:#1f2937;line-height:1.5;">
  <div style="border-bottom:2px solid #e5e7eb;padding-bottom:16px;margin-bottom:24px;">
    <h1 style="margin:0 0 4px 0;font-size:22px;font-weight:600;color:#111827;">竞品快速分析报告</h1>
    <div style="font-size:15px;color:#374151;font-weight:500;">${params.marketplace} 站点</div>
    <div style="font-size:13px;color:#6b7280;margin-top:4px;">报告日期: ${reportDate}${params.my_asin ? ` | 我的 ASIN: ${params.my_asin}` : ''}</div>
  </div>
  <div style="display:flex;gap:12px;margin-bottom:28px;">
    <div style="flex:1;background:#f8fafc;border:1px solid #e5e7eb;border-radius:6px;padding:14px;text-align:center;">
      <div style="font-size:24px;font-weight:600;color:#111827;">${formatNumber(products.length)}</div>
      <div style="font-size:12px;color:#6b7280;margin-top:2px;">分析产品</div>
    </div>
    <div style="flex:1;background:#f8fafc;border:1px solid #e5e7eb;border-radius:6px;padding:14px;text-align:center;">
      <div style="font-size:24px;font-weight:600;color:#059669;">${formatNumber(validProducts.length)}</div>
      <div style="font-size:12px;color:#6b7280;margin-top:2px;">成功获取</div>
    </div>
    <div style="flex:1;background:#f8fafc;border:1px solid #e5e7eb;border-radius:6px;padding:14px;text-align:center;">
      <div style="font-size:24px;font-weight:600;color:#0369a1;">$${avgPrice}</div>
      <div style="font-size:12px;color:#6b7280;margin-top:2px;">平均价格</div>
    </div>
    <div style="flex:1;background:#f8fafc;border:1px solid #e5e7eb;border-radius:6px;padding:14px;text-align:center;">
      <div style="font-size:24px;font-weight:600;color:#6b7280;">$${minPrice} - $${maxPrice}</div>
      <div style="font-size:12px;color:#6b7280;margin-top:2px;">价格区间</div>
    </div>
  </div>
  ${comparisonSection}
  <div style="margin-bottom:28px;">
    <h2 style="font-size:15px;font-weight:600;color:#111827;margin:0 0 12px 0;">产品数据一览</h2>
    ${validProducts.length > 0 ? `<table style="width:100%;border-collapse:collapse;font-size:13px;"><thead><tr style="background:#f8fafc;"><th style="padding:8px 12px;text-align:left;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">ASIN</th><th style="padding:6px;width:50px;text-align:center;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">图片</th><th style="padding:8px 12px;text-align:left;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">产品标题</th><th style="padding:8px 12px;text-align:right;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">价格</th><th style="padding:8px 12px;text-align:center;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">评分</th><th style="padding:8px 12px;text-align:right;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">评论</th><th style="padding:8px 12px;text-align:right;font-weight:500;color:#6b7280;border-bottom:1px solid #e5e7eb;">BSR</th></tr></thead><tbody>${productRows}</tbody></table>` : '<div style="padding:16px;background:#f8fafc;border-radius:6px;color:#6b7280;font-size:13px;">暂无产品数据</div>'}
  </div>
  ${failedProducts.length > 0 ? `
  <div style="margin-bottom:28px;">
    <h2 style="font-size:15px;font-weight:600;color:#dc2626;margin:0 0 12px 0;">获取失败的产品</h2>
    <div style="padding:12px;background:#fef2f2;border:1px solid #fecaca;border-radius:6px;">
      ${failedProducts.map(p => `<div style="font-size:13px;color:#991b1b;margin-bottom:4px;">${p.asin}: ${p.error}</div>`).join('')}
    </div>
  </div>` : ''}
  <div style="border-top:1px solid #e5e7eb;padding-top:12px;text-align:center;font-size:12px;color:#9ca3af;">
    报告由系统自动生成 · ${reportDate}
  </div>
</div>`.trim();

      return {
        success: true,
        report_date: reportDate,
        html,
        summary: {
          total_products: products.length,
          success_fetched: validProducts.length,
          avg_price: avgPrice,
          price_range: `$${minPrice} - $${maxPrice}`,
        },
      };
    } catch (error) {
      console.error('生成快速报告失败:', error);
      return {
        success: false,
        error: `生成报告失败: ${error}`,
      };
    }
  }
);

// ==================== 导出工具集 ====================

export const competitorIntelligenceTools: Tool[] = [
  fetchCompetitorListingTool,
  fetchCompetitorsBatchTool,
  compareCompetitorHistoryTool,
  generateCompetitorReportTool,
  generateQuickReportTool,
];

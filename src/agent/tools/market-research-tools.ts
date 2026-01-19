// 市场调研 Agent 工具集

import { createTool } from '../Tool';
import type { Tool } from '../types';
import { invoke } from '@tauri-apps/api/core';
import {
  generateWeeklyReport,
  buildReportDataFromTools,
  type ToolResults,
} from '../report-template';

// ==================== BSR 数据获取工具 ====================

export const fetchBsrDataTool: Tool = createTool(
  'fetch_bsr_data',
  '获取指定类目的 Best Seller 排名数据（Top 100）',
  {
    type: 'object',
    properties: {
      marketplace: {
        type: 'string',
        description: 'Amazon 站点代码（如 US, UK, DE, JP 等）',
        enum: ['US', 'CA', 'MX', 'BR', 'UK', 'DE', 'FR', 'IT', 'ES', 'NL', 'SE', 'PL', 'JP', 'AU'],
      },
      category_id: {
        type: 'string',
        description: '类目 ID',
      },
      category_name: {
        type: 'string',
        description: '类目名称（可选，用于显示）',
      },
    },
    required: ['marketplace', 'category_id'],
  },
  async (params) => {
    try {
      // 调用 Rust 后端爬取 BSR 数据
      const result = await invoke('fetch_category_bsr', {
        marketplace: params.marketplace,
        categoryId: params.category_id,
      });
      return result;
    } catch (error) {
      // 返回错误信息，不再静默返回模拟数据
      console.error('BSR 数据获取失败:', error);
      return {
        marketplace: params.marketplace,
        category_id: params.category_id,
        products: [],
        snapshot_date: new Date().toISOString(),
        error: `爬虫调用失败: ${error}`,
      };
    }
  }
);

// ==================== 子类目发现工具 ====================

export const discoverSubcategoriesTool: Tool = createTool(
  'discover_subcategories',
  '自动发现指定类目下的所有子类目，返回子类目名称和 ID',
  {
    type: 'object',
    properties: {
      marketplace: {
        type: 'string',
        description: 'Amazon 站点代码（如 US, UK, DE, FR, JP 等）',
        enum: ['US', 'CA', 'MX', 'BR', 'UK', 'DE', 'FR', 'IT', 'ES', 'NL', 'SE', 'PL', 'JP', 'AU'],
      },
      parent_category: {
        type: 'string',
        description: '父类目 ID（如 beauty, electronics, home-garden 等英文类目）',
      },
    },
    required: ['marketplace', 'parent_category'],
  },
  async (params) => {
    try {
      const result = await invoke('discover_subcategories', {
        marketplace: params.marketplace,
        parentCategory: params.parent_category,
      });
      return result;
    } catch (error) {
      console.error('子类目发现失败:', error);
      return {
        marketplace: params.marketplace,
        parent_category: params.parent_category,
        subcategories: [],
        error: `子类目发现失败: ${error}`,
      };
    }
  }
);

// ==================== 历史数据对比工具 ====================

interface BsrSnapshot {
  id: number;
  marketplace: string;
  category_id: string;
  category_name?: string;
  snapshot_date: string;
  products_json: string;
  product_count: number;
  created_at: string;
}

interface BsrProduct {
  rank: number;
  asin?: string;
  title?: string;
  price?: string;
  rating?: number;
  reviews: number;
  image_url?: string;
}

export const compareBsrHistoryTool: Tool = createTool(
  'compare_bsr_history',
  '对比当前 BSR 数据与历史数据，识别排名变化',
  {
    type: 'object',
    properties: {
      marketplace: {
        type: 'string',
        description: 'Amazon 站点代码',
      },
      category_id: {
        type: 'string',
        description: '类目 ID',
      },
      days: {
        type: 'number',
        description: '对比多少天内的数据（默认 7 天）',
      },
    },
    required: ['marketplace', 'category_id'],
  },
  async (params) => {
    try {
      const days = params.days || 7;
      const history = await invoke<BsrSnapshot[]>('get_bsr_history', {
        marketplace: params.marketplace,
        categoryId: params.category_id,
        days,
      });

      if (history.length < 2) {
        return {
          status: 'insufficient_data',
          message: `历史数据不足，当前仅有 ${history.length} 天的数据，需要至少2天的数据进行对比`,
          marketplace: params.marketplace,
          category_id: params.category_id,
          available_snapshots: history.length,
        };
      }

      // 获取最新和最早的快照
      const latestSnapshot = history[0];
      const oldestSnapshot = history[history.length - 1];

      const latestProducts: BsrProduct[] = JSON.parse(latestSnapshot.products_json);
      const oldestProducts: BsrProduct[] = JSON.parse(oldestSnapshot.products_json);

      // 创建 ASIN -> 排名的映射
      const latestRanks = new Map(latestProducts.map(p => [p.asin, p.rank]));
      const oldestRanks = new Map(oldestProducts.map(p => [p.asin, p.rank]));

      // 分析排名变化
      const rankChanges: { asin: string; oldRank: number; newRank: number; change: number; title?: string; image_url?: string }[] = [];
      const newEntries: { asin: string; rank: number; title?: string; image_url?: string }[] = [];
      const droppedOut: { asin: string; oldRank: number; title?: string }[] = [];

      // 检查当前产品
      for (const product of latestProducts) {
        if (!product.asin) continue;
        const oldRank = oldestRanks.get(product.asin);
        if (oldRank !== undefined) {
          const change = oldRank - product.rank; // 正数表示上升
          if (Math.abs(change) >= 5) {
            rankChanges.push({
              asin: product.asin,
              oldRank,
              newRank: product.rank,
              change,
              title: product.title,
              image_url: product.image_url,
            });
          }
        } else {
          newEntries.push({
            asin: product.asin,
            rank: product.rank,
            title: product.title,
            image_url: product.image_url,
          });
        }
      }

      // 检查跌出榜单的产品
      for (const product of oldestProducts) {
        if (!product.asin) continue;
        if (!latestRanks.has(product.asin)) {
          droppedOut.push({
            asin: product.asin,
            oldRank: product.rank,
            title: product.title,
          });
        }
      }

      // 排序：按变化幅度排序
      rankChanges.sort((a, b) => Math.abs(b.change) - Math.abs(a.change));

      return {
        status: 'success',
        marketplace: params.marketplace,
        category_id: params.category_id,
        comparison_period: {
          from: oldestSnapshot.snapshot_date,
          to: latestSnapshot.snapshot_date,
          days: history.length,
        },
        summary: {
          total_products: latestProducts.length,
          significant_rank_changes: rankChanges.length,
          new_entries: newEntries.length,
          dropped_out: droppedOut.length,
        },
        top_rank_improvements: rankChanges.filter(c => c.change > 0).slice(0, 10),
        top_rank_declines: rankChanges.filter(c => c.change < 0).slice(0, 10),
        new_entries: newEntries.slice(0, 10),
        dropped_out: droppedOut.slice(0, 10),
      };
    } catch (error) {
      return {
        status: 'error',
        message: `获取历史数据失败: ${error}`,
        marketplace: params.marketplace,
        category_id: params.category_id,
      };
    }
  }
);

// ==================== 新品识别工具 ====================

export const identifyNewProductsTool: Tool = createTool(
  'identify_new_products',
  '识别本周新进入 Top 100 的产品',
  {
    type: 'object',
    properties: {
      marketplace: {
        type: 'string',
        description: 'Amazon 站点代码',
      },
      category_id: {
        type: 'string',
        description: '类目 ID',
      },
    },
    required: ['marketplace', 'category_id'],
  },
  async (params) => {
    try {
      const history = await invoke<BsrSnapshot[]>('get_bsr_history', {
        marketplace: params.marketplace,
        categoryId: params.category_id,
        days: 7,
      });

      if (history.length < 2) {
        return {
          status: 'insufficient_data',
          message: `历史数据不足，当前仅有 ${history.length} 天的数据`,
          marketplace: params.marketplace,
          category_id: params.category_id,
        };
      }

      const latestSnapshot = history[0];
      const oldestSnapshot = history[history.length - 1];

      const latestProducts: BsrProduct[] = JSON.parse(latestSnapshot.products_json);
      const oldestAsins = new Set(
        JSON.parse(oldestSnapshot.products_json)
          .map((p: BsrProduct) => p.asin)
          .filter(Boolean)
      );

      const newProducts = latestProducts
        .filter(p => p.asin && !oldestAsins.has(p.asin))
        .map(p => ({
          asin: p.asin,
          rank: p.rank,
          title: p.title,
          price: p.price,
          rating: p.rating,
          reviews: p.reviews,
          image_url: p.image_url,
        }));

      return {
        status: 'success',
        marketplace: params.marketplace,
        category_id: params.category_id,
        period: {
          from: oldestSnapshot.snapshot_date,
          to: latestSnapshot.snapshot_date,
        },
        new_products_count: newProducts.length,
        new_products: newProducts,
      };
    } catch (error) {
      return {
        status: 'error',
        message: `识别新品失败: ${error}`,
        marketplace: params.marketplace,
        category_id: params.category_id,
      };
    }
  }
);

// ==================== 价格分析工具 ====================

export const analyzePriceTrendsTool: Tool = createTool(
  'analyze_price_trends',
  '分析类目内产品价格变动趋势',
  {
    type: 'object',
    properties: {
      marketplace: {
        type: 'string',
        description: 'Amazon 站点代码',
      },
      category_id: {
        type: 'string',
        description: '类目 ID',
      },
    },
    required: ['marketplace', 'category_id'],
  },
  async (params) => {
    try {
      const history = await invoke<BsrSnapshot[]>('get_bsr_history', {
        marketplace: params.marketplace,
        categoryId: params.category_id,
        days: 7,
      });

      if (history.length < 2) {
        return {
          status: 'insufficient_data',
          message: `历史数据不足，当前仅有 ${history.length} 天的数据`,
          marketplace: params.marketplace,
          category_id: params.category_id,
        };
      }

      // 解析价格字符串
      const parsePrice = (priceStr?: string): number | null => {
        if (!priceStr) return null;
        const match = priceStr.match(/[\d.]+/);
        return match ? parseFloat(match[0]) : null;
      };

      // 计算每日的价格统计
      const dailyStats = history.map(snapshot => {
        const products: BsrProduct[] = JSON.parse(snapshot.products_json);
        const prices = products
          .map(p => parsePrice(p.price))
          .filter((p): p is number => p !== null);

        if (prices.length === 0) {
          return {
            date: snapshot.snapshot_date,
            avg_price: null,
            min_price: null,
            max_price: null,
            product_count: products.length,
          };
        }

        return {
          date: snapshot.snapshot_date,
          avg_price: prices.reduce((a, b) => a + b, 0) / prices.length,
          min_price: Math.min(...prices),
          max_price: Math.max(...prices),
          product_count: products.length,
        };
      });

      // 计算价格变化趋势
      const latestStats = dailyStats[0];
      const oldestStats = dailyStats[dailyStats.length - 1];

      let priceChange = null;
      let priceChangePercent = null;
      if (latestStats.avg_price && oldestStats.avg_price) {
        priceChange = latestStats.avg_price - oldestStats.avg_price;
        priceChangePercent = (priceChange / oldestStats.avg_price) * 100;
      }

      return {
        status: 'success',
        marketplace: params.marketplace,
        category_id: params.category_id,
        period: {
          from: oldestStats.date,
          to: latestStats.date,
          days: history.length,
        },
        summary: {
          current_avg_price: latestStats.avg_price?.toFixed(2),
          current_min_price: latestStats.min_price?.toFixed(2),
          current_max_price: latestStats.max_price?.toFixed(2),
          price_change: priceChange?.toFixed(2),
          price_change_percent: priceChangePercent?.toFixed(2),
          trend: priceChange !== null ? (priceChange > 0 ? 'increasing' : priceChange < 0 ? 'decreasing' : 'stable') : 'unknown',
        },
        daily_stats: dailyStats,
      };
    } catch (error) {
      return {
        status: 'error',
        message: `价格趋势分析失败: ${error}`,
        marketplace: params.marketplace,
        category_id: params.category_id,
      };
    }
  }
);

// ==================== 生成周报工具（一键生成，自动获取所有数据）====================

export const generateWeeklyReportTool: Tool = createTool(
  'generate_weekly_report',
  '一键生成完整的市场调研周报。返回 report_content 字段包含 HTML 格式报告（以 <div 开头），请直接原样输出该字段内容',
  {
    type: 'object',
    properties: {
      marketplace: {
        type: 'string',
        description: 'Amazon 站点代码（如 US, UK, DE, FR, JP 等）',
        enum: ['US', 'CA', 'MX', 'BR', 'UK', 'DE', 'FR', 'IT', 'ES', 'NL', 'SE', 'PL', 'JP', 'AU'],
      },
      category_id: {
        type: 'string',
        description: '类目 ID',
      },
      category_name: {
        type: 'string',
        description: '类目名称',
      },
    },
    required: ['marketplace', 'category_id', 'category_name'],
  },
  async (params) => {
    const { marketplace, category_id, category_name } = params;

    // 0. 先爬取最新的 BSR 数据（确保有新数据）
    try {
      const bsrResult = await invoke<{ products: BsrProduct[]; error?: string }>('fetch_category_bsr', {
        marketplace,
        categoryId: category_id,
      });
      // 保存爬取结果到数据库
      if (!bsrResult.error && bsrResult.products && bsrResult.products.length > 0) {
        await invoke('save_bsr_snapshot', {
          marketplace,
          categoryId: category_id,
          categoryName: category_name,
          productsJson: JSON.stringify(bsrResult.products),
          productCount: bsrResult.products.length,
        }).catch(() => {}); // 忽略保存错误
      }
    } catch {
      // 爬取失败时使用历史数据
    }

    // 1. 获取 BSR 对比数据（包含刚爬取的新数据）
    let bsrComparison: any = null;
    try {
      const history = await invoke<BsrSnapshot[]>('get_bsr_history', {
        marketplace,
        categoryId: category_id,
        days: 7,
      });

      if (history.length >= 2) {
        const latestSnapshot = history[0];
        const oldestSnapshot = history[history.length - 1];
        const latestProducts: BsrProduct[] = JSON.parse(latestSnapshot.products_json);
        const oldestProducts: BsrProduct[] = JSON.parse(oldestSnapshot.products_json);

        const latestRanks = new Map(latestProducts.map(p => [p.asin, p.rank]));
        const oldestRanks = new Map(oldestProducts.map(p => [p.asin, p.rank]));

        const rankChanges: any[] = [];
        const newEntries: any[] = [];
        const droppedOut: any[] = [];

        for (const product of latestProducts) {
          if (!product.asin) continue;
          const oldRank = oldestRanks.get(product.asin);
          if (oldRank !== undefined) {
            const change = oldRank - product.rank;
            if (Math.abs(change) >= 5) {
              rankChanges.push({
                asin: product.asin,
                oldRank,
                newRank: product.rank,
                change,
                title: product.title,
                image_url: product.image_url,
              });
            }
          } else {
            newEntries.push({
              asin: product.asin,
              rank: product.rank,
              title: product.title,
              image_url: product.image_url,
              price: product.price,
              rating: product.rating,
              reviews: product.reviews,
            });
          }
        }

        for (const product of oldestProducts) {
          if (!product.asin) continue;
          if (!latestRanks.has(product.asin)) {
            droppedOut.push({ asin: product.asin, oldRank: product.rank, title: product.title });
          }
        }

        rankChanges.sort((a, b) => Math.abs(b.change) - Math.abs(a.change));

        bsrComparison = {
          status: 'success',
          comparison_period: { from: oldestSnapshot.snapshot_date, to: latestSnapshot.snapshot_date },
          summary: {
            total_products: latestProducts.length,
            significant_rank_changes: rankChanges.length,
            new_entries: newEntries.length,
            dropped_out: droppedOut.length,
          },
          top_rank_improvements: rankChanges.filter(c => c.change > 0).slice(0, 10),
          top_rank_declines: rankChanges.filter(c => c.change < 0).slice(0, 10),
          new_entries: newEntries.slice(0, 10),
        };
      } else {
        bsrComparison = { status: 'insufficient_data', message: '历史数据不足' };
      }
    } catch (error) {
      bsrComparison = { status: 'error', message: String(error) };
    }

    // 2. 获取新品数据
    let newProductsData: any = null;
    try {
      const history = await invoke<BsrSnapshot[]>('get_bsr_history', {
        marketplace,
        categoryId: category_id,
        days: 7,
      });

      if (history.length >= 2) {
        const latestSnapshot = history[0];
        const oldestSnapshot = history[history.length - 1];
        const latestProducts: BsrProduct[] = JSON.parse(latestSnapshot.products_json);
        const oldestAsins = new Set(
          JSON.parse(oldestSnapshot.products_json).map((p: BsrProduct) => p.asin).filter(Boolean)
        );

        const newProducts = latestProducts
          .filter(p => p.asin && !oldestAsins.has(p.asin))
          .map(p => ({
            asin: p.asin,
            rank: p.rank,
            title: p.title,
            image_url: p.image_url,
            price: p.price,
            rating: p.rating,
            reviews: p.reviews,
          }));

        newProductsData = {
          status: 'success',
          period: { from: oldestSnapshot.snapshot_date, to: latestSnapshot.snapshot_date },
          new_products: newProducts,
        };
      } else {
        newProductsData = { status: 'insufficient_data', new_products: [] };
      }
    } catch (error) {
      newProductsData = { status: 'error', new_products: [] };
    }

    // 3. 获取价格趋势
    let priceTrendsData: any = null;
    try {
      const history = await invoke<BsrSnapshot[]>('get_bsr_history', {
        marketplace,
        categoryId: category_id,
        days: 7,
      });

      if (history.length >= 2) {
        const parsePrice = (priceStr?: string): number | null => {
          if (!priceStr) return null;
          const match = priceStr.match(/[\d.]+/);
          return match ? parseFloat(match[0]) : null;
        };

        const latestSnapshot = history[0];
        const oldestSnapshot = history[history.length - 1];

        const latestProducts: BsrProduct[] = JSON.parse(latestSnapshot.products_json);
        const oldestProducts: BsrProduct[] = JSON.parse(oldestSnapshot.products_json);

        const latestPrices = latestProducts.map(p => parsePrice(p.price)).filter((p): p is number => p !== null);
        const oldestPrices = oldestProducts.map(p => parsePrice(p.price)).filter((p): p is number => p !== null);

        const latestAvg = latestPrices.length > 0 ? latestPrices.reduce((a, b) => a + b, 0) / latestPrices.length : null;
        const oldestAvg = oldestPrices.length > 0 ? oldestPrices.reduce((a, b) => a + b, 0) / oldestPrices.length : null;

        let priceChange = null;
        let priceChangePercent = null;
        let trend = 'unknown';

        if (latestAvg && oldestAvg) {
          priceChange = latestAvg - oldestAvg;
          priceChangePercent = (priceChange / oldestAvg) * 100;
          trend = priceChange > 0 ? 'increasing' : priceChange < 0 ? 'decreasing' : 'stable';
        }

        priceTrendsData = {
          status: 'success',
          summary: {
            current_avg_price: latestAvg?.toFixed(2),
            current_min_price: latestPrices.length > 0 ? Math.min(...latestPrices).toFixed(2) : null,
            current_max_price: latestPrices.length > 0 ? Math.max(...latestPrices).toFixed(2) : null,
            price_change: priceChange?.toFixed(2),
            price_change_percent: priceChangePercent?.toFixed(2),
            trend,
          },
        };
      } else {
        priceTrendsData = { status: 'insufficient_data', summary: {} };
      }
    } catch (error) {
      priceTrendsData = { status: 'error', summary: {} };
    }

    // 4. 使用固定模板生成报告
    const toolResults: ToolResults = {
      bsrComparison,
      newProducts: newProductsData,
      priceTrends: priceTrendsData,
    };

    const reportData = buildReportDataFromTools(marketplace, category_id, category_name, toolResults);
    const reportContent = generateWeeklyReport(reportData);

    // 5. 报告由 AgentTab.vue 保存，这里直接返回
    return {
      success: true,
      report_content: reportContent,
      summary: {
        total_products: reportData.totalProducts,
        significant_changes: reportData.significantChanges,
        new_entries: reportData.newEntries,
        has_enough_data: reportData.hasEnoughData,
      },
    };
  }
);

// ==================== 保存报告工具 ====================

export const saveReportTool: Tool = createTool(
  'save_report',
  '保存生成的周报到数据库',
  {
    type: 'object',
    properties: {
      marketplace: {
        type: 'string',
        description: 'Amazon 站点代码',
      },
      category_id: {
        type: 'string',
        description: '类目 ID',
      },
      report_content: {
        type: 'string',
        description: '周报内容（Markdown 格式）',
      },
    },
    required: ['marketplace', 'category_id', 'report_content'],
  },
  async () => {
    // 报告由 AgentTab.vue 自动保存，此工具仅作兼容
    return { success: true, message: '报告已通过系统自动保存' };
  }
);

// ==================== 导出所有工具 ====================

export const marketResearchTools: Tool[] = [
  fetchBsrDataTool,
  discoverSubcategoriesTool,
  compareBsrHistoryTool,
  identifyNewProductsTool,
  analyzePriceTrendsTool,
  generateWeeklyReportTool,
  saveReportTool,
];


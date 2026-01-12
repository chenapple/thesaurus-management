/**
 * 亚马逊广告搜索词报告 Excel 解析器
 * 支持中英文列名自动映射
 */

import * as XLSX from 'xlsx';
import type { AdSearchTerm } from '../types';

// 亚马逊搜索词报告列名映射（英文 -> 标准字段名）
const COLUMN_MAPPING_EN: Record<string, keyof AdSearchTerm> = {
  // Date
  'date': 'report_date',

  // Portfolio
  'portfolio name': 'portfolio_name',
  'portfolio': 'portfolio_name',

  // Campaign 相关
  'campaign name': 'campaign_name',
  'campaign': 'campaign_name',

  // Ad Group 相关
  'ad group name': 'ad_group_name',
  'ad group': 'ad_group_name',

  // Country/Region
  'country': 'country',
  'country/region': 'country',
  'region': 'country',

  // Targeting 相关
  'targeting': 'targeting',
  'keyword': 'targeting',

  // Match Type
  'match type': 'match_type',

  // Customer Search Term
  'customer search term': 'customer_search_term',
  'search term': 'customer_search_term',
  'query': 'customer_search_term',

  // 指标
  'impressions': 'impressions',
  'clicks': 'clicks',
  'click-thru rate (ctr)': 'ctr',
  'ctr': 'ctr',
  'spend': 'spend',
  'cost': 'spend',
  'sales': 'sales',
  '7 day total sales': 'sales',
  '14 day total sales': 'sales',
  'orders': 'orders',
  '7 day total orders': 'orders',
  '7 day total orders (#)': 'orders',
  '14 day total orders': 'orders',
  'acos': 'acos',
  'total advertising cost of sales (acos)': 'acos',
  'roas': 'roas',
  'total return on advertising spend (roas)': 'roas',
  'conversion rate': 'conversion_rate',
  '7 day conversion rate': 'conversion_rate',
  'cpc': 'cpc',
  'cost per click (cpc)': 'cpc',

  // SKU
  'sku': 'sku',
  'advertised sku': 'sku',
};

// 亚马逊搜索词报告列名映射（中文 -> 标准字段名）
// 基于实际下载的报告列名：日期、广告组合名称、货币、广告活动名称、广告组名称、国家/地区、投放、匹配类型、客户搜索词、展示量、点击量、点击率(CTR)、每次点击成本(CPC)、花费、7天总销售额、广告投入产出比 (ACOS) 总计、总广告投资回报率 (ROAS)、7天总订单数(#)、7天总销售量(#)、7天的转化率、7天内广告SKU销售量(#)、7天内其他SKU销售量(#)、7天内广告SKU销售额、7天内其他SKU销售额
const COLUMN_MAPPING_CN: Record<string, keyof AdSearchTerm> = {
  // 日期
  '日期': 'report_date',

  // Portfolio 相关
  '广告组合名称': 'portfolio_name',

  // Campaign 相关
  '广告活动名称': 'campaign_name',
  '广告系列名称': 'campaign_name',

  // Ad Group 相关
  '广告组名称': 'ad_group_name',

  // 国家/地区
  '国家/地区': 'country',
  '国家': 'country',
  '地区': 'country',

  // Targeting 相关
  '投放': 'targeting',
  '关键词': 'targeting',
  '投放词': 'targeting',

  // Match Type
  '匹配类型': 'match_type',

  // Customer Search Term
  '客户搜索词': 'customer_search_term',
  '搜索词': 'customer_search_term',

  // 指标
  '展示量': 'impressions',
  '曝光量': 'impressions',
  '点击量': 'clicks',
  '点击次数': 'clicks',
  '点击率(ctr)': 'ctr',
  '点击率': 'ctr',
  '每次点击成本(cpc)': 'cpc',
  '单次点击成本': 'cpc',
  'cpc': 'cpc',
  '花费': 'spend',
  '支出': 'spend',
  '成本': 'spend',
  '7天总销售额': 'sales',
  '14天总销售额': 'sales',
  '销售额': 'sales',
  '广告投入产出比 (acos) 总计': 'acos',
  '广告投入产出比(acos)': 'acos',
  'acos': 'acos',
  '广告销售成本': 'acos',
  '总广告投资回报率 (roas)': 'roas',
  '总广告投资回报率(roas)': 'roas',
  'roas': 'roas',
  '广告投资回报率': 'roas',
  '7天总订单数(#)': 'orders',
  '7天总订单数': 'orders',
  '14天总订单数': 'orders',
  '订单': 'orders',
  '订单数': 'orders',
  '7天的转化率': 'conversion_rate',
  '转化率': 'conversion_rate',

  // SKU
  'sku': 'sku',
  '广告sku': 'sku',
};

// 合并映射
const COLUMN_MAPPING = { ...COLUMN_MAPPING_EN, ...COLUMN_MAPPING_CN };

// Match Type 值映射
const MATCH_TYPE_MAPPING: Record<string, string> = {
  'broad': 'broad',
  'phrase': 'phrase',
  'exact': 'exact',
  'auto': 'auto',
  'negative phrase': 'phrase',
  'negative exact': 'exact',
  '广泛': 'broad',
  '词组': 'phrase',
  '精准': 'exact',
  '自动': 'auto',
};

/**
 * 解析 Excel 文件并转换为 AdSearchTerm 数组
 */
export async function parseAdExcel(file: File): Promise<AdSearchTerm[]> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();

    reader.onload = (e) => {
      try {
        const data = new Uint8Array(e.target?.result as ArrayBuffer);
        const workbook = XLSX.read(data, { type: 'array' });

        // 获取第一个 sheet
        const firstSheetName = workbook.SheetNames[0];
        const worksheet = workbook.Sheets[firstSheetName];

        // 转换为 JSON
        const jsonData = XLSX.utils.sheet_to_json(worksheet, { header: 1 }) as any[][];

        if (jsonData.length < 2) {
          reject(new Error('Excel 文件为空或只有表头'));
          return;
        }

        // 获取表头行并建立列映射
        const headers = jsonData[0] as string[];
        const columnIndexMap = buildColumnIndexMap(headers);

        // 验证必要列是否存在
        const requiredColumns: (keyof AdSearchTerm)[] = [
          'customer_search_term',
          'spend'
        ];

        for (const col of requiredColumns) {
          if (columnIndexMap[col] === undefined) {
            reject(new Error(`缺少必要列: ${col}。请检查 Excel 文件格式是否正确。`));
            return;
          }
        }

        // 解析数据行
        const searchTerms: AdSearchTerm[] = [];

        for (let i = 1; i < jsonData.length; i++) {
          const row = jsonData[i];
          if (!row || row.length === 0) continue;

          const searchTerm = parseRow(row, columnIndexMap);
          if (searchTerm) {
            searchTerms.push(searchTerm);
          }
        }

        resolve(searchTerms);
      } catch (error) {
        reject(error);
      }
    };

    reader.onerror = () => {
      reject(new Error('读取文件失败'));
    };

    reader.readAsArrayBuffer(file);
  });
}

/**
 * 根据表头建立列索引映射
 */
function buildColumnIndexMap(headers: string[]): Partial<Record<keyof AdSearchTerm, number>> {
  const indexMap: Partial<Record<keyof AdSearchTerm, number>> = {};

  headers.forEach((header, index) => {
    if (!header) return;

    const normalizedHeader = header.toString().toLowerCase().trim();
    const fieldName = COLUMN_MAPPING[normalizedHeader];

    if (fieldName && indexMap[fieldName] === undefined) {
      indexMap[fieldName] = index;
    }
  });

  return indexMap;
}

/**
 * 解析单行数据
 */
function parseRow(
  row: any[],
  columnIndexMap: Partial<Record<keyof AdSearchTerm, number>>
): AdSearchTerm | null {
  const getValue = (field: keyof AdSearchTerm): any => {
    const index = columnIndexMap[field];
    return index !== undefined ? row[index] : undefined;
  };

  const customerSearchTerm = getValue('customer_search_term');
  if (!customerSearchTerm) return null;

  // 解析数值字段
  const parseNumber = (value: any): number => {
    if (value === undefined || value === null || value === '') return 0;
    if (typeof value === 'number') return value;
    // 处理货币格式 "$1,234.56" -> 1234.56
    const cleaned = value.toString().replace(/[$,￥]/g, '').trim();
    const num = parseFloat(cleaned);
    return isNaN(num) ? 0 : num;
  };

  // 解析百分比字段
  const parsePercent = (value: any): number => {
    if (value === undefined || value === null || value === '') return 0;
    if (typeof value === 'number') {
      // Excel 中百分比可能已经是小数（0.15 表示 15%）
      return value > 1 ? value : value * 100;
    }
    // 处理 "15%" 或 "15.5%" 格式
    const cleaned = value.toString().replace('%', '').trim();
    const num = parseFloat(cleaned);
    return isNaN(num) ? 0 : num;
  };

  // 解析匹配类型
  const parseMatchType = (value: any): 'broad' | 'phrase' | 'exact' | 'auto' => {
    if (!value) return 'auto';
    const normalized = value.toString().toLowerCase().trim();
    return (MATCH_TYPE_MAPPING[normalized] as any) || 'auto';
  };

  const spend = parseNumber(getValue('spend'));
  const sales = parseNumber(getValue('sales'));
  const clicks = parseNumber(getValue('clicks'));
  const orders = parseNumber(getValue('orders'));
  const impressions = parseNumber(getValue('impressions'));

  // 计算派生指标（如果报表中没有）
  let acos = parsePercent(getValue('acos'));
  if (acos === 0 && sales > 0) {
    acos = (spend / sales) * 100;
  }

  let roas = parseNumber(getValue('roas'));
  if (roas === 0 && spend > 0) {
    roas = sales / spend;
  }

  let ctr = parsePercent(getValue('ctr'));
  if (ctr === 0 && impressions > 0) {
    ctr = (clicks / impressions) * 100;
  }

  let conversionRate = parsePercent(getValue('conversion_rate'));
  if (conversionRate === 0 && clicks > 0) {
    conversionRate = (orders / clicks) * 100;
  }

  let cpc = parseNumber(getValue('cpc'));
  if (cpc === 0 && clicks > 0) {
    cpc = spend / clicks;
  }

  return {
    id: 0, // 会在导入时由数据库分配
    project_id: 0, // 会在导入时设置
    portfolio_name: getValue('portfolio_name')?.toString() || null,
    campaign_name: getValue('campaign_name')?.toString() || null,
    ad_group_name: getValue('ad_group_name')?.toString() || null,
    country: getValue('country')?.toString() || null,
    targeting: getValue('targeting')?.toString() || null,
    match_type: parseMatchType(getValue('match_type')),
    customer_search_term: customerSearchTerm.toString(),
    impressions: Math.round(impressions),
    clicks: Math.round(clicks),
    ctr: parseFloat(ctr.toFixed(2)),
    spend: parseFloat(spend.toFixed(2)),
    sales: parseFloat(sales.toFixed(2)),
    orders: Math.round(orders),
    acos: parseFloat(acos.toFixed(2)),
    roas: parseFloat(roas.toFixed(2)),
    conversion_rate: parseFloat(conversionRate.toFixed(2)),
    cpc: parseFloat(cpc.toFixed(2)),
    report_date: getValue('report_date')?.toString() || null,
    sku: getValue('sku')?.toString() || null,
  };
}

/**
 * 验证解析结果
 */
export function validateParseResult(searchTerms: AdSearchTerm[]): {
  valid: boolean;
  errors: string[];
  warnings: string[];
  stats: {
    total: number;
    withSpend: number;
    withSales: number;
    withOrders: number;
  };
} {
  const errors: string[] = [];
  const warnings: string[] = [];

  if (searchTerms.length === 0) {
    errors.push('没有解析到任何搜索词数据');
  }

  const withSpend = searchTerms.filter(t => t.spend > 0).length;
  const withSales = searchTerms.filter(t => t.sales > 0).length;
  const withOrders = searchTerms.filter(t => t.orders > 0).length;

  if (withSpend === 0) {
    warnings.push('所有搜索词的花费都为 0，请检查数据是否正确');
  }

  if (withSales === 0) {
    warnings.push('所有搜索词的销售额都为 0');
  }

  // 检查异常值
  const highAcos = searchTerms.filter(t => t.acos > 500).length;
  if (highAcos > 0) {
    warnings.push(`有 ${highAcos} 个搜索词的 ACOS 超过 500%`);
  }

  return {
    valid: errors.length === 0,
    errors,
    warnings,
    stats: {
      total: searchTerms.length,
      withSpend,
      withSales,
      withOrders,
    },
  };
}

/**
 * 获取支持的文件类型
 */
export function getSupportedFileTypes(): string[] {
  return ['.xlsx', '.xls', '.csv'];
}

/**
 * 检查文件类型是否支持
 */
export function isSupportedFileType(fileName: string): boolean {
  const ext = fileName.toLowerCase().split('.').pop();
  return ['xlsx', 'xls', 'csv'].includes(ext || '');
}

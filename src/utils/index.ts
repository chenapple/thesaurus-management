/**
 * 工具函数集合
 */

/**
 * 根据阈值计算流量级别
 */
export function getTrafficLevel(
  rank: number | null | undefined,
  bigThreshold: number,
  mediumThreshold: number
): string | null {
  if (rank === null || rank === undefined) return null;
  if (rank <= bigThreshold) return "大词";
  if (rank <= mediumThreshold) return "中词";
  return "小词";
}

/**
 * 格式化数字为带千分位的字符串
 */
export function formatNumber(num: number | null | undefined): string {
  if (num === null || num === undefined) return "-";
  return num.toLocaleString("zh-CN");
}

/**
 * 格式化百分比
 */
export function formatPercent(value: number | null | undefined, decimals: number = 2): string {
  if (value === null || value === undefined) return "-";
  return `${(value * 100).toFixed(decimals)}%`;
}

/**
 * 格式化日期时间
 */
export function formatDateTime(dateStr: string | null | undefined): string {
  if (!dateStr) return "-";
  try {
    const date = new Date(dateStr);
    return date.toLocaleString("zh-CN", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return dateStr;
  }
}

/**
 * 截断文本并添加省略号
 */
export function truncate(text: string | null | undefined, maxLength: number): string {
  if (!text) return "";
  if (text.length <= maxLength) return text;
  return text.slice(0, maxLength) + "...";
}

/**
 * 解析关键词（按行分割，去除空行和重复）
 */
export function parseKeywords(text: string): string[] {
  const lines = text.split(/[\r\n]+/);
  const keywords = lines
    .map(line => line.trim())
    .filter(line => line.length > 0);
  return [...new Set(keywords)]; // 去重
}

/**
 * 验证是否为有效的 Excel 文件名
 */
export function isValidExcelFile(filename: string): boolean {
  const ext = filename.toLowerCase().split(".").pop();
  return ext === "xlsx" || ext === "xls";
}

/**
 * 生成默认导出文件名
 */
export function generateExportFilename(productName: string, suffix: string = ""): string {
  const date = new Date().toISOString().slice(0, 10);
  const safeName = productName.replace(/[<>:"/\\|?*]/g, "_");
  return `${safeName}${suffix}_${date}.xlsx`;
}

/**
 * 深度克隆对象
 */
export function deepClone<T>(obj: T): T {
  return JSON.parse(JSON.stringify(obj));
}

/**
 * 防抖函数
 */
export function debounce<T extends (...args: any[]) => any>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout> | null = null;
  return (...args: Parameters<T>) => {
    if (timeoutId) clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  };
}

/**
 * 判断两个数组是否相等（浅比较）
 */
export function arraysEqual<T>(a: T[], b: T[]): boolean {
  if (a.length !== b.length) return false;
  return a.every((val, index) => val === b[index]);
}

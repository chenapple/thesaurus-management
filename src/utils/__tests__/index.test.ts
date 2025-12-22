import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  getTrafficLevel,
  formatNumber,
  formatPercent,
  formatDateTime,
  truncate,
  parseKeywords,
  isValidExcelFile,
  generateExportFilename,
  deepClone,
  debounce,
  arraysEqual,
} from '../index';

describe('getTrafficLevel', () => {
  const bigThreshold = 500;
  const mediumThreshold = 5000;

  it('排名小于等于大词阈值时返回"大词"', () => {
    expect(getTrafficLevel(100, bigThreshold, mediumThreshold)).toBe('大词');
    expect(getTrafficLevel(500, bigThreshold, mediumThreshold)).toBe('大词');
  });

  it('排名在两个阈值之间时返回"中词"', () => {
    expect(getTrafficLevel(501, bigThreshold, mediumThreshold)).toBe('中词');
    expect(getTrafficLevel(3000, bigThreshold, mediumThreshold)).toBe('中词');
    expect(getTrafficLevel(5000, bigThreshold, mediumThreshold)).toBe('中词');
  });

  it('排名大于中词阈值时返回"小词"', () => {
    expect(getTrafficLevel(5001, bigThreshold, mediumThreshold)).toBe('小词');
    expect(getTrafficLevel(100000, bigThreshold, mediumThreshold)).toBe('小词');
  });

  it('null 或 undefined 返回 null', () => {
    expect(getTrafficLevel(null, bigThreshold, mediumThreshold)).toBeNull();
    expect(getTrafficLevel(undefined, bigThreshold, mediumThreshold)).toBeNull();
  });
});

describe('formatNumber', () => {
  it('应该格式化数字为带千分位的字符串', () => {
    expect(formatNumber(1000)).toBe('1,000');
    expect(formatNumber(1000000)).toBe('1,000,000');
  });

  it('小数应该保持原样', () => {
    expect(formatNumber(1234.56)).toBe('1,234.56');
  });

  it('null 或 undefined 返回 "-"', () => {
    expect(formatNumber(null)).toBe('-');
    expect(formatNumber(undefined)).toBe('-');
  });

  it('0 应该返回 "0"', () => {
    expect(formatNumber(0)).toBe('0');
  });
});

describe('formatPercent', () => {
  it('应该格式化为百分比', () => {
    expect(formatPercent(0.5)).toBe('50.00%');
    expect(formatPercent(0.123)).toBe('12.30%');
    expect(formatPercent(1)).toBe('100.00%');
  });

  it('应该支持自定义小数位数', () => {
    expect(formatPercent(0.12345, 1)).toBe('12.3%');
    expect(formatPercent(0.12345, 3)).toBe('12.345%');
  });

  it('null 或 undefined 返回 "-"', () => {
    expect(formatPercent(null)).toBe('-');
    expect(formatPercent(undefined)).toBe('-');
  });
});

describe('formatDateTime', () => {
  it('应该格式化日期时间字符串', () => {
    const result = formatDateTime('2024-12-22T10:30:00');
    expect(result).toContain('2024');
    expect(result).toContain('12');
    expect(result).toContain('22');
  });

  it('空值返回 "-"', () => {
    expect(formatDateTime(null)).toBe('-');
    expect(formatDateTime(undefined)).toBe('-');
    expect(formatDateTime('')).toBe('-');
  });
});

describe('truncate', () => {
  it('短于最大长度时返回原文', () => {
    expect(truncate('hello', 10)).toBe('hello');
  });

  it('长于最大长度时截断并加省略号', () => {
    expect(truncate('hello world', 5)).toBe('hello...');
  });

  it('正好等于最大长度时返回原文', () => {
    expect(truncate('hello', 5)).toBe('hello');
  });

  it('空值返回空字符串', () => {
    expect(truncate(null, 10)).toBe('');
    expect(truncate(undefined, 10)).toBe('');
    expect(truncate('', 10)).toBe('');
  });
});

describe('parseKeywords', () => {
  it('应该按行分割关键词', () => {
    const result = parseKeywords('apple\nbanana\norange');
    expect(result).toEqual(['apple', 'banana', 'orange']);
  });

  it('应该去除空行', () => {
    const result = parseKeywords('apple\n\nbanana\n\n\norange');
    expect(result).toEqual(['apple', 'banana', 'orange']);
  });

  it('应该去除重复', () => {
    const result = parseKeywords('apple\nbanana\napple\norange\nbanana');
    expect(result).toEqual(['apple', 'banana', 'orange']);
  });

  it('应该去除首尾空格', () => {
    const result = parseKeywords('  apple  \n  banana  ');
    expect(result).toEqual(['apple', 'banana']);
  });

  it('支持 Windows 换行符', () => {
    const result = parseKeywords('apple\r\nbanana\r\norange');
    expect(result).toEqual(['apple', 'banana', 'orange']);
  });

  it('空字符串返回空数组', () => {
    expect(parseKeywords('')).toEqual([]);
    expect(parseKeywords('   ')).toEqual([]);
  });
});

describe('isValidExcelFile', () => {
  it('xlsx 文件返回 true', () => {
    expect(isValidExcelFile('data.xlsx')).toBe(true);
    expect(isValidExcelFile('DATA.XLSX')).toBe(true);
  });

  it('xls 文件返回 true', () => {
    expect(isValidExcelFile('data.xls')).toBe(true);
    expect(isValidExcelFile('DATA.XLS')).toBe(true);
  });

  it('其他格式返回 false', () => {
    expect(isValidExcelFile('data.csv')).toBe(false);
    expect(isValidExcelFile('data.txt')).toBe(false);
    expect(isValidExcelFile('data.pdf')).toBe(false);
  });

  it('无扩展名返回 false', () => {
    expect(isValidExcelFile('data')).toBe(false);
  });
});

describe('generateExportFilename', () => {
  it('应该生成正确格式的文件名', () => {
    const filename = generateExportFilename('测试产品');
    expect(filename).toMatch(/^测试产品_\d{4}-\d{2}-\d{2}\.xlsx$/);
  });

  it('应该支持后缀', () => {
    const filename = generateExportFilename('产品', '_导出');
    expect(filename).toMatch(/^产品_导出_\d{4}-\d{2}-\d{2}\.xlsx$/);
  });

  it('应该替换非法字符', () => {
    const filename = generateExportFilename('产品<test>');
    expect(filename).not.toContain('<');
    expect(filename).not.toContain('>');
  });
});

describe('deepClone', () => {
  it('应该深度克隆对象', () => {
    const original = { a: 1, b: { c: 2 } };
    const cloned = deepClone(original);

    expect(cloned).toEqual(original);
    expect(cloned).not.toBe(original);
    expect(cloned.b).not.toBe(original.b);
  });

  it('应该深度克隆数组', () => {
    const original = [1, [2, 3], { a: 4 }];
    const cloned = deepClone(original);

    expect(cloned).toEqual(original);
    expect(cloned).not.toBe(original);
    expect(cloned[1]).not.toBe(original[1]);
  });

  it('修改克隆不应影响原对象', () => {
    const original = { a: 1, b: { c: 2 } };
    const cloned = deepClone(original);

    cloned.b.c = 999;
    expect(original.b.c).toBe(2);
  });
});

describe('debounce', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('应该延迟执行函数', () => {
    const fn = vi.fn();
    const debouncedFn = debounce(fn, 100);

    debouncedFn();
    expect(fn).not.toHaveBeenCalled();

    vi.advanceTimersByTime(100);
    expect(fn).toHaveBeenCalledTimes(1);
  });

  it('多次调用只执行最后一次', () => {
    const fn = vi.fn();
    const debouncedFn = debounce(fn, 100);

    debouncedFn();
    debouncedFn();
    debouncedFn();

    vi.advanceTimersByTime(100);
    expect(fn).toHaveBeenCalledTimes(1);
  });

  it('应该传递正确的参数', () => {
    const fn = vi.fn();
    const debouncedFn = debounce(fn, 100);

    debouncedFn('arg1', 'arg2');
    vi.advanceTimersByTime(100);

    expect(fn).toHaveBeenCalledWith('arg1', 'arg2');
  });
});

describe('arraysEqual', () => {
  it('相同数组返回 true', () => {
    expect(arraysEqual([1, 2, 3], [1, 2, 3])).toBe(true);
    expect(arraysEqual(['a', 'b'], ['a', 'b'])).toBe(true);
    expect(arraysEqual([], [])).toBe(true);
  });

  it('不同长度返回 false', () => {
    expect(arraysEqual([1, 2], [1, 2, 3])).toBe(false);
  });

  it('不同内容返回 false', () => {
    expect(arraysEqual([1, 2, 3], [1, 2, 4])).toBe(false);
  });

  it('顺序不同返回 false', () => {
    expect(arraysEqual([1, 2, 3], [3, 2, 1])).toBe(false);
  });
});

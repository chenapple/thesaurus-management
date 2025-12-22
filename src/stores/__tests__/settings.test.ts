import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useSettingsStore, columnDefinitions } from '../settings';

// Mock localStorage
const localStorageMock = {
  store: {} as Record<string, string>,
  getItem: vi.fn((key: string) => localStorageMock.store[key] || null),
  setItem: vi.fn((key: string, value: string) => {
    localStorageMock.store[key] = value;
  }),
  removeItem: vi.fn((key: string) => {
    delete localStorageMock.store[key];
  }),
  clear: vi.fn(() => {
    localStorageMock.store = {};
  }),
};

Object.defineProperty(window, 'localStorage', {
  value: localStorageMock,
});

// Mock matchMedia
Object.defineProperty(window, 'matchMedia', {
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

describe('Settings Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorageMock.clear();
    vi.clearAllMocks();
  });

  describe('columnDefinitions', () => {
    it('应该包含关键词列作为必需列', () => {
      const keywordCol = columnDefinitions.find(c => c.key === 'keyword');
      expect(keywordCol).toBeDefined();
      expect(keywordCol?.required).toBe(true);
    });

    it('应该有正确数量的列定义', () => {
      expect(columnDefinitions.length).toBeGreaterThan(10);
    });

    it('每个列定义应该有 key 和 label', () => {
      for (const col of columnDefinitions) {
        expect(col.key).toBeDefined();
        expect(col.label).toBeDefined();
        expect(typeof col.key).toBe('string');
        expect(typeof col.label).toBe('string');
      }
    });
  });

  describe('isDarkMode', () => {
    it('初始值应该为 false（当系统不是深色模式时）', () => {
      const store = useSettingsStore();
      expect(store.isDarkMode).toBe(false);
    });

    it('toggleDarkMode 应该切换主题', () => {
      const store = useSettingsStore();
      expect(store.isDarkMode).toBe(false);

      store.toggleDarkMode();
      expect(store.isDarkMode).toBe(true);

      store.toggleDarkMode();
      expect(store.isDarkMode).toBe(false);
    });

    it('setDarkMode 应该设置指定值', () => {
      const store = useSettingsStore();

      store.setDarkMode(true);
      expect(store.isDarkMode).toBe(true);

      store.setDarkMode(false);
      expect(store.isDarkMode).toBe(false);
    });

    it('主题变化应该保存到 localStorage', async () => {
      const store = useSettingsStore();
      store.setDarkMode(true);

      // Vue 的 watch 是异步的，需要等待
      await new Promise(resolve => setTimeout(resolve, 0));

      expect(localStorageMock.setItem).toHaveBeenCalledWith('darkMode', 'true');
    });
  });

  describe('columnConfig', () => {
    it('默认应该启用必需列', () => {
      const store = useSettingsStore();
      expect(store.columnConfig.keyword).toBe(true);
    });

    it('默认应该启用 default: true 的列', () => {
      const store = useSettingsStore();
      const defaultCols = columnDefinitions.filter(c => c.default);

      for (const col of defaultCols) {
        expect(store.columnConfig[col.key]).toBe(true);
      }
    });

    it('updateColumnConfig 应该更新指定列', () => {
      const store = useSettingsStore();

      store.updateColumnConfig('translation', false);
      expect(store.columnConfig.translation).toBe(false);

      store.updateColumnConfig('translation', true);
      expect(store.columnConfig.translation).toBe(true);
    });

    it('setAllColumns(true) 应该启用所有列', () => {
      const store = useSettingsStore();
      store.setAllColumns(true);

      for (const col of columnDefinitions) {
        expect(store.columnConfig[col.key]).toBe(true);
      }
    });

    it('setAllColumns(false) 应该禁用非必需列', () => {
      const store = useSettingsStore();
      store.setAllColumns(false);

      for (const col of columnDefinitions) {
        if (col.required) {
          expect(store.columnConfig[col.key]).toBe(true);
        } else {
          expect(store.columnConfig[col.key]).toBe(false);
        }
      }
    });

    it('resetColumnConfig 应该恢复默认值', () => {
      const store = useSettingsStore();

      // 先修改一些值
      store.setAllColumns(false);

      // 重置
      store.resetColumnConfig();

      // 检查是否恢复默认
      for (const col of columnDefinitions) {
        const expected = col.required || col.default || false;
        expect(store.columnConfig[col.key]).toBe(expected);
      }
    });
  });

  describe('viewMode', () => {
    it('默认应该是 keywords', () => {
      const store = useSettingsStore();
      expect(store.viewMode).toBe('keywords');
    });

    it('setViewMode 应该切换视图模式', () => {
      const store = useSettingsStore();

      store.setViewMode('roots');
      expect(store.viewMode).toBe('roots');

      store.setViewMode('wordcloud');
      expect(store.viewMode).toBe('wordcloud');

      store.setViewMode('keywords');
      expect(store.viewMode).toBe('keywords');
    });
  });

  describe('sidebarWidth', () => {
    it('默认应该是 240', () => {
      const store = useSettingsStore();
      expect(store.sidebarWidth).toBe(240);
    });

    it('setSidebarWidth 应该限制在最小最大值之间', () => {
      const store = useSettingsStore();

      // 设置为小于最小值
      store.setSidebarWidth(100);
      expect(store.sidebarWidth).toBe(store.MIN_SIDEBAR_WIDTH);

      // 设置为大于最大值
      store.setSidebarWidth(500);
      expect(store.sidebarWidth).toBe(store.MAX_SIDEBAR_WIDTH);

      // 设置正常值
      store.setSidebarWidth(300);
      expect(store.sidebarWidth).toBe(300);
    });
  });
});

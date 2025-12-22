import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useProductStore, countryOptions, amazonDomains } from '../product';

// Mock API
vi.mock('../../api', () => ({
  getProducts: vi.fn(),
  createProduct: vi.fn(),
  updateProduct: vi.fn(),
  deleteProduct: vi.fn(),
  getStats: vi.fn(),
  getWorkflowStatus: vi.fn(),
  getBackups: vi.fn(),
  createBackup: vi.fn(),
  restoreBackup: vi.fn(),
  deleteBackup: vi.fn(),
  clearProductData: vi.fn(),
}));

import * as api from '../../api';

describe('Product Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  describe('countryOptions', () => {
    it('应该包含 6 个国家选项', () => {
      expect(countryOptions.length).toBe(6);
    });

    it('每个国家应该有 code, name, flag', () => {
      for (const country of countryOptions) {
        expect(country.code).toBeDefined();
        expect(country.name).toBeDefined();
        expect(country.flag).toBeDefined();
        expect(country.flag).toContain('<svg');
      }
    });

    it('应该包含 US, UK, DE, FR, IT, ES', () => {
      const codes = countryOptions.map(c => c.code);
      expect(codes).toContain('US');
      expect(codes).toContain('UK');
      expect(codes).toContain('DE');
      expect(codes).toContain('FR');
      expect(codes).toContain('IT');
      expect(codes).toContain('ES');
    });
  });

  describe('amazonDomains', () => {
    it('应该有正确的 Amazon 域名映射', () => {
      expect(amazonDomains.US).toBe('www.amazon.com');
      expect(amazonDomains.UK).toBe('www.amazon.co.uk');
      expect(amazonDomains.DE).toBe('www.amazon.de');
      expect(amazonDomains.FR).toBe('www.amazon.fr');
      expect(amazonDomains.IT).toBe('www.amazon.it');
      expect(amazonDomains.ES).toBe('www.amazon.es');
    });
  });

  describe('初始状态', () => {
    it('products 应该为空数组', () => {
      const store = useProductStore();
      expect(store.products).toEqual([]);
    });

    it('selectedProduct 应该为 null', () => {
      const store = useProductStore();
      expect(store.selectedProduct).toBeNull();
    });

    it('hasSelectedProduct 应该为 false', () => {
      const store = useProductStore();
      expect(store.hasSelectedProduct).toBe(false);
    });

    it('stats 应该是默认值', () => {
      const store = useProductStore();
      expect(store.stats).toEqual({ keywordCount: 0, rootCount: 0 });
    });

    it('workflowStatus 应该全部为 false', () => {
      const store = useProductStore();
      expect(store.workflowStatus).toEqual({
        has_data: false,
        has_traffic_level: false,
        has_category: false,
        has_phrase_tag: false,
        has_orderliness: false,
      });
    });
  });

  describe('loadProducts', () => {
    it('应该调用 API 并更新 products', async () => {
      const mockProducts = [
        { id: 1, name: '产品1', country: 'US' },
        { id: 2, name: '产品2', country: 'UK' },
      ];
      vi.mocked(api.getProducts).mockResolvedValue(mockProducts as any);

      const store = useProductStore();
      await store.loadProducts();

      expect(api.getProducts).toHaveBeenCalled();
      expect(store.products).toEqual(mockProducts);
    });

    it('加载过程中 loading 应该为 true', async () => {
      vi.mocked(api.getProducts).mockImplementation(() => new Promise(resolve => {
        setTimeout(() => resolve([]), 100);
      }));

      const store = useProductStore();
      const promise = store.loadProducts();

      expect(store.loading).toBe(true);
      await promise;
      expect(store.loading).toBe(false);
    });
  });

  describe('selectProduct', () => {
    it('选择产品时应该加载统计和工作流状态', async () => {
      vi.mocked(api.getStats).mockResolvedValue([100, 50]);
      vi.mocked(api.getWorkflowStatus).mockResolvedValue({
        has_data: true,
        has_traffic_level: false,
        has_category: false,
        has_phrase_tag: false,
        has_orderliness: false,
      });

      const store = useProductStore();
      const product = { id: 1, name: '测试产品', country: 'US' } as any;

      await store.selectProduct(product);

      expect(store.selectedProduct).toEqual(product);
      expect(store.hasSelectedProduct).toBe(true);
      expect(api.getStats).toHaveBeenCalledWith(1);
      expect(api.getWorkflowStatus).toHaveBeenCalledWith(1);
      expect(store.stats).toEqual({ keywordCount: 100, rootCount: 50 });
    });

    it('取消选择时应该重置状态', async () => {
      const store = useProductStore();
      store.selectedProduct = { id: 1, name: '测试' } as any;
      store.stats = { keywordCount: 100, rootCount: 50 };

      await store.selectProduct(null);

      expect(store.selectedProduct).toBeNull();
      expect(store.hasSelectedProduct).toBe(false);
      expect(store.stats).toEqual({ keywordCount: 0, rootCount: 0 });
    });
  });

  describe('getCountryName', () => {
    it('应该返回正确的国家名称', () => {
      const store = useProductStore();
      expect(store.getCountryName('US')).toBe('美国');
      expect(store.getCountryName('UK')).toBe('英国');
      expect(store.getCountryName('DE')).toBe('德国');
    });

    it('空值应该返回空字符串', () => {
      const store = useProductStore();
      expect(store.getCountryName(null)).toBe('');
      expect(store.getCountryName('')).toBe('');
    });

    it('未知代码应该返回代码本身', () => {
      const store = useProductStore();
      expect(store.getCountryName('XX')).toBe('XX');
    });
  });

  describe('getCountryFlag', () => {
    it('应该返回 SVG 字符串', () => {
      const store = useProductStore();
      const flag = store.getCountryFlag('US');
      expect(flag).toContain('<svg');
      expect(flag).toContain('</svg>');
    });

    it('空值应该返回空字符串', () => {
      const store = useProductStore();
      expect(store.getCountryFlag(null)).toBe('');
    });
  });

  describe('getAmazonDomain', () => {
    it('有选中产品时应该返回正确域名', () => {
      const store = useProductStore();
      store.selectedProduct = { id: 1, name: '测试', country: 'US' } as any;

      expect(store.getAmazonDomain()).toBe('www.amazon.com');
    });

    it('无选中产品时应该返回 null', () => {
      const store = useProductStore();
      expect(store.getAmazonDomain()).toBeNull();
    });

    it('产品无国家时应该返回 null', () => {
      const store = useProductStore();
      store.selectedProduct = { id: 1, name: '测试', country: null } as any;

      expect(store.getAmazonDomain()).toBeNull();
    });
  });

  describe('createProduct', () => {
    it('应该调用 API 并刷新产品列表', async () => {
      vi.mocked(api.createProduct).mockResolvedValue(1);
      vi.mocked(api.getProducts).mockResolvedValue([]);

      const store = useProductStore();
      const id = await store.createProduct('新产品', 'US');

      expect(api.createProduct).toHaveBeenCalledWith('新产品', 'US');
      expect(api.getProducts).toHaveBeenCalled();
      expect(id).toBe(1);
    });
  });

  describe('deleteProduct', () => {
    it('删除选中产品时应该清除选中状态', async () => {
      vi.mocked(api.deleteProduct).mockResolvedValue(undefined);
      vi.mocked(api.getProducts).mockResolvedValue([]);

      const store = useProductStore();
      store.selectedProduct = { id: 1, name: '测试' } as any;

      await store.deleteProduct(1);

      expect(api.deleteProduct).toHaveBeenCalledWith(1);
      expect(store.selectedProduct).toBeNull();
    });

    it('删除非选中产品时不应该影响选中状态', async () => {
      vi.mocked(api.deleteProduct).mockResolvedValue(undefined);
      vi.mocked(api.getProducts).mockResolvedValue([]);

      const store = useProductStore();
      const product = { id: 1, name: '测试' } as any;
      store.selectedProduct = product;

      await store.deleteProduct(2);

      expect(store.selectedProduct).toEqual(product);
    });
  });
});

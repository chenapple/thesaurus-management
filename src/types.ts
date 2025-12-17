export interface Product {
  id: number;
  name: string;
  sku: string | null;
  asin: string | null;
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

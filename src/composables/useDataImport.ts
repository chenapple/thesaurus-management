import { ref, onUnmounted, type Ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { readFile } from "@tauri-apps/plugin-fs";
import * as XLSX from "xlsx";
import * as api from "../api";
import type { KeywordData, Product } from "../types";
import type { UnlistenFn } from "@tauri-apps/api/event";

interface DataImportOptions {
  selectedProduct: Ref<Product | null>;
  products: Ref<Product[]>;
  viewMode: Ref<string>;
  stats: Ref<{ keywordCount: number; rootCount: number }>;
  loadKeywordData: () => Promise<void>;
  loadRoots: () => Promise<void>;
  loadStats: () => Promise<void>;
  loadWorkflowStatus: () => Promise<void>;
}

export function useDataImport(options: DataImportOptions) {
  const {
    selectedProduct,
    products,
    viewMode,
    stats,
    loadKeywordData,
    loadRoots,
    loadStats,
    loadWorkflowStatus,
  } = options;

  const importing = ref(false);
  const isDragging = ref(false);
  let unlistenDragDrop: UnlistenFn | null = null;

  async function processExcelBuffer(buffer: ArrayBuffer) {
    if (!selectedProduct.value) {
      ElMessage.warning("请先选择或创建一个产品");
      return;
    }

    const workbook = XLSX.read(buffer, { type: "array" });
    const sheetName = workbook.SheetNames[0];
    const sheet = workbook.Sheets[sheetName];
    const data = XLSX.utils.sheet_to_json<{ [key: string]: any }>(sheet, { header: 1 });

    if (data.length < 2) {
      ElMessage.warning("Excel中没有数据");
      return;
    }

    // Get headers (first row)
    const headers = data[0] as string[];
    const rows = data.slice(1) as any[][];

    // Parse keywords and full data
    const keywords: string[] = [];
    const keywordDataList: KeywordData[] = [];

    // Find ASIN columns after column P
    const asinColumns: { index: number; name: string }[] = [];
    for (let i = 16; i < headers.length; i++) {
      if (headers[i]) {
        asinColumns.push({ index: i, name: String(headers[i]) });
      }
    }

    for (const row of rows) {
      const keyword = row[0] ? String(row[0]).trim() : "";
      if (!keyword) continue;

      keywords.push(keyword);

      // Collect ASIN data
      const asinData: { [key: string]: any } = {};
      for (const col of asinColumns) {
        if (row[col.index] !== undefined && row[col.index] !== null) {
          asinData[col.name] = row[col.index];
        }
      }

      // Column mapping (A-P):
      // A(0): Keyword, B(1): Translation, C(2): Relevance Score, D(3): Relevance Level
      // E(4): Traffic Total, F(5): Avg Keyword Rank, G(6): Avg Search Volume
      // H(7): CPC Bid, I(8): Bid Range, J(9): Click Rate
      // K(10): Conversion Competition, L(11): Competition Level, M(12): Natural Position Flow
      // N(13): Top3 Click Share, O(14): Avg Conversion Share, P(15): ASIN Count
      const kwData: KeywordData = {
        id: 0,
        product_id: selectedProduct.value.id,
        keyword: keyword,
        translation: row[1] ? String(row[1]) : null,
        relevance_score: row[2] !== undefined ? String(row[2]) : null,
        relevance_level: row[3] ? String(row[3]) : null,
        traffic_total: row[4] !== undefined ? Number(row[4]) : null,
        avg_keyword_rank: row[5] ? String(row[5]) : null,
        avg_search_volume: row[6] !== undefined ? Number(row[6]) : null,
        cpc_bid: row[7] ? String(row[7]) : null,
        bid_range: row[8] ? String(row[8]) : null,
        click_rate: row[9] ? String(row[9]) : null,
        conversion_competition: row[10] ? String(row[10]) : null,
        competition_level: row[11] ? String(row[11]) : null,
        natural_position_flow: row[12] ? String(row[12]) : null,
        top3_click_share: row[13] ? String(row[13]) : null,
        avg_conversion_share: row[14] ? String(row[14]) : null,
        asin_count: row[15] !== undefined ? Number(row[15]) : null,
        // New calculated columns (initially empty)
        traffic_level: null,
        negative_word: null,
        orderliness: null,
        phrase_tag: null,
        primary_category: null,
        secondary_category: null,
        search_intent: null,
        traffic_share: null,
        asin_data: Object.keys(asinData).length > 0 ? JSON.stringify(asinData) : null,
      };
      keywordDataList.push(kwData);
    }

    if (keywords.length === 0) {
      ElMessage.warning("Excel中没有找到关键词");
      return;
    }

    // Save Excel headers (CPC and bid range columns contain currency symbols)
    const cpcHeader = headers[7] ? String(headers[7]) : null;
    const bidRangeHeader = headers[8] ? String(headers[8]) : null;
    if (cpcHeader || bidRangeHeader) {
      await api.updateProductHeaders(selectedProduct.value.id, cpcHeader || undefined, bidRangeHeader || undefined);
      // Update local product data
      selectedProduct.value.cpc_header = cpcHeader;
      selectedProduct.value.bid_range_header = bidRangeHeader;
      // Sync update product list
      const idx = products.value.findIndex(p => p.id === selectedProduct.value!.id);
      if (idx >= 0) {
        products.value[idx].cpc_header = cpcHeader;
        products.value[idx].bid_range_header = bidRangeHeader;
      }
    }

    // Check if there's existing data, auto backup if so
    const hasExistingData = stats.value.keywordCount > 0;
    if (hasExistingData) {
      try {
        await ElMessageBox.confirm(
          '导入新数据将覆盖现有数据。系统将自动创建备份，您可以随时回滚到当前版本。',
          '导入确认',
          {
            confirmButtonText: '确认导入',
            cancelButtonText: '取消',
            type: 'warning',
          }
        );

        // Create auto backup
        const timestamp = new Date().toLocaleString('zh-CN');
        await api.createBackup(
          selectedProduct.value.id,
          `导入前自动备份 - ${timestamp}`
        );
        ElMessage.success('已创建备份');
      } catch (e) {
        if (e === 'cancel') {
          ElMessage.info('已取消导入');
          return;
        }
        throw e;
      }
    }

    // Import full keyword data
    await api.importKeywordData(selectedProduct.value.id, keywordDataList);

    // Also import keywords for root analysis
    await api.importKeywords(selectedProduct.value.id, keywords);

    ElMessage.success(`成功导入 ${keywords.length} 个关键词到"${selectedProduct.value.name}"`);

    // Auto calculate traffic levels (use product saved thresholds or defaults)
    const bigThreshold = selectedProduct.value.big_word_threshold || 20000;
    const mediumThreshold = selectedProduct.value.medium_word_threshold || 100000;
    await api.calculateTrafficLevels(selectedProduct.value.id, bigThreshold, mediumThreshold);

    // Auto calculate traffic share
    await api.calculateTrafficShare(selectedProduct.value.id);

    // Refresh data
    await loadKeywordData();
    await loadRoots();
    await loadStats();
    await loadWorkflowStatus();
  }

  async function handleImport() {
    if (!selectedProduct.value) {
      ElMessage.warning("请先选择或创建一个产品");
      return;
    }

    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "Excel", extensions: ["xlsx", "xls"] }],
      });

      if (!selected) return;

      importing.value = true;
      const fileData = await readFile(selected);
      const buffer = fileData.buffer;
      await processExcelBuffer(buffer);
    } catch (e) {
      ElMessage.error("导入失败: " + e);
    } finally {
      importing.value = false;
    }
  }

  async function setupDragDrop() {
    const webview = getCurrentWebview();
    unlistenDragDrop = await webview.onDragDropEvent(async (event) => {
      // Global Excel drag import only works in keywords/roots views
      if (!['keywords', 'roots'].includes(viewMode.value)) {
        isDragging.value = false;
        return;
      }

      if (event.payload.type === "over") {
        isDragging.value = true;
      } else if (event.payload.type === "leave") {
        isDragging.value = false;
      } else if (event.payload.type === "drop") {
        isDragging.value = false;

        if (!selectedProduct.value) {
          ElMessage.warning("请先选择或创建一个产品");
          return;
        }

        const paths = event.payload.paths;
        if (paths.length === 0) return;

        const filePath = paths[0];
        const validExtensions = [".xlsx", ".xls"];
        const isValidFile = validExtensions.some((ext) =>
          filePath.toLowerCase().endsWith(ext)
        );

        if (!isValidFile) {
          ElMessage.warning("请拖入Excel文件（.xlsx或.xls）");
          return;
        }

        try {
          importing.value = true;
          const fileData = await readFile(filePath);
          const buffer = fileData.buffer;
          await processExcelBuffer(buffer);
        } catch (e) {
          ElMessage.error("导入失败: " + e);
        } finally {
          importing.value = false;
        }
      }
    });
  }

  onUnmounted(() => {
    if (unlistenDragDrop) {
      unlistenDragDrop();
    }
  });

  return {
    importing,
    isDragging,
    handleImport,
    setupDragDrop,
    processExcelBuffer,
  };
}

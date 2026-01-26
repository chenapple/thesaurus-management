import { onMounted, onUnmounted, type Ref } from "vue";
import type { Product } from "../types";

interface KeyboardShortcutsOptions {
  products: Ref<Product[]>;
  selectedProduct: Ref<Product | null>;
  viewMode: Ref<string>;
  analyzing: Ref<boolean>;
  classifying: Ref<boolean>;
  editingId: Ref<number | null>;
  showShortcutsDialog: Ref<boolean>;
  openAddProductDialog: () => void;
  handleImport: () => void;
  handleExport: () => void;
  handleKeywordExport: () => void;
  handleAIAnalysis: () => void;
  handleKeywordClassify: () => void;
  toggleTheme: () => void;
  openHelp: (tab?: string) => void;
  cancelEdit: () => void;
  selectProduct: (product: Product) => void;
}

export function useKeyboardShortcuts(options: KeyboardShortcutsOptions) {
  const {
    products,
    selectedProduct,
    viewMode,
    analyzing,
    classifying,
    editingId,
    showShortcutsDialog,
    openAddProductDialog,
    handleImport,
    handleExport,
    handleKeywordExport,
    handleAIAnalysis,
    handleKeywordClassify,
    toggleTheme,
    openHelp,
    cancelEdit,
    selectProduct,
  } = options;

  function handleKeyboard(e: KeyboardEvent) {
    // Detect Ctrl (Windows) or Cmd (Mac)
    const isMod = e.ctrlKey || e.metaKey;

    // If in input field, only respond to Escape
    const isInputting =
      document.activeElement?.tagName === "INPUT" ||
      document.activeElement?.tagName === "TEXTAREA";

    if (isInputting && e.key !== "Escape") {
      return;
    }

    // Ctrl/Cmd + N: Create new product
    if (isMod && e.key === "n") {
      e.preventDefault();
      openAddProductDialog();
      return;
    }

    // Ctrl/Cmd + I: Import Excel
    if (isMod && e.key === "i") {
      e.preventDefault();
      if (selectedProduct.value) {
        handleImport();
      }
      return;
    }

    // Ctrl/Cmd + E: Export Excel
    if (isMod && e.key === "e") {
      e.preventDefault();
      if (selectedProduct.value) {
        if (viewMode.value === 'keywords') {
          handleKeywordExport();
        } else {
          handleExport();
        }
      }
      return;
    }

    // Ctrl/Cmd + F: Focus search box
    if (isMod && e.key === "f") {
      e.preventDefault();
      const searchInput = document.querySelector(
        ".header-right .el-input__inner"
      ) as HTMLInputElement;
      searchInput?.focus();
      return;
    }

    // Ctrl/Cmd + Enter: Trigger AI operation based on view
    if (isMod && e.key === "Enter") {
      e.preventDefault();
      if (selectedProduct.value) {
        if (viewMode.value === 'keywords' && !classifying.value) {
          handleKeywordClassify();
        } else if (viewMode.value === 'roots' && !analyzing.value) {
          handleAIAnalysis();
        }
      }
      return;
    }

    // Ctrl/Cmd + D: Toggle dark mode
    if (isMod && e.key === "d") {
      e.preventDefault();
      toggleTheme();
      return;
    }

    // Ctrl/Cmd + H: Show help center
    if (isMod && e.key === "h") {
      e.preventDefault();
      openHelp(viewMode.value);
      return;
    }

    // ? or Ctrl/Cmd + /: Show shortcuts help
    if (e.key === "?" || (isMod && e.key === "/")) {
      e.preventDefault();
      showShortcutsDialog.value = true;
      return;
    }

    // Escape: Close dialog / Cancel edit
    if (e.key === "Escape") {
      if (showShortcutsDialog.value) {
        showShortcutsDialog.value = false;
      } else if (editingId.value !== null) {
        cancelEdit();
      }
      return;
    }

    // Arrow up/down: Navigate product list
    if (e.key === "ArrowUp" || e.key === "ArrowDown") {
      if (products.value.length === 0) return;

      const currentIndex = selectedProduct.value
        ? products.value.findIndex((p) => p.id === selectedProduct.value?.id)
        : -1;

      let newIndex: number;
      if (e.key === "ArrowUp") {
        newIndex = currentIndex > 0 ? currentIndex - 1 : products.value.length - 1;
      } else {
        newIndex = currentIndex < products.value.length - 1 ? currentIndex + 1 : 0;
      }

      selectProduct(products.value[newIndex]);
      return;
    }
  }

  onMounted(() => {
    window.addEventListener("keydown", handleKeyboard);
  });

  onUnmounted(() => {
    window.removeEventListener("keydown", handleKeyboard);
  });

  return {
    handleKeyboard,
  };
}

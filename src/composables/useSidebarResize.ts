import { ref, onMounted, onUnmounted } from "vue";

export function useSidebarResize() {
  const sidebarWidth = ref(240);
  const isResizing = ref(false);
  const MIN_SIDEBAR_WIDTH = 180;
  const MAX_SIDEBAR_WIDTH = 400;

  function initSidebarWidth() {
    const saved = localStorage.getItem("sidebarWidth");
    if (saved) {
      const width = parseInt(saved, 10);
      if (width >= MIN_SIDEBAR_WIDTH && width <= MAX_SIDEBAR_WIDTH) {
        sidebarWidth.value = width;
      }
    }
  }

  function startResize(e: MouseEvent) {
    isResizing.value = true;
    document.addEventListener("mousemove", handleResize);
    document.addEventListener("mouseup", stopResize);
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    e.preventDefault();
  }

  function handleResize(e: MouseEvent) {
    if (!isResizing.value) return;
    const newWidth = Math.min(Math.max(e.clientX, MIN_SIDEBAR_WIDTH), MAX_SIDEBAR_WIDTH);
    sidebarWidth.value = newWidth;
  }

  function stopResize() {
    isResizing.value = false;
    document.removeEventListener("mousemove", handleResize);
    document.removeEventListener("mouseup", stopResize);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    localStorage.setItem("sidebarWidth", sidebarWidth.value.toString());
  }

  onMounted(() => {
    initSidebarWidth();
  });

  onUnmounted(() => {
    // Cleanup if resize was in progress
    if (isResizing.value) {
      stopResize();
    }
  });

  return {
    sidebarWidth,
    isResizing,
    MIN_SIDEBAR_WIDTH,
    MAX_SIDEBAR_WIDTH,
    startResize,
  };
}

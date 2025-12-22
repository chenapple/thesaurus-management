<script setup lang="ts">
defineProps<{
  visible: boolean;
  appVersion: string;
}>();

defineEmits<{
  (e: 'update:visible', value: boolean): void;
}>();
</script>

<template>
  <el-dialog
    :model-value="visible"
    @update:model-value="$emit('update:visible', $event)"
    title="键盘快捷键"
    width="480px"
    :show-close="true"
  >
    <div class="shortcuts-list">
      <div class="shortcut-group">
        <h4>通用操作</h4>
        <div class="shortcut-item">
          <span class="shortcut-desc">创建新产品</span>
          <span class="shortcut-key"><kbd>⌘</kbd> <kbd>N</kbd></span>
        </div>
        <div class="shortcut-item">
          <span class="shortcut-desc">切换深色模式</span>
          <span class="shortcut-key"><kbd>⌘</kbd> <kbd>D</kbd></span>
        </div>
        <div class="shortcut-item">
          <span class="shortcut-desc">显示快捷键帮助</span>
          <span class="shortcut-key"><kbd>?</kbd></span>
        </div>
      </div>
      <div class="shortcut-group">
        <h4>数据操作</h4>
        <div class="shortcut-item">
          <span class="shortcut-desc">导入 Excel</span>
          <span class="shortcut-key"><kbd>⌘</kbd> <kbd>I</kbd></span>
        </div>
        <div class="shortcut-item">
          <span class="shortcut-desc">导出 Excel</span>
          <span class="shortcut-key"><kbd>⌘</kbd> <kbd>E</kbd></span>
        </div>
        <div class="shortcut-item">
          <span class="shortcut-desc">AI分类 / 智能分析</span>
          <span class="shortcut-key"><kbd>⌘</kbd> <kbd>↵</kbd></span>
        </div>
      </div>
      <div class="shortcut-group">
        <h4>导航</h4>
        <div class="shortcut-item">
          <span class="shortcut-desc">聚焦搜索框</span>
          <span class="shortcut-key"><kbd>⌘</kbd> <kbd>F</kbd></span>
        </div>
        <div class="shortcut-item">
          <span class="shortcut-desc">切换产品</span>
          <span class="shortcut-key"><kbd>↑</kbd> <kbd>↓</kbd></span>
        </div>
        <div class="shortcut-item">
          <span class="shortcut-desc">取消/关闭</span>
          <span class="shortcut-key"><kbd>Esc</kbd></span>
        </div>
      </div>
    </div>
    <template #footer>
      <div class="shortcuts-footer">
        <span class="shortcuts-tip">Windows 用户请将 ⌘ 替换为 Ctrl</span>
        <span class="app-version">版本号：v{{ appVersion }}</span>
      </div>
    </template>
  </el-dialog>
</template>

<style scoped>
.shortcuts-list {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.shortcut-group h4 {
  font-size: 13px;
  color: var(--text-muted);
  margin-bottom: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.shortcut-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid var(--border-color);
}

.shortcut-item:last-child {
  border-bottom: none;
}

.shortcut-desc {
  color: var(--text-primary);
  font-size: 14px;
}

.shortcut-key {
  display: flex;
  gap: 4px;
}

.shortcut-key kbd {
  display: inline-block;
  padding: 4px 8px;
  font-size: 12px;
  font-family: inherit;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-secondary);
  min-width: 28px;
  text-align: center;
}

.shortcuts-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.shortcuts-tip {
  font-size: 12px;
  color: var(--text-muted);
}

.app-version {
  font-size: 12px;
  color: var(--text-muted);
}
</style>

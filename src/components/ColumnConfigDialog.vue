<script setup lang="ts">
interface ColumnDef {
  key: string;
  label: string;
  default?: boolean;
  required?: boolean;
}

defineProps<{
  visible: boolean;
  columnDefinitions: ColumnDef[];
  columnConfig: Record<string, boolean>;
  isAllColumnsSelected: boolean;
}>();

defineEmits<{
  (e: 'update:visible', value: boolean): void;
  (e: 'update:columnConfig', key: string, value: boolean): void;
  (e: 'toggleAll', value: boolean): void;
  (e: 'resetDefault'): void;
}>();
</script>

<template>
  <el-dialog
    :model-value="visible"
    @update:model-value="$emit('update:visible', $event)"
    title="列配置"
    width="500px"
  >
    <div class="column-config">
      <p class="column-config-desc">选择要显示的列（关键词列始终显示）</p>
      <div class="column-config-select-all">
        <el-checkbox
          :model-value="isAllColumnsSelected"
          @change="(val: boolean) => $emit('toggleAll', val)"
        >
          全选
        </el-checkbox>
      </div>
      <div class="column-config-grid">
        <div
          v-for="col in columnDefinitions"
          :key="col.key"
          class="column-config-item"
        >
          <el-checkbox
            :model-value="columnConfig[col.key]"
            :disabled="col.required"
            @change="(val: boolean) => $emit('update:columnConfig', col.key, val)"
          >
            {{ col.label }}
          </el-checkbox>
        </div>
      </div>
    </div>
    <template #footer>
      <el-button @click="$emit('resetDefault')">
        恢复默认
      </el-button>
      <el-button type="primary" @click="$emit('update:visible', false)">
        确定
      </el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.column-config {
  padding: 0 10px;
}

.column-config-desc {
  margin-bottom: 16px;
  color: var(--el-text-color-secondary);
  font-size: 14px;
}

.column-config-select-all {
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.column-config-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}

.column-config-item {
  padding: 4px 0;
}
</style>

<script setup lang="ts">
import { Clock, Document, RefreshLeft, Delete } from '@element-plus/icons-vue';
import type { BackupInfo } from '../types';

defineProps<{
  visible: boolean;
  backups: BackupInfo[];
  restoring: boolean;
}>();

defineEmits<{
  (e: 'update:visible', value: boolean): void;
  (e: 'restore', backup: BackupInfo): void;
  (e: 'delete', backup: BackupInfo): void;
}>();

function formatBackupTime(dateStr: string): string {
  try {
    const date = new Date(dateStr);
    return date.toLocaleString('zh-CN');
  } catch {
    return dateStr;
  }
}
</script>

<template>
  <el-dialog
    :model-value="visible"
    @update:model-value="$emit('update:visible', $event)"
    title="备份管理"
    width="600px"
  >
    <div class="backup-dialog">
      <p class="backup-desc">
        每次导入Excel前会自动创建备份，最多保留3个历史版本
      </p>

      <div v-if="backups.length === 0" class="backup-empty">
        <el-empty description="暂无备份" :image-size="80" />
      </div>

      <div v-else class="backup-list">
        <div
          v-for="backup in backups"
          :key="backup.id"
          class="backup-item"
        >
          <div class="backup-info">
            <div class="backup-name">
              {{ backup.backup_name || '自动备份' }}
            </div>
            <div class="backup-meta">
              <span class="backup-time">
                <el-icon><Clock /></el-icon>
                {{ formatBackupTime(backup.created_at) }}
              </span>
              <span class="backup-count">
                <el-icon><Document /></el-icon>
                {{ backup.keyword_data_count }} 条数据
              </span>
            </div>
          </div>
          <div class="backup-actions">
            <el-button
              type="primary"
              size="small"
              :loading="restoring"
              @click="$emit('restore', backup)"
            >
              <el-icon><RefreshLeft /></el-icon>
              回滚
            </el-button>
            <el-button
              type="danger"
              size="small"
              plain
              @click="$emit('delete', backup)"
            >
              <el-icon><Delete /></el-icon>
            </el-button>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <el-button @click="$emit('update:visible', false)">关闭</el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.backup-dialog {
  padding: 0 10px;
}

.backup-desc {
  color: var(--el-text-color-secondary);
  font-size: 14px;
  margin-bottom: 20px;
}

.backup-empty {
  padding: 20px 0;
}

.backup-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.backup-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
  border: 1px solid var(--el-border-color-light);
  transition: all 0.2s;
}

.backup-item:hover {
  border-color: var(--el-color-primary-light-5);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

.backup-info {
  flex: 1;
}

.backup-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 6px;
}

.backup-meta {
  display: flex;
  gap: 16px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.backup-time,
.backup-count {
  display: flex;
  align-items: center;
  gap: 4px;
}

.backup-time .el-icon,
.backup-count .el-icon {
  font-size: 14px;
}

.backup-actions {
  display: flex;
  gap: 8px;
}
</style>

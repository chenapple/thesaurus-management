<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { EditPen, Delete, Check, Close, Rank, Calendar, Refresh, Search } from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
import draggable from 'vuedraggable';
import * as api from '../api';
import type { QuickNote } from '../types';

// 状态
const isOpen = ref(false);
const notes = ref<QuickNote[]>([]);
const newContent = ref('');
const loading = ref(false);
const filter = ref<'all' | 'pending' | 'completed'>('pending');
const editingId = ref<number | null>(null);
const editingContent = ref('');
const hasShownStartupNotification = ref(false); // 防止重复发送启动通知
const searchText = ref('');
const typeFilter = ref<'all' | 'repeat' | 'normal'>('all');
const newDueDate = ref<string | null>(null);
const newRepeatType = ref<string | null>(null);

// 计算属性
const pendingCount = computed(() => notes.value.filter(n => !n.completed).length);

// 紧急事项（已过期 + 今天到期）
const urgentNotes = computed(() => {
  return notes.value.filter(n => {
    if (n.completed || !n.due_date) return false;
    const days = getDueDays(n);
    return days <= 0; // 过期或今天到期
  });
});

const urgentCount = computed(() => urgentNotes.value.length);


const filteredNotes = computed(() => {
  let result = notes.value;
  if (filter.value === 'pending') {
    result = result.filter(n => !n.completed);
  } else if (filter.value === 'completed') {
    result = result.filter(n => n.completed);
  }
  if (typeFilter.value === 'repeat') {
    result = result.filter(n => !!n.repeat_type);
  } else if (typeFilter.value === 'normal') {
    result = result.filter(n => !n.repeat_type);
  }
  if (searchText.value.trim()) {
    const keyword = searchText.value.trim().toLowerCase();
    result = result.filter(n => n.content.toLowerCase().includes(keyword));
  }
  return result;
});

// 检查是否过期
function isOverdue(note: QuickNote): boolean {
  if (!note.due_date || note.completed) return false;
  const dueDate = new Date(note.due_date);
  dueDate.setHours(23, 59, 59, 999);
  return new Date() > dueDate;
}

// 计算距离截止日期的天数
function getDueDays(note: QuickNote): number {
  if (!note.due_date) return 0;
  const dueDate = new Date(note.due_date);
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  dueDate.setHours(0, 0, 0, 0);
  return Math.ceil((dueDate.getTime() - today.getTime()) / (1000 * 60 * 60 * 24));
}

// 格式化截止日期显示
function formatDueDate(note: QuickNote): string {
  if (!note.due_date) return '';
  const days = getDueDays(note);
  if (days < 0) {
    return `已过期 ${Math.abs(days)} 天`;
  }
  if (days === 0) return '今天到期';
  if (days === 1) return '明天到期';
  if (days <= 7) return `${days}天后到期`;
  const date = new Date(note.due_date);
  return `${date.getMonth() + 1}/${date.getDate()} 到期`;
}

// 加载备忘录
async function loadNotes() {
  try {
    loading.value = true;
    notes.value = await api.getQuickNotes();
  } catch (e) {
    console.error('加载备忘录失败:', e);
  } finally {
    loading.value = false;
  }
}

// 添加备忘录
async function addNote() {
  const content = newContent.value.trim();
  if (!content) return;

  try {
    const id = await api.addQuickNote(content);
    if (newDueDate.value) {
      await api.updateQuickNoteDueDate(id, newDueDate.value);
      if (newRepeatType.value) {
        await api.updateQuickNoteRepeat(id, newRepeatType.value);
      }
      newDueDate.value = null;
      newRepeatType.value = null;
    }
    newContent.value = '';
    await loadNotes();
  } catch (e) {
    console.error('添加备忘录失败:', e);
  }
}

// 切换完成状态
async function toggleNote(id: number) {
  try {
    await api.toggleQuickNote(id);
    await loadNotes();
  } catch (e) {
    console.error('切换状态失败:', e);
  }
}

// 开始编辑
function startEdit(note: QuickNote) {
  editingId.value = note.id;
  editingContent.value = note.content;
}

// 保存编辑
async function saveEdit() {
  if (editingId.value === null) return;
  const content = editingContent.value.trim();
  if (!content) return;

  try {
    await api.updateQuickNote(editingId.value, content);
    editingId.value = null;
    editingContent.value = '';
    await loadNotes();
  } catch (e) {
    console.error('更新备忘录失败:', e);
  }
}

// 取消编辑
function cancelEdit() {
  editingId.value = null;
  editingContent.value = '';
}

// 删除备忘录
async function deleteNote(id: number) {
  try {
    await api.deleteQuickNote(id);
    await loadNotes();
  } catch (e) {
    console.error('删除备忘录失败:', e);
  }
}

// 更新截止日期
async function updateDueDate(id: number, dueDate: string | null) {
  try {
    await api.updateQuickNoteDueDate(id, dueDate);
    await loadNotes();
  } catch (e) {
    console.error('更新截止日期失败:', e);
  }
}

// 更新重复设置
async function updateRepeat(id: number, repeatType: string | null, repeatInterval: number = 1) {
  try {
    await api.updateQuickNoteRepeat(id, repeatType, repeatInterval);
    await loadNotes();
  } catch (e) {
    console.error('更新重复设置失败:', e);
  }
}

// 获取重复类型显示文本
function getRepeatLabel(note: QuickNote): string {
  if (!note.repeat_type) return '';
  const interval = note.repeat_interval || 1;
  if (interval === 1) {
    switch (note.repeat_type) {
      case 'daily': return '每天';
      case 'weekday': return '工作日';
      case 'weekly': return '每周';
      case 'monthly': return '每月';
      default: return '';
    }
  } else {
    switch (note.repeat_type) {
      case 'daily': return `每${interval}天`;
      case 'weekday': return `每${interval}个工作日`;
      case 'weekly': return `每${interval}周`;
      case 'monthly': return `每${interval}月`;
      default: return '';
    }
  }
}

// 拖拽结束处理
async function onDragEnd() {
  try {
    // 获取当前筛选后的备忘录ID列表
    const ids = filteredNotes.value.map(n => n.id);
    await api.reorderQuickNotes(ids);
  } catch (e) {
    console.error('更新排序失败:', e);
    // 如果失败，重新加载恢复原始顺序
    await loadNotes();
  }
}

// 切换面板
function togglePanel() {
  isOpen.value = !isOpen.value;
  if (isOpen.value) {
    loadNotes();
    // 显示紧急提醒
    showUrgentReminder();
  }
}

// 显示紧急事项提醒（打开抽屉时）
function showUrgentReminder() {
  const overdueNotes = notes.value.filter(n => !n.completed && n.due_date && getDueDays(n) < 0);
  const todayNotes = notes.value.filter(n => !n.completed && n.due_date && getDueDays(n) === 0);
  const tomorrowNotes = notes.value.filter(n => !n.completed && n.due_date && getDueDays(n) === 1);

  if (overdueNotes.length > 0) {
    ElMessage.error({
      message: `有 ${overdueNotes.length} 项备忘已过期！`,
      duration: 4000,
      showClose: true,
    });
  } else if (todayNotes.length > 0) {
    ElMessage.warning({
      message: `有 ${todayNotes.length} 项备忘今天到期`,
      duration: 3000,
      showClose: true,
    });
  } else if (tomorrowNotes.length > 0) {
    ElMessage.info({
      message: `有 ${tomorrowNotes.length} 项备忘明天到期`,
      duration: 3000,
      showClose: true,
    });
  }
}

// 发送系统通知
async function sendSystemNotification() {
  try {
    let permissionGranted = await isPermissionGranted();
    if (!permissionGranted) {
      const permission = await requestPermission();
      permissionGranted = permission === 'granted';
    }

    if (!permissionGranted) return;

    const overdueNotes = notes.value.filter(n => !n.completed && n.due_date && getDueDays(n) < 0);
    const todayNotes = notes.value.filter(n => !n.completed && n.due_date && getDueDays(n) === 0);

    if (overdueNotes.length > 0 || todayNotes.length > 0) {
      let title = '备忘录提醒';
      let body = '';

      if (overdueNotes.length > 0 && todayNotes.length > 0) {
        body = `有 ${overdueNotes.length} 项已过期，${todayNotes.length} 项今天到期`;
      } else if (overdueNotes.length > 0) {
        body = `有 ${overdueNotes.length} 项备忘已过期：${overdueNotes[0].content.slice(0, 30)}${overdueNotes[0].content.length > 30 ? '...' : ''}`;
      } else {
        body = `有 ${todayNotes.length} 项备忘今天到期：${todayNotes[0].content.slice(0, 30)}${todayNotes[0].content.length > 30 ? '...' : ''}`;
      }

      sendNotification({ title, body });
    }
  } catch (e) {
    console.error('发送系统通知失败:', e);
  }
}

// 格式化时间
function formatTime(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const days = Math.floor(diff / (1000 * 60 * 60 * 24));

  if (days === 0) {
    return `今天 ${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}`;
  }
  if (days === 1) {
    return '昨天';
  }
  if (days < 7) {
    return `${days}天前`;
  }
  return `${date.getMonth() + 1}/${date.getDate()}`;
}

onMounted(async () => {
  // 预加载备忘录数据
  await loadNotes();
  // 应用启动时发送系统通知
  if (!hasShownStartupNotification.value && notes.value.length > 0) {
    hasShownStartupNotification.value = true;
    sendSystemNotification();
  }
});
</script>

<template>
  <!-- 浮动按钮 - 边缘隐藏 -->
  <div class="quick-notes-fab-container" :class="{ 'is-open': isOpen }">
    <div class="quick-notes-fab" @click="togglePanel">
      <el-icon :size="20"><EditPen /></el-icon>
      <!-- 紧急事项显示红色徽章（闪烁），普通待办显示蓝色徽章 -->
      <span
        v-if="(urgentCount > 0 || pendingCount > 0) && !isOpen"
        class="fab-badge"
        :class="{ urgent: urgentCount > 0 }"
      >
        {{ urgentCount > 0 ? urgentCount : pendingCount }}
      </span>
    </div>
  </div>

  <!-- 抽屉面板 -->
  <el-drawer
    v-model="isOpen"
    title="备忘录"
    direction="rtl"
    size="360px"
    :show-close="true"
    :with-header="true"
    class="quick-notes-drawer"
  >
    <template #header>
      <div class="drawer-header">
        <span class="drawer-title">备忘录</span>
        <el-radio-group v-model="filter" size="small">
          <el-radio-button value="all">全部</el-radio-button>
          <el-radio-button value="pending">待办</el-radio-button>
          <el-radio-button value="completed">已完成</el-radio-button>
        </el-radio-group>
      </div>
    </template>

    <div class="notes-container">
      <!-- 搜索和类型筛选 -->
      <div class="notes-toolbar">
        <el-input
          v-model="searchText"
          placeholder="搜索备忘..."
          clearable
          size="small"
          :prefix-icon="Search"
          class="search-input"
        />
        <el-radio-group v-model="typeFilter" size="small" class="type-filter">
          <el-radio-button value="all">全部</el-radio-button>
          <el-radio-button value="repeat">重复</el-radio-button>
          <el-radio-button value="normal">普通</el-radio-button>
        </el-radio-group>
      </div>

      <!-- 添加区域 -->
      <div class="add-note-area">
        <el-input
          v-model="newContent"
          placeholder="添加新备忘..."
          clearable
          @keyup.enter="addNote"
        >
          <template #append>
            <div class="add-note-append">
              <el-popover placement="bottom" :width="280" trigger="click">
                <template #reference>
                  <el-button
                    :icon="Calendar"
                    :type="newDueDate ? 'primary' : 'default'"
                    :title="newDueDate ? `截止: ${newDueDate}` : '设置截止日期'"
                  />
                </template>
                <div class="due-date-picker">
                  <el-date-picker
                    v-model="newDueDate"
                    type="date"
                    placeholder="选择截止日期"
                    format="YYYY-MM-DD"
                    value-format="YYYY-MM-DD"
                    :clearable="true"
                    style="width: 100%"
                  />
                  <div class="quick-date-btns">
                    <el-button size="small" @click="newDueDate = new Date().toISOString().split('T')[0]">今天</el-button>
                    <el-button size="small" @click="newDueDate = new Date(Date.now() + 86400000).toISOString().split('T')[0]">明天</el-button>
                    <el-button size="small" @click="newDueDate = new Date(Date.now() + 7 * 86400000).toISOString().split('T')[0]">一周后</el-button>
                  </div>
                  <!-- 重复设置（仅当有截止日期时显示） -->
                  <div v-if="newDueDate" class="repeat-setting">
                    <span class="repeat-label-text">重复:</span>
                    <el-select
                      :model-value="newRepeatType || ''"
                      size="small"
                      placeholder="不重复"
                      style="width: 100%"
                      @change="(val: string) => newRepeatType = val || null"
                    >
                      <el-option label="不重复" value="" />
                      <el-option label="每天" value="daily" />
                      <el-option label="仅工作日" value="weekday" />
                      <el-option label="每周" value="weekly" />
                      <el-option label="每月" value="monthly" />
                    </el-select>
                  </div>
                </div>
              </el-popover>
              <span class="append-divider"></span>
              <el-button @click="addNote" :disabled="!newContent.trim()">添加</el-button>
            </div>
          </template>
        </el-input>
      </div>

      <!-- 备忘列表 -->
      <div class="notes-list" v-loading="loading">
        <draggable
          v-model="notes"
          item-key="id"
          handle=".drag-handle"
          ghost-class="ghost"
          @end="onDragEnd"
          :disabled="filter !== 'all'"
        >
          <template #item="{ element: note }">
            <div
              v-show="(filter === 'all' || (filter === 'pending' && !note.completed) || (filter === 'completed' && note.completed)) && (typeFilter === 'all' || (typeFilter === 'repeat' && !!note.repeat_type) || (typeFilter === 'normal' && !note.repeat_type)) && (!searchText.trim() || note.content.toLowerCase().includes(searchText.trim().toLowerCase()))"
              class="note-item"
              :class="{ completed: note.completed, overdue: isOverdue(note) }"
            >
              <!-- 编辑模式 -->
              <template v-if="editingId === note.id">
                <el-input
                  v-model="editingContent"
                  size="small"
                  @keyup.enter="saveEdit"
                  @keyup.escape="cancelEdit"
                  autofocus
                />
                <div class="edit-actions">
                  <el-button size="small" type="primary" :icon="Check" @click="saveEdit" />
                  <el-button size="small" :icon="Close" @click="cancelEdit" />
                </div>
              </template>

              <!-- 查看模式 -->
              <template v-else>
                <div class="drag-handle" title="拖拽排序">
                  <el-icon><Rank /></el-icon>
                </div>
                <el-checkbox
                  :model-value="note.completed"
                  @change="toggleNote(note.id)"
                  class="note-checkbox"
                />
                <div class="note-content" @dblclick="startEdit(note)">
                  <span class="note-text">{{ note.content }}</span>
                  <!-- 截止日期、重复标识和创建时间 -->
                  <div class="note-meta">
                    <span v-if="note.due_date" class="due-date-label" :class="{ overdue: isOverdue(note) }">
                      <el-icon v-if="isOverdue(note)" class="warning-icon"><Calendar /></el-icon>
                      <el-icon v-else><Calendar /></el-icon>
                      {{ formatDueDate(note) }}
                    </span>
                    <span v-if="note.repeat_type" class="repeat-label" :title="getRepeatLabel(note)">
                      <el-icon><Refresh /></el-icon>
                      {{ getRepeatLabel(note) }}
                    </span>
                    <span class="note-time">{{ formatTime(note.created_at) }}</span>
                  </div>
                </div>
                <div class="note-actions">
                  <!-- 设置截止日期 -->
                  <el-popover
                    placement="bottom"
                    :width="280"
                    trigger="click"
                  >
                    <template #reference>
                      <el-button
                        size="small"
                        :icon="Calendar"
                        link
                        :type="note.due_date ? 'primary' : 'default'"
                        title="设置截止日期"
                      />
                    </template>
                    <div class="due-date-picker">
                      <el-date-picker
                        :model-value="note.due_date"
                        type="date"
                        placeholder="选择截止日期"
                        format="YYYY-MM-DD"
                        value-format="YYYY-MM-DD"
                        :clearable="true"
                        @update:model-value="(val: string | null) => updateDueDate(note.id, val)"
                        style="width: 100%"
                      />
                      <div class="quick-date-btns">
                        <el-button size="small" @click="updateDueDate(note.id, new Date().toISOString().split('T')[0])">今天</el-button>
                        <el-button size="small" @click="updateDueDate(note.id, new Date(Date.now() + 86400000).toISOString().split('T')[0])">明天</el-button>
                        <el-button size="small" @click="updateDueDate(note.id, new Date(Date.now() + 7 * 86400000).toISOString().split('T')[0])">一周后</el-button>
                      </div>
                      <!-- 重复设置（仅当有截止日期时显示） -->
                      <div v-if="note.due_date" class="repeat-setting">
                        <span class="repeat-label-text">重复:</span>
                        <el-select
                          :model-value="note.repeat_type || ''"
                          size="small"
                          placeholder="不重复"
                          style="width: 100%"
                          @change="(val: string) => updateRepeat(note.id, val || null)"
                        >
                          <el-option label="不重复" value="" />
                          <el-option label="每天" value="daily" />
                          <el-option label="仅工作日" value="weekday" />
                          <el-option label="每周" value="weekly" />
                          <el-option label="每月" value="monthly" />
                        </el-select>
                      </div>
                    </div>
                  </el-popover>
                  <el-button
                    size="small"
                    :icon="EditPen"
                    link
                    @click="startEdit(note)"
                  />
                  <el-button
                    size="small"
                    :icon="Delete"
                    link
                    type="danger"
                    @click="deleteNote(note.id)"
                  />
                </div>
              </template>
            </div>
          </template>
        </draggable>

        <!-- 空状态 -->
        <div v-if="filteredNotes.length === 0 && !loading" class="empty-state">
          <p v-if="filter === 'pending'">没有待办事项</p>
          <p v-else-if="filter === 'completed'">没有已完成事项</p>
          <p v-else>暂无备忘录</p>
        </div>
      </div>

      <!-- 底部统计 -->
      <div class="notes-footer" v-if="notes.length > 0">
        <span>{{ pendingCount }} 项待办</span>
        <span>{{ notes.length - pendingCount }} 项已完成</span>
      </div>
    </div>
  </el-drawer>
</template>

<style scoped>
/* 浮动按钮容器 - 边缘隐藏效果 */
.quick-notes-fab-container {
  position: fixed;
  right: 0;
  bottom: 100px;
  z-index: 999;
  transform: translateX(36px);
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.quick-notes-fab-container:hover,
.quick-notes-fab-container.is-open {
  transform: translateX(0);
}

.quick-notes-fab {
  position: relative;
  width: 48px;
  height: 48px;
  border-radius: 24px 0 0 24px;
  background: var(--gradient-primary, linear-gradient(135deg, #2563EB 0%, #7C3AED 100%));
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  box-shadow: -2px 2px 12px rgba(37, 99, 235, 0.3);
  transition: all 0.3s ease;
  padding-left: 4px;
}

.quick-notes-fab-container:hover .quick-notes-fab {
  box-shadow: -4px 4px 16px rgba(37, 99, 235, 0.4);
}

.fab-badge {
  position: absolute;
  top: -6px;
  left: -6px;
  min-width: 18px;
  height: 18px;
  background: #3B82F6;
  color: white;
  border-radius: 9px;
  font-size: 11px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 5px;
  box-shadow: 0 2px 6px rgba(59, 130, 246, 0.4);
}

/* 紧急事项徽章（红色闪烁） */
.fab-badge.urgent {
  background: #EF4444;
  box-shadow: 0 2px 6px rgba(239, 68, 68, 0.4);
  animation: urgentPulse 1.5s ease-in-out infinite;
}

@keyframes urgentPulse {
  0%, 100% {
    transform: scale(1);
    box-shadow: 0 2px 6px rgba(239, 68, 68, 0.4);
  }
  50% {
    transform: scale(1.1);
    box-shadow: 0 2px 12px rgba(239, 68, 68, 0.6);
  }
}

/* 抽屉样式 */
.drawer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.drawer-title {
  font-size: 16px;
  font-weight: 600;
}

.notes-container {
  display: flex;
  flex-direction: column;
  height: 100%;
}

/* 添加区域 */
.notes-toolbar {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 12px;
}

.notes-toolbar .search-input {
  flex: 1;
}

.notes-toolbar .type-filter {
  flex-shrink: 0;
}

.add-note-area {
  padding: 0 0 16px 0;
  border-bottom: 1px solid var(--el-border-color-light);
  margin-bottom: 16px;
}

.add-note-append {
  display: flex;
  align-items: center;
}

.append-divider {
  width: 1px;
  height: 16px;
  background: var(--el-border-color);
  margin: 0 15px;
}

/* 备忘列表 */
.notes-list {
  flex: 1;
  overflow-y: auto;
  min-height: 200px;
}

.note-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 12px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
  margin-bottom: 8px;
  transition: all 0.2s ease;
  border-left: 3px solid transparent;
}

.note-item:hover {
  background: var(--el-fill-color);
}

.note-item.completed {
  opacity: 0.6;
}

.note-item.completed .note-text {
  text-decoration: line-through;
  color: var(--el-text-color-secondary);
}

/* 过期样式 */
.note-item.overdue {
  border-left-color: #EF4444;
  background: rgba(239, 68, 68, 0.05);
}

.note-item.overdue:hover {
  background: rgba(239, 68, 68, 0.08);
}

/* 拖拽手柄 */
.drag-handle {
  cursor: grab;
  color: var(--el-text-color-placeholder);
  padding: 2px;
  display: flex;
  align-items: center;
  opacity: 0;
  transition: opacity 0.2s;
}

.note-item:hover .drag-handle {
  opacity: 1;
}

.drag-handle:active {
  cursor: grabbing;
}

/* 拖拽时的幽灵元素样式 */
.ghost {
  opacity: 0.5;
  background: var(--el-color-primary-light-9);
  border: 1px dashed var(--el-color-primary);
}

.note-checkbox {
  margin-top: 2px;
}

.note-content {
  flex: 1;
  min-width: 0;
  cursor: pointer;
}

.note-text {
  display: block;
  font-size: 14px;
  line-height: 1.5;
  word-break: break-word;
}

.note-meta {
  margin-top: 4px;
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.note-time {
  font-size: 12px;
  color: var(--el-text-color-placeholder);
}

/* 截止日期标签 */
.due-date-label {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--el-color-primary);
  padding: 2px 6px;
  background: var(--el-color-primary-light-9);
  border-radius: 4px;
}

.due-date-label.overdue {
  color: #EF4444;
  background: rgba(239, 68, 68, 0.1);
}

.due-date-label .warning-icon {
  animation: pulse 1.5s infinite;
}

/* 重复任务标签 */
.repeat-label {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--el-color-success);
  padding: 2px 6px;
  background: var(--el-color-success-light-9);
  border-radius: 4px;
}

/* 重复设置区域 */
.repeat-setting {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--el-border-color-lighter);
}

.repeat-label-text {
  display: block;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 6px;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.note-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.2s;
}

.note-item:hover .note-actions {
  opacity: 1;
}

.edit-actions {
  display: flex;
  gap: 4px;
  margin-left: auto;
}

/* 截止日期选择器 */
.due-date-picker {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.quick-date-btns {
  display: flex;
  gap: 4px;
  justify-content: flex-start;
}

/* 空状态 */
.empty-state {
  text-align: center;
  padding: 40px 20px;
  color: var(--el-text-color-secondary);
}

/* 底部统计 */
.notes-footer {
  display: flex;
  justify-content: space-between;
  padding: 12px 0 0;
  border-top: 1px solid var(--el-border-color-light);
  margin-top: auto;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

/* 暗色模式 */
html.dark .quick-notes-fab {
  box-shadow: -2px 2px 12px rgba(37, 99, 235, 0.25);
}

html.dark .quick-notes-fab-container:hover .quick-notes-fab {
  box-shadow: -4px 4px 16px rgba(37, 99, 235, 0.35);
}

html.dark .note-item.overdue {
  background: rgba(239, 68, 68, 0.1);
}

html.dark .note-item.overdue:hover {
  background: rgba(239, 68, 68, 0.15);
}
</style>

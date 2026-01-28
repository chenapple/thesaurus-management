<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import {
  ArrowLeft, ArrowRight, Plus, Edit, Delete, Clock,
  FolderOpened, MagicStick, Download, Refresh, Check,
  Document, Calendar, List
} from '@element-plus/icons-vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import * as api from '../api';
import type {
  WeeklyReport, WeeklyReportEntry, WeeklyReportContent,
  WeeklyReportEntryCategory
} from '../types';
import { EVENT_SUB_TYPES, type AIProvider } from '../types';
import ReportHistoryDialog from './ReportHistoryDialog.vue';
import { chatStream, checkApiKeyConfigured } from '../ai-service';
import * as XLSX from 'xlsx';
import { save } from '@tauri-apps/plugin-dialog';
import { writeFile } from '@tauri-apps/plugin-fs';

defineEmits<{
  (e: 'show-help'): void;
}>();

// ==================== 周次管理 ====================

// 获取指定日期所在周的周一（周一为一周的开始）
function getWeekStart(date: Date): Date {
  const d = new Date(date);
  d.setHours(0, 0, 0, 0);
  const dayOfWeek = d.getDay(); // 0=周日, 1=周一, ..., 6=周六
  // 计算需要回退的天数：周日回退6天，周一回退0天，周二回退1天...
  const daysToSubtract = dayOfWeek === 0 ? 6 : dayOfWeek - 1;
  d.setDate(d.getDate() - daysToSubtract);
  return d;
}

// 获取周日（一周的结束）
function getWeekEnd(date: Date): Date {
  const start = getWeekStart(date);
  const end = new Date(start);
  end.setDate(start.getDate() + 6); // 周一 + 6天 = 周日
  return end;
}

// 格式化日期为 YYYY-MM-DD（使用本地时间，避免时区问题）
function formatDate(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

// 当前选中的周
const currentWeekStart = ref<Date>(getWeekStart(new Date()));
const currentWeekEnd = computed(() => getWeekEnd(currentWeekStart.value));

// 周次显示
const weekLabel = computed(() => {
  const start = currentWeekStart.value;
  const year = start.getFullYear();
  const weekNum = getWeekNumber(start);
  return `${year}年第${weekNum}周`;
});

const dateRangeLabel = computed(() => {
  const start = currentWeekStart.value;
  const end = currentWeekEnd.value;
  return `${formatDate(start)} ~ ${formatDate(end)}`;
});

// 获取一年中的第几周
function getWeekNumber(date: Date): number {
  const firstDayOfYear = new Date(date.getFullYear(), 0, 1);
  const pastDaysOfYear = (date.getTime() - firstDayOfYear.getTime()) / 86400000;
  return Math.ceil((pastDaysOfYear + firstDayOfYear.getDay() + 1) / 7);
}

// 切换周次
function prevWeek() {
  const newStart = new Date(currentWeekStart.value);
  newStart.setDate(newStart.getDate() - 7);
  currentWeekStart.value = newStart;
  loadReportData();
}

function nextWeek() {
  const newStart = new Date(currentWeekStart.value);
  newStart.setDate(newStart.getDate() + 7);
  currentWeekStart.value = newStart;
  loadReportData();
}

function goToCurrentWeek() {
  currentWeekStart.value = getWeekStart(new Date());
  loadReportData();
}

// ==================== 周报数据 ====================

const loading = ref(false);
const reportData = ref<WeeklyReportContent | null>(null);
const report = computed(() => reportData.value?.report);
const entries = computed(() => reportData.value?.entries);
const quickNotesCompleted = computed(() => reportData.value?.quick_notes_completed || []);
const optimizationEvents = computed(() => reportData.value?.optimization_events || []);

async function loadReportData() {
  loading.value = true;
  try {
    const weekStart = formatDate(currentWeekStart.value);
    const weekEnd = formatDate(currentWeekEnd.value);
    reportData.value = await api.getWeeklyReportData(weekStart, weekEnd);
  } catch (error) {
    console.error('加载周报数据失败:', error);
    ElMessage.error('加载周报数据失败');
  } finally {
    loading.value = false;
  }
}

// 创建或初始化周报
async function ensureReport(): Promise<number | null> {
  const weekStart = formatDate(currentWeekStart.value);
  const weekEnd = formatDate(currentWeekEnd.value);
  const title = `${weekLabel.value}工作周报`;

  try {
    const id = await api.createWeeklyReport(weekStart, weekEnd, title);
    await loadReportData();
    return id;
  } catch (error) {
    console.error('创建周报失败:', error);
    ElMessage.error('创建周报失败');
    return null;
  }
}

// ==================== 常规任务 ====================

// 常规任务显示结构
interface RoutineTaskDisplay {
  content: string;
  repeat_type: 'daily' | 'weekly' | 'monthly';
  repeat_interval: number;
  completedCount: number;  // 本周完成次数
  expectedCount: number;   // 本周预期完成次数
}

const routineTasks = ref<RoutineTaskDisplay[]>([]);
const routineLoading = ref(false);

// 加载常规任务（有 repeat_type 的备忘录任务）
async function loadRoutineTasks() {
  routineLoading.value = true;
  try {
    const allNotes = await api.getQuickNotes();
    // 筛选有 repeat_type 的任务
    const repeatNotes = allNotes.filter(note => note.repeat_type);

    // 获取本周日期范围
    const weekStart = formatDate(currentWeekStart.value);
    const weekEnd = formatDate(currentWeekEnd.value);

    // 按任务内容分组，统计本周完成的【不同日期】数量
    const taskMap = new Map<string, {
      repeat_type: 'daily' | 'weekly' | 'monthly';
      repeat_interval: number;
      completedDates: Set<string>; // 用 Set 存储完成日期，自动去重
    }>();

    for (const note of repeatNotes) {
      const existing = taskMap.get(note.content);

      // 检查是否在本周内完成
      const completedAt = note.completed_at?.split(' ')[0]; // 取日期部分
      const isCompletedThisWeek = note.completed && completedAt &&
        completedAt >= weekStart && completedAt <= weekEnd;

      if (!existing) {
        const completedDates = new Set<string>();
        if (isCompletedThisWeek && completedAt) {
          completedDates.add(completedAt);
        }
        taskMap.set(note.content, {
          repeat_type: note.repeat_type as 'daily' | 'weekly' | 'monthly',
          repeat_interval: note.repeat_interval || 1,
          completedDates,
        });
      } else {
        if (isCompletedThisWeek && completedAt) {
          existing.completedDates.add(completedAt); // Set 会自动去重同一天的多次完成
        }
      }
    }

    // 计算预期完成次数并生成显示数据
    const today = new Date();
    const isCurrentWeek = formatDate(getWeekStart(today)) === weekStart;

    // 计算本周已过的【工作日】天数（周一到周五）
    let workdaysInWeek = 5; // 默认一周5个工作日
    if (isCurrentWeek) {
      // 当前周：计算从周一到今天经过了几个工作日
      const dayOfWeek = today.getDay(); // 0=周日, 1=周一, ..., 6=周六
      if (dayOfWeek === 0) {
        // 周日：本周工作日已全部过完
        workdaysInWeek = 5;
      } else if (dayOfWeek === 6) {
        // 周六：本周工作日已全部过完
        workdaysInWeek = 5;
      } else {
        // 周一到周五：已过的工作日数 = dayOfWeek
        workdaysInWeek = dayOfWeek;
      }
    }

    const result: RoutineTaskDisplay[] = [];
    for (const [content, data] of taskMap) {
      let expectedCount = 1;

      if (data.repeat_type === 'daily') {
        // 每日任务：预期次数 = 工作日天数 / 间隔
        expectedCount = Math.ceil(workdaysInWeek / data.repeat_interval);
      } else if (data.repeat_type === 'weekly') {
        // 每周任务：预期1次
        expectedCount = 1;
      } else if (data.repeat_type === 'monthly') {
        // 每月任务：预期1次（简化处理）
        expectedCount = 1;
      }

      result.push({
        content,
        repeat_type: data.repeat_type,
        repeat_interval: data.repeat_interval,
        completedCount: data.completedDates.size, // 使用去重后的日期数量
        expectedCount,
      });
    }

    routineTasks.value = result;
  } catch (error) {
    console.error('加载常规任务失败:', error);
  } finally {
    routineLoading.value = false;
  }
}

// 自动刷新定时器
let routineRefreshTimer: ReturnType<typeof setInterval> | null = null;

// 启动自动刷新（每5秒检查一次）
function startAutoRefresh() {
  if (routineRefreshTimer) return;
  routineRefreshTimer = setInterval(() => {
    loadRoutineTasks();
  }, 5000);
}

// 停止自动刷新
function stopAutoRefresh() {
  if (routineRefreshTimer) {
    clearInterval(routineRefreshTimer);
    routineRefreshTimer = null;
  }
}

// 获取常规任务频率标签
function getFrequencyLabel(task: RoutineTaskDisplay): string {
  const interval = task.repeat_interval;
  if (interval === 1) {
    switch (task.repeat_type) {
      case 'daily': return '每日';
      case 'weekly': return '每周';
      case 'monthly': return '每月';
      default: return '';
    }
  } else {
    switch (task.repeat_type) {
      case 'daily': return `每${interval}天`;
      case 'weekly': return `每${interval}周`;
      case 'monthly': return `每${interval}月`;
      default: return '';
    }
  }
}

// 获取任务进度文本（如 "3/5"）
function getProgressText(task: RoutineTaskDisplay): string {
  return `${task.completedCount}/${task.expectedCount}`;
}

// 判断任务是否全部完成
function isTaskFullyCompleted(task: RoutineTaskDisplay): boolean {
  return task.completedCount >= task.expectedCount;
}

// 判断任务是否部分完成
function isTaskPartiallyCompleted(task: RoutineTaskDisplay): boolean {
  return task.completedCount > 0 && task.completedCount < task.expectedCount;
}

// 常规任务总完成次数
const routineTotalCompleted = computed(() =>
  routineTasks.value.reduce((sum, t) => sum + t.completedCount, 0)
);

// 常规任务总预期次数
const routineTotalExpected = computed(() =>
  routineTasks.value.reduce((sum, t) => sum + t.expectedCount, 0)
);

// 常规任务进度百分比（上限100%）
const routineProgress = computed(() => {
  if (routineTotalExpected.value === 0) return 0;
  const progress = Math.round((routineTotalCompleted.value / routineTotalExpected.value) * 100);
  return Math.min(progress, 100); // 最大100%
});

// ==================== 周报编辑 ====================

// 总结编辑
const editingSummary = ref(false);
const summaryText = ref('');

function startEditSummary() {
  summaryText.value = report.value?.summary || '';
  editingSummary.value = true;
}

async function saveSummary() {
  if (!report.value) {
    await ensureReport();
  }
  if (!report.value) return;

  try {
    await api.updateWeeklyReport(
      report.value.id,
      report.value.title,
      summaryText.value || undefined,
      report.value.next_week_plan || undefined,
      report.value.status
    );
    await loadReportData();
    editingSummary.value = false;
    ElMessage.success('已保存');
  } catch (error) {
    ElMessage.error('保存失败');
  }
}

// 下周计划编辑
const editingPlan = ref(false);
const planText = ref('');

function startEditPlan() {
  planText.value = report.value?.next_week_plan || '';
  editingPlan.value = true;
}

async function savePlan() {
  if (!report.value) {
    await ensureReport();
  }
  if (!report.value) return;

  try {
    await api.updateWeeklyReport(
      report.value.id,
      report.value.title,
      report.value.summary || undefined,
      planText.value || undefined,
      report.value.status
    );
    await loadReportData();
    editingPlan.value = false;
    ElMessage.success('已保存');
  } catch (error) {
    ElMessage.error('保存失败');
  }
}

// ==================== 条目管理 ====================

const showAddEntry = ref(false);
const addEntryCategory = ref<WeeklyReportEntryCategory>('completed');
const addEntryContent = ref('');
const addEntryDescription = ref('');
const addEntryTaskCategory = ref('运营');
const addEntryPriorityLevel = ref<'low' | 'medium' | 'high'>('medium');
const addEntryProgress = ref(100);

function openAddEntry(category: WeeklyReportEntryCategory) {
  addEntryCategory.value = category;
  addEntryContent.value = '';
  addEntryDescription.value = '';
  addEntryTaskCategory.value = '运营';
  addEntryPriorityLevel.value = 'medium';
  addEntryProgress.value = 100;
  showAddEntry.value = true;
}

async function confirmAddEntry() {
  if (!addEntryContent.value.trim()) {
    ElMessage.warning('请输入任务标题');
    return;
  }

  await ensureReport();
  const weekStart = formatDate(currentWeekStart.value);

  try {
    await api.addReportEntry(
      weekStart,
      addEntryCategory.value,
      addEntryContent.value.trim(),
      addEntryDescription.value.trim() || undefined,
      addEntryTaskCategory.value,
      addEntryPriorityLevel.value,
      addEntryProgress.value
    );
    await loadReportData();
    showAddEntry.value = false;
    ElMessage.success('已添加');
  } catch (error) {
    ElMessage.error('添加失败');
  }
}

// 编辑条目
const showEditEntry = ref(false);
const editingEntry = ref<WeeklyReportEntry | null>(null);
const editEntryContent = ref('');
const editEntryDescription = ref('');
const editEntryTaskCategory = ref('运营');
const editEntryPriorityLevel = ref<'low' | 'medium' | 'high'>('medium');
const editEntryProgress = ref(100);

function startEditEntry(entry: WeeklyReportEntry) {
  editingEntry.value = entry;
  editEntryContent.value = entry.content;
  editEntryDescription.value = entry.description || '';
  editEntryTaskCategory.value = entry.task_category || '运营';
  editEntryPriorityLevel.value = entry.priority_level || 'medium';
  editEntryProgress.value = entry.progress;
  showEditEntry.value = true;
}

async function saveEditEntry() {
  if (!editingEntry.value) return;
  if (!editEntryContent.value.trim()) {
    ElMessage.warning('请输入任务标题');
    return;
  }

  try {
    await api.updateReportEntry(
      editingEntry.value.id,
      editEntryContent.value.trim(),
      editEntryDescription.value.trim() || null,
      editEntryTaskCategory.value,
      editingEntry.value.category,
      editEntryPriorityLevel.value,
      editingEntry.value.priority,
      editEntryProgress.value
    );
    await loadReportData();
    showEditEntry.value = false;
    editingEntry.value = null;
    ElMessage.success('已保存');
  } catch (error) {
    ElMessage.error('保存失败');
  }
}

async function deleteEntry(entry: WeeklyReportEntry) {
  try {
    await ElMessageBox.confirm('确定删除这条记录吗？', '确认删除', {
      confirmButtonText: '删除',
      cancelButtonText: '取消',
      type: 'warning',
    });
    await api.deleteReportEntry(entry.id);
    await loadReportData();
    ElMessage.success('已删除');
  } catch (e) {
    if (e !== 'cancel') ElMessage.error('删除失败');
  }
}

// ==================== 历史记录 ====================

const showHistoryDialog = ref(false);

function handleSelectHistory(report: WeeklyReport) {
  // 跳转到历史周报
  currentWeekStart.value = new Date(report.week_start);
  showHistoryDialog.value = false;
  loadReportData();
}

// ==================== AI 功能 ====================

const aiProcessing = ref(false);

// 检查是否有可用的 AI 配置
async function hasAIConfigured(): Promise<boolean> {
  const providers: AIProvider[] = ['qwen', 'deepseek', 'gemini'];
  for (const p of providers) {
    if (await checkApiKeyConfigured(p)) return true;
  }
  return false;
}

// 获取可用的 provider
async function getAvailableProvider(): Promise<AIProvider | null> {
  const providers: AIProvider[] = ['qwen', 'deepseek', 'gemini'];
  for (const p of providers) {
    if (await checkApiKeyConfigured(p)) return p;
  }
  return null;
}

// 调用 AI 对话
async function callAI(prompt: string): Promise<string> {
  const provider = await getAvailableProvider();
  if (!provider) {
    throw new Error('请先配置 AI API Key');
  }

  let result = '';
  const stream = chatStream(
    [{ role: 'user', content: prompt }],
    { provider }
  );

  for await (const chunk of stream) {
    result += chunk.content;
    if (chunk.done) break;
  }

  return result.trim();
}

// AI 一键生成本周总结和下周计划
async function aiGenerateReport() {
  if (!await hasAIConfigured()) {
    ElMessage.warning('请先配置 AI API Key');
    return;
  }

  // 常规任务完成情况
  const routineStatus = routineTasks.value.map(t =>
    `${t.content}（${t.completedCount}/${t.expectedCount}）`
  );

  // 本周完成的所有事项
  const completedItems = allCompletedItems.value.map(item => item.content);

  if (routineStatus.length === 0 && completedItems.length === 0) {
    ElMessage.warning('请先添加工作内容');
    return;
  }

  const prompt = `根据本周工作内容，生成周报：

本周工作数据：
${routineStatus.length > 0 ? routineStatus.map(s => `- ${s}`).join('\n') : ''}
${completedItems.length > 0 ? completedItems.map(c => `- ${c}`).join('\n') : ''}

格式要求（非常重要）：
1. 每条必须换行，以"- "开头
2. 每条20-30字，概括具体成果
3. 3-5条即可，突出重点

严格按以下格式返回：

【本周总结】
- 完成xxx工作，达成xxx效果
- 处理xxx事项，保障xxx运营
- 优化xxx内容，提升xxx指标

【下周计划】
- 继续推进xxx工作
- 重点关注xxx事项
- 完成xxx目标`;

  aiProcessing.value = true;
  // 显示加载提示
  const loadingMsg = ElMessage({
    message: 'AI 正在生成中...',
    type: 'info',
    duration: 0, // 不自动关闭
    icon: 'Loading',
  });

  try {
    const result = await callAI(prompt);

    // 解析返回结果
    const summaryMatch = result.match(/【本周总结】\s*([\s\S]*?)(?=【下周计划】|$)/);
    const planMatch = result.match(/【下周计划】\s*([\s\S]*?)$/);

    // 格式化函数：确保每个条目独占一行
    const formatBulletPoints = (text: string): string => {
      // 统一处理各种可能的分隔符格式
      // 1. 先把所有可能的条目分隔符统一转为特殊标记
      let normalized = text
        .replace(/\n\s*-\s*/g, '|||ITEM|||')  // 换行后跟 -
        .replace(/\s+-\s+/g, '|||ITEM|||')     // 空格-空格 分隔
        .replace(/^-\s*/g, '|||ITEM|||');      // 开头的 -

      // 2. 按标记分割成数组
      const items = normalized
        .split('|||ITEM|||')
        .map(s => s.trim())
        .filter(s => s.length > 0);

      // 3. 重新组装为标准格式
      if (items.length === 0) return text.trim();
      return items.map(item => `- ${item}`).join('\n');
    };

    if (summaryMatch && summaryMatch[1]) {
      summaryText.value = formatBulletPoints(summaryMatch[1]);
      editingSummary.value = true;
    }

    if (planMatch && planMatch[1]) {
      planText.value = formatBulletPoints(planMatch[1]);
      editingPlan.value = true;
    }

    loadingMsg.close();
    ElMessage.success('已生成本周总结和下周计划');
  } catch (error) {
    loadingMsg.close();
    ElMessage.error('AI 生成失败');
  } finally {
    aiProcessing.value = false;
  }
}

// ==================== 导出功能 ====================

// 导出为 Excel
async function exportExcel() {
  if (!report.value && !entries.value && !hasCompletedItems.value) {
    ElMessage.warning('没有可导出的内容');
    return;
  }

  const title = report.value?.title || weekLabel.value + '工作周报';

  // 创建工作簿
  const wb = XLSX.utils.book_new();

  // 准备数据
  const data: (string | number)[][] = [];

  // 标题行
  data.push([title]);
  data.push([dateRangeLabel.value]);
  data.push([]); // 空行

  // 常规任务检查
  if (routineTasks.value.length > 0) {
    data.push(['【常规任务检查】']);
    data.push(['任务', '频率', '完成情况']);
    routineTasks.value.forEach(task => {
      data.push([
        task.content,
        getFrequencyLabel(task),
        `${task.completedCount}/${task.expectedCount}`
      ]);
    });
    data.push([]);
  }

  // 本周完成
  if (hasCompletedItems.value) {
    data.push(['【本周完成】']);
    data.push(['内容', '描述', '分类', '优先级', '进度', '来源', '日期']);
    allCompletedItems.value.forEach(item => {
      const sourceLabel = item.source === 'quicknotes' ? '备忘录' :
                         item.source === 'optimization' ? '优化' : '手动';
      const priorityLabel = item.priorityLevel === 'high' ? '高' :
                           item.priorityLevel === 'low' ? '低' : '中';
      const progressText = item.progress < 100 ? `${item.progress}%` : '已完成';
      data.push([
        item.content,
        item.description || '',
        item.taskCategory || '',
        priorityLabel,
        progressText,
        sourceLabel,
        item.date || ''
      ]);
    });
    data.push([]);
  }

  // 本周总结
  if (report.value?.summary) {
    data.push(['【本周总结】']);
    data.push([report.value.summary]);
    data.push([]);
  }

  // 下周计划
  if (report.value?.next_week_plan) {
    data.push(['【下周计划】']);
    data.push([report.value.next_week_plan]);
  }

  // 创建工作表
  const ws = XLSX.utils.aoa_to_sheet(data);

  // 设置列宽
  ws['!cols'] = [
    { wch: 40 },  // 内容
    { wch: 30 },  // 描述
    { wch: 10 },  // 分类
    { wch: 8 },   // 优先级
    { wch: 10 },  // 进度
    { wch: 10 },  // 来源
    { wch: 12 }   // 日期
  ];

  // 添加工作表到工作簿
  XLSX.utils.book_append_sheet(wb, ws, '周报');

  // 生成文件
  const wbout = XLSX.write(wb, { bookType: 'xlsx', type: 'array' });

  // 保存文件
  try {
    const filePath = await save({
      defaultPath: `${title}.xlsx`,
      filters: [{ name: 'Excel', extensions: ['xlsx'] }]
    });

    if (filePath) {
      await writeFile(filePath, new Uint8Array(wbout));
      ElMessage.success('导出成功');
    }
  } catch (error) {
    console.error('导出失败:', error);
    ElMessage.error('导出失败');
  }
}

// 复制 Markdown 到剪贴板
async function exportMarkdown() {
  if (!report.value && !entries.value) {
    ElMessage.warning('没有可导出的内容');
    return;
  }

  const title = report.value?.title || weekLabel.value + '工作周报';
  const dateRange = dateRangeLabel.value;

  let md = `# ${title}\n\n`;
  md += `> ${dateRange}\n\n`;

  // 常规任务
  if (routineTasks.value.length > 0) {
    md += `## 常规任务检查\n\n`;
    routineTasks.value.forEach(task => {
      const check = task.completedCount >= task.expectedCount ? '✓' : '○';
      md += `- [${check}] ${task.content} (${task.completedCount}/${task.expectedCount})\n`;
    });
    md += '\n';
  }

  if (hasCompletedItems.value) {
    md += `## 本周完成\n\n`;
    allCompletedItems.value.forEach(item => {
      const dateStr = item.date ? ` (${item.date})` : '';
      const progressStr = item.progress < 100 ? ` [${item.progress}%]` : '';
      const categoryStr = item.taskCategory ? ` 【${item.taskCategory}】` : '';
      const priorityStr = item.priorityLevel === 'high' ? ' ⚡' : '';
      md += `- ${item.content}${categoryStr}${priorityStr}${progressStr}${dateStr}\n`;
      if (item.description) {
        md += `  > ${item.description}\n`;
      }
    });
    md += '\n';
  }

  if (report.value?.summary) {
    md += `## 本周总结\n\n${report.value.summary}\n\n`;
  }

  if (report.value?.next_week_plan) {
    md += `## 下周计划\n\n${report.value.next_week_plan}\n`;
  }

  // 复制到剪贴板
  try {
    await navigator.clipboard.writeText(md);
    ElMessage.success('Markdown 已复制到剪贴板');
  } catch {
    ElMessage.error('复制失败');
  }
}

// ==================== 生命周期 ====================

onMounted(() => {
  loadReportData();
  loadRoutineTasks();
  startAutoRefresh();
});

onUnmounted(() => {
  stopAutoRefresh();
});

// 合并所有"本周完成"的内容（手动条目 + 备忘录 + 优化事件）
interface CompletedItem {
  id: number | string;
  content: string;
  description?: string; // 详细描述
  source: 'manual' | 'quicknotes' | 'optimization';
  date?: string;
  progress: number; // 进度百分比 0-100
  taskCategory?: string; // 任务分类
  priorityLevel?: string; // 优先级
  isEntry?: boolean; // 是否为手动添加的条目（可编辑/删除）
  originalEntry?: WeeklyReportEntry;
}

const allCompletedItems = computed<CompletedItem[]>(() => {
  const items: CompletedItem[] = [];

  // 1. 手动添加的条目
  if (entries.value?.completed) {
    for (const entry of entries.value.completed) {
      items.push({
        id: entry.id,
        content: entry.content,
        description: entry.description || undefined,
        source: entry.source as 'manual' | 'quicknotes' | 'optimization',
        progress: entry.progress,
        taskCategory: entry.task_category || undefined,
        priorityLevel: entry.priority_level || undefined,
        isEntry: true,
        originalEntry: entry,
      });
    }
  }

  // 2. 备忘录已完成任务（排除已在常规任务板块显示的内容）
  // 获取所有常规任务的内容（用于排除）
  const routineTaskContents = new Set(routineTasks.value.map(t => t.content));

  // 过滤掉：有 repeat_type 的 或 内容已在常规任务中的
  const nonRepeatNotes = quickNotesCompleted.value.filter(note =>
    !note.repeat_type && !routineTaskContents.has(note.content)
  );

  // 按内容去重，保留最新完成的
  const seenNoteContents = new Set<string>();
  const sortedNotes = [...nonRepeatNotes].sort((a, b) => {
    const dateA = a.completed_at || '';
    const dateB = b.completed_at || '';
    return dateB.localeCompare(dateA);
  });

  for (const note of sortedNotes) {
    if (seenNoteContents.has(note.content)) continue;
    seenNoteContents.add(note.content);

    items.push({
      id: `note-${note.id}`,
      content: note.content,
      source: 'quicknotes',
      date: note.completed_at?.split(' ')[0],
      progress: 100,
      isEntry: false,
    });
  }

  // 3. 优化事件（自动导入，包含详细描述）
  for (const event of optimizationEvents.value) {
    const typeConfig = EVENT_SUB_TYPES[event.event_type] || {};
    const subTypes = event.event_sub_type ? JSON.parse(event.event_sub_type) : [];
    const subTypeText = subTypes.map((st: string) => typeConfig[st]?.label || st).join('、');
    const typeLabel = event.event_type === 'listing' ? '文案' : '广告';

    items.push({
      id: `event-${event.id}`,
      content: `[${typeLabel}${subTypeText ? '-' + subTypeText : ''}] ${event.title}`,
      description: event.description,
      source: 'optimization',
      date: event.event_date,
      progress: 100,
      isEntry: false,
    });
  }

  return items;
});

// 检查是否有任何完成的内容
const hasCompletedItems = computed(() => allCompletedItems.value.length > 0);

</script>

<template>
  <div class="weekly-report-container">
    <!-- 头部 -->
    <header class="report-header">
      <div class="week-selector">
        <button class="nav-btn" @click="prevWeek">
          <el-icon><ArrowLeft /></el-icon>
        </button>
        <div class="week-info">
          <span class="week-label">{{ weekLabel }}</span>
          <span class="date-range">{{ dateRangeLabel }}</span>
        </div>
        <button class="nav-btn" @click="nextWeek">
          <el-icon><ArrowRight /></el-icon>
        </button>
        <button class="today-btn" @click="goToCurrentWeek">
          <el-icon><Clock /></el-icon>
          本周
        </button>
      </div>

      <div class="header-actions">
        <button class="action-btn" :class="{ loading: aiProcessing }" @click="aiGenerateReport" :disabled="aiProcessing">
          <el-icon :class="{ 'is-loading': aiProcessing }"><MagicStick /></el-icon>
          {{ aiProcessing ? '生成中...' : 'AI 生成' }}
        </button>

        <el-dropdown trigger="click">
          <button class="action-btn">
            <el-icon><Download /></el-icon>
            导出
          </button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item @click="exportExcel">导出 Excel</el-dropdown-item>
              <el-dropdown-item @click="exportMarkdown">复制 Markdown</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>

        <button class="action-btn" @click="showHistoryDialog = true">
          <el-icon><FolderOpened /></el-icon>
          历史
        </button>
      </div>
    </header>

    <!-- 主内容区域 -->
    <main class="report-main" v-loading="loading">
      <!-- 常规任务检查板块 -->
      <section v-if="routineTasks.length > 0" class="routine-section" v-loading="routineLoading">
        <div class="routine-header">
          <div class="routine-title">
            <span class="title-icon routine"><el-icon><Refresh /></el-icon></span>
            <h2>常规任务检查</h2>
            <span class="routine-stats">{{ routineTotalCompleted }}/{{ routineTotalExpected }}</span>
          </div>
          <div class="routine-actions">
            <span class="progress-text">{{ routineProgress }}%</span>
          </div>
        </div>

        <!-- 进度条 -->
        <div class="routine-progress-bar">
          <div class="progress-fill" :style="{ width: routineProgress + '%' }"></div>
        </div>

        <!-- 任务网格（只读展示） -->
        <div class="routine-grid">
          <div
            v-for="task in routineTasks"
            :key="task.content"
            class="routine-item"
            :class="{
              completed: isTaskFullyCompleted(task),
              partial: isTaskPartiallyCompleted(task)
            }"
          >
            <span class="routine-checkbox">
              <el-icon v-if="isTaskFullyCompleted(task)" class="check-mark"><Check /></el-icon>
              <span v-else-if="isTaskPartiallyCompleted(task)" class="partial-mark">―</span>
            </span>
            <div class="routine-content">
              <span class="routine-name" :class="{ 'line-through': isTaskFullyCompleted(task) }">
                {{ task.content }}
              </span>
              <div class="routine-meta">
                <span class="routine-freq">{{ getFrequencyLabel(task) }}</span>
                <span class="routine-progress" :class="{
                  'full': isTaskFullyCompleted(task),
                  'partial': isTaskPartiallyCompleted(task),
                  'zero': task.completedCount === 0
                }">{{ getProgressText(task) }}</span>
              </div>
            </div>
          </div>
        </div>

      </section>

      <!-- 双栏布局容器 -->
      <div class="two-column-layout">
        <!-- 左侧：本周完成 -->
        <section class="left-panel">
        <div class="panel-header">
          <div class="panel-title">
            <span class="title-icon completed"><el-icon><Check /></el-icon></span>
            <h2>本周完成</h2>
            <span class="item-count" v-if="hasCompletedItems">{{ allCompletedItems.length }}</span>
          </div>
          <button class="add-btn" @click="openAddEntry('completed')">
            <el-icon><Plus /></el-icon>
            添加
          </button>
        </div>

        <div class="completed-list">
          <div v-if="hasCompletedItems" class="task-cards">
            <div
              v-for="item in allCompletedItems"
              :key="item.id"
              class="task-card"
              :class="{ 'has-description': item.description }"
            >
              <!-- 显示模式 -->
                <div class="task-row">
                  <span v-if="item.progress === 100" class="check-icon"><el-icon><Check /></el-icon></span>
                  <el-progress
                    v-else
                    type="circle"
                    :percentage="item.progress"
                    :width="22"
                    :stroke-width="3"
                    :show-text="false"
                    class="mini-circle-progress"
                  />
                  <div class="task-content">
                    <span class="task-title">{{ item.content }}</span>
                  </div>
                  <div class="task-meta">
                    <span v-if="item.priorityLevel === 'high'" class="priority-tag high">高</span>
                    <span v-if="item.taskCategory" class="category-tag">{{ item.taskCategory }}</span>
                    <span v-if="item.source === 'quicknotes'" class="source-tag memo">备忘录</span>
                    <span v-else-if="item.source === 'optimization'" class="source-tag optimize">优化</span>
                    <span v-if="item.date" class="task-date">{{ item.date }}</span>
                  </div>
                  <div v-if="item.isEntry" class="task-actions">
                    <button class="icon-btn" @click="startEditEntry(item.originalEntry!)">
                      <el-icon><Edit /></el-icon>
                    </button>
                    <button class="icon-btn danger" @click="deleteEntry(item.originalEntry!)">
                      <el-icon><Delete /></el-icon>
                    </button>
                  </div>
                </div>
                <!-- 详细描述（可折叠） -->
                <div v-if="item.description" class="task-description">
                  {{ item.description }}
                </div>
            </div>
          </div>
          <div v-else class="empty-state">
            <div class="empty-icon"><el-icon><List /></el-icon></div>
            <p>本周暂无完成事项</p>
            <span class="empty-hint">备忘录完成任务和优化事件会自动同步</span>
          </div>
        </div>
      </section>

      <!-- 右侧：总结、风险、计划 -->
      <aside class="right-panel">
        <!-- 本周总结 -->
        <section class="side-card summary-card">
          <div class="side-header">
            <div class="side-title">
              <span class="title-icon summary"><el-icon><Document /></el-icon></span>
              <h3>本周总结</h3>
            </div>
            <button v-if="!editingSummary" class="icon-btn" @click="startEditSummary">
              <el-icon><Edit /></el-icon>
            </button>
          </div>
          <div class="side-content">
            <template v-if="editingSummary">
              <el-input
                v-model="summaryText"
                type="textarea"
                :rows="4"
                placeholder="输入本周工作总结..."
                resize="none"
              />
              <div class="edit-actions">
                <el-button size="small" @click="editingSummary = false">取消</el-button>
                <el-button size="small" type="primary" @click="saveSummary">保存</el-button>
              </div>
            </template>
            <template v-else>
              <p v-if="report?.summary" class="content-text pre-wrap">{{ report.summary }}</p>
              <p v-else class="placeholder-text">点击编辑添加本周总结...</p>
            </template>
          </div>
        </section>

        <!-- 下周计划 -->
        <section class="side-card plan-card">
          <div class="side-header">
            <div class="side-title">
              <span class="title-icon plan"><el-icon><Calendar /></el-icon></span>
              <h3>下周计划</h3>
            </div>
            <button v-if="!editingPlan" class="icon-btn" @click="startEditPlan">
              <el-icon><Edit /></el-icon>
            </button>
          </div>
          <div class="side-content">
            <template v-if="editingPlan">
              <el-input
                v-model="planText"
                type="textarea"
                :rows="4"
                placeholder="输入下周工作计划..."
                resize="none"
              />
              <div class="edit-actions">
                <el-button size="small" @click="editingPlan = false">取消</el-button>
                <el-button size="small" type="primary" @click="savePlan">保存</el-button>
              </div>
            </template>
            <template v-else>
              <p v-if="report?.next_week_plan" class="content-text pre-wrap">{{ report.next_week_plan }}</p>
              <p v-else class="placeholder-text">点击编辑或使用 AI 生成下周计划...</p>
            </template>
          </div>
        </section>
      </aside>
      </div>
    </main>

    <!-- 添加条目对话框 -->
    <el-dialog
      v-model="showAddEntry"
      title="添加任务"
      width="500px"
      class="add-entry-dialog"
    >
      <div class="form-group">
        <label class="form-label">任务标题</label>
        <el-input
          v-model="addEntryContent"
          placeholder="输入任务标题..."
        />
      </div>
      <div class="form-group">
        <label class="form-label">任务描述</label>
        <el-input
          v-model="addEntryDescription"
          type="textarea"
          :rows="3"
          placeholder="描述任务详情..."
        />
      </div>
      <div class="form-row">
        <div class="form-group half">
          <label class="form-label">优先级</label>
          <el-select v-model="addEntryPriorityLevel" style="width: 100%">
            <el-option label="低优先级" value="low" />
            <el-option label="中优先级" value="medium" />
            <el-option label="高优先级" value="high" />
          </el-select>
        </div>
        <div class="form-group half">
          <label class="form-label">分类</label>
          <el-select v-model="addEntryTaskCategory" style="width: 100%">
            <el-option label="运营" value="运营" />
            <el-option label="开发" value="开发" />
            <el-option label="设计" value="设计" />
            <el-option label="会议" value="会议" />
            <el-option label="学习" value="学习" />
            <el-option label="其他" value="其他" />
          </el-select>
        </div>
      </div>
      <div class="form-group">
        <label class="form-label">完成进度 ({{ addEntryProgress }}%)</label>
        <el-slider
          v-model="addEntryProgress"
          :min="0"
          :max="100"
          :step="5"
        />
      </div>
      <template #footer>
        <el-button @click="showAddEntry = false">取消</el-button>
        <el-button type="primary" @click="confirmAddEntry">保存</el-button>
      </template>
    </el-dialog>

    <!-- 编辑条目对话框 -->
    <el-dialog
      v-model="showEditEntry"
      title="编辑任务"
      width="500px"
      class="add-entry-dialog"
    >
      <div class="form-group">
        <label class="form-label">任务标题</label>
        <el-input
          v-model="editEntryContent"
          placeholder="输入任务标题..."
        />
      </div>
      <div class="form-group">
        <label class="form-label">任务描述</label>
        <el-input
          v-model="editEntryDescription"
          type="textarea"
          :rows="3"
          placeholder="描述任务详情..."
        />
      </div>
      <div class="form-row">
        <div class="form-group half">
          <label class="form-label">优先级</label>
          <el-select v-model="editEntryPriorityLevel" style="width: 100%">
            <el-option label="低优先级" value="low" />
            <el-option label="中优先级" value="medium" />
            <el-option label="高优先级" value="high" />
          </el-select>
        </div>
        <div class="form-group half">
          <label class="form-label">分类</label>
          <el-select v-model="editEntryTaskCategory" style="width: 100%">
            <el-option label="运营" value="运营" />
            <el-option label="开发" value="开发" />
            <el-option label="设计" value="设计" />
            <el-option label="会议" value="会议" />
            <el-option label="学习" value="学习" />
            <el-option label="其他" value="其他" />
          </el-select>
        </div>
      </div>
      <div class="form-group">
        <label class="form-label">完成进度 ({{ editEntryProgress }}%)</label>
        <el-slider
          v-model="editEntryProgress"
          :min="0"
          :max="100"
          :step="5"
        />
      </div>
      <template #footer>
        <el-button @click="showEditEntry = false">取消</el-button>
        <el-button type="primary" @click="saveEditEntry">保存</el-button>
      </template>
    </el-dialog>

    <!-- 历史周报对话框 -->
    <ReportHistoryDialog
      v-model="showHistoryDialog"
      @select="handleSelectHistory"
    />
  </div>
</template>

<style scoped>
/* ========== 容器与基础 ========== */
.weekly-report-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

/* ========== 头部 ========== */
.report-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 28px;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(12px);
  border-bottom: 1px solid rgba(0, 0, 0, 0.06);
  position: sticky;
  top: 0;
  z-index: 10;
}

.week-selector {
  display: flex;
  align-items: center;
  gap: 8px;
}

.nav-btn {
  width: 36px;
  height: 36px;
  border: none;
  background: transparent;
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #64748b;
  transition: all 0.2s ease-out;
}

.nav-btn:hover {
  background: #f1f5f9;
  color: #334155;
}

.week-info {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 0 16px;
  min-width: 160px;
}

.week-label {
  font-size: 20px;
  font-weight: 700;
  color: #1e293b;
  letter-spacing: -0.02em;
}

.date-range {
  font-size: 12px;
  color: #64748b;  /* 提高对比度 */
  margin-top: 2px;
}

.today-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border: 1px solid #e2e8f0;
  background: white;
  border-radius: 8px;
  font-size: 13px;
  color: #475569;
  cursor: pointer;
  transition: all 0.2s;
  margin-left: 8px;
}

.today-btn:hover {
  border-color: #3b82f6;
  color: #3b82f6;
  background: #eff6ff;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border: none;
  background: #f1f5f9;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  color: #475569;
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:hover {
  background: #e2e8f0;
  color: #1e293b;
}

.action-btn.loading {
  opacity: 0.7;
  cursor: not-allowed;
}

.action-btn .is-loading {
  animation: rotating 1.5s linear infinite;
}

@keyframes rotating {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* ========== 主内容区 ========== */
.report-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 20px;
  padding: 24px 28px;
  overflow-y: auto;
}

/* ========== 常规任务板块 ========== */
.routine-section {
  background: white;
  border-radius: 16px;
  padding: 20px 24px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04), 0 4px 12px rgba(0, 0, 0, 0.03);
}

.routine-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.routine-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.routine-title h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 700;
  color: #1e293b;
}

.title-icon.routine {
  width: 32px;
  height: 32px;
  background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  color: #d97706;
}

.routine-stats {
  background: #f1f5f9;
  color: #64748b;
  font-size: 13px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 20px;
}

.routine-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.routine-actions .progress-text {
  font-size: 14px;
  font-weight: 600;
  color: #16a34a;
}

.routine-progress-bar {
  height: 6px;
  background: #e2e8f0;
  border-radius: 3px;
  margin-bottom: 16px;
  overflow: hidden;
}

.routine-progress-bar .progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #22c55e 0%, #16a34a 100%);
  border-radius: 3px;
  transition: width 0.3s ease-out;
}

.routine-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 10px;
}

.routine-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 14px;
  background: #fafbfc;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  transition: all 0.2s;
}

.routine-item.completed {
  background: #f0fdf4;
  border-color: #bbf7d0;
}

.routine-item.partial {
  background: #fffbeb;
  border-color: #fde68a;
}

.routine-checkbox {
  width: 20px;
  height: 20px;
  border: 2px solid #cbd5e1;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.2s;
}

.routine-item.completed .routine-checkbox {
  background: #22c55e;
  border-color: #22c55e;
}

.routine-item.partial .routine-checkbox {
  background: #f59e0b;
  border-color: #f59e0b;
}

.routine-checkbox .check-mark {
  color: white;
  font-size: 14px;
}

.routine-checkbox .partial-mark {
  color: white;
  font-size: 14px;
  font-weight: bold;
  line-height: 1;
}

.routine-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.routine-name {
  font-size: 13px;
  color: #334155;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.routine-name.line-through {
  text-decoration: line-through;
  color: #94a3b8;
}

.routine-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 2px;
}

.routine-freq {
  font-size: 11px;
  color: #64748b;  /* 提高对比度 */
}

.routine-progress {
  font-size: 11px;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 4px;
}

.routine-progress.full {
  background: #dcfce7;
  color: #16a34a;
}

.routine-progress.partial {
  background: #fef3c7;
  color: #d97706;
}

.routine-progress.zero {
  background: #f1f5f9;
  color: #94a3b8;
}


/* ========== 双栏布局 ========== */
.two-column-layout {
  display: grid;
  grid-template-columns: 1fr 380px;
  gap: 24px;
  flex: 1;
  min-height: 0;
}

/* ========== 左侧面板 ========== */
.left-panel {
  display: flex;
  flex-direction: column;
  background: white;
  border-radius: 16px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04), 0 4px 12px rgba(0, 0, 0, 0.03);
  overflow: hidden;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid #f1f5f9;
}

.panel-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.panel-title h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
  color: #1e293b;
}

.title-icon {
  width: 32px;
  height: 32px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
}

.title-icon .el-icon {
  font-size: 18px;
}

.title-icon.completed {
  background: linear-gradient(135deg, #dcfce7 0%, #bbf7d0 100%);
  color: #16a34a;
}

.title-icon.summary {
  background: linear-gradient(135deg, #dbeafe 0%, #bfdbfe 100%);
  color: #2563eb;
}

.title-icon.risk {
  background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
  color: #d97706;
}

.title-icon.plan {
  background: linear-gradient(135deg, #e0e7ff 0%, #c7d2fe 100%);
  color: #4f46e5;
}

.item-count {
  background: #f1f5f9;
  color: #64748b;
  font-size: 13px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 20px;
}

.add-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border: 1px dashed #cbd5e1;
  background: transparent;
  border-radius: 8px;
  font-size: 13px;
  color: #64748b;
  cursor: pointer;
  transition: all 0.2s;
}

.add-btn:hover {
  border-color: #3b82f6;
  color: #3b82f6;
  background: #eff6ff;
}

.completed-list {
  flex: 1;
  overflow-y: auto;
  padding: 16px 24px;
}

/* ========== 任务卡片 ========== */
.task-cards {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.task-card {
  background: #fafbfc;
  border: 1px solid #f1f5f9;
  border-radius: 12px;
  padding: 16px;
  transition: all 0.2s ease-out;
  cursor: pointer;
}

.task-card:hover {
  border-color: #e2e8f0;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
}

.task-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.check-icon {
  width: 22px;
  height: 22px;
  background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%);
  color: white;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  margin-top: 1px;
}

.check-icon .el-icon {
  font-size: 14px;
}

.task-content {
  flex: 1;
  min-width: 0;
}

.task-title {
  font-size: 14px;
  color: #334155;
  line-height: 1.5;
  word-break: break-word;
}

.task-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.source-tag {
  font-size: 11px;
  padding: 3px 8px;
  border-radius: 4px;
  font-weight: 500;
}

.source-tag.memo {
  background: #dcfce7;
  color: #16a34a;
}

.source-tag.optimize {
  background: #fef3c7;
  color: #d97706;
}

.priority-tag {
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 4px;
  font-weight: 600;
}

.priority-tag.high {
  background: #fee2e2;
  color: #dc2626;
}

.category-tag {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  background: #e0e7ff;
  color: #4f46e5;
  font-weight: 500;
}

.task-date {
  font-size: 12px;
  color: #64748b;  /* 提高对比度 */
}

.task-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.2s;
}

.task-card:hover .task-actions {
  opacity: 1;
}

.task-description {
  margin-top: 12px;
  padding: 12px;
  background: white;
  border-radius: 8px;
  font-size: 13px;
  color: #64748b;
  line-height: 1.6;
  white-space: pre-wrap;
  border-left: 3px solid #e2e8f0;
}

/* ========== 图标按钮 ========== */
.icon-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #94a3b8;
  transition: all 0.15s;
}

.icon-btn:hover {
  background: #f1f5f9;
  color: #475569;
}

.icon-btn.danger:hover {
  background: #fef2f2;
  color: #ef4444;
}

.icon-btn.small {
  width: 24px;
  height: 24px;
}

/* ========== 空状态 ========== */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  text-align: center;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
  color: #94a3b8;
}

.empty-icon .el-icon {
  font-size: 48px;
}

.empty-state p {
  margin: 0;
  font-size: 15px;
  color: #64748b;
  font-weight: 500;
}

.empty-state .empty-hint {
  font-size: 13px;
  color: #64748b;  /* 提高对比度 */
  margin-top: 6px;
}

/* ========== 右侧面板 ========== */
.right-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
  overflow-y: auto;
}

.side-card {
  background: white;
  border-radius: 16px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04), 0 4px 12px rgba(0, 0, 0, 0.03);
  overflow: hidden;
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.side-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid #f1f5f9;
  flex-shrink: 0;
}

.side-title {
  display: flex;
  align-items: center;
  gap: 10px;
}

.side-title h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: #1e293b;
}

.side-content {
  padding: 16px 20px;
  flex: 1;
  overflow-y: auto;
}

.content-text {
  margin: 0;
  font-size: 13px;
  color: #475569;
  line-height: 1.7;
}

.placeholder-text {
  margin: 0;
  font-size: 13px;
  color: #64748b;  /* 提高对比度 */
  font-style: italic;
}

.pre-wrap {
  white-space: pre-wrap;
}

/* ========== 风险列表 ========== */
.risk-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.risk-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px 12px;
  background: #fffbeb;
  border-radius: 8px;
  border-left: 3px solid #f59e0b;
}

.risk-dot {
  width: 6px;
  height: 6px;
  background: #f59e0b;
  border-radius: 50%;
  flex-shrink: 0;
  margin-top: 6px;
}

.risk-text {
  flex: 1;
  font-size: 13px;
  color: #92400e;
  line-height: 1.5;
}

.risk-actions {
  display: flex;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.2s;
}

.risk-item:hover .risk-actions {
  opacity: 1;
}

/* ========== 编辑模式 ========== */
.edit-mode {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.edit-btns {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.edit-btns.inline {
  margin-top: 8px;
}

.edit-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 12px;
}

/* ========== 进度条相关 ========== */
.edit-progress {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 8px;
}

.edit-progress .progress-label {
  font-size: 13px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.edit-progress .progress-value {
  font-size: 13px;
  color: var(--text-primary);
  font-weight: 500;
  min-width: 40px;
  text-align: right;
}

.edit-progress .el-slider {
  flex: 1;
}

/* 添加对话框表单样式 */
.form-group {
  margin-bottom: 20px;
}

.form-group:last-child {
  margin-bottom: 0;
}

.form-group.half {
  flex: 1;
}

.form-row {
  display: flex;
  gap: 16px;
}

.form-label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  margin-bottom: 8px;
}

.add-entry-dialog .el-slider {
  margin-top: 8px;
  padding: 0 6px;
}

.mini-circle-progress {
  flex-shrink: 0;
  margin-right: 2px;
}

/* ========== 响应式 ========== */
@media (max-width: 1100px) {
  .two-column-layout {
    grid-template-columns: 1fr;
  }

  .left-panel {
    max-height: none;
  }

  .right-panel {
    flex-direction: row;
    flex-wrap: wrap;
    overflow: visible;
  }

  .side-card {
    flex: 1 1 280px;
    min-height: 200px;
  }

  .routine-grid {
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  }
}

@media (max-width: 768px) {
  .report-header {
    flex-direction: column;
    gap: 16px;
    padding: 16px;
  }

  .header-actions {
    width: 100%;
    justify-content: center;
    flex-wrap: wrap;
  }

  .report-main {
    padding: 16px;
    gap: 16px;
  }

  .right-panel {
    flex-direction: column;
  }

  .side-card {
    flex: none;
  }

  .routine-grid {
    grid-template-columns: 1fr;
  }
}

/* ========== 对话框 ========== */
.add-entry-dialog :deep(.el-dialog__body) {
  padding: 20px 24px;
}

/* ========== 暗色模式 ========== */
html.dark .weekly-report-container {
  background: linear-gradient(135deg, #1a1a1a 0%, #242424 100%);
}

html.dark .report-header {
  background: rgba(36, 36, 36, 0.95);
  border-bottom-color: rgba(255, 255, 255, 0.08);
}

html.dark .nav-btn {
  color: #a3a3a3;
}

html.dark .nav-btn:hover {
  background: #2c2c2c;
  color: #e5e5e5;
}

html.dark .week-label {
  color: #e5e5e5;
}

html.dark .date-range {
  color: #737373;
}

html.dark .today-btn {
  border-color: #3a3a3a;
  background: #242424;
  color: #a3a3a3;
}

html.dark .today-btn:hover {
  border-color: #3b82f6;
  color: #60a5fa;
  background: rgba(59, 130, 246, 0.15);
}

html.dark .action-btn {
  background: #2c2c2c;
  color: #a3a3a3;
}

html.dark .action-btn:hover {
  background: #3a3a3a;
  color: #e5e5e5;
}

html.dark .routine-section,
html.dark .left-panel,
html.dark .side-card {
  background: #242424;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2), 0 4px 12px rgba(0, 0, 0, 0.15);
}

html.dark .routine-title h2,
html.dark .panel-title h2,
html.dark .side-title h3 {
  color: #e5e5e5;
}

html.dark .panel-header,
html.dark .side-header,
html.dark .routine-header {
  border-bottom-color: #3a3a3a;
}

html.dark .routine-stats,
html.dark .item-count {
  background: #2c2c2c;
  color: #a3a3a3;
}

html.dark .routine-name {
  color: #d4d4d4;
}

html.dark .routine-name.line-through {
  color: #737373;
}

html.dark .routine-freq {
  color: #737373;
}

html.dark .routine-progress.zero {
  background: #2c2c2c;
  color: #737373;
}

html.dark .routine-progress-bar {
  background: #3a3a3a;
}

html.dark .add-btn {
  border-color: #4a4a4a;
  color: #a3a3a3;
}

html.dark .add-btn:hover {
  border-color: #60a5fa;
  background: rgba(59, 130, 246, 0.1);
  color: #60a5fa;
}

html.dark .task-card {
  background: #2c2c2c;
}

html.dark .task-card:hover {
  background: #333333;
}

html.dark .task-title {
  color: #e5e5e5;
}

html.dark .task-date {
  color: #737373;
}

html.dark .task-description {
  background: #1a1a1a;
  color: #a3a3a3;
  border-left-color: #3a3a3a;
}

html.dark .priority-tag.high {
  background: rgba(220, 38, 38, 0.2);
  color: #f87171;
}

html.dark .category-tag {
  background: rgba(79, 70, 229, 0.2);
  color: #a5b4fc;
}

html.dark .icon-btn {
  color: #737373;
}

html.dark .icon-btn:hover {
  background: #3a3a3a;
  color: #a3a3a3;
}

html.dark .icon-btn.danger:hover {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
}

html.dark .empty-state p {
  color: #a3a3a3;
}

html.dark .empty-state .empty-hint {
  color: #737373;
}

html.dark .content-text {
  color: #a3a3a3;
}

html.dark .placeholder-text {
  color: #737373;
}

html.dark .risk-item {
  background: rgba(245, 158, 11, 0.1);
  border-left-color: #f59e0b;
}

html.dark .risk-text {
  color: #fbbf24;
}

html.dark .form-label {
  color: #e5e5e5;
}

html.dark .routine-card {
  background: #2c2c2c;
  border-color: #3a3a3a;
}

html.dark .routine-card:hover {
  background: #333333;
  border-color: #4a4a4a;
}

html.dark .routine-card.completed {
  background: rgba(22, 163, 74, 0.1);
  border-color: rgba(22, 163, 74, 0.3);
}

html.dark .title-icon.completed {
  background: linear-gradient(135deg, rgba(22, 163, 74, 0.3) 0%, rgba(34, 197, 94, 0.3) 100%);
  color: #4ade80;
}

html.dark .title-icon.summary {
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.3) 0%, rgba(96, 165, 250, 0.3) 100%);
  color: #60a5fa;
}

html.dark .title-icon.risk {
  background: linear-gradient(135deg, rgba(245, 158, 11, 0.3) 0%, rgba(251, 191, 36, 0.3) 100%);
  color: #fbbf24;
}

html.dark .title-icon.plan {
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.3) 0%, rgba(129, 140, 248, 0.3) 100%);
  color: #a5b4fc;
}

html.dark .title-icon.routine {
  background: linear-gradient(135deg, rgba(245, 158, 11, 0.3) 0%, rgba(251, 191, 36, 0.3) 100%);
  color: #fbbf24;
}

html.dark .empty-icon {
  color: #737373;
}
</style>

<template>
  <el-dialog
    :model-value="modelValue"
    :title="editingEvent ? '编辑优化事件' : '记录优化事件'"
    width="500px"
    @update:model-value="$emit('update:modelValue', $event)"
    @close="handleClose"
  >
    <el-form
      ref="formRef"
      :model="form"
      :rules="rules"
      label-width="80px"
    >
      <el-form-item label="事件日期" prop="eventDate">
        <el-date-picker
          v-model="form.eventDate"
          type="date"
          placeholder="选择日期"
          value-format="YYYY-MM-DD"
          style="width: 100%"
        />
      </el-form-item>

      <el-form-item label="优化类型" prop="eventType">
        <el-select
          v-model="form.eventType"
          placeholder="选择主类型"
          style="width: 100%"
          @change="handleMainTypeChange"
        >
          <el-option
            v-for="(config, key) in EVENT_MAIN_TYPES"
            :key="key"
            :label="config.label"
            :value="key"
          >
            <span class="event-type-option">
              <span
                class="event-type-dot"
                :style="{ backgroundColor: config.color }"
              ></span>
              <span>{{ config.label }}</span>
            </span>
          </el-option>
        </el-select>
      </el-form-item>

      <el-form-item label="具体操作" prop="eventSubTypes">
        <el-select
          v-model="form.eventSubTypes"
          multiple
          collapse-tags
          collapse-tags-tooltip
          placeholder="选择具体操作（可多选）"
          style="width: 100%"
        >
          <el-option
            v-for="(config, key) in subTypeOptions"
            :key="key"
            :label="config.label"
            :value="key"
          />
        </el-select>
      </el-form-item>

      <el-form-item label="事件标题" prop="title">
        <el-input
          v-model="form.title"
          placeholder="简要描述优化内容"
          clearable
          maxlength="100"
          show-word-limit
        />
      </el-form-item>

      <el-form-item label="详细描述">
        <el-input
          v-model="form.description"
          type="textarea"
          :rows="3"
          placeholder="可选：详细说明优化操作"
          maxlength="500"
          show-word-limit
        />
      </el-form-item>

      <el-form-item label="事件范围">
        <el-radio-group v-model="form.eventScope" @change="handleScopeChange">
          <el-radio value="product">
            全部
            <el-tooltip content="该事件针对产品下的所有 ASIN" placement="top">
              <el-icon class="scope-help"><QuestionFilled /></el-icon>
            </el-tooltip>
          </el-radio>
          <el-radio value="asin" :disabled="availableAsins.length === 0">
            ASIN
            <el-tooltip content="该事件仅针对选定的 ASIN" placement="top">
              <el-icon class="scope-help"><QuestionFilled /></el-icon>
            </el-tooltip>
          </el-radio>
          <el-radio value="keyword" :disabled="availableAsins.length === 0">
            关键词
            <el-tooltip content="该事件仅针对选定的 ASIN 的关键词" placement="top">
              <el-icon class="scope-help"><QuestionFilled /></el-icon>
            </el-tooltip>
          </el-radio>
        </el-radio-group>
      </el-form-item>

      <el-form-item v-if="form.eventScope !== 'product'" label="选择ASIN">
        <el-select
          v-model="form.targetAsins"
          multiple
          collapse-tags
          collapse-tags-tooltip
          placeholder="请选择 ASIN（可多选）"
          style="width: 100%"
          @change="handleAsinChange"
        >
          <el-option
            v-for="item in availableAsins"
            :key="item.asin"
            :label="item.asin"
            :value="item.asin"
          >
            <div class="asin-option">
              <img
                v-if="item.imageUrl"
                :src="item.imageUrl"
                class="asin-image"
                @error="(e: Event) => (e.target as HTMLImageElement).style.display = 'none'"
              />
              <span v-else class="asin-image-placeholder">
                <el-icon><Picture /></el-icon>
              </span>
              <span class="asin-text">{{ item.asin }}</span>
            </div>
          </el-option>
        </el-select>
      </el-form-item>

      <el-form-item v-if="form.eventScope === 'keyword'" label="选择关键词">
        <el-select
          v-model="selectedKeywords"
          multiple
          filterable
          :disabled="form.targetAsins.length === 0"
          :placeholder="form.targetAsins.length > 0 ? '选择关联的关键词' : '请先选择 ASIN'"
          style="width: 100%"
        >
          <el-option
            v-for="kw in keywordsForSelectedAsin"
            :key="kw"
            :label="kw"
            :value="kw"
          />
        </el-select>
      </el-form-item>

      <el-form-item label="截图">
        <div
          class="screenshot-upload-area"
          @paste="handlePaste"
          @dragover.prevent
          @drop.prevent="handleDrop"
          tabindex="0"
        >
          <el-upload
            ref="uploadRef"
            :auto-upload="false"
            :show-file-list="false"
            accept="image/*"
            multiple
            :on-change="handleFileChange"
            :disabled="uploadedImages.length >= 5"
          >
            <div class="upload-trigger" v-if="uploadedImages.length < 5">
              <el-icon class="upload-icon"><Plus /></el-icon>
              <div class="upload-text">
                <span>点击/拖拽/粘贴</span>
                <span class="upload-hint">最多 5 张，最大 5MB/张</span>
              </div>
            </div>
          </el-upload>

          <div class="uploaded-images" v-if="uploadedImages.length > 0">
            <div
              v-for="(img, index) in uploadedImages"
              :key="index"
              class="image-item"
            >
              <el-image
                :src="img.url"
                fit="cover"
                class="image-preview"
                @click="previewImage(index)"
              />
              <el-icon class="remove-btn" @click="removeImage(index)"><Close /></el-icon>
            </div>
          </div>
        </div>
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button
        type="primary"
        :loading="submitting"
        @click="handleSubmit"
      >
        {{ editingEvent ? '保存' : '添加' }}
      </el-button>
    </template>
  </el-dialog>

  <!-- 图片预览 -->
  <teleport to="body">
    <el-image-viewer
      v-if="previewVisible"
      :url-list="previewUrls"
      :initial-index="previewIndex"
      @close="previewVisible = false"
    />
  </teleport>
</template>

<script setup lang="ts">
import { ref, reactive, watch, computed } from 'vue';
import type { FormInstance, FormRules, UploadInstance, UploadFile } from 'element-plus';
import { ElMessage } from 'element-plus';
import { QuestionFilled, Picture, Plus, Close } from '@element-plus/icons-vue';
import { addOptimizationEvent, updateOptimizationEvent, saveEventScreenshot, deleteEventScreenshot, getScreenshotsDir } from '../api';
import { convertFileSrc } from '@tauri-apps/api/core';
import {
  EVENT_MAIN_TYPES,
  EVENT_SUB_TYPES,
  type OptimizationEvent,
  type EventMainType,
  type EventSubType
} from '../types';

// ASIN 信息类型
interface AsinInfo {
  asin: string;
  imageUrl: string | null;
}

const props = defineProps<{
  modelValue: boolean;
  productId: number;
  editingEvent?: OptimizationEvent | null;
  asins?: AsinInfo[];  // 当前产品的 ASIN 列表（包含图片）
  keywordsByAsin?: Record<string, string[]>;  // 每个 ASIN 下的关键词
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'success'): void;
}>();

const formRef = ref<FormInstance>();
const uploadRef = ref<UploadInstance>();
const submitting = ref(false);
const selectedKeywords = ref<string[]>([]);
const screenshotsDir = ref<string>('');

// 上传的图片列表
interface UploadedImage {
  url: string;       // 预览 URL (data: URL 或 file: URL)
  base64: string;    // Base64 数据（用于新上传的图片）
  filename?: string; // 已保存的文件名（用于编辑时已有的图片）
  isNew: boolean;    // 是否是新上传的
}
const uploadedImages = ref<UploadedImage[]>([]);

// 图片预览状态
const previewVisible = ref(false);
const previewIndex = ref(0);
const previewUrls = computed(() => uploadedImages.value.map(img => img.url));

// 事件范围类型
type EventScope = 'product' | 'asin' | 'keyword';

// 表单数据
const form = reactive({
  eventDate: '',
  eventType: 'listing' as EventMainType,
  eventSubTypes: ['title'] as EventSubType[],  // 改为数组支持多选
  title: '',
  description: '',
  eventScope: 'product' as EventScope,
  targetAsins: [] as string[],  // 改为数组支持多选
});

// 子类型选项（根据主类型动态变化）
const subTypeOptions = computed(() => {
  return EVENT_SUB_TYPES[form.eventType] || {};
});

// 主类型变化时重置子类型
function handleMainTypeChange() {
  const subTypes = EVENT_SUB_TYPES[form.eventType];
  const keys = Object.keys(subTypes);
  form.eventSubTypes = keys.length > 0 ? [keys[0] as EventSubType] : [];
}

// 表单验证规则
const rules: FormRules = {
  eventDate: [
    { required: true, message: '请选择事件日期', trigger: 'change' },
  ],
  eventType: [
    { required: true, message: '请选择优化类型', trigger: 'change' },
  ],
  eventSubTypes: [
    { required: true, type: 'array', min: 1, message: '请至少选择一个具体操作', trigger: 'change' },
  ],
  title: [
    { required: true, message: '请输入事件标题', trigger: 'blur' },
    { max: 100, message: '标题不能超过100个字符', trigger: 'blur' },
  ],
};

// 可用的 ASIN 列表
const availableAsins = computed((): AsinInfo[] => {
  return props.asins || [];
});

// 当前选中 ASIN 下的关键词列表（合并多个 ASIN 的关键词）
const keywordsForSelectedAsin = computed(() => {
  if (form.targetAsins.length === 0 || !props.keywordsByAsin) return [];
  const allKeywords = new Set<string>();
  for (const asin of form.targetAsins) {
    const keywords = props.keywordsByAsin[asin] || [];
    keywords.forEach(kw => allKeywords.add(kw));
  }
  return Array.from(allKeywords);
});

// 事件范围变化时的处理
function handleScopeChange() {
  if (form.eventScope === 'product') {
    form.targetAsins = [];
    selectedKeywords.value = [];
  } else if (form.eventScope === 'asin') {
    selectedKeywords.value = [];
  }
}

// ASIN 变化时的处理
function handleAsinChange() {
  selectedKeywords.value = [];
}

// 解析 event_sub_type 字段（兼容旧数据：单字符串 或 新数据：JSON 数组）
function parseEventSubTypes(value: string | undefined): EventSubType[] {
  if (!value) return ['title'];
  // 尝试解析为 JSON 数组
  if (value.startsWith('[')) {
    try {
      return JSON.parse(value) as EventSubType[];
    } catch {
      return [value as EventSubType];
    }
  }
  // 旧数据：单个字符串
  return [value as EventSubType];
}

// 初始化表单
function initForm() {
  if (props.editingEvent) {
    form.eventDate = props.editingEvent.event_date;
    form.eventType = props.editingEvent.event_type as EventMainType;
    form.eventSubTypes = parseEventSubTypes(props.editingEvent.event_sub_type as string);
    form.title = props.editingEvent.title;
    form.description = props.editingEvent.description || '';

    // 解析 target_asin（支持旧的单值和新的 JSON 数组格式）
    if (props.editingEvent.target_asin) {
      if (props.editingEvent.target_asin.startsWith('[')) {
        try {
          form.targetAsins = JSON.parse(props.editingEvent.target_asin);
        } catch {
          form.targetAsins = [props.editingEvent.target_asin];
        }
      } else {
        form.targetAsins = [props.editingEvent.target_asin];
      }
    } else {
      form.targetAsins = [];
    }

    // 解析关联关键词
    if (props.editingEvent.affected_keywords) {
      try {
        selectedKeywords.value = JSON.parse(props.editingEvent.affected_keywords);
      } catch {
        selectedKeywords.value = [];
      }
    } else {
      selectedKeywords.value = [];
    }

    // 根据数据确定事件范围
    if (form.targetAsins.length === 0) {
      form.eventScope = 'product';
    } else if (selectedKeywords.value.length > 0) {
      form.eventScope = 'keyword';
    } else {
      form.eventScope = 'asin';
    }

    // 加载已有的截图
    loadExistingScreenshots();
  } else {
    // 新建事件，默认今天
    const today = new Date();
    form.eventDate = today.toISOString().split('T')[0];
    form.eventType = 'listing';
    form.eventSubTypes = ['title'];
    form.title = '';
    form.description = '';
    form.eventScope = 'product';
    form.targetAsins = [];
    selectedKeywords.value = [];
    uploadedImages.value = [];
  }
}

// 加载已有的截图
async function loadExistingScreenshots() {
  uploadedImages.value = [];
  if (!props.editingEvent?.screenshots) return;

  try {
    const filenames = JSON.parse(props.editingEvent.screenshots) as string[];
    if (!screenshotsDir.value) {
      screenshotsDir.value = await getScreenshotsDir();
    }

    for (const filename of filenames) {
      const filePath = `${screenshotsDir.value}/${filename}`;
      const url = convertFileSrc(filePath);
      uploadedImages.value.push({
        url,
        base64: '',
        filename,
        isNew: false,
      });
    }
  } catch (e) {
    console.error('Failed to load existing screenshots:', e);
  }
}

// 处理文件选择
async function handleFileChange(file: UploadFile) {
  if (!file.raw) return;

  // 检查文件大小
  if (file.raw.size > 5 * 1024 * 1024) {
    ElMessage.warning('图片大小不能超过 5MB');
    return;
  }

  // 检查数量限制
  if (uploadedImages.value.length >= 5) {
    ElMessage.warning('最多只能上传 5 张图片');
    return;
  }

  // 转换为 base64
  const base64 = await fileToBase64(file.raw);
  uploadedImages.value.push({
    url: base64,
    base64,
    isNew: true,
  });
}

// 处理粘贴事件
async function handlePaste(event: ClipboardEvent) {
  const items = event.clipboardData?.items;
  if (!items) return;

  for (const item of items) {
    if (item.type.startsWith('image/')) {
      const file = item.getAsFile();
      if (!file) continue;

      // 检查文件大小
      if (file.size > 5 * 1024 * 1024) {
        ElMessage.warning('图片大小不能超过 5MB');
        return;
      }

      // 检查数量限制
      if (uploadedImages.value.length >= 5) {
        ElMessage.warning('最多只能上传 5 张图片');
        return;
      }

      const base64 = await fileToBase64(file);
      uploadedImages.value.push({
        url: base64,
        base64,
        isNew: true,
      });
    }
  }
}

// 处理拖放事件
async function handleDrop(event: DragEvent) {
  const files = event.dataTransfer?.files;
  if (!files) return;

  for (const file of files) {
    if (!file.type.startsWith('image/')) continue;

    // 检查文件大小
    if (file.size > 5 * 1024 * 1024) {
      ElMessage.warning('图片大小不能超过 5MB');
      continue;
    }

    // 检查数量限制
    if (uploadedImages.value.length >= 5) {
      ElMessage.warning('最多只能上传 5 张图片');
      break;
    }

    const base64 = await fileToBase64(file);
    uploadedImages.value.push({
      url: base64,
      base64,
      isNew: true,
    });
  }
}

// 文件转 base64
function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}

// 移除图片
async function removeImage(index: number) {
  const img = uploadedImages.value[index];

  // 如果是已保存的图片，需要删除文件
  if (!img.isNew && img.filename) {
    try {
      await deleteEventScreenshot(img.filename);
    } catch (e) {
      console.error('Failed to delete screenshot file:', e);
    }
  }

  uploadedImages.value.splice(index, 1);
}

// 预览图片
function previewImage(index: number) {
  previewIndex.value = index;
  previewVisible.value = true;
}

// 监听对话框打开
watch(() => props.modelValue, (val) => {
  if (val) {
    initForm();
  }
});

// 关闭对话框
function handleClose() {
  formRef.value?.resetFields();
  form.eventScope = 'product';
  form.targetAsins = [];
  selectedKeywords.value = [];
  uploadedImages.value = [];
  emit('update:modelValue', false);
}

// 提交表单
async function handleSubmit() {
  if (!formRef.value) return;

  const valid = await formRef.value.validate().catch(() => false);
  if (!valid) return;

  submitting.value = true;

  try {
    // 根据事件范围确定 targetAsin 和 affectedKeywords
    // 多个 ASIN 时存储为 JSON 数组
    const targetAsin = form.eventScope !== 'product' && form.targetAsins.length > 0
      ? (form.targetAsins.length === 1 ? form.targetAsins[0] : JSON.stringify(form.targetAsins))
      : undefined;

    const affectedKeywords = form.eventScope === 'keyword' && selectedKeywords.value.length > 0
      ? JSON.stringify(selectedKeywords.value)
      : undefined;

    // 将多选的具体操作序列化为 JSON 字符串
    const eventSubTypesJson = JSON.stringify(form.eventSubTypes);

    // 处理截图
    // 对于编辑模式，先获取事件 ID；对于新增模式，先创建事件获取 ID
    let eventId: number;
    let screenshotsJson: string | undefined;

    if (props.editingEvent) {
      eventId = props.editingEvent.id;
    } else {
      // 先创建事件（不含截图），获取 ID
      eventId = await addOptimizationEvent(
        props.productId,
        form.eventDate,
        form.eventType,
        eventSubTypesJson,
        form.title,
        form.description || undefined,
        targetAsin,
        affectedKeywords,
        undefined
      );
    }

    // 保存新上传的截图
    const screenshotFilenames: string[] = [];
    for (let i = 0; i < uploadedImages.value.length; i++) {
      const img = uploadedImages.value[i];
      if (img.isNew && img.base64) {
        // 新上传的图片，保存到文件
        const filename = await saveEventScreenshot(eventId, img.base64, i);
        screenshotFilenames.push(filename);
      } else if (img.filename) {
        // 已有的图片，保留文件名
        screenshotFilenames.push(img.filename);
      }
    }

    // 生成截图 JSON
    screenshotsJson = screenshotFilenames.length > 0
      ? JSON.stringify(screenshotFilenames)
      : undefined;

    if (props.editingEvent) {
      // 更新事件（包含截图）
      await updateOptimizationEvent(
        props.editingEvent.id,
        form.eventDate,
        form.eventType,
        eventSubTypesJson,
        form.title,
        form.description || undefined,
        targetAsin,
        affectedKeywords,
        screenshotsJson
      );
      ElMessage.success('事件已更新');
    } else {
      // 更新事件以包含截图
      if (screenshotsJson) {
        await updateOptimizationEvent(
          eventId,
          form.eventDate,
          form.eventType,
          eventSubTypesJson,
          form.title,
          form.description || undefined,
          targetAsin,
          affectedKeywords,
          screenshotsJson
        );
      }
      ElMessage.success('事件已记录');
    }

    emit('success');
    handleClose();
  } catch (err) {
    ElMessage.error(`操作失败: ${err}`);
  } finally {
    submitting.value = false;
  }
}
</script>

<style scoped>
.event-type-option {
  display: flex;
  align-items: center;
  gap: 8px;
}

.event-type-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.scope-help {
  margin-left: 4px;
  color: var(--el-text-color-placeholder);
  cursor: help;
  font-size: 14px;
  vertical-align: middle;
}

.scope-help:hover {
  color: var(--el-color-primary);
}

/* ASIN 下拉选项样式 */
.asin-option {
  display: flex;
  align-items: center;
  gap: 10px;
}

.asin-image {
  width: 32px;
  height: 32px;
  object-fit: contain;
  border-radius: 4px;
  background: var(--el-fill-color-light);
}

.asin-image-placeholder {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--el-fill-color-light);
  border-radius: 4px;
  color: var(--el-text-color-placeholder);
}

.asin-text {
  font-family: monospace;
  font-size: 13px;
}

/* 截图上传区域样式 */
.screenshot-upload-area {
  width: 100%;
  outline: none;
}

.screenshot-upload-area:focus {
  outline: none;
}

.upload-trigger {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;
  min-height: 80px;
  border: 1px dashed var(--el-border-color);
  border-radius: 6px;
  background: var(--el-fill-color-lighter);
  cursor: pointer;
  transition: all 0.2s;
}

.upload-trigger:hover {
  border-color: var(--el-color-primary);
  background: var(--el-color-primary-light-9);
}

.upload-icon {
  font-size: 24px;
  color: var(--el-text-color-placeholder);
  margin-bottom: 4px;
}

.upload-text {
  display: flex;
  flex-direction: column;
  align-items: center;
  color: var(--el-text-color-secondary);
  font-size: 13px;
}

.upload-hint {
  font-size: 10px;
  color: var(--el-text-color-placeholder);
  margin-top: 2px;
}

.uploaded-images {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 12px;
}

.image-item {
  position: relative;
  width: 80px;
  height: 80px;
  border-radius: 6px;
  overflow: hidden;
  border: 1px solid var(--el-border-color-lighter);
}

.image-preview {
  width: 100%;
  height: 100%;
  cursor: pointer;
}

.remove-btn {
  position: absolute;
  top: 4px;
  right: 4px;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.5);
  color: white;
  border-radius: 50%;
  cursor: pointer;
  font-size: 12px;
  opacity: 0;
  transition: opacity 0.2s;
}

.image-item:hover .remove-btn {
  opacity: 1;
}

.remove-btn:hover {
  background: var(--el-color-danger);
}
</style>

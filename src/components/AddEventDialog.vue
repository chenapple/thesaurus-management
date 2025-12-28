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

      <el-form-item label="具体操作" prop="eventSubType">
        <el-select
          v-model="form.eventSubType"
          placeholder="选择具体操作"
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
          v-model="form.targetAsin"
          placeholder="请选择 ASIN"
          style="width: 100%"
          @change="handleAsinChange"
        >
          <el-option
            v-for="asin in availableAsins"
            :key="asin"
            :label="asin"
            :value="asin"
          />
        </el-select>
      </el-form-item>

      <el-form-item v-if="form.eventScope === 'keyword'" label="选择关键词">
        <el-select
          v-model="selectedKeywords"
          multiple
          filterable
          :disabled="!form.targetAsin"
          :placeholder="form.targetAsin ? '选择关联的关键词' : '请先选择 ASIN'"
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
</template>

<script setup lang="ts">
import { ref, reactive, watch, computed } from 'vue';
import type { FormInstance, FormRules } from 'element-plus';
import { ElMessage } from 'element-plus';
import { QuestionFilled } from '@element-plus/icons-vue';
import { addOptimizationEvent, updateOptimizationEvent } from '../api';
import {
  EVENT_MAIN_TYPES,
  EVENT_SUB_TYPES,
  type OptimizationEvent,
  type EventMainType,
  type EventSubType
} from '../types';

const props = defineProps<{
  modelValue: boolean;
  productId: number;
  editingEvent?: OptimizationEvent | null;
  asins?: string[];  // 当前产品的 ASIN 列表
  keywordsByAsin?: Record<string, string[]>;  // 每个 ASIN 下的关键词
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'success'): void;
}>();

const formRef = ref<FormInstance>();
const submitting = ref(false);
const selectedKeywords = ref<string[]>([]);

// 事件范围类型
type EventScope = 'product' | 'asin' | 'keyword';

// 表单数据
const form = reactive({
  eventDate: '',
  eventType: 'listing' as EventMainType,
  eventSubType: 'title' as EventSubType,
  title: '',
  description: '',
  eventScope: 'product' as EventScope,
  targetAsin: '' as string,
});

// 子类型选项（根据主类型动态变化）
const subTypeOptions = computed(() => {
  return EVENT_SUB_TYPES[form.eventType] || {};
});

// 主类型变化时重置子类型
function handleMainTypeChange() {
  const subTypes = EVENT_SUB_TYPES[form.eventType];
  const keys = Object.keys(subTypes);
  form.eventSubType = (keys[0] || 'title') as EventSubType;
}

// 表单验证规则
const rules: FormRules = {
  eventDate: [
    { required: true, message: '请选择事件日期', trigger: 'change' },
  ],
  eventType: [
    { required: true, message: '请选择优化类型', trigger: 'change' },
  ],
  eventSubType: [
    { required: true, message: '请选择具体操作', trigger: 'change' },
  ],
  title: [
    { required: true, message: '请输入事件标题', trigger: 'blur' },
    { max: 100, message: '标题不能超过100个字符', trigger: 'blur' },
  ],
};

// 可用的 ASIN 列表
const availableAsins = computed(() => {
  return props.asins || [];
});

// 当前选中 ASIN 下的关键词列表
const keywordsForSelectedAsin = computed(() => {
  if (!form.targetAsin || !props.keywordsByAsin) return [];
  return props.keywordsByAsin[form.targetAsin] || [];
});

// 事件范围变化时的处理
function handleScopeChange() {
  if (form.eventScope === 'product') {
    form.targetAsin = '';
    selectedKeywords.value = [];
  } else if (form.eventScope === 'asin') {
    selectedKeywords.value = [];
  }
}

// ASIN 变化时的处理
function handleAsinChange() {
  selectedKeywords.value = [];
}

// 初始化表单
function initForm() {
  if (props.editingEvent) {
    form.eventDate = props.editingEvent.event_date;
    form.eventType = props.editingEvent.event_type as EventMainType;
    form.eventSubType = props.editingEvent.event_sub_type as EventSubType;
    form.title = props.editingEvent.title;
    form.description = props.editingEvent.description || '';
    form.targetAsin = props.editingEvent.target_asin || '';

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
    if (!props.editingEvent.target_asin) {
      form.eventScope = 'product';
    } else if (selectedKeywords.value.length > 0) {
      form.eventScope = 'keyword';
    } else {
      form.eventScope = 'asin';
    }
  } else {
    // 新建事件，默认今天
    const today = new Date();
    form.eventDate = today.toISOString().split('T')[0];
    form.eventType = 'listing';
    form.eventSubType = 'title';
    form.title = '';
    form.description = '';
    form.eventScope = 'product';
    form.targetAsin = '';
    selectedKeywords.value = [];
  }
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
  form.targetAsin = '';
  selectedKeywords.value = [];
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
    const targetAsin = form.eventScope !== 'product' && form.targetAsin
      ? form.targetAsin
      : undefined;

    const affectedKeywords = form.eventScope === 'keyword' && selectedKeywords.value.length > 0
      ? JSON.stringify(selectedKeywords.value)
      : undefined;

    if (props.editingEvent) {
      // 更新事件
      await updateOptimizationEvent(
        props.editingEvent.id,
        form.eventDate,
        form.eventType,
        form.eventSubType,
        form.title,
        form.description || undefined,
        targetAsin,
        affectedKeywords
      );
      ElMessage.success('事件已更新');
    } else {
      // 添加事件
      await addOptimizationEvent(
        props.productId,
        form.eventDate,
        form.eventType,
        form.eventSubType,
        form.title,
        form.description || undefined,
        targetAsin,
        affectedKeywords
      );
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
</style>

<template>
  <el-dialog
    :model-value="modelValue"
    :title="isEdit ? '编辑监控任务' : '创建监控任务'"
    width="600px"
    @update:model-value="$emit('update:modelValue', $event)"
    @close="handleClose"
  >
    <el-form
      ref="formRef"
      :model="form"
      :rules="rules"
      label-width="100px"
    >
      <el-form-item label="任务名称" prop="name">
        <el-input
          v-model="form.name"
          placeholder="例如: 美妆类目周报"
          clearable
        />
      </el-form-item>

      <el-form-item label="站点" prop="marketplace">
        <el-select v-model="form.marketplace" style="width: 100%">
          <el-option
            v-for="country in COUNTRY_OPTIONS"
            :key="country.value"
            :label="country.label"
            :value="country.value"
          >
            <div style="display: flex; align-items: center; gap: 8px;">
              <span class="country-flag" v-html="country.flag"></span>
              <span>{{ country.label }}</span>
            </div>
          </el-option>
        </el-select>
      </el-form-item>

      <el-form-item label="类目 ID" prop="category_id">
        <el-input
          v-model="form.category_id"
          placeholder="例如: beauty/211005031"
        />
        <div class="form-hint">类目 ID 可从 Amazon BSR 页面 URL 获取</div>
      </el-form-item>

      <el-form-item label="类目名称">
        <el-input
          v-model="form.category_name"
          placeholder="例如: Skin Care (可选)"
        />
      </el-form-item>

      <el-form-item label="AI 服务" prop="ai_provider">
        <el-select v-model="form.ai_provider" style="width: 100%">
          <el-option
            v-for="(config, key) in AI_PROVIDERS"
            :key="key"
            :value="key"
            :label="config.name"
          />
        </el-select>
      </el-form-item>

      <el-form-item label="模型">
        <el-select v-model="form.ai_model" style="width: 100%">
          <el-option
            v-for="model in availableModels"
            :key="model"
            :value="model"
            :label="model"
          />
        </el-select>
      </el-form-item>

      <el-form-item label="运行频率" prop="schedule_type">
        <el-radio-group v-model="form.schedule_type">
          <el-radio value="daily">每天</el-radio>
          <el-radio value="weekly">每周</el-radio>
        </el-radio-group>
      </el-form-item>

      <el-form-item v-if="form.schedule_type === 'weekly'" label="运行日期" prop="schedule_days">
        <el-checkbox-group v-model="selectedDays">
          <el-checkbox :value="1">周一</el-checkbox>
          <el-checkbox :value="2">周二</el-checkbox>
          <el-checkbox :value="3">周三</el-checkbox>
          <el-checkbox :value="4">周四</el-checkbox>
          <el-checkbox :value="5">周五</el-checkbox>
          <el-checkbox :value="6">周六</el-checkbox>
          <el-checkbox :value="0">周日</el-checkbox>
        </el-checkbox-group>
      </el-form-item>

      <el-form-item label="运行时间" prop="schedule_time">
        <el-time-select
          v-model="form.schedule_time"
          start="00:00"
          step="00:30"
          end="23:30"
          placeholder="选择时间"
          style="width: 200px"
        />
      </el-form-item>

      <el-form-item v-if="isEdit" label="状态">
        <el-switch
          v-model="form.is_enabled"
          active-text="启用"
          inactive-text="禁用"
        />
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button
        type="primary"
        :loading="submitting"
        @click="handleSubmit"
      >
        {{ isEdit ? '保存' : '创建' }}
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue';
import type { FormInstance, FormRules } from 'element-plus';
import { ElMessage } from 'element-plus';
import { invoke } from '@tauri-apps/api/core';
import { AI_PROVIDERS, type AIProvider, COUNTRY_OPTIONS } from '../types';

export interface MarketResearchTask {
  id: number;
  name: string;
  marketplace: string;
  category_id: string;
  category_name?: string;
  ai_provider: string;
  ai_model?: string;
  schedule_type: string;
  schedule_days?: string;
  schedule_time: string;
  is_enabled: boolean;
  last_run_at?: string;
  last_run_status?: string;
  created_at: string;
}

const props = defineProps<{
  modelValue: boolean;
  task?: MarketResearchTask | null;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'success'): void;
}>();

const isEdit = computed(() => !!props.task);
const submitting = ref(false);
const formRef = ref<FormInstance>();

const form = reactive({
  name: '',
  marketplace: 'US',
  category_id: '',
  category_name: '',
  ai_provider: 'deepseek' as AIProvider,
  ai_model: AI_PROVIDERS.deepseek.defaultModel,
  schedule_type: 'weekly',
  schedule_time: '09:00',
  is_enabled: true,
});

const availableModels = computed(() => {
  return AI_PROVIDERS[form.ai_provider].models;
});

watch(() => form.ai_provider, (newProvider) => {
  form.ai_model = AI_PROVIDERS[newProvider].defaultModel;
});

const selectedDays = ref<number[]>([1, 3, 5]);

const rules: FormRules = {
  name: [{ required: true, message: '请输入任务名称', trigger: 'blur' }],
  marketplace: [{ required: true, message: '请选择站点', trigger: 'change' }],
  category_id: [{ required: true, message: '请输入类目 ID', trigger: 'blur' }],
  ai_provider: [{ required: true, message: '请选择 AI 模型', trigger: 'change' }],
  schedule_type: [{ required: true, message: '请选择运行频率', trigger: 'change' }],
  schedule_time: [{ required: true, message: '请选择运行时间', trigger: 'change' }],
};

watch(() => props.modelValue, (visible) => {
  if (visible && props.task) {
    form.name = props.task.name;
    form.marketplace = props.task.marketplace;
    form.category_id = props.task.category_id;
    form.category_name = props.task.category_name || '';
    form.ai_provider = props.task.ai_provider as AIProvider;
    form.ai_model = props.task.ai_model || AI_PROVIDERS[form.ai_provider].defaultModel;
    form.schedule_type = props.task.schedule_type;
    form.schedule_time = props.task.schedule_time;
    form.is_enabled = props.task.is_enabled;
    if (props.task.schedule_days) {
      try {
        selectedDays.value = JSON.parse(props.task.schedule_days);
      } catch {
        selectedDays.value = [1, 3, 5];
      }
    }
  } else if (visible) {
    resetForm();
  }
});

function resetForm() {
  form.name = '';
  form.marketplace = 'US';
  form.category_id = '';
  form.category_name = '';
  form.ai_provider = 'deepseek';
  form.ai_model = AI_PROVIDERS.deepseek.defaultModel;
  form.schedule_type = 'weekly';
  form.schedule_time = '09:00';
  form.is_enabled = true;
  selectedDays.value = [1, 3, 5];
}

function handleClose() {
  emit('update:modelValue', false);
}

async function handleSubmit() {
  if (!formRef.value) return;

  const valid = await formRef.value.validate().catch(() => false);
  if (!valid) return;

  if (form.schedule_type === 'weekly' && selectedDays.value.length === 0) {
    ElMessage.warning('请至少选择一天');
    return;
  }

  submitting.value = true;
  try {
    const scheduleDays = form.schedule_type === 'weekly'
      ? JSON.stringify(selectedDays.value)
      : null;

    if (isEdit.value && props.task) {
      await invoke('update_market_research_task', {
        id: props.task.id,
        name: form.name,
        marketplace: form.marketplace,
        categoryId: form.category_id,
        categoryName: form.category_name || null,
        aiProvider: form.ai_provider,
        aiModel: form.ai_model,
        scheduleType: form.schedule_type,
        scheduleDays,
        scheduleTime: form.schedule_time,
        isEnabled: form.is_enabled,
      });
      ElMessage.success('任务已更新');
    } else {
      await invoke('create_market_research_task', {
        name: form.name,
        marketplace: form.marketplace,
        categoryId: form.category_id,
        categoryName: form.category_name || null,
        aiProvider: form.ai_provider,
        aiModel: form.ai_model,
        scheduleType: form.schedule_type,
        scheduleDays,
        scheduleTime: form.schedule_time,
      });
      ElMessage.success('任务已创建');
    }

    emit('success');
    handleClose();
  } catch (error) {
    ElMessage.error(`操作失败: ${error}`);
  } finally {
    submitting.value = false;
  }
}
</script>

<style scoped>
.form-hint {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 4px;
}

.country-flag {
  display: inline-flex;
  width: 20px;
  height: 14px;
  border-radius: 2px;
  overflow: hidden;
  flex-shrink: 0;
}

.country-flag :deep(svg) {
  width: 100%;
  height: 100%;
}
</style>

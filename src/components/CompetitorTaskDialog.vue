<template>
  <el-dialog
    :model-value="modelValue"
    :title="isEdit ? '编辑监控任务' : '创建监控任务'"
    width="650px"
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
          placeholder="例如: 主要竞品监控"
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

      <el-form-item label="我的 ASIN">
        <el-input
          v-model="form.my_asin"
          placeholder="可选，用于对比分析"
        />
        <div class="form-hint">填写您的产品 ASIN，用于与竞品进行对比分析</div>
      </el-form-item>

      <el-form-item label="竞品 ASIN" prop="asins">
        <div class="asin-input-area">
          <el-input
            v-model="newAsin"
            placeholder="输入竞品 ASIN，按回车添加"
            @keyup.enter="addAsin"
            style="flex: 1"
          >
            <template #append>
              <el-button @click="addAsin">添加</el-button>
            </template>
          </el-input>
        </div>
        <div class="form-hint">支持批量粘贴（每行一个或逗号分隔）</div>

        <div v-if="form.asins.length > 0" class="asin-list">
          <el-tag
            v-for="(asin, index) in form.asins"
            :key="index"
            closable
            type="info"
            @close="removeAsin(index)"
            class="asin-tag"
          >
            {{ asin.asin }}
            <span v-if="asin.title" class="asin-title">{{ truncateTitle(asin.title) }}</span>
          </el-tag>
        </div>
        <div v-else class="empty-asins">
          请添加至少一个竞品 ASIN
        </div>
      </el-form-item>

      <el-divider />

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
        <el-checkbox-group v-model="form.schedule_days">
          <el-checkbox v-for="day in WEEKDAY_OPTIONS" :key="day.value" :value="day.value">
            {{ day.label }}
          </el-checkbox>
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
import { AI_PROVIDERS, type AIProvider, COUNTRY_OPTIONS, type CompetitorTask, type CompetitorAsin } from '../types';

const WEEKDAY_OPTIONS = [
  { value: 1, label: '周一' },
  { value: 2, label: '周二' },
  { value: 3, label: '周三' },
  { value: 4, label: '周四' },
  { value: 5, label: '周五' },
  { value: 6, label: '周六' },
  { value: 0, label: '周日' },
];

const props = defineProps<{
  modelValue: boolean;
  task?: CompetitorTask | null;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'success'): void;
}>();

const isEdit = computed(() => !!props.task);
const submitting = ref(false);
const formRef = ref<FormInstance>();
const newAsin = ref('');

interface AsinItem {
  asin: string;
  title?: string;
}

const form = reactive({
  name: '',
  marketplace: 'US',
  my_asin: '',
  asins: [] as AsinItem[],
  ai_provider: 'deepseek' as AIProvider,
  ai_model: AI_PROVIDERS.deepseek.defaultModel,
  schedule_type: 'daily',
  schedule_days: [1, 2, 3, 4, 5] as number[],  // 默认周一到周五
  schedule_time: '09:00',
  is_enabled: true,
});

const availableModels = computed(() => {
  return AI_PROVIDERS[form.ai_provider].models;
});

watch(() => form.ai_provider, (newProvider) => {
  form.ai_model = AI_PROVIDERS[newProvider].defaultModel;
});

const rules: FormRules = {
  name: [{ required: true, message: '请输入任务名称', trigger: 'blur' }],
  marketplace: [{ required: true, message: '请选择站点', trigger: 'change' }],
  ai_provider: [{ required: true, message: '请选择 AI 服务', trigger: 'change' }],
  schedule_type: [{ required: true, message: '请选择运行频率', trigger: 'change' }],
  schedule_time: [{ required: true, message: '请选择运行时间', trigger: 'change' }],
};

// 添加 ASIN
function addAsin() {
  const input = newAsin.value.trim();
  if (!input) return;

  // 支持多种分隔符
  const asins = input
    .split(/[\n,\s]+/)
    .map(s => s.trim().toUpperCase())
    .filter(s => s && /^[A-Z0-9]{10}$/.test(s));

  if (asins.length === 0) {
    ElMessage.warning('请输入有效的 ASIN（10位字母数字）');
    return;
  }

  for (const asin of asins) {
    if (!form.asins.find(a => a.asin === asin)) {
      form.asins.push({ asin });
    }
  }

  newAsin.value = '';
}

// 移除 ASIN
function removeAsin(index: number) {
  form.asins.splice(index, 1);
}

// 截断标题
function truncateTitle(title: string): string {
  return title.length > 20 ? title.slice(0, 20) + '...' : title;
}

// 加载任务数据
watch(() => props.modelValue, async (visible) => {
  if (visible && props.task) {
    form.name = props.task.name;
    form.marketplace = props.task.marketplace;
    form.my_asin = props.task.my_asin || '';
    form.ai_provider = props.task.ai_provider as AIProvider;
    form.ai_model = props.task.ai_model || AI_PROVIDERS[form.ai_provider].defaultModel;
    form.schedule_type = props.task.schedule_type;
    // 解析 schedule_days
    if (props.task.schedule_days) {
      try {
        const days = JSON.parse(props.task.schedule_days);
        form.schedule_days = Array.isArray(days) ? days : [1, 2, 3, 4, 5];
      } catch {
        form.schedule_days = [1, 2, 3, 4, 5];
      }
    } else {
      form.schedule_days = [1, 2, 3, 4, 5];
    }
    form.schedule_time = props.task.schedule_time;
    form.is_enabled = props.task.is_enabled;

    // 加载已有的 ASIN 列表
    try {
      const asins = await invoke<CompetitorAsin[]>('get_competitor_asins', { taskId: props.task.id });
      form.asins = asins.map(a => ({ asin: a.asin, title: a.title }));
    } catch (e) {
      console.error('加载 ASIN 列表失败:', e);
    }
  } else if (visible) {
    resetForm();
  }
});

function resetForm() {
  form.name = '';
  form.marketplace = 'US';
  form.my_asin = '';
  form.asins = [];
  form.ai_provider = 'deepseek';
  form.ai_model = AI_PROVIDERS.deepseek.defaultModel;
  form.schedule_type = 'daily';
  form.schedule_days = [1, 2, 3, 4, 5];  // 默认周一到周五
  form.schedule_time = '09:00';
  form.is_enabled = true;
  newAsin.value = '';
}

function handleClose() {
  emit('update:modelValue', false);
}

async function handleSubmit() {
  if (!formRef.value) return;

  const valid = await formRef.value.validate().catch(() => false);
  if (!valid) return;

  if (form.asins.length === 0) {
    ElMessage.warning('请至少添加一个竞品 ASIN');
    return;
  }

  submitting.value = true;
  try {
    if (isEdit.value && props.task) {
      // 更新任务
      await invoke('update_competitor_task', {
        id: props.task.id,
        name: form.name,
        marketplace: form.marketplace,
        myAsin: form.my_asin || null,
        aiProvider: form.ai_provider,
        aiModel: form.ai_model,
        scheduleType: form.schedule_type,
        scheduleDays: form.schedule_type === 'weekly' ? JSON.stringify(form.schedule_days) : null,
        scheduleTime: form.schedule_time,
        isEnabled: form.is_enabled,
      });

      // 更新 ASIN 列表（先删除旧的，再添加新的）
      const existingAsins = await invoke<CompetitorAsin[]>('get_competitor_asins', { taskId: props.task.id });
      for (const existing of existingAsins) {
        if (!form.asins.find(a => a.asin === existing.asin)) {
          await invoke('remove_competitor_asin', { taskId: props.task.id, asin: existing.asin });
        }
      }
      for (const asin of form.asins) {
        if (!existingAsins.find(e => e.asin === asin.asin)) {
          await invoke('add_competitor_asin', {
            taskId: props.task.id,
            asin: asin.asin,
            title: asin.title || null,
            tags: null,
          });
        }
      }

      ElMessage.success('任务已更新');
    } else {
      // 创建任务
      const taskId = await invoke<number>('create_competitor_task', {
        name: form.name,
        marketplace: form.marketplace,
        myAsin: form.my_asin || null,
        aiProvider: form.ai_provider,
        aiModel: form.ai_model,
        scheduleType: form.schedule_type,
        scheduleDays: form.schedule_type === 'weekly' ? JSON.stringify(form.schedule_days) : null,
        scheduleTime: form.schedule_time,
      });

      // 添加 ASIN 列表
      for (const asin of form.asins) {
        await invoke('add_competitor_asin', {
          taskId,
          asin: asin.asin,
          title: asin.title || null,
          tags: null,
        });
      }

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

.asin-input-area {
  display: flex;
  gap: 8px;
}

.asin-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 12px;
  padding: 12px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
  max-height: 200px;
  overflow-y: auto;
}

.asin-tag {
  display: flex;
  align-items: center;
  gap: 6px;
}

.asin-title {
  font-size: 11px;
  color: var(--el-text-color-secondary);
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.empty-asins {
  margin-top: 12px;
  padding: 20px;
  text-align: center;
  color: var(--el-text-color-placeholder);
  background: var(--el-fill-color-light);
  border-radius: 8px;
  font-size: 13px;
}
</style>

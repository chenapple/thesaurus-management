<template>
  <el-dialog
    :model-value="modelValue"
    title="添加关键词监控"
    width="600px"
    @update:model-value="$emit('update:modelValue', $event)"
    @close="handleClose"
  >
    <el-tabs v-model="activeTab">
      <!-- 单个添加 -->
      <el-tab-pane label="单个添加" name="single">
        <el-form
          ref="singleFormRef"
          :model="singleForm"
          :rules="singleRules"
          label-width="80px"
        >
          <el-form-item label="关键词" prop="keyword">
            <el-input
              v-model="singleForm.keyword"
              placeholder="输入要监控的关键词"
              clearable
            />
          </el-form-item>
          <el-form-item label="ASIN" prop="asin">
            <el-input
              v-model="singleForm.asin"
              placeholder="输入目标产品的ASIN"
              clearable
            />
          </el-form-item>
          <el-form-item label="站点" prop="country">
            <el-select v-model="singleForm.country" style="width: 100%">
              <el-option
                v-for="opt in COUNTRY_OPTIONS"
                :key="opt.value"
                :value="opt.value"
              >
                <span class="country-option">
                  <span class="country-flag-small" v-html="opt.flag"></span>
                  <span>{{ opt.label }}</span>
                </span>
              </el-option>
            </el-select>
          </el-form-item>
          <el-form-item label="优先级" prop="priority">
            <el-select v-model="singleForm.priority" style="width: 100%">
              <el-option
                v-for="opt in PRIORITY_OPTIONS"
                :key="opt.value"
                :label="opt.label"
                :value="opt.value"
              />
            </el-select>
          </el-form-item>
        </el-form>
      </el-tab-pane>

      <!-- 批量添加 -->
      <el-tab-pane label="批量添加" name="batch">
        <div class="batch-tip">
          每行一条，格式：<code>关键词,ASIN,站点,优先级</code>
          <br>
          示例：<code>wireless mouse,B09X1234AB,US,high</code>
          <br>
          站点可选: US, UK, DE, FR, IT, ES；优先级可选: high, medium, low
        </div>
        <el-input
          v-model="batchText"
          type="textarea"
          :rows="10"
          placeholder="每行一条，用逗号分隔"
        />
        <div class="batch-preview" v-if="batchItems.length">
          <div class="preview-header">预览 ({{ batchItems.length }} 条)</div>
          <el-table :data="batchItems.slice(0, 5)" size="small" border>
            <el-table-column prop="keyword" label="关键词" />
            <el-table-column prop="asin" label="ASIN" width="120" />
            <el-table-column prop="country" label="站点" width="80" />
            <el-table-column prop="priority" label="优先级" width="80" />
          </el-table>
          <div v-if="batchItems.length > 5" class="more-hint">
            ... 还有 {{ batchItems.length - 5 }} 条
          </div>
        </div>
      </el-tab-pane>
    </el-tabs>

    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button
        type="primary"
        :loading="submitting"
        @click="handleSubmit"
      >
        确定
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue';
import type { FormInstance, FormRules } from 'element-plus';
import { ElMessage } from 'element-plus';
import { addKeywordMonitoring, batchAddKeywordMonitoring } from '../api';
import { COUNTRY_OPTIONS, PRIORITY_OPTIONS } from '../types';

const props = defineProps<{
  modelValue: boolean;
  productId: number;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'success'): void;
}>();

const activeTab = ref('single');
const submitting = ref(false);

// 单个添加表单
const singleFormRef = ref<FormInstance>();
const singleForm = reactive({
  keyword: '',
  asin: '',
  country: 'US',
  priority: 'medium',
});

const singleRules: FormRules = {
  keyword: [
    { required: true, message: '请输入关键词', trigger: 'blur' },
  ],
  asin: [
    { required: true, message: '请输入ASIN', trigger: 'blur' },
    { pattern: /^[A-Z0-9]{10}$/, message: 'ASIN格式不正确', trigger: 'blur' },
  ],
  country: [
    { required: true, message: '请选择站点', trigger: 'change' },
  ],
};

// 批量添加
const batchText = ref('');

const batchItems = computed(() => {
  if (!batchText.value.trim()) return [];

  const lines = batchText.value.trim().split('\n');
  const items: { keyword: string; asin: string; country: string; priority: string }[] = [];

  for (const line of lines) {
    const parts = line.split(',').map(s => s.trim());
    if (parts.length >= 2) {
      const keyword = parts[0];
      const asin = parts[1].toUpperCase();
      const country = (parts[2] || 'US').toUpperCase();
      const priority = (parts[3] || 'medium').toLowerCase();

      if (keyword && /^[A-Z0-9]{10}$/.test(asin)) {
        items.push({
          keyword,
          asin,
          country: COUNTRY_OPTIONS.find(c => c.value === country) ? country : 'US',
          priority: PRIORITY_OPTIONS.find(p => p.value === priority) ? priority : 'medium',
        });
      }
    }
  }

  return items;
});

// 提交
async function handleSubmit() {
  if (activeTab.value === 'single') {
    await submitSingle();
  } else {
    await submitBatch();
  }
}

async function submitSingle() {
  try {
    await singleFormRef.value?.validate();
  } catch {
    return;
  }

  submitting.value = true;
  try {
    await addKeywordMonitoring(
      props.productId,
      singleForm.keyword,
      singleForm.asin.toUpperCase(),
      singleForm.country,
      singleForm.priority
    );
    ElMessage.success('添加成功');
    emit('success');
    handleClose();
  } catch (e) {
    ElMessage.error(`添加失败: ${e}`);
  } finally {
    submitting.value = false;
  }
}

async function submitBatch() {
  if (!batchItems.value.length) {
    ElMessage.warning('请输入有效的监控数据');
    return;
  }

  submitting.value = true;
  try {
    await batchAddKeywordMonitoring(props.productId, batchItems.value);
    ElMessage.success(`成功添加 ${batchItems.value.length} 条监控`);
    emit('success');
    handleClose();
  } catch (e) {
    ElMessage.error(`批量添加失败: ${e}`);
  } finally {
    submitting.value = false;
  }
}

// 关闭
function handleClose() {
  singleForm.keyword = '';
  singleForm.asin = '';
  singleForm.country = 'US';
  singleForm.priority = 'medium';
  batchText.value = '';
  singleFormRef.value?.resetFields();
  emit('update:modelValue', false);
}

// 监听打开
watch(() => props.modelValue, (val) => {
  if (val) {
    activeTab.value = 'single';
  }
});
</script>

<style scoped>
.batch-tip {
  margin-bottom: 12px;
  padding: 12px;
  background: var(--el-fill-color-light);
  border-radius: 4px;
  font-size: 13px;
  line-height: 1.6;
}

.batch-tip code {
  background: var(--el-fill-color-darker);
  padding: 2px 6px;
  border-radius: 3px;
  font-family: monospace;
}

.batch-preview {
  margin-top: 16px;
}

.preview-header {
  margin-bottom: 8px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.more-hint {
  margin-top: 8px;
  font-size: 12px;
  color: var(--el-text-color-placeholder);
  text-align: center;
}

/* 国旗样式 */
.country-option {
  display: flex;
  align-items: center;
  gap: 6px;
}

.country-flag-small {
  display: inline-flex;
  width: 18px;
  height: 12px;
}

.country-flag-small :deep(svg) {
  width: 100%;
  height: 100%;
  border-radius: 2px;
  box-shadow: 0 0 1px rgba(0, 0, 0, 0.2);
}
</style>

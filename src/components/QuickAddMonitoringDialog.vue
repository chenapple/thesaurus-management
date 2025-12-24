<template>
  <el-dialog
    :model-value="modelValue"
    title="批量添加到排名监控"
    width="700px"
    @update:model-value="$emit('update:modelValue', $event)"
    @close="handleClose"
  >
    <!-- 已选关键词预览 -->
    <div class="selected-keywords-preview">
      <div class="preview-header">
        <span>已选择 {{ localKeywords.length }} 个关键词</span>
        <el-button text type="primary" size="small" @click="showAllKeywords = !showAllKeywords">
          {{ showAllKeywords ? '收起' : '展开全部' }}
        </el-button>
      </div>
      <div class="keywords-list" :class="{ expanded: showAllKeywords }">
        <el-tag
          v-for="kw in displayKeywords"
          :key="kw.id"
          size="small"
          class="keyword-tag"
          closable
          @close="removeKeyword(kw)"
        >
          {{ kw.keyword }}
        </el-tag>
        <span v-if="!showAllKeywords && localKeywords.length > 5" class="more-count">
          +{{ localKeywords.length - 5 }} 更多
        </span>
      </div>
    </div>

    <el-divider />

    <!-- ASIN 输入表单 -->
    <el-form
      ref="formRef"
      :model="form"
      :rules="rules"
      label-width="80px"
    >
      <el-form-item label="ASIN" prop="asin">
        <el-input
          v-model="form.asin"
          placeholder="输入目标产品的ASIN (10位)"
          clearable
          style="width: 100%"
          @input="form.asin = form.asin.toUpperCase()"
        />
        <div class="form-tip">
          所有选中的关键词将使用相同的 ASIN 进行排名监控
        </div>
      </el-form-item>

      <el-form-item label="站点" prop="country">
        <el-select v-model="form.country" style="width: 100%">
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
        <el-select v-model="form.priority" style="width: 100%">
          <el-option
            v-for="opt in PRIORITY_OPTIONS"
            :key="opt.value"
            :label="opt.label"
            :value="opt.value"
          />
        </el-select>
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button
        type="primary"
        :loading="submitting"
        :disabled="localKeywords.length === 0"
        @click="handleSubmit"
      >
        添加 {{ localKeywords.length }} 个关键词
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue';
import type { FormInstance, FormRules } from 'element-plus';
import { ElMessage } from 'element-plus';
import { batchAddKeywordMonitoring } from '../api';
import { COUNTRY_OPTIONS, PRIORITY_OPTIONS } from '../types';
import type { KeywordData } from '../types';

const props = defineProps<{
  modelValue: boolean;
  productId: number;
  productCountry: string | null;
  keywords: KeywordData[];
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'success'): void;
}>();

const formRef = ref<FormInstance>();
const submitting = ref(false);
const showAllKeywords = ref(false);

// 本地关键词列表（可移除）
const localKeywords = ref<KeywordData[]>([]);

// 表单
const form = reactive({
  asin: '',
  country: 'US',
  priority: 'medium',
});

// 校验规则
const rules: FormRules = {
  asin: [
    { required: true, message: '请输入ASIN', trigger: 'blur' },
    { pattern: /^[A-Z0-9]{10}$/, message: 'ASIN必须是10位字母数字', trigger: 'blur' },
  ],
  country: [
    { required: true, message: '请选择站点', trigger: 'change' },
  ],
};

// 计算展示的关键词（最多5个或全部）
const displayKeywords = computed(() => {
  if (showAllKeywords.value) {
    return localKeywords.value;
  }
  return localKeywords.value.slice(0, 5);
});

// 移除单个关键词
function removeKeyword(kw: KeywordData) {
  const index = localKeywords.value.findIndex(k => k.id === kw.id);
  if (index > -1) {
    localKeywords.value.splice(index, 1);
  }
}

// 提交
async function handleSubmit() {
  try {
    await formRef.value?.validate();
  } catch {
    return;
  }

  if (localKeywords.value.length === 0) {
    ElMessage.warning('请至少选择一个关键词');
    return;
  }

  submitting.value = true;
  try {
    const items = localKeywords.value.map(kw => ({
      keyword: kw.keyword,
      asin: form.asin,
      country: form.country,
      priority: form.priority,
    }));

    await batchAddKeywordMonitoring(props.productId, items);
    ElMessage.success(`成功添加 ${items.length} 个关键词监控`);
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
  form.asin = '';
  form.country = props.productCountry || 'US';
  form.priority = 'medium';
  showAllKeywords.value = false;
  formRef.value?.resetFields();
  emit('update:modelValue', false);
}

// 监听打开
watch(() => props.modelValue, (val) => {
  if (val) {
    // 初始化本地关键词列表
    localKeywords.value = [...props.keywords];
    // 自动填充 country
    if (props.productCountry) {
      form.country = props.productCountry;
    }
  }
});
</script>

<style scoped>
.selected-keywords-preview {
  margin-bottom: 16px;
}

.preview-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  font-size: 14px;
  color: var(--el-text-color-regular);
}

.keywords-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  max-height: 80px;
  overflow: hidden;
}

.keywords-list.expanded {
  max-height: none;
}

.keyword-tag {
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.more-count {
  color: var(--el-text-color-secondary);
  font-size: 12px;
  align-self: center;
}

.form-tip {
  margin-top: 4px;
  font-size: 12px;
  color: var(--el-text-color-placeholder);
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

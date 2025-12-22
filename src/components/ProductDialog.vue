<script setup lang="ts">
interface ProductForm {
  id: number;
  name: string;
  country: string;
}

interface CountryOption {
  code: string;
  name: string;
  flag: string;
}

defineProps<{
  visible: boolean;
  isEditing: boolean;
  form: ProductForm;
  countryOptions: CountryOption[];
}>();

defineEmits<{
  (e: 'update:visible', value: boolean): void;
  (e: 'update:form', value: ProductForm): void;
  (e: 'save'): void;
}>();
</script>

<template>
  <el-dialog
    :model-value="visible"
    @update:model-value="$emit('update:visible', $event)"
    :title="isEditing ? '编辑产品' : '创建产品'"
    width="400px"
  >
    <el-form :model="form" label-width="80px">
      <el-form-item label="产品名称" required>
        <el-input
          :model-value="form.name"
          @update:model-value="$emit('update:form', { ...form, name: $event })"
          placeholder="请输入产品名称"
        />
      </el-form-item>
      <el-form-item label="国家">
        <el-select
          :model-value="form.country"
          @update:model-value="$emit('update:form', { ...form, country: $event })"
          placeholder="请选择国家"
          clearable
          style="width: 100%"
        >
          <el-option
            v-for="country in countryOptions"
            :key="country.code"
            :label="country.name"
            :value="country.code"
          >
            <span class="country-option">
              <span class="country-flag" v-html="country.flag"></span>
              <span>{{ country.name }}</span>
            </span>
          </el-option>
        </el-select>
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="$emit('update:visible', false)">取消</el-button>
      <el-button type="primary" @click="$emit('save')">确定</el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.country-option {
  display: flex;
  align-items: center;
  gap: 8px;
}

.country-flag {
  width: 24px;
  height: 16px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 2px;
  overflow: hidden;
  flex-shrink: 0;
}

.country-flag :deep(svg) {
  width: 100%;
  height: 100%;
}
</style>

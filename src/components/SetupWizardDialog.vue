<script setup lang="ts">
import { ref, computed } from 'vue';
import { ElMessage } from 'element-plus';
import { Key, View, Hide, Check, Right, InfoFilled, DataBoard } from '@element-plus/icons-vue';
import * as api from '../api';

defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
  (e: 'complete'): void;
  (e: 'start-onboarding'): void;
}>();

// 向导步骤
const currentStep = ref(0);
const loading = ref(false);

// API Key 状态
const deepseekKey = ref('');
const qwenKey = ref('');
const deepseekConfigured = ref(false);
const qwenConfigured = ref(false);
const showDeepseekPassword = ref(false);
const showQwenPassword = ref(false);

// 步骤定义
const steps = [
  { title: '欢迎', description: '开始配置' },
  { title: 'DeepSeek', description: 'AI 分析' },
  { title: '通义千问', description: '向量化 & OCR' },
  { title: '完成', description: '开始使用' },
];

// 计算属性
const isLastStep = computed(() => currentStep.value === steps.length - 1);

// 检查已有配置
async function checkExistingConfig() {
  loading.value = true;
  try {
    deepseekConfigured.value = await api.hasApiKey('deepseek');
    qwenConfigured.value = await api.hasApiKey('qwen');
  } catch (e) {
    console.error('检查 API Key 状态失败:', e);
  } finally {
    loading.value = false;
  }
}

// 保存 DeepSeek API Key
async function saveDeepseekKey() {
  if (!deepseekKey.value.trim()) return true; // 跳过空值

  loading.value = true;
  try {
    await api.setApiKey('deepseek', deepseekKey.value.trim());
    deepseekConfigured.value = true;
    ElMessage.success('DeepSeek API Key 已保存');
    return true;
  } catch (e) {
    ElMessage.error('保存失败: ' + e);
    return false;
  } finally {
    loading.value = false;
  }
}

// 保存通义千问 API Key
async function saveQwenKey() {
  if (!qwenKey.value.trim()) return true; // 跳过空值

  loading.value = true;
  try {
    await api.setApiKey('qwen', qwenKey.value.trim());
    qwenConfigured.value = true;
    ElMessage.success('通义千问 API Key 已保存');
    return true;
  } catch (e) {
    ElMessage.error('保存失败: ' + e);
    return false;
  } finally {
    loading.value = false;
  }
}

// 下一步
async function nextStep() {
  // 保存当前步骤的数据
  if (currentStep.value === 1 && deepseekKey.value.trim()) {
    const success = await saveDeepseekKey();
    if (!success) return;
  }
  if (currentStep.value === 2 && qwenKey.value.trim()) {
    const success = await saveQwenKey();
    if (!success) return;
  }

  if (isLastStep.value) {
    complete();
  } else {
    currentStep.value++;
  }
}

// 上一步
function prevStep() {
  if (currentStep.value > 0) {
    currentStep.value--;
  }
}

// 跳过当前步骤
function skipStep() {
  if (isLastStep.value) {
    complete();
  } else {
    currentStep.value++;
  }
}

// 完成向导
async function complete() {
  // 标记向导已完成（使用 setApiKey 存储到 settings 表）
  try {
    await api.setApiKey('__setup_wizard_completed', 'true');
  } catch (e) {
    console.error('保存向导状态失败:', e);
  }
  emit('complete');
  emit('update:visible', false);
  // 触发新手教程
  emit('start-onboarding');
}

// 稍后配置
function skipAll() {
  complete();
}

// 初始化
checkExistingConfig();
</script>

<template>
  <el-dialog
    :model-value="visible"
    @update:model-value="$emit('update:visible', $event)"
    title=""
    width="560px"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    :show-close="false"
    class="setup-wizard-dialog"
  >
    <!-- 步骤条 -->
    <el-steps :active="currentStep" finish-status="success" align-center class="wizard-steps">
      <el-step v-for="step in steps" :key="step.title" :title="step.title" />
    </el-steps>

    <div class="wizard-content" v-loading="loading">
      <!-- 步骤 0: 欢迎 -->
      <div v-if="currentStep === 0" class="step-content welcome-step">
        <div class="welcome-icon">
          <el-icon><DataBoard /></el-icon>
        </div>
        <h2>欢迎使用词库管理工具</h2>
        <p class="welcome-desc">
          在开始之前，我们需要配置一些必要的 API Key 以启用 AI 功能。
        </p>

        <el-alert type="info" :closable="false" show-icon class="info-alert">
          <template #title>
            <strong>需要配置的 API Key：</strong>
          </template>
          <div class="api-list">
            <div class="api-item">
              <el-icon :class="{ configured: deepseekConfigured }"><Check v-if="deepseekConfigured" /><Right v-else /></el-icon>
              <span><strong>DeepSeek</strong> - 词根分析、关键词分类</span>
            </div>
            <div class="api-item">
              <el-icon :class="{ configured: qwenConfigured }"><Check v-if="qwenConfigured" /><Right v-else /></el-icon>
              <span><strong>通义千问</strong> - 文档向量化、OCR 识别</span>
            </div>
          </div>
        </el-alert>

        <p class="note" v-if="deepseekConfigured && qwenConfigured">
          <el-icon><Check /></el-icon>
          所有必要的 API Key 已配置，可以直接开始使用！
        </p>
      </div>

      <!-- 步骤 1: DeepSeek -->
      <div v-else-if="currentStep === 1" class="step-content">
        <div class="step-header">
          <el-icon class="step-icon deepseek"><Key /></el-icon>
          <div>
            <h3>配置 DeepSeek API Key</h3>
            <p class="step-desc">用于 AI 词根分析、关键词分类和知识库问答</p>
          </div>
        </div>

        <div v-if="deepseekConfigured" class="already-configured">
          <el-icon><Check /></el-icon>
          <span>DeepSeek API Key 已配置</span>
        </div>

        <div v-else class="key-input-section">
          <el-input
            v-model="deepseekKey"
            :type="showDeepseekPassword ? 'text' : 'password'"
            placeholder="请输入 DeepSeek API Key (sk-...)"
            size="large"
            clearable
          >
            <template #prefix>
              <el-icon><Key /></el-icon>
            </template>
            <template #suffix>
              <el-icon class="password-toggle" @click="showDeepseekPassword = !showDeepseekPassword">
                <View v-if="showDeepseekPassword" />
                <Hide v-else />
              </el-icon>
            </template>
          </el-input>

          <div class="help-links">
            <el-link type="primary" href="https://platform.deepseek.com/api_keys" target="_blank">
              <el-icon><InfoFilled /></el-icon>
              如何获取 DeepSeek API Key？
            </el-link>
          </div>
        </div>
      </div>

      <!-- 步骤 2: 通义千问 -->
      <div v-else-if="currentStep === 2" class="step-content">
        <div class="step-header">
          <el-icon class="step-icon qwen"><Key /></el-icon>
          <div>
            <h3>配置通义千问 API Key</h3>
            <p class="step-desc">用于知识库文档向量化和图片/PDF OCR 识别</p>
          </div>
        </div>

        <div v-if="qwenConfigured" class="already-configured">
          <el-icon><Check /></el-icon>
          <span>通义千问 API Key 已配置</span>
        </div>

        <div v-else class="key-input-section">
          <el-input
            v-model="qwenKey"
            :type="showQwenPassword ? 'text' : 'password'"
            placeholder="请输入通义千问 API Key (sk-...)"
            size="large"
            clearable
          >
            <template #prefix>
              <el-icon><Key /></el-icon>
            </template>
            <template #suffix>
              <el-icon class="password-toggle" @click="showQwenPassword = !showQwenPassword">
                <View v-if="showQwenPassword" />
                <Hide v-else />
              </el-icon>
            </template>
          </el-input>

          <div class="help-links">
            <el-link type="primary" href="https://dashscope.console.aliyun.com/apiKey" target="_blank">
              <el-icon><InfoFilled /></el-icon>
              如何获取通义千问 API Key？
            </el-link>
          </div>
        </div>
      </div>

      <!-- 步骤 3: 完成 -->
      <div v-else-if="currentStep === 3" class="step-content complete-step">
        <div class="complete-icon">
          <el-icon><Check /></el-icon>
        </div>
        <h2>配置完成！</h2>

        <div class="config-summary">
          <div class="summary-item" :class="{ success: deepseekConfigured }">
            <el-icon><Check v-if="deepseekConfigured" /><Right v-else /></el-icon>
            <span>DeepSeek: {{ deepseekConfigured ? '已配置' : '未配置' }}</span>
          </div>
          <div class="summary-item" :class="{ success: qwenConfigured }">
            <el-icon><Check v-if="qwenConfigured" /><Right v-else /></el-icon>
            <span>通义千问: {{ qwenConfigured ? '已配置' : '未配置' }}</span>
          </div>
        </div>

        <p class="complete-note" v-if="!deepseekConfigured || !qwenConfigured">
          部分 API Key 未配置，相关功能将受限。<br/>
          你可以稍后在「设置 → API Key」中配置。
        </p>
        <p class="complete-note success" v-else>
          所有必要的 API Key 已配置，可以使用全部功能！
        </p>
      </div>
    </div>

    <template #footer>
      <div class="wizard-footer">
        <div class="left-actions">
          <el-button v-if="currentStep === 0" text @click="skipAll">
            稍后配置
          </el-button>
          <el-button v-else-if="currentStep > 0 && currentStep < 3" text @click="skipStep">
            跳过
          </el-button>
        </div>
        <div class="right-actions">
          <el-button v-if="currentStep > 0 && currentStep < 3" @click="prevStep">
            上一步
          </el-button>
          <el-button type="primary" @click="nextStep" :disabled="loading">
            {{ isLastStep ? '开始使用' : '下一步' }}
          </el-button>
        </div>
      </div>
    </template>
  </el-dialog>
</template>

<style scoped>
.wizard-steps {
  margin-bottom: 30px;
}

.wizard-content {
  min-height: 280px;
}

.step-content {
  padding: 10px 20px;
}

/* 欢迎页 */
.welcome-step {
  text-align: center;
}

.welcome-icon {
  margin-bottom: 20px;
}

.welcome-icon .el-icon {
  font-size: 64px;
  color: var(--el-color-primary);
  padding: 16px;
  background: var(--el-color-primary-light-9);
  border-radius: 16px;
}

.welcome-step h2 {
  margin: 0 0 10px 0;
  font-size: 22px;
  font-weight: 600;
}

.welcome-desc {
  color: var(--el-text-color-secondary);
  margin-bottom: 20px;
}

.info-alert {
  text-align: left;
  margin-bottom: 16px;
}

.api-list {
  margin-top: 8px;
}

.api-item {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 6px 0;
  font-size: 14px;
}

.api-item .el-icon {
  color: var(--el-text-color-secondary);
}

.api-item .el-icon.configured {
  color: var(--el-color-success);
}

.note {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  color: var(--el-color-success);
  font-size: 14px;
}

/* 配置步骤 */
.step-header {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 24px;
}

.step-icon {
  font-size: 40px;
  padding: 12px;
  border-radius: 12px;
  background: var(--el-fill-color-light);
}

.step-icon.deepseek {
  color: #4a6cf7;
  background: rgba(74, 108, 247, 0.1);
}

.step-icon.qwen {
  color: #ff6a00;
  background: rgba(255, 106, 0, 0.1);
}

.step-header h3 {
  margin: 0 0 4px 0;
  font-size: 18px;
  font-weight: 600;
}

.step-desc {
  margin: 0;
  color: var(--el-text-color-secondary);
  font-size: 14px;
}

.already-configured {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 40px;
  background: var(--el-color-success-light-9);
  border-radius: 8px;
  color: var(--el-color-success);
  font-size: 16px;
}

.already-configured .el-icon {
  font-size: 24px;
}

.key-input-section {
  margin-top: 20px;
}

.key-input-section .el-input {
  margin-bottom: 12px;
}

.help-links {
  display: flex;
  justify-content: flex-end;
}

.help-links .el-link {
  font-size: 13px;
}

.password-toggle {
  cursor: pointer;
  color: var(--el-text-color-secondary);
}

.password-toggle:hover {
  color: var(--el-color-primary);
}

/* 完成页 */
.complete-step {
  text-align: center;
}

.complete-icon {
  width: 80px;
  height: 80px;
  margin: 0 auto 20px;
  background: var(--el-color-success-light-9);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.complete-icon .el-icon {
  font-size: 40px;
  color: var(--el-color-success);
}

.complete-step h2 {
  margin: 0 0 20px 0;
  font-size: 22px;
  font-weight: 600;
}

.config-summary {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 20px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
  margin-bottom: 16px;
}

.summary-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  color: var(--el-text-color-secondary);
}

.summary-item.success {
  color: var(--el-color-success);
}

.summary-item .el-icon {
  font-size: 18px;
}

.complete-note {
  color: var(--el-text-color-secondary);
  font-size: 14px;
  line-height: 1.6;
}

.complete-note.success {
  color: var(--el-color-success);
}

/* 底部按钮 */
.wizard-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.left-actions,
.right-actions {
  display: flex;
  gap: 8px;
}
</style>

<style>
/* 全局样式 - 隐藏对话框头部 */
.setup-wizard-dialog .el-dialog__header {
  display: none;
}

.setup-wizard-dialog .el-dialog__body {
  padding-top: 30px;
}
</style>

<template>
  <el-dialog
    :model-value="modelValue"
    title="安装运行时依赖"
    width="500px"
    :close-on-click-modal="!installing"
    :close-on-press-escape="!installing"
    :show-close="!installing"
    @update:model-value="$emit('update:modelValue', $event)"
  >
    <!-- 检测中 -->
    <div v-if="checking" class="checking-container">
      <el-icon class="is-loading" :size="32"><Loading /></el-icon>
      <span>正在检测依赖...</span>
    </div>

    <!-- 依赖状态列表 -->
    <div v-else-if="!installing" class="status-container">
      <el-alert
        type="warning"
        :closable="false"
        show-icon
        class="status-alert"
      >
        <template #title>检测到缺少运行时依赖</template>
        <template #default>
          关键词排名检测功能需要 Python 和 Playwright 支持。
          点击下方按钮自动安装所需依赖。
        </template>
      </el-alert>

      <div class="status-list">
        <div class="status-item">
          <div class="status-icon">
            <el-icon :class="status.python_installed ? 'success' : 'error'">
              <CircleCheck v-if="status.python_installed" />
              <CircleClose v-else />
            </el-icon>
          </div>
          <div class="status-info">
            <div class="status-name">Python 3</div>
            <div class="status-detail">
              {{ status.python_installed
                ? status.python_version || '已安装'
                : '未安装' }}
            </div>
          </div>
        </div>

        <div class="status-item">
          <div class="status-icon">
            <el-icon :class="status.playwright_installed ? 'success' : 'error'">
              <CircleCheck v-if="status.playwright_installed" />
              <CircleClose v-else />
            </el-icon>
          </div>
          <div class="status-info">
            <div class="status-name">Playwright</div>
            <div class="status-detail">
              {{ status.playwright_installed ? '已安装' : '未安装' }}
            </div>
          </div>
        </div>

        <div class="status-item">
          <div class="status-icon">
            <el-icon :class="status.chromium_installed ? 'success' : 'error'">
              <CircleCheck v-if="status.chromium_installed" />
              <CircleClose v-else />
            </el-icon>
          </div>
          <div class="status-info">
            <div class="status-name">Chromium 浏览器</div>
            <div class="status-detail">
              {{ status.chromium_installed ? '已安装' : '未安装 (约150MB)' }}
            </div>
          </div>
        </div>
      </div>

      <!-- macOS/Linux 手动安装提示 -->
      <el-alert
        v-if="!isWindows && !status.python_installed"
        type="info"
        :closable="false"
        class="manual-hint"
      >
        <template #title>请先安装 Python</template>
        <template #default>
          <div class="manual-commands">
            <div><strong>macOS:</strong> <code>brew install python@3.11</code></div>
            <div><strong>Linux:</strong> <code>sudo apt install python3</code></div>
          </div>
        </template>
      </el-alert>
    </div>

    <!-- 安装进度 -->
    <div v-else class="install-container">
      <div class="install-header">
        <el-icon class="is-loading" :size="24"><Loading /></el-icon>
        <span>{{ currentProgress.step_name }}</span>
      </div>

      <div class="install-steps">
        <div
          v-for="step in installSteps"
          :key="step.key"
          class="install-step"
          :class="{
            'is-active': currentProgress.step === step.key,
            'is-done': completedSteps.has(step.key),
            'is-error': errorSteps.has(step.key),
          }"
        >
          <div class="step-icon">
            <el-icon v-if="completedSteps.has(step.key)" class="success"><CircleCheck /></el-icon>
            <el-icon v-else-if="errorSteps.has(step.key)" class="error"><CircleClose /></el-icon>
            <el-icon v-else-if="currentProgress.step === step.key" class="is-loading"><Loading /></el-icon>
            <span v-else class="step-number">{{ step.number }}</span>
          </div>
          <div class="step-name">{{ step.name }}</div>
        </div>
      </div>

      <el-progress
        :percentage="currentProgress.progress"
        :status="currentProgress.is_error ? 'exception' : undefined"
        :stroke-width="10"
      />

      <div class="install-message">{{ currentProgress.message }}</div>

      <el-alert
        v-if="installError"
        type="error"
        :closable="false"
        class="install-error"
      >
        <template #title>安装失败</template>
        <template #default>{{ installError }}</template>
      </el-alert>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button
          v-if="!installing && !installComplete"
          @click="$emit('update:modelValue', false)"
        >
          稍后安装
        </el-button>
        <el-button
          v-if="!installing && !installComplete && needsInstall"
          type="primary"
          @click="startInstall"
          :disabled="!canInstall"
        >
          <el-icon><Download /></el-icon>
          一键安装
        </el-button>
        <el-button
          v-if="installing && installError"
          type="primary"
          @click="retryInstall"
        >
          重试安装
        </el-button>
        <el-button
          v-if="installComplete"
          type="success"
          @click="handleComplete"
        >
          <el-icon><CircleCheck /></el-icon>
          完成
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, onUnmounted } from 'vue';
import { ElMessage } from 'element-plus';
import { Loading, CircleCheck, CircleClose, Download } from '@element-plus/icons-vue';
import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { checkDependencies, installAllDependencies, installPlaywrightOnly } from '../api';
import type { DependencyStatus, InstallProgress } from '../types';

const props = defineProps<{
  modelValue: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'installed'): void;
}>();

// 状态
const checking = ref(false);
const installing = ref(false);
const installComplete = ref(false);
const installError = ref<string | null>(null);

const status = reactive<DependencyStatus>({
  python_installed: false,
  python_version: null,
  python_path: null,
  playwright_installed: false,
  chromium_installed: false,
  pdf2image_installed: false,
  poppler_installed: false,
  error_message: null,
});

const currentProgress = reactive<InstallProgress>({
  step: 'python',
  step_name: '准备安装...',
  progress: 0,
  message: '',
  is_error: false,
});

const completedSteps = ref<Set<string>>(new Set());
const errorSteps = ref<Set<string>>(new Set());

// 事件监听
let unlistenProgress: UnlistenFn | null = null;
let unlistenComplete: UnlistenFn | null = null;

// 计算属性
const isWindows = computed(() => {
  return navigator.userAgent.includes('Windows');
});

const needsInstall = computed(() => {
  return !status.python_installed ||
         !status.playwright_installed ||
         !status.chromium_installed;
});

const canInstall = computed(() => {
  // Windows 可以自动安装 Python
  // macOS 需要 Homebrew，如果有 Python 则可以继续
  return isWindows.value || status.python_installed;
});

const installSteps = computed(() => {
  const steps: { key: string; number: number; name: string }[] = [];
  let num = 1;

  if (!status.python_installed) {
    steps.push({ key: 'python', number: num++, name: 'Python 3' });
  }
  if (!status.playwright_installed) {
    steps.push({ key: 'playwright', number: num++, name: 'Playwright' });
  }
  if (!status.chromium_installed) {
    steps.push({ key: 'chromium', number: num++, name: 'Chromium' });
  }
  return steps;
});

// 检查依赖
async function checkStatus() {
  checking.value = true;
  try {
    const result = await checkDependencies();
    Object.assign(status, result);
  } catch (e) {
    ElMessage.error(`检查依赖失败: ${e}`);
  } finally {
    checking.value = false;
  }
}

// 开始安装
async function startInstall() {
  installing.value = true;
  installComplete.value = false;
  installError.value = null;
  completedSteps.value.clear();
  errorSteps.value.clear();

  // 重置进度
  currentProgress.step = status.python_installed ? 'playwright' : 'python';
  currentProgress.step_name = '准备安装...';
  currentProgress.progress = 0;
  currentProgress.message = '';
  currentProgress.is_error = false;

  // 设置进度监听
  unlistenProgress = await listen<InstallProgress>('install-progress', (event) => {
    Object.assign(currentProgress, event.payload);

    // 进度达到100表示该步骤完成
    if (event.payload.progress >= 100 && !event.payload.is_error) {
      completedSteps.value.add(event.payload.step);
    }
    if (event.payload.is_error) {
      errorSteps.value.add(event.payload.step);
    }
  });

  unlistenComplete = await listen('install-complete', () => {
    installComplete.value = true;
    installing.value = false;
  });

  try {
    const result = status.python_installed
      ? await installPlaywrightOnly()
      : await installAllDependencies();

    if (result.success) {
      installComplete.value = true;
      ElMessage.success('依赖安装完成!');
    } else {
      installError.value = result.message;
    }
  } catch (e) {
    installError.value = String(e);
  } finally {
    installing.value = false;
    // 清理监听器
    if (unlistenProgress) {
      unlistenProgress();
      unlistenProgress = null;
    }
    if (unlistenComplete) {
      unlistenComplete();
      unlistenComplete = null;
    }
  }
}

// 重试安装
function retryInstall() {
  installError.value = null;
  startInstall();
}

// 完成
function handleComplete() {
  emit('update:modelValue', false);
  emit('installed');
}

// 监听对话框打开
watch(() => props.modelValue, (val) => {
  if (val) {
    // 重置状态
    installComplete.value = false;
    installError.value = null;
    completedSteps.value.clear();
    errorSteps.value.clear();
    // 检查依赖
    checkStatus();
  }
});

// 清理
onUnmounted(() => {
  if (unlistenProgress) unlistenProgress();
  if (unlistenComplete) unlistenComplete();
});
</script>

<style scoped>
.checking-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 32px;
  color: var(--el-text-color-secondary);
}

.status-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.status-alert {
  margin-bottom: 8px;
}

.status-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
  padding: 16px;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-icon {
  font-size: 24px;
}

.status-icon .success {
  color: var(--el-color-success);
}

.status-icon .error {
  color: var(--el-color-danger);
}

.status-info {
  flex: 1;
}

.status-name {
  font-weight: 600;
  font-size: 14px;
}

.status-detail {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 2px;
}

.manual-hint {
  margin-top: 8px;
}

.manual-commands {
  margin-top: 8px;
}

.manual-commands code {
  background: var(--el-fill-color);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: monospace;
}

.install-container {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.install-header {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 16px;
  font-weight: 600;
}

.install-steps {
  display: flex;
  justify-content: center;
  gap: 24px;
}

.install-step {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  opacity: 0.4;
  transition: opacity 0.3s;
}

.install-step.is-active,
.install-step.is-done,
.install-step.is-error {
  opacity: 1;
}

.step-icon {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: var(--el-fill-color);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
}

.step-icon .success {
  color: var(--el-color-success);
}

.step-icon .error {
  color: var(--el-color-danger);
}

.step-number {
  font-weight: 600;
  color: var(--el-text-color-secondary);
}

.step-name {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.install-message {
  font-size: 13px;
  color: var(--el-text-color-secondary);
  text-align: center;
  min-height: 20px;
  word-break: break-all;
}

.install-error {
  margin-top: 8px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Key, View, Hide, Delete } from '@element-plus/icons-vue';
import * as api from '../api';

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
}>();

// API Key 配置
interface ApiKeyConfig {
  name: string;
  displayName: string;
  description: string;
}

const apiKeyConfigs: ApiKeyConfig[] = [
  {
    name: 'deepseek',
    displayName: 'DeepSeek',
    description: '用于 AI 词根分析、关键词分类和知识库问答',
  },
  {
    name: 'openai',
    displayName: 'OpenAI',
    description: '用于知识库问答 (GPT-4o, GPT-4 等)',
  },
  {
    name: 'claude',
    displayName: 'Claude',
    description: '用于知识库问答 (Claude 3.5 Sonnet 等)',
  },
  {
    name: 'gemini',
    displayName: 'Gemini',
    description: '用于知识库问答 (Gemini 2.0, 1.5 等)',
  },
];

// 状态
const loading = ref(false);
const apiKeyStatus = ref<Record<string, boolean>>({});
const editingKey = ref<string | null>(null);
const newApiKey = ref('');
const showPassword = ref(false);

// 当对话框打开时检查 API Key 状态
watch(() => props.visible, async (newVisible) => {
  if (newVisible) {
    await checkApiKeyStatus();
  } else {
    // 关闭时重置状态
    editingKey.value = null;
    newApiKey.value = '';
    showPassword.value = false;
  }
});

async function checkApiKeyStatus() {
  loading.value = true;
  try {
    for (const config of apiKeyConfigs) {
      apiKeyStatus.value[config.name] = await api.hasApiKey(config.name);
    }
  } catch (e) {
    console.error('检查 API Key 状态失败:', e);
  } finally {
    loading.value = false;
  }
}

async function startEdit(keyName: string) {
  editingKey.value = keyName;
  showPassword.value = false;
  // 如果已配置，加载现有的 Key
  if (apiKeyStatus.value[keyName]) {
    try {
      const existingKey = await api.getApiKey(keyName);
      newApiKey.value = existingKey || '';
    } catch (e) {
      newApiKey.value = '';
    }
  } else {
    newApiKey.value = '';
  }
}

function cancelEdit() {
  editingKey.value = null;
  newApiKey.value = '';
  showPassword.value = false;
}

async function saveApiKey(keyName: string) {
  if (!newApiKey.value.trim()) {
    ElMessage.warning('请输入 API Key');
    return;
  }

  loading.value = true;
  try {
    await api.setApiKey(keyName, newApiKey.value.trim());
    apiKeyStatus.value[keyName] = true;
    editingKey.value = null;
    newApiKey.value = '';
    ElMessage.success('API Key 已保存');
  } catch (e) {
    ElMessage.error('保存失败: ' + e);
  } finally {
    loading.value = false;
  }
}

async function deleteApiKey(keyName: string, displayName: string) {
  try {
    await ElMessageBox.confirm(
      `确定要删除 ${displayName} API Key 吗？删除后 AI 分析功能将无法使用。`,
      '确认删除',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );

    loading.value = true;
    await api.deleteApiKey(keyName);
    apiKeyStatus.value[keyName] = false;
    ElMessage.success('API Key 已删除');
  } catch (e) {
    if (e !== 'cancel') {
      ElMessage.error('删除失败: ' + e);
    }
  } finally {
    loading.value = false;
  }
}

function closeDialog() {
  emit('update:visible', false);
}
</script>

<template>
  <el-dialog
    :model-value="visible"
    @update:model-value="$emit('update:visible', $event)"
    title="API Key 设置"
    width="500px"
    :close-on-click-modal="false"
  >
    <div class="api-key-dialog" v-loading="loading">
      <el-alert
        type="info"
        :closable="false"
        show-icon
        style="margin-bottom: 20px"
      >
        <template #title>
          API Key 安全存储于本地数据库
        </template>
      </el-alert>

      <div v-for="config in apiKeyConfigs" :key="config.name" class="api-key-item">
        <div class="api-key-header">
          <div class="api-key-info">
            <el-icon class="key-icon"><Key /></el-icon>
            <div>
              <div class="key-name">{{ config.displayName }}</div>
              <div class="key-desc">{{ config.description }}</div>
            </div>
          </div>
          <el-tag :type="apiKeyStatus[config.name] ? 'success' : 'info'" size="small">
            {{ apiKeyStatus[config.name] ? '已配置' : '未配置' }}
          </el-tag>
        </div>

        <!-- 编辑模式 -->
        <div v-if="editingKey === config.name" class="api-key-edit">
          <el-input
            v-model="newApiKey"
            :type="showPassword ? 'text' : 'password'"
            placeholder="请输入 API Key"
            clearable
          >
            <template #suffix>
              <el-icon
                class="password-toggle"
                @click="showPassword = !showPassword"
              >
                <View v-if="showPassword" />
                <Hide v-else />
              </el-icon>
            </template>
          </el-input>
          <div class="edit-actions">
            <el-button size="small" @click="cancelEdit">取消</el-button>
            <el-button type="primary" size="small" @click="saveApiKey(config.name)">
              保存
            </el-button>
          </div>
        </div>

        <!-- 查看模式 -->
        <div v-else class="api-key-actions">
          <el-button
            v-if="apiKeyStatus[config.name]"
            type="primary"
            text
            @click="startEdit(config.name)"
          >
            更新
          </el-button>
          <el-button
            v-else
            type="primary"
            @click="startEdit(config.name)"
          >
            配置
          </el-button>
          <el-button
            v-if="apiKeyStatus[config.name]"
            type="danger"
            text
            :icon="Delete"
            @click="deleteApiKey(config.name, config.displayName)"
          >
            删除
          </el-button>
        </div>
      </div>
    </div>

    <template #footer>
      <el-button @click="closeDialog">关闭</el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.api-key-dialog {
  min-height: 150px;
}

.api-key-item {
  padding: 16px;
  border: 1px solid var(--el-border-color-light);
  border-radius: 8px;
  margin-bottom: 12px;
}

.api-key-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.api-key-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.key-icon {
  font-size: 24px;
  color: var(--el-color-primary);
}

.key-name {
  font-weight: 600;
  font-size: 14px;
}

.key-desc {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 2px;
}

.api-key-edit {
  margin-top: 12px;
}

.edit-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 8px;
}

.api-key-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 12px;
}

.password-toggle {
  cursor: pointer;
  color: var(--el-text-color-secondary);
}

.password-toggle:hover {
  color: var(--el-color-primary);
}
</style>

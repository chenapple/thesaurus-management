<script setup lang="ts">
import { ref, onMounted, computed, nextTick, watch } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { open } from '@tauri-apps/plugin-dialog';
import * as api from '../api';
import { chatStream, buildRAGSystemPrompt, parseSourceReferences, checkApiKeyConfigured } from '../ai-service';
import type { ChatMessage } from '../ai-service';
import type { KbDocument, KbConversation, KbMessage, KbSearchResult, AIProvider } from '../types';
import { AI_PROVIDERS } from '../types';

// ==================== 状态 ====================

// 文档管理
const documents = ref<KbDocument[]>([]);
const loadingDocuments = ref(false);
const processingDocuments = ref<Set<number>>(new Set());

// AI 对话
const conversations = ref<KbConversation[]>([]);
const currentConversation = ref<KbConversation | null>(null);
const messages = ref<KbMessage[]>([]);
const inputMessage = ref('');
const isGenerating = ref(false);
const streamingContent = ref('');

// AI 设置
const selectedProvider = ref<AIProvider>('deepseek');
const selectedModel = ref(AI_PROVIDERS.deepseek.defaultModel);

// UI 状态
const activeSection = ref<'documents' | 'chat'>('chat');
const messagesContainer = ref<HTMLElement | null>(null);
const abortController = ref<AbortController | null>(null);

// ==================== 计算属性 ====================

const availableModels = computed(() => {
  return AI_PROVIDERS[selectedProvider.value].models;
});

const displayMessages = computed(() => {
  const result: (KbMessage | { role: 'assistant'; content: string; isStreaming: true })[] = [...messages.value];
  if (isGenerating.value && streamingContent.value) {
    result.push({
      role: 'assistant',
      content: streamingContent.value,
      isStreaming: true,
    });
  }
  return result;
});

// ==================== 文档管理 ====================

async function loadDocuments() {
  loadingDocuments.value = true;
  try {
    documents.value = await api.kbGetDocuments();
  } catch (e) {
    console.error('加载文档失败:', e);
    ElMessage.error('加载文档失败');
  } finally {
    loadingDocuments.value = false;
  }
}

async function handleUploadDocument() {
  const selected = await open({
    multiple: true,
    filters: [
      {
        name: '支持的文档',
        extensions: ['pdf', 'docx', 'xlsx', 'xls', 'md', 'txt'],
      },
    ],
  });

  if (!selected) return;

  const files = Array.isArray(selected) ? selected : [selected];

  for (const filePath of files) {
    try {
      // 提取文件信息
      const parts = filePath.split(/[/\\]/);
      const fileName = parts[parts.length - 1];
      const ext = fileName.split('.').pop()?.toLowerCase() || 'txt';
      const title = fileName.replace(/\.[^.]+$/, '');

      // 添加文档记录
      const docId = await api.kbAddDocument(null, title, fileName, filePath, ext);

      // 处理文档（解析 + 分块）
      processingDocuments.value.add(docId);
      await loadDocuments();

      const chunkCount = await api.kbProcessDocument(docId, filePath);

      processingDocuments.value.delete(docId);
      await loadDocuments();

      ElMessage.success(`文档 "${title}" 处理完成，生成 ${chunkCount} 个分块`);
    } catch (e) {
      console.error('上传文档失败:', e);
      ElMessage.error(`上传失败: ${e}`);
    }
  }
}

async function handleDeleteDocument(doc: KbDocument) {
  try {
    await ElMessageBox.confirm(`确定要删除文档 "${doc.title}" 吗？`, '确认删除', {
      confirmButtonText: '删除',
      cancelButtonText: '取消',
      type: 'warning',
    });

    await api.kbDeleteDocument(doc.id);
    await loadDocuments();
    ElMessage.success('文档已删除');
  } catch (e) {
    if (e !== 'cancel') {
      console.error('删除文档失败:', e);
      ElMessage.error('删除失败');
    }
  }
}

function getFileTypeIcon(fileType: string): string {
  const icons: Record<string, string> = {
    pdf: 'DocumentCopy',
    docx: 'Document',
    xlsx: 'Grid',
    xls: 'Grid',
    md: 'EditPen',
    txt: 'Tickets',
  };
  return icons[fileType] || 'Document';
}

function getStatusTag(status: string): { type: 'success' | 'warning' | 'danger' | 'info'; label: string } {
  const statusMap: Record<string, { type: 'success' | 'warning' | 'danger' | 'info'; label: string }> = {
    pending: { type: 'info', label: '待处理' },
    processing: { type: 'warning', label: '处理中' },
    completed: { type: 'success', label: '已完成' },
    failed: { type: 'danger', label: '失败' },
  };
  return statusMap[status] || { type: 'info', label: status };
}

// ==================== 对话管理 ====================

async function loadConversations() {
  try {
    conversations.value = await api.kbGetConversations();
  } catch (e) {
    console.error('加载对话失败:', e);
  }
}

async function createNewConversation() {
  try {
    const id = await api.kbCreateConversation(
      selectedProvider.value,
      selectedModel.value,
      '新对话'
    );
    await loadConversations();

    // 选中新对话
    currentConversation.value = conversations.value.find(c => c.id === id) || null;
    messages.value = [];
    inputMessage.value = '';
  } catch (e) {
    console.error('创建对话失败:', e);
    ElMessage.error('创建对话失败');
  }
}

async function selectConversation(conv: KbConversation) {
  currentConversation.value = conv;
  selectedProvider.value = (conv.ai_provider as AIProvider) || 'deepseek';
  selectedModel.value = conv.ai_model || AI_PROVIDERS[selectedProvider.value].defaultModel;

  try {
    messages.value = await api.kbGetMessages(conv.id);
    await nextTick();
    scrollToBottom();
  } catch (e) {
    console.error('加载消息失败:', e);
    messages.value = [];
  }
}

async function deleteConversation(conv: KbConversation) {
  try {
    await ElMessageBox.confirm('确定要删除这个对话吗？', '确认删除', {
      confirmButtonText: '删除',
      cancelButtonText: '取消',
      type: 'warning',
    });

    await api.kbDeleteConversation(conv.id);
    await loadConversations();

    if (currentConversation.value?.id === conv.id) {
      currentConversation.value = null;
      messages.value = [];
    }
  } catch (e) {
    if (e !== 'cancel') {
      console.error('删除对话失败:', e);
    }
  }
}

// ==================== AI 对话 ====================

async function sendMessage() {
  const content = inputMessage.value.trim();
  if (!content || isGenerating.value) return;

  // 检查 API Key
  const hasKey = await checkApiKeyConfigured(selectedProvider.value);
  if (!hasKey) {
    ElMessage.warning(`请先在设置中配置 ${AI_PROVIDERS[selectedProvider.value].name} API Key`);
    return;
  }

  // 如果没有对话，创建新对话
  if (!currentConversation.value) {
    await createNewConversation();
  }

  const convId = currentConversation.value!.id;

  // 保存用户消息
  await api.kbAddMessage(convId, 'user', content);

  // 添加到本地显示
  messages.value.push({
    id: Date.now(),
    conversation_id: convId,
    role: 'user',
    content,
    sources: null,
    created_at: new Date().toISOString(),
  });

  inputMessage.value = '';
  await nextTick();
  scrollToBottom();

  // 搜索相关文档
  let searchResults: KbSearchResult[] = [];
  try {
    searchResults = await api.kbSearch(content, 5);
  } catch (e) {
    console.error('搜索失败:', e);
  }

  // 构建消息
  const systemPrompt = buildRAGSystemPrompt(searchResults);
  const chatMessages: ChatMessage[] = [
    { role: 'system', content: systemPrompt },
    ...messages.value.map(m => ({
      role: m.role as 'user' | 'assistant',
      content: m.content,
    })),
  ];

  // 开始生成
  isGenerating.value = true;
  streamingContent.value = '';
  abortController.value = new AbortController();

  try {
    // 使用流式响应
    for await (const chunk of chatStream(chatMessages, {
      provider: selectedProvider.value,
      model: selectedModel.value,
      signal: abortController.value.signal,
    })) {
      if (chunk.done) break;
      streamingContent.value += chunk.content;
      await nextTick();
      scrollToBottom();
    }

    // 解析来源引用
    const sources = parseSourceReferences(streamingContent.value, searchResults);

    // 保存助手消息
    await api.kbAddMessage(
      convId,
      'assistant',
      streamingContent.value,
      sources.length > 0 ? JSON.stringify(sources) : undefined
    );

    // 更新对话标题（如果是第一条消息）
    if (messages.value.length === 1) {
      const title = content.substring(0, 30) + (content.length > 30 ? '...' : '');
      await api.kbUpdateConversationTitle(convId, title);
      await loadConversations();
    }

    // 添加到本地显示
    messages.value.push({
      id: Date.now() + 1,
      conversation_id: convId,
      role: 'assistant',
      content: streamingContent.value,
      sources: sources.length > 0 ? JSON.stringify(sources) : null,
      created_at: new Date().toISOString(),
    });

  } catch (e: any) {
    if (e.name === 'AbortError') {
      ElMessage.info('已停止生成');
    } else {
      console.error('生成失败:', e);
      ElMessage.error(`生成失败: ${e.message}`);
    }
  } finally {
    isGenerating.value = false;
    streamingContent.value = '';
    abortController.value = null;
  }
}

function stopGeneration() {
  if (abortController.value) {
    abortController.value.abort();
  }
}

function scrollToBottom() {
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
  }
}

function handleKeyDown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    sendMessage();
  }
}

// 当切换 provider 时，更新默认 model
watch(selectedProvider, (newProvider) => {
  selectedModel.value = AI_PROVIDERS[newProvider].defaultModel;
});

// ==================== 生命周期 ====================

onMounted(async () => {
  await Promise.all([loadDocuments(), loadConversations()]);
});
</script>

<template>
  <div class="knowledge-base-container">
    <!-- 左侧边栏 -->
    <div class="sidebar">
      <!-- 切换标签 -->
      <div class="sidebar-tabs">
        <el-button-group class="section-toggle">
          <el-button
            :type="activeSection === 'chat' ? 'primary' : 'default'"
            size="small"
            @click="activeSection = 'chat'"
          >
            <el-icon><ChatDotRound /></el-icon>
            AI 问答
          </el-button>
          <el-button
            :type="activeSection === 'documents' ? 'primary' : 'default'"
            size="small"
            @click="activeSection = 'documents'"
          >
            <el-icon><FolderOpened /></el-icon>
            知识库
          </el-button>
        </el-button-group>
      </div>

      <!-- 对话列表 -->
      <div v-if="activeSection === 'chat'" class="conversation-list">
        <div class="list-header">
          <span>对话历史</span>
          <el-button type="primary" size="small" @click="createNewConversation">
            <el-icon><Plus /></el-icon>
            新对话
          </el-button>
        </div>
        <div class="list-content">
          <div
            v-for="conv in conversations"
            :key="conv.id"
            class="conversation-item"
            :class="{ active: currentConversation?.id === conv.id }"
            @click="selectConversation(conv)"
          >
            <div class="conv-title">{{ conv.title || '新对话' }}</div>
            <div class="conv-meta">
              <span class="conv-provider">{{ AI_PROVIDERS[conv.ai_provider as AIProvider]?.name || conv.ai_provider }}</span>
              <el-button
                type="danger"
                text
                size="small"
                class="delete-btn"
                @click.stop="deleteConversation(conv)"
              >
                <el-icon><Delete /></el-icon>
              </el-button>
            </div>
          </div>
          <div v-if="conversations.length === 0" class="empty-state">
            暂无对话历史
          </div>
        </div>
      </div>

      <!-- 文档列表 -->
      <div v-else class="document-list">
        <div class="list-header">
          <span>文档列表</span>
          <el-button type="primary" size="small" @click="handleUploadDocument">
            <el-icon><Upload /></el-icon>
            上传
          </el-button>
        </div>
        <div class="list-content" v-loading="loadingDocuments">
          <div
            v-for="doc in documents"
            :key="doc.id"
            class="document-item"
          >
            <div class="doc-info">
              <el-icon class="doc-icon">
                <component :is="getFileTypeIcon(doc.file_type)" />
              </el-icon>
              <div class="doc-details">
                <div class="doc-title">{{ doc.title }}</div>
                <div class="doc-meta">
                  <el-tag
                    :type="getStatusTag(doc.status).type"
                    size="small"
                  >
                    {{ getStatusTag(doc.status).label }}
                  </el-tag>
                  <span v-if="doc.status === 'completed'">
                    {{ doc.chunk_count }} 个分块
                  </span>
                  <span v-if="processingDocuments.has(doc.id)" class="processing-hint">
                    处理中...
                  </span>
                </div>
              </div>
            </div>
            <el-button
              type="danger"
              text
              size="small"
              @click="handleDeleteDocument(doc)"
            >
              <el-icon><Delete /></el-icon>
            </el-button>
          </div>
          <div v-if="documents.length === 0" class="empty-state">
            <p>暂无文档</p>
            <p class="empty-hint">上传 PDF、Word、Excel、Markdown 或文本文件</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 右侧主内容区 -->
    <div class="main-content">
      <!-- AI 设置栏 -->
      <div class="ai-settings">
        <div class="setting-item">
          <span class="label">AI 服务：</span>
          <el-select v-model="selectedProvider" size="small" style="width: 120px">
            <el-option
              v-for="(config, key) in AI_PROVIDERS"
              :key="key"
              :label="config.name"
              :value="key"
            />
          </el-select>
        </div>
        <div class="setting-item">
          <span class="label">模型：</span>
          <el-select v-model="selectedModel" size="small" style="width: 180px">
            <el-option
              v-for="model in availableModels"
              :key="model"
              :label="model"
              :value="model"
            />
          </el-select>
        </div>
        <div class="doc-count">
          <el-icon><FolderOpened /></el-icon>
          {{ documents.filter(d => d.status === 'completed').length }} 个文档已就绪
        </div>
      </div>

      <!-- 消息区域 -->
      <div class="messages-area" ref="messagesContainer">
        <div v-if="displayMessages.length === 0" class="welcome-message">
          <div class="welcome-icon">
            <el-icon :size="48"><ChatDotRound /></el-icon>
          </div>
          <h3>企业知识库助手</h3>
          <p>基于您上传的文档回答问题，支持 PDF、Word、Excel、Markdown 等格式</p>
          <div class="quick-tips">
            <div class="tip">
              <el-icon><Document /></el-icon>
              上传文档到知识库
            </div>
            <div class="tip">
              <el-icon><Search /></el-icon>
              AI 自动检索相关内容
            </div>
            <div class="tip">
              <el-icon><ChatLineRound /></el-icon>
              获取带来源引用的回答
            </div>
          </div>
        </div>

        <div
          v-for="(msg, index) in displayMessages"
          :key="index"
          class="message"
          :class="[msg.role]"
        >
          <div class="message-content">
            <div class="message-text">{{ msg.content }}</div>
            <div
              v-if="'sources' in msg && msg.sources"
              class="message-sources"
            >
              <div class="sources-title">
                <el-icon><Link /></el-icon>
                参考来源
              </div>
              <div class="source-list">
                <div
                  v-for="(source, sIndex) in JSON.parse(msg.sources as string)"
                  :key="sIndex"
                  class="source-item"
                >
                  <span class="source-title">{{ source.document_title }}</span>
                  <span class="source-snippet">{{ source.snippet }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 输入区域 -->
      <div class="input-area">
        <el-input
          v-model="inputMessage"
          type="textarea"
          :rows="2"
          placeholder="输入问题，按 Enter 发送..."
          :disabled="isGenerating"
          @keydown="handleKeyDown"
        />
        <div class="input-actions">
          <el-button
            v-if="!isGenerating"
            type="primary"
            :disabled="!inputMessage.trim()"
            @click="sendMessage"
          >
            <el-icon><Promotion /></el-icon>
            发送
          </el-button>
          <el-button
            v-else
            type="danger"
            @click="stopGeneration"
          >
            <el-icon><VideoPause /></el-icon>
            停止
          </el-button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.knowledge-base-container {
  display: flex;
  height: 100%;
  background: var(--el-bg-color);
}

/* 侧边栏 */
.sidebar {
  width: 280px;
  border-right: 1px solid var(--el-border-color);
  display: flex;
  flex-direction: column;
  background: var(--el-bg-color-page);
}

.sidebar-tabs {
  padding: 12px;
  border-bottom: 1px solid var(--el-border-color);
}

.section-toggle {
  width: 100%;
}

.section-toggle .el-button {
  flex: 1;
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  border-bottom: 1px solid var(--el-border-color);
  font-weight: 500;
}

.list-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

/* 对话列表 */
.conversation-item {
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  margin-bottom: 4px;
  transition: background 0.2s;
}

.conversation-item:hover {
  background: var(--el-fill-color);
}

.conversation-item.active {
  background: var(--el-color-primary-light-9);
}

.conv-title {
  font-size: 14px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.conv-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 4px;
}

.conv-provider {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.delete-btn {
  opacity: 0;
  transition: opacity 0.2s;
}

.conversation-item:hover .delete-btn {
  opacity: 1;
}

/* 文档列表 */
.document-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  border-radius: 6px;
  margin-bottom: 4px;
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-light);
}

.doc-info {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
  min-width: 0;
}

.doc-icon {
  font-size: 24px;
  color: var(--el-color-primary);
}

.doc-details {
  flex: 1;
  min-width: 0;
}

.doc-title {
  font-size: 14px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.doc-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.processing-hint {
  color: var(--el-color-warning);
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.empty-state {
  text-align: center;
  padding: 40px 20px;
  color: var(--el-text-color-secondary);
}

.empty-hint {
  font-size: 12px;
  margin-top: 8px;
}

/* 主内容区 */
.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

/* AI 设置栏 */
.ai-settings {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--el-border-color);
  background: var(--el-bg-color-page);
}

.setting-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.setting-item .label {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.doc-count {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

/* 消息区域 */
.messages-area {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.welcome-message {
  text-align: center;
  padding: 60px 20px;
  color: var(--el-text-color-secondary);
}

.welcome-icon {
  margin-bottom: 16px;
  color: var(--el-color-primary);
}

.welcome-message h3 {
  margin: 0 0 8px;
  font-size: 18px;
  color: var(--el-text-color-primary);
}

.welcome-message p {
  margin: 0;
  font-size: 14px;
}

.quick-tips {
  display: flex;
  justify-content: center;
  gap: 24px;
  margin-top: 32px;
}

.tip {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
}

/* 消息样式 */
.message {
  margin-bottom: 16px;
}

.message.user {
  display: flex;
  justify-content: flex-end;
}

.message.user .message-content {
  background: var(--el-color-primary);
  color: white;
  border-radius: 12px 12px 4px 12px;
}

.message.assistant .message-content {
  background: var(--el-fill-color);
  border-radius: 12px 12px 12px 4px;
}

.message-content {
  max-width: 80%;
  padding: 12px 16px;
}

.message-text {
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.6;
}

.message-sources {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--el-border-color);
}

.sources-title {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 8px;
}

.source-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.source-item {
  font-size: 12px;
  padding: 6px 10px;
  background: var(--el-bg-color);
  border-radius: 4px;
}

.source-title {
  font-weight: 500;
  color: var(--el-color-primary);
}

.source-snippet {
  display: block;
  margin-top: 4px;
  color: var(--el-text-color-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 输入区域 */
.input-area {
  padding: 16px;
  border-top: 1px solid var(--el-border-color);
  background: var(--el-bg-color-page);
}

.input-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 10px;
}
</style>

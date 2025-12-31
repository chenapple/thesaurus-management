# 知识库功能实现笔记

## 进度跟踪

### Phase 1: 基础架构 [已完成]
- [x] 分析现有 db.rs 结构
- [x] 添加知识库数据库表 (db.rs:3175-3650)
- [x] 添加文档解析依赖到 Cargo.toml
- [x] 创建知识库 Tauri Commands (lib.rs:699-846)

### Phase 2: 文档处理 [已完成]
- [x] PDF 解析器 (pdf-extract)
- [x] Word 解析器 (docx-rs)
- [x] Excel 解析器 (calamine)
- [x] Markdown 解析器 (pulldown-cmark)
- [x] 纯文本解析器
- [x] 文本分块器（递归分割算法，支持重叠）
- [x] 文档处理模块 (knowledge_base.rs)

### Phase 3: 搜索与 RAG [已完成]
- [x] FTS5 全文搜索 (db.rs kb_search)
- [x] RAG Prompt 构建 (ai-service.ts)
- [x] 来源引用处理 (ai-service.ts)

### Phase 4: 多 AI 服务 [已完成]
- [x] 创建 AI 服务抽象层 (ai-service.ts)
- [x] DeepSeek 支持
- [x] OpenAI 支持
- [x] Claude 支持
- [x] 流式响应支持

### Phase 5: 前端界面 [已完成]
- [x] 知识库管理界面 (KnowledgeBaseTab.vue)
- [x] AI 问答界面 (KnowledgeBaseTab.vue)
- [x] 添加知识库视图模式到 App.vue

---

## 已完成的关键文件

| 文件 | 修改内容 |
|------|---------|
| `src-tauri/src/db.rs` | 添加知识库数据库表和 CRUD 函数 |
| `src-tauri/src/lib.rs` | 添加 18 个知识库 Tauri Commands |
| `src-tauri/src/knowledge_base.rs` | 新增文档解析和分块模块 |
| `src-tauri/Cargo.toml` | 添加文档解析依赖 |
| `src/types.ts` | 添加知识库相关类型定义 |
| `src/api.ts` | 添加知识库 API 调用函数 |
| `src/ai-service.ts` | 新增多 AI 服务抽象层 |
| `src/components/KnowledgeBaseTab.vue` | 新增知识库前端界面 |
| `src/App.vue` | 添加知识库视图模式 |

---

## 已实现的 Tauri Commands

```typescript
// 分类管理
kb_create_category(name, parent_id?)
kb_get_categories()
kb_delete_category(id)

// 文档管理
kb_add_document(category_id?, title, file_name, file_path, file_type, file_size?)
kb_update_document_status(id, status, chunk_count)
kb_get_documents(category_id?)
kb_delete_document(id)
kb_process_document(document_id, file_path)  // 解析+分块

// 分块管理
kb_add_chunk(document_id, chunk_index, content, page_number?)
kb_add_chunks_batch(document_id, chunks)
kb_get_chunks(document_id)

// 搜索
kb_search(query, limit)

// AI 对话
kb_create_conversation(ai_provider, ai_model?, title?)
kb_get_conversations()
kb_update_conversation_title(id, title)
kb_delete_conversation(id)
kb_add_message(conversation_id, role, content, sources?)
kb_get_messages(conversation_id)
```

---

## 数据库表

```sql
-- 已创建的表
kb_categories       -- 文档分类
kb_documents        -- 文档信息
kb_chunks           -- 文档分块
kb_chunks_fts       -- FTS5 全文搜索索引
kb_conversations    -- AI 对话
kb_messages         -- AI 消息
```

---

## 支持的 AI 服务

| 服务 | 模型 | 状态 |
|------|------|------|
| DeepSeek | deepseek-chat, deepseek-reasoner | 已支持 |
| OpenAI | gpt-4o, gpt-4o-mini, gpt-4-turbo, gpt-3.5-turbo | 已支持 |
| Claude | claude-3-5-sonnet-20241022, claude-3-5-haiku-20241022 | 已支持 |
| Gemini | gemini-3-flash-preview, gemini-2.5-flash, gemini-2.5-pro, gemini-2.0-flash | 已支持 |

---

## 功能特性

1. **文档管理**
   - 支持 PDF, Word (docx), Excel (xlsx/xls), Markdown, 纯文本
   - 自动解析和分块
   - 递归分割算法，支持重叠

2. **RAG 问答**
   - FTS5 全文搜索
   - 智能构建上下文
   - 来源引用追踪

3. **多 AI 服务**
   - 统一抽象接口
   - 流式响应
   - 可切换服务和模型

4. **用户界面**
   - 文档列表管理
   - 对话历史
   - 实时流式回复
   - 来源引用展示

---

## 使用方式

1. 在应用顶部点击「知识库」按钮切换到知识库视图
2. 点击「知识库」标签页上传文档
3. 文档会自动解析和分块
4. 在「AI 问答」标签页开始对话
5. AI 会自动检索相关文档内容并回答问题

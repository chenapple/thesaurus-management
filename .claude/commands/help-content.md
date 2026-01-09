---
description: 更新 App.vue 中指定模块的帮助页面内容
argument-hint: <模块名 如 排名监控/智能广告/关键词>
allowed-tools: Read, Edit, Grep
---

为「$ARGUMENTS」模块更新帮助页面内容。

## 步骤

### 1. 了解模块功能
- 读取相关组件代码，理解模块的所有功能
- 如果是 AI 相关功能，读取对应的 prompts 文件了解分析逻辑

### 2. 找到帮助内容位置
帮助内容在 `src/App.vue` 的 `<el-dialog>` 帮助弹窗中，搜索 `el-tab-pane label="$ARGUMENTS"`

### 3. 更新内容结构
按以下结构编写帮助内容：

```vue
<el-tab-pane label="模块名" name="xxx">
  <div class="help-content">
    <h4>功能说明</h4>
    <p>简要描述模块的核心功能和价值</p>
    <ul>
      <li><strong>功能1：</strong>说明</li>
      <li><strong>功能2：</strong>说明</li>
    </ul>

    <h4>核心功能详解</h4>
    <p style="color: var(--el-text-color-secondary); margin-bottom: 12px;">点击展开查看详细说明：</p>

    <el-collapse class="agent-prompts-collapse">
      <el-collapse-item name="feature1">
        <template #title>
          <span class="agent-title">功能标题</span>
          <span class="agent-subtitle">副标题</span>
        </template>
        <div class="prompt-section">
          <h5>小标题</h5>
          <div class="prompt-block">
            <ul>
              <li>详细内容</li>
            </ul>
          </div>
        </div>
      </el-collapse-item>
    </el-collapse>

    <h4>使用流程</h4>
    <ol>
      <li>步骤1</li>
      <li>步骤2</li>
    </ol>

    <h4>注意事项</h4>
    <ul>
      <li>注意点1</li>
      <li>注意点2</li>
    </ul>
  </div>
</el-tab-pane>
```

### 4. 验证
运行 `npm run build` 确保无编译错误

# 词库管理 App 优化清单

> 记录与专业应用的差距，逐步优化

---

## 1. 代码架构

- [ ] **App.vue 拆分** - ~~3700+~~ 3055 行，已部分拆分
  - [ ] 拆分产品管理组件
  - [ ] 拆分关键词视图组件
  - [ ] 拆分词根视图组件
  - [x] 拆分词云组件容器 (WordCloud.vue)
  - [x] 拆分各种弹窗为独立组件
    - [x] TrafficSettingsDialog - 流量设置
    - [x] KeywordExportDialog - 关键词导出
    - [x] BackupDialog - 备份管理
    - [x] ColumnConfigDialog - 列配置
    - [x] ProductDialog - 产品编辑
    - [x] ShortcutsDialog - 快捷键帮助

- [x] **引入状态管理** - 使用 Pinia 管理全局状态
  - [x] 产品状态 (stores/product.ts)
  - [x] 关键词数据状态 (stores/keyword.ts)
  - [x] 用户偏好状态 (stores/settings.ts)

- [x] **添加单元测试** - 提高代码质量和重构信心
  - [x] 前端 Store 测试 (Vitest) - 41 个测试用例
  - [x] 工具函数测试 - 40 个测试用例
  - [ ] Rust 后端测试 (可选)

---

## 2. UI/UX 设计

- [x] **空状态设计** - 提供引导性的空状态
  - [x] 无产品时的引导（侧边栏 + 主区域）
  - [x] 关键词表格空状态（无数据/筛选无结果）
  - [x] 词根表格空状态（无数据/搜索无结果）

- [x] **快捷键支持** - 提高操作效率
  - [x] Ctrl/Cmd + N 新建产品
  - [x] Ctrl/Cmd + D 切换深色模式
  - [x] Ctrl/Cmd + I 导入 Excel
  - [x] Ctrl/Cmd + E 导出 Excel
  - [x] Ctrl/Cmd + F 搜索
  - [x] Ctrl/Cmd + Enter AI分析
  - [x] ? 显示快捷键帮助

---

## 3. 性能优化

- [x] **虚拟滚动** - 大数据量时保持流畅
  - [x] 关键词表格 - 已有分页机制处理大数据
  - [x] 词根表格 - 已有分页机制处理大数据
  - 注：Element Plus el-table 不原生支持虚拟滚动，当前分页方案已足够

- [x] **懒加载** - 按需加载
  - [x] 组件懒加载 (defineAsyncComponent)
  - [x] 数据分页优化 (后端分页)

- [x] **缓存策略**
  - [x] 数据缓存 (stores/cache.ts)
  - [x] 减少重复请求 (5分钟TTL缓存)

---

## 4. 安全性

- [x] **API Key 安全存储**
  - [x] 使用系统密钥链存储 (keyring crate)
  - [x] 前端 API Key 设置 UI (ApiKeyDialog.vue)
  - [x] 移除硬编码 API Key (deepseek.ts)

- [~] **数据库加密** (已跳过 - 词库数据不敏感)
  - [ ] SQLite 加密 (SQLCipher)
  - [ ] 敏感数据保护

---

## 5. 用户体验细节

- [x] **偏好设置持久化**
  - [x] 列配置保存 (localStorage)
  - [x] 主题设置保存 (localStorage)
  - [x] 窗口大小/位置记忆 (tauri-plugin-window-state)

- [~] **多语言支持 (i18n)** (已跳过)

- [~] **导入模板** (已跳过)

- [~] **帮助文档** (已跳过)

- [~] **更新日志 (Changelog)** (已跳过)

---

## 6. 运维相关

- [~] **错误监控** (已跳过 - 内部工具不需要)

- [~] **使用统计** (已跳过 - 内部工具不需要)

- [x] **自动更新优化**
  - [x] 更新提示弹窗 (已有)
  - [x] 下载进度条显示
  - [x] 用户可选择更新或稍后

---

## 优化进度

| 类别 | 完成 | 总计 | 进度 |
|------|------|------|------|
| 代码架构 | 2.5 | 3 | 83% |
| UI/UX | 2 | 2 | 100% |
| 性能优化 | 3 | 3 | 100% |
| 安全性 | 1 | 2 | 50% |
| 用户体验 | 1.5 | 5 | 30% |
| 运维 | 1 | 3 | 33% |
| **总计** | **11** | **18** | **61%** |

---

## 优化记录

### 2024-12-22 (自动更新优化)
- ✅ 添加下载进度条弹窗
  - 显示版本号、下载百分比进度条
  - 下载期间禁止关闭弹窗
  - 下载完成提示重启

### 2024-12-22 (窗口状态记忆)
- ✅ 窗口大小/位置记忆
  - 使用 tauri-plugin-window-state 插件
  - 自动保存和恢复窗口状态

### 2024-12-22 (API Key 安全存储)
- ✅ API Key 安全存储实现
  - Rust 后端: keyring crate 集成系统密钥链 (macOS Keychain / Windows Credential Manager)
  - Tauri 命令: set_api_key, get_api_key, delete_api_key, has_api_key
  - 前端 API: src/api.ts 添加密钥管理函数
  - UI 组件: ApiKeyDialog.vue 设置界面
  - 移除硬编码: deepseek.ts 改用动态获取 API Key

### 2024-12-22 (续)
- ✅ 性能优化完成
  - 组件懒加载 (defineAsyncComponent) - WordCloud, 各种Dialog组件
  - 数据缓存 Store (stores/cache.ts) - 5分钟TTL, LRU淘汰策略
  - 虚拟滚动评估 - 当前分页方案已满足需求

### 2024-12-22
- ✅ App.vue 弹窗拆分为独立组件 (6个对话框组件)
  - TrafficSettingsDialog, KeywordExportDialog, BackupDialog
  - ColumnConfigDialog, ProductDialog, ShortcutsDialog
- ✅ App.vue 行数从 3700+ 减少到 3055 行
- ✅ 快捷键支持已实现 (Cmd+N/D/I/E/F/Enter, ?)
- ✅ 偏好设置持久化 (列配置、主题)
- ✅ 引入 Pinia 状态管理
  - 创建 stores/product.ts - 产品相关状态和方法
  - 创建 stores/keyword.ts - 关键词和词根数据状态
  - 创建 stores/settings.ts - 用户偏好设置（主题、列配置）
  - 修改 main.ts 注册 Pinia
- ✅ 添加单元测试 (Vitest)
  - 安装 vitest, @vue/test-utils, happy-dom
  - 创建 vitest.config.ts 配置文件
  - 编写 stores 测试: settings.test.ts, product.test.ts (41 个测试)
  - 创建 utils/index.ts 工具函数库
  - 编写工具函数测试: utils/index.test.ts (40 个测试)
  - 总计 81 个测试用例，全部通过
- ✅ 添加空状态设计
  - 侧边栏产品列表空状态（图标 + 引导文案 + 按钮）
  - 主区域无产品提示
  - 关键词表格空状态（区分无数据/筛选无结果）
  - 词根表格空状态（区分无数据/搜索无结果）

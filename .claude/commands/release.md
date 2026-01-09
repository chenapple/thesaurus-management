---
description: 自动化版本发布流程
argument-hint: <版本号 如 0.8.1>
allowed-tools: Bash, Read, Edit
---

执行版本发布流程，目标版本：$ARGUMENTS

## 步骤

### 1. 验证版本号格式
确保版本号符合语义化版本格式（如 0.8.1, 1.0.0）

### 2. 更新版本号
修改以下文件中的版本号为 $ARGUMENTS：
- `package.json` 中的 `"version": "x.x.x"`
- `src-tauri/tauri.conf.json` 中的 `"version": "x.x.x"`
- `src-tauri/Cargo.toml` 中的 `version = "x.x.x"`

### 3. 构建验证
运行 `npm run build` 确保编译通过，如有错误需先修复

### 4. Git 提交
- 暂存所有改动：`git add -A`
- 提交，信息格式：`chore: bump version to $ARGUMENTS`
- 创建 tag：`git tag -a v$ARGUMENTS -m "v$ARGUMENTS"`

### 5. 推送发布
- 推送代码：`git push origin main`
- 推送 tag：`git push origin v$ARGUMENTS`

### 6. 确认
显示 GitHub Actions 构建状态链接，确认 CI/CD 已触发

---
description: 智能生成 commit 信息并提交代码
allowed-tools: Bash(git:*)
---

请执行以下步骤：

1. 运行 `git status` 和 `git diff --staged` 查看当前改动
2. 如果没有暂存的改动，先运行 `git add -A` 暂存所有改动
3. 分析改动内容，生成符合以下规范的 commit 信息：
   - `feat`: 新功能
   - `fix`: Bug 修复
   - `docs`: 文档更新
   - `style`: 代码格式（不影响功能）
   - `refactor`: 重构（不是新功能也不是修复）
   - `perf`: 性能优化
   - `test`: 测试相关
   - `chore`: 构建/工具变动

4. Commit 信息格式：
   ```
   <type>: <简短描述>

   <详细说明（可选）>

   Co-Authored-By: Claude <noreply@anthropic.com>
   ```

5. 执行 git commit，使用 HEREDOC 格式传递多行信息
6. 显示 commit 结果

# 优化事件记录功能设计

## 功能概述

记录每次对产品/广告的优化操作，在排名趋势图上标注事件点，方便分析"优化 → 排名变化"的因果关系。

## 数据模型

```sql
-- 优化事件表
CREATE TABLE optimization_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_id INTEGER NOT NULL,          -- 关联产品
    event_date TEXT NOT NULL,             -- 事件日期 (YYYY-MM-DD)
    event_type TEXT NOT NULL,             -- 类型: listing/ad_keyword/ad_bid/ad_budget/other
    title TEXT NOT NULL,                  -- 简短标题 (如: "优化标题关键词")
    description TEXT,                     -- 详细描述
    affected_keywords TEXT,               -- 受影响的关键词 (JSON数组，可选)
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

## 事件类型

| 类型 | 标签 | 图标颜色 |
|------|------|----------|
| `listing` | 文案优化 | 绿色 |
| `ad_keyword` | 广告关键词 | 蓝色 |
| `ad_bid` | 竞价调整 | 橙色 |
| `ad_budget` | 预算调整 | 紫色 |
| `other` | 其他 | 灰色 |

## UI 设计

### 1. 添加事件入口
- 在监控页面工具栏添加「+ 记录事件」按钮
- 或在趋势图上右键点击某天 → 「在此添加事件」

### 2. 事件表单
```
┌─────────────────────────────────────┐
│  记录优化事件                    ✕  │
├─────────────────────────────────────┤
│  日期:     [2024-12-24      📅]     │
│  类型:     [文案优化         ▼]     │
│  标题:     [优化了产品标题关键词  ]  │
│  详细描述: [                      ]  │
│            [添加了 ponceuse 等词  ]  │
│  关联关键词: [可选，多选监控中的词] │
├─────────────────────────────────────┤
│              [取消]  [保存]         │
└─────────────────────────────────────┘
```

### 3. 趋势图标注
- 在 RankingHistoryChart 的 X 轴对应日期位置显示垂直虚线
- 鼠标悬停显示事件详情 tooltip
- 不同类型用不同颜色的标记点

### 4. 事件列表视图
- 在监控页面底部或单独 Tab 显示事件时间线
- 每个事件显示：日期、类型标签、标题、排名变化摘要

## 排名变化分析

事件卡片自动显示：
```
📅 2024-12-24 文案优化
   "优化了产品标题关键词"

   📊 事件后7天排名变化:
   • ponceuse pour ongles: 60名 → 45名 (↑15)
   • ponceuse a ongle: 128名 → 98名 (↑30)
   • 平均提升: 22.5 名
```

## 实现步骤

1. **后端 (Rust)**
   - 在 db.rs 添加 optimization_events 表的 CRUD 函数
   - 在 lib.rs 添加 Tauri 命令

2. **前端 (Vue)**
   - 在 types.ts 添加 OptimizationEvent 接口
   - 在 api.ts 添加 API 调用函数
   - 创建 AddEventDialog.vue 组件
   - 修改 RankingHistoryChart.vue 添加事件标注
   - 在 KeywordMonitoringTab.vue 添加事件列表

## 相关文件

- `src-tauri/src/db.rs` - 数据库操作
- `src-tauri/src/lib.rs` - Tauri 命令
- `src/types.ts` - TypeScript 类型
- `src/api.ts` - API 函数
- `src/components/RankingHistoryChart.vue` - 趋势图组件
- `src/components/KeywordMonitoringTab.vue` - 监控页面

---

*Created: 2024-12-24*

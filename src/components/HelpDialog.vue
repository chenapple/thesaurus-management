<script setup lang="ts">
import { ref, computed } from "vue";
import { Search, DataAnalysis, Collection, TrendCharts, EditPen, Promotion, FolderOpened, Calendar } from "@element-plus/icons-vue";

defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
}>();

const activeHelpTab = ref('dashboard');
const helpSearchQuery = ref('');

const helpSections = [
  { id: 'dashboard', title: '数据概览', icon: 'DataAnalysis', keywords: ['首页', '数据', '统计', '关键词', '监控', '排名变化', '待办'] },
  { id: 'keywords', title: '词库管理', icon: 'Collection', keywords: ['关键词', '导入', '分类', '词根', '流量', '搜索量', 'AI', '西柚', '卖家精灵', '否词', '批量', '标记'] },
  { id: 'monitoring', title: '排名监控', icon: 'TrendCharts', keywords: ['排名', '监控', '追踪', '变化', '调度', '定时', '通知'] },
  { id: 'smartcopy', title: '智能文案', icon: 'EditPen', keywords: ['文案', '标题', '五点', 'listing', 'AI', '竞品', '分析', '新品', '老品'] },
  { id: 'ads', title: '智能广告', icon: 'Promotion', keywords: ['广告', 'ACOS', 'CPC', '否词', '优化', '预算', '投放', '趋势图', '四象限', '导入', '追加', '替换', '时间筛选', '国家'] },
  { id: 'knowledge', title: '知识库', icon: 'FolderOpened', keywords: ['知识', '文档', 'AI问答', '向量', '搜索', 'RAG', '分类', '双向链接', '知识图谱', 'Callout', '大纲', '保存笔记', 'Obsidian'] },
  { id: 'weekly_report', title: '工作周报', icon: 'Calendar', keywords: ['周报', '工作', '总结', '计划', '任务', '常规', '导出', 'AI', 'Excel', 'Markdown'] },
];

const filteredHelpSections = computed(() => {
  const query = helpSearchQuery.value.toLowerCase().trim();
  if (!query) return helpSections;
  return helpSections.filter(section =>
    section.title.toLowerCase().includes(query) ||
    section.keywords.some(k => k.toLowerCase().includes(query))
  );
});

const isSearchingHelp = computed(() => helpSearchQuery.value.trim().length > 0);

function handleClose() {
  emit('update:visible', false);
}
</script>

<template>
  <el-dialog
    :model-value="visible"
    @update:model-value="handleClose"
    title="帮助中心"
    width="950px"
    class="help-dialog"
  >
    <div class="help-layout">
      <!-- Left navigation -->
      <div class="help-nav">
        <el-input
          v-model="helpSearchQuery"
          placeholder="搜索帮助..."
          clearable
          class="help-search"
        >
          <template #prefix>
            <el-icon><Search /></el-icon>
          </template>
        </el-input>
        <el-menu
          :default-active="activeHelpTab"
          @select="(key: string) => activeHelpTab = key"
          class="help-menu"
        >
          <el-menu-item
            v-for="section in filteredHelpSections"
            :key="section.id"
            :index="section.id"
          >
            <el-icon>
              <DataAnalysis v-if="section.icon === 'DataAnalysis'" />
              <Collection v-else-if="section.icon === 'Collection'" />
              <TrendCharts v-else-if="section.icon === 'TrendCharts'" />
              <EditPen v-else-if="section.icon === 'EditPen'" />
              <Promotion v-else-if="section.icon === 'Promotion'" />
              <FolderOpened v-else-if="section.icon === 'FolderOpened'" />
              <Calendar v-else-if="section.icon === 'Calendar'" />
            </el-icon>
            <span>{{ section.title }}</span>
          </el-menu-item>
          <!-- No search results -->
          <div v-if="filteredHelpSections.length === 0 && isSearchingHelp" class="help-no-result">
            <el-icon><Search /></el-icon>
            <span>未找到相关帮助</span>
          </div>
        </el-menu>
        <div class="help-shortcut-hint">
          <kbd>⌘</kbd> + <kbd>H</kbd> 快速打开
        </div>
      </div>

      <!-- Right content area -->
      <div class="help-content-area">
        <!-- Dashboard help -->
        <div v-show="activeHelpTab === 'dashboard'" class="help-content">
          <h4>功能说明</h4>
          <p>首页是数据总览面板，展示所有产品的关键指标汇总，并提供智慧大屏和备忘录等辅助工具。</p>
          <ul>
            <li><strong>关键词统计：</strong>显示各产品的关键词总数、已分类数量等</li>
            <li><strong>监控概览：</strong>显示正在监控的关键词数量和排名变化趋势</li>
            <li><strong>排名变化榜：</strong>展示排名上升/下降最多的关键词</li>
            <li><strong>待办提醒：</strong>提示需要关注的事项，如未分类关键词等</li>
            <li><strong>市场时钟：</strong>显示目标市场当地时间</li>
            <li><strong>电商日历：</strong>点击日历按钮查看全年电商节日和促销活动</li>
            <li><strong>节日提醒：</strong>距离节日 14 天内时自动弹出提醒</li>
          </ul>

          <h4>实时汇率</h4>
          <p>顶部状态栏显示主要货币对人民币的实时汇率，帮助快速换算成本和利润。</p>
          <ul>
            <li><strong>支持币种：</strong>美元、欧元、英镑、日元、加元、澳元、墨西哥比索、印度卢比、新加坡元、阿联酋迪拉姆、沙特里亚尔、波兰兹罗提、瑞典克朗、巴西雷亚尔、土耳其里拉</li>
            <li><strong>自动轮播：</strong>状态栏自动轮播显示各币种汇率</li>
            <li><strong>点击展开：</strong>点击汇率区域可展开查看所有币种的完整汇率列表</li>
            <li><strong>实时更新：</strong>汇率数据每小时自动更新，确保数据准确性</li>
            <li><strong>涨跌标识：</strong>显示汇率涨跌趋势，绿色表示上涨，红色表示下跌</li>
          </ul>

          <h4>使用建议</h4>
          <ul>
            <li>每天打开首页快速了解整体数据变化</li>
            <li>关注排名变化榜，及时发现异常波动</li>
            <li>留意节日提醒，提前为促销活动做准备</li>
            <li>使用电商日历规划全年营销节点</li>
            <li>关注汇率变化，在汇率有利时进行资金结算</li>
          </ul>
        </div>

        <!-- Keywords help -->
        <div v-show="activeHelpTab === 'keywords'" class="help-content">
          <h4>功能说明</h4>
          <p>管理和分析亚马逊关键词数据，支持导入第三方工具数据、AI智能分类、流量分级、否词标记等功能。</p>
          <ul>
            <li><strong>数据导入：</strong>支持导入西柚找词、卖家精灵、H10等工具导出的关键词数据</li>
            <li><strong>多维度展示：</strong>关键词视图、词根视图、词云视图三种查看方式</li>
            <li><strong>智能分类：</strong>AI自动分析关键词并进行一级分类</li>
            <li><strong>流量分级：</strong>根据搜索量自动划分大词、中词、小词</li>
            <li><strong>否词标记：</strong>在词根视图中标记与产品不相关的词根，自动同步到关键词</li>
          </ul>

          <h4>否词功能</h4>
          <p>在词根视图中标记与产品不相关的词根为"否词"，系统会自动将包含该词根的所有关键词标记为否词。</p>
          <ul>
            <li><strong>单个标记：</strong>在词根列表的"否词"列点击开关，即可标记/取消标记</li>
            <li><strong>批量标记：</strong>勾选多个词根后，使用顶部工具栏的"批量标记否词"或"批量取消否词"按钮</li>
            <li><strong>AI 智能识别：</strong>运行"智能分析"时，AI 会根据产品名称自动识别不相关词根并标记为否词</li>
            <li><strong>实时同步：</strong>标记否词后，关键词视图的否词列会实时更新显示</li>
          </ul>

          <h4>使用流程</h4>
          <ol>
            <li>创建产品，选择对应的亚马逊站点</li>
            <li>导入关键词数据</li>
            <li>设置流量分级阈值</li>
            <li>运行 AI 分析进行智能分类和否词识别</li>
            <li>切换到词根视图，手动调整否词标记</li>
            <li>使用筛选功能定位目标关键词</li>
            <li>将重点关键词添加到排名监控</li>
          </ol>
        </div>

        <!-- Monitoring help -->
        <div v-show="activeHelpTab === 'monitoring'" class="help-content">
          <h4>功能说明</h4>
          <p>实时监控关键词在亚马逊搜索结果中的自然排名和广告排名变化，并记录优化事件以追踪效果。</p>
          <ul>
            <li><strong>双维度监控：</strong>同时监控自然排名和广告位排名</li>
            <li><strong>多国家支持：</strong>支持美国、英国、德国等主要站点</li>
            <li><strong>多视图模式：</strong>列表视图、按产品分组、按关键词分组</li>
            <li><strong>趋势可视化：</strong>迷你图展示近7天排名趋势</li>
            <li><strong>标签管理：</strong>为监控项添加标签，支持按标签筛选</li>
          </ul>

          <h4>优化事件记录</h4>
          <p>记录 Listing 优化操作，便于追踪优化效果和前后对比分析。</p>
          <ul>
            <li><strong>事件类型：</strong>支持 Listing 优化、广告优化、价格调整、库存补货等多种类型</li>
            <li><strong>子类型细分：</strong>如 Listing 优化可细分为标题、五点、描述、图片等</li>
            <li><strong>事件范围：</strong>可关联到产品、特定 ASIN 或特定关键词</li>
            <li><strong>截图上传：</strong>保存优化前的截图，方便后续对比
              <ul>
                <li>支持点击上传、拖拽上传、粘贴上传（Ctrl+V / Cmd+V）</li>
                <li>每个事件最多 5 张图片，单张最大 5MB</li>
                <li>点击缩略图可放大查看，多张图片支持左右切换</li>
              </ul>
            </li>
            <li><strong>日历视图：</strong>切换到日历模式，按日期查看优化事件分布</li>
          </ul>

          <h4>使用流程</h4>
          <ol>
            <li>点击"添加监控"，输入关键词和 ASIN</li>
            <li>选择站点并设置优先级</li>
            <li>点击"检测全部"获取初始排名</li>
            <li>配置自动检测时间间隔</li>
            <li>点击趋势图查看历史排名曲线</li>
            <li>进行 Listing 优化前，先记录优化事件并上传截图</li>
            <li>优化后持续观察排名变化，对比优化效果</li>
          </ol>
        </div>

        <!-- Smart copy help -->
        <div v-show="activeHelpTab === 'smartcopy'" class="help-content">
          <h4>功能说明</h4>
          <p>基于AI分析竞品数据，生成符合亚马逊算法的优质 Listing 文案建议。</p>
          <ul>
            <li><strong>新品打造：</strong>从零开始创建 Listing</li>
            <li><strong>老品优化：</strong>基于现有文案生成优化建议</li>
            <li><strong>竞品分析：</strong>抓取竞品标题、五点、评论等信息</li>
          </ul>
          <h4>使用流程</h4>
          <ol>
            <li>创建项目，选择场景</li>
            <li>添加 3-5 个主要竞品 ASIN</li>
            <li>点击"批量获取"抓取竞品数据</li>
            <li>点击"开始分析"，AI 会逐步生成分析报告</li>
            <li>参考建议优化自己的 Listing</li>
          </ol>
        </div>

        <!-- Ads help -->
        <div v-show="activeHelpTab === 'ads'" class="help-content">
          <h4>功能说明</h4>
          <p>基于AI多智能体架构分析亚马逊广告搜索词报告，提供优化建议。</p>
          <ul>
            <li><strong>数据导入：</strong>支持导入亚马逊搜索词报告（Excel/CSV 格式）</li>
            <li><strong>多国家支持：</strong>自动识别国家并按国家分组分析</li>
            <li><strong>数据可视化：</strong>时间趋势图、效率四象限图、花费效率散点图等多维度图表</li>
            <li><strong>多智能体分析：</strong>4个AI专家并行分析，提供专业优化建议</li>
            <li><strong>智能采样：</strong>大数据量时自动采样，确保各类问题都能被发现</li>
          </ul>

          <h4>数据可视化</h4>
          <p>导入数据后，系统会自动生成多维度可视化图表：</p>
          <ul>
            <li><strong>时间趋势图：</strong>展示 ACOS、花费、订单、销售额随时间的变化趋势</li>
            <li><strong>效率四象限图：</strong>按转化率和 ACOS 将搜索词分为高潜力、待优化、稳定、淘汰四类</li>
            <li><strong>花费效率散点图：</strong>分析花费与 ROAS 的关系</li>
            <li><strong>综合仪表盘：</strong>ACOS 分布、花费占比、Top10 排行榜</li>
            <li><strong>匹配类型对比：</strong>对比不同匹配类型的表现</li>
          </ul>

          <h4>数据筛选与分析范围</h4>
          <p>支持按国家和时间两个维度筛选数据，筛选结果直接用于 AI 分析：</p>
          <ul>
            <li><strong>国家筛选：</strong>选择特定国家查看该国数据，或选择"全部国家"</li>
            <li><strong>时间筛选：</strong>快速选择近7天、近15天、近30天，或自定义时间范围</li>
            <li><strong>分析范围信息：</strong>开始分析前会显示当前筛选条件（时间、国家、数据量）</li>
            <li><strong>筛选器联动：</strong>AI 分析使用的数据与图表筛选结果一致</li>
          </ul>

          <h4>智能采样</h4>
          <p>当单个国家的数据量超过 200 条时，系统会自动进行智能采样：</p>
          <ul>
            <li><strong>高花费词（40%）：</strong>按花费降序选取，确保大额花费词被分析</li>
            <li><strong>高 ACOS 词（20%）：</strong>ACOS 超过目标值的词，发现效率问题</li>
            <li><strong>低转化词（20%）：</strong>点击多但无转化的词，识别浪费</li>
            <li><strong>高潜力词（20%）：</strong>ACOS 低于目标且有转化，发现机会</li>
          </ul>
          <p>这种采样策略确保各类问题都能被 AI 发现，而不只是分析高花费词。</p>

          <h4>数据导入模式</h4>
          <p>导入搜索词报告时可选择两种模式：</p>
          <ul>
            <li><strong>替换全部：</strong>清空现有数据，导入新数据（适用于每次导入完整报告）</li>
            <li><strong>追加合并：</strong>保留现有数据，智能合并新数据（适用于积累历史数据）
              <ul>
                <li>相同搜索词+日期+广告组+国家的记录会更新</li>
                <li>新记录会追加到数据库</li>
              </ul>
            </li>
          </ul>

          <h4>使用流程</h4>
          <ol>
            <li>创建广告项目，设置目标 ACOS</li>
            <li>从亚马逊后台下载搜索词报告（Excel 格式）</li>
            <li>点击"导入数据"，选择导入模式（替换/追加）</li>
            <li>查看数据可视化图表，了解整体表现</li>
            <li>使用国家和时间筛选器聚焦特定范围</li>
            <li>查看"分析范围"确认要分析的数据</li>
            <li>选择 AI 服务商和模型，点击"开始分析"</li>
            <li>查看 AI 生成的优化建议（否定词、竞价调整、新词机会）</li>
          </ol>

          <h4>使用建议</h4>
          <ul>
            <li>定期导入新数据，使用"追加合并"模式积累历史趋势</li>
            <li>先用时间筛选器选择要分析的时间范围，再开始 AI 分析</li>
            <li>关注时间趋势图，观察 ACOS 和花费的变化规律</li>
            <li>使用四象限图快速识别高潜力和需淘汰的搜索词</li>
            <li>对比不同时间段的数据，评估优化效果</li>
          </ul>
        </div>

        <!-- Knowledge base help -->
        <div v-show="activeHelpTab === 'knowledge'" class="help-content">
          <h4>功能说明</h4>
          <p>上传产品相关文档，构建企业专属知识库。通过向量检索 + 关键词搜索实现精准 AI 问答。</p>
          <ul>
            <li><strong>文档管理：</strong>支持 PDF、Word、Excel、PPT、Markdown 等格式</li>
            <li><strong>分类整理：</strong>创建分类管理文档，支持颜色标识</li>
            <li><strong>双向链接：</strong>文档间建立关联，支持反向链接查看</li>
            <li><strong>知识图谱：</strong>可视化展示文档之间的关联关系</li>
            <li><strong>智能问答：</strong>三种对话模式，支持流式输出</li>
          </ul>
          <h4>使用流程</h4>
          <ol>
            <li>配置 AI 服务 API Key</li>
            <li>点击"添加文档"或拖拽文件上传</li>
            <li>等待文档处理完成</li>
            <li>为文档添加标签</li>
            <li>建立文档间的双向链接</li>
            <li>在对话界面输入问题进行问答</li>
          </ol>
        </div>

        <!-- Weekly report help -->
        <div v-show="activeHelpTab === 'weekly_report'" class="help-content">
          <h4>功能说明</h4>
          <p>工作周报帮助你记录和总结每周工作，支持数据自动同步、AI 智能生成和多格式导出。</p>
          <ul>
            <li><strong>周次导航：</strong>使用左右箭头切换查看不同周的周报，点击"本周"快速回到当前周</li>
            <li><strong>常规任务：</strong>自动统计备忘录中重复任务的本周完成情况</li>
            <li><strong>本周完成：</strong>展示手动添加的任务、备忘录完成项、优化事件</li>
            <li><strong>本周总结：</strong>记录本周工作总结，支持 AI 润色</li>
            <li><strong>下周计划：</strong>规划下周工作，支持 AI 智能生成</li>
          </ul>

          <h4>数据来源</h4>
          <p>周报会自动整合以下数据，无需手动录入：</p>
          <ul>
            <li><strong>备忘录任务：</strong>本周完成的备忘录任务自动同步到"本周完成"</li>
            <li><strong>常规任务：</strong>有重复周期的备忘录任务自动汇总到"常规任务检查"</li>
            <li><strong>优化事件：</strong>排名监控中记录的优化事件自动同步</li>
          </ul>

          <h4>添加任务</h4>
          <p>点击"添加"按钮可手动添加工作任务，支持以下字段：</p>
          <ul>
            <li><strong>任务标题：</strong>简要描述任务内容</li>
            <li><strong>任务描述：</strong>详细说明任务细节</li>
            <li><strong>优先级：</strong>低/中/高三档，高优先级会显示红色标签</li>
            <li><strong>分类：</strong>运营、开发、设计、会议、学习、其他</li>
            <li><strong>完成进度：</strong>0-100%，未完成的任务显示进度环</li>
          </ul>

          <h4>AI 功能</h4>
          <p>点击"AI 生成"按钮，AI 会根据本周完成内容自动生成：</p>
          <ul>
            <li><strong>本周总结：</strong>基于完成事项生成工作总结</li>
            <li><strong>下周计划：</strong>根据本周情况智能规划下周工作</li>
          </ul>

          <h4>导出功能</h4>
          <p>支持多种格式导出，方便分享和存档：</p>
          <ul>
            <li><strong>导出 Excel：</strong>生成 .xlsx 文件，包含完整周报内容</li>
            <li><strong>复制 Markdown：</strong>复制为 Markdown 格式，可粘贴到其他平台</li>
          </ul>

          <h4>使用流程</h4>
          <ol>
            <li>每天随时记录完成的任务（或使用备忘录自动同步）</li>
            <li>周五点击"AI 生成"自动生成本周总结和下周计划</li>
            <li>检查并编辑 AI 生成的内容</li>
            <li>导出 Excel 或复制 Markdown 分享给领导</li>
            <li>点击"历史"查看过往周报</li>
          </ol>
        </div>
      </div>
    </div>
  </el-dialog>
</template>

<style scoped>
.help-dialog :deep(.el-dialog__body) {
  padding: 0 !important;
}

.help-layout {
  display: flex;
  height: 520px;
}

.help-nav {
  width: 200px;
  border-right: 1px solid var(--el-border-color-light);
  display: flex;
  flex-direction: column;
  background: var(--el-bg-color-page);
}

.help-search {
  margin: 12px;
  width: calc(100% - 24px);
}

.help-menu {
  flex: 1;
  border-right: none !important;
}

.help-menu .el-menu-item {
  height: 44px;
  line-height: 44px;
}

.help-menu .el-menu-item.is-active {
  background: var(--el-color-primary-light-9);
}

.help-shortcut-hint {
  padding: 12px;
  font-size: 12px;
  color: var(--el-text-color-placeholder);
  text-align: center;
  border-top: 1px solid var(--el-border-color-lighter);
}

.help-shortcut-hint kbd {
  display: inline-block;
  padding: 2px 6px;
  background: var(--el-fill-color);
  border: 1px solid var(--el-border-color);
  border-radius: 4px;
  font-family: inherit;
  font-size: 11px;
}

.help-no-result {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 24px 12px;
  color: var(--el-text-color-placeholder);
  font-size: 13px;
}

.help-no-result .el-icon {
  font-size: 24px;
}

.help-content-area {
  flex: 1;
  padding: 20px 24px;
  overflow-y: auto;
}

.help-content {
  line-height: 1.8;
  color: var(--el-text-color-regular);
}

.help-content h4 {
  margin: 16px 0 8px;
  color: var(--el-text-color-primary);
  font-size: 15px;
}

.help-content h4:first-child {
  margin-top: 0;
}

.help-content p {
  margin: 0 0 12px;
  color: var(--el-text-color-secondary);
}

.help-content ul,
.help-content ol {
  margin: 0 0 12px;
  padding-left: 20px;
}

.help-content li {
  margin-bottom: 6px;
}

.help-content strong {
  color: var(--el-text-color-primary);
}
</style>

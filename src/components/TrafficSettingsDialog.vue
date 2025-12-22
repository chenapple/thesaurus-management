<script setup lang="ts">
import { ref, watch } from 'vue';
import { ElMessage } from 'element-plus';
import { MagicStick } from '@element-plus/icons-vue';
import type { Product, TrafficLevelStats } from '../types';
import * as api from '../api';

const props = defineProps<{
  visible: boolean;
  product: Product | null;
}>();

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
  (e: 'applied', bigThreshold: number, mediumThreshold: number): void;
}>();

const trafficForm = ref({
  bigThreshold: 20000,
  mediumThreshold: 100000,
});
const trafficStats = ref<TrafficLevelStats>({ big_count: 0, medium_count: 0, small_count: 0 });
const calculatingTraffic = ref(false);

// 当对话框打开时初始化数据
watch(() => props.visible, async (newVisible) => {
  if (newVisible && props.product) {
    trafficForm.value.bigThreshold = props.product.big_word_threshold || 20000;
    trafficForm.value.mediumThreshold = props.product.medium_word_threshold || 100000;
    await loadTrafficStats();
  }
});

async function loadTrafficStats() {
  if (!props.product) return;

  try {
    trafficStats.value = await api.getTrafficLevelStats(props.product.id);
  } catch (e) {
    console.error("加载流量统计失败:", e);
  }
}

async function applyTrafficLevels() {
  if (!props.product) return;

  if (trafficForm.value.bigThreshold >= trafficForm.value.mediumThreshold) {
    ElMessage.warning("大词阈值必须小于中词阈值");
    return;
  }

  calculatingTraffic.value = true;
  try {
    // 保存阈值到产品
    await api.updateProductThresholds(
      props.product.id,
      trafficForm.value.bigThreshold,
      trafficForm.value.mediumThreshold
    );

    // 计算流量级别
    await api.calculateTrafficLevels(
      props.product.id,
      trafficForm.value.bigThreshold,
      trafficForm.value.mediumThreshold
    );

    // 重新加载统计
    await loadTrafficStats();

    ElMessage.success("流量级别已更新");

    // 通知父组件更新
    emit('applied', trafficForm.value.bigThreshold, trafficForm.value.mediumThreshold);
    emit('update:visible', false);
  } catch (e) {
    ElMessage.error("更新失败: " + e);
  } finally {
    calculatingTraffic.value = false;
  }
}

async function recommendBigThreshold() {
  if (!props.product) return;

  try {
    const recommendedThreshold = await api.recommendThreshold(props.product.id, 20);
    if (recommendedThreshold > 0) {
      trafficForm.value.bigThreshold = recommendedThreshold;
      ElMessage.success(`推荐大词阈值: ${recommendedThreshold.toLocaleString()}`);
    } else {
      ElMessage.info("数据不足，无法推荐阈值");
    }
  } catch (e) {
    ElMessage.error("推荐失败: " + e);
  }
}

function handleClose() {
  emit('update:visible', false);
}
</script>

<template>
  <el-dialog
    :model-value="visible"
    @update:model-value="$emit('update:visible', $event)"
    title="流量级别设置"
    width="480px"
  >
    <div class="traffic-settings">
      <p class="traffic-desc">
        根据周平均排名划分关键词流量级别：大词、中词、小词
      </p>

      <el-form :model="trafficForm" label-width="100px">
        <el-form-item label="大词阈值">
          <el-input-number
            v-model="trafficForm.bigThreshold"
            :min="1"
            :max="trafficForm.mediumThreshold - 1"
            :step="1000"
            style="width: 200px"
          />
          <span class="threshold-hint">排名 ≤ 此值为大词</span>
        </el-form-item>
        <el-form-item label="中词阈值">
          <el-input-number
            v-model="trafficForm.mediumThreshold"
            :min="trafficForm.bigThreshold + 1"
            :step="10000"
            style="width: 200px"
          />
          <span class="threshold-hint">排名 > 此值为小词</span>
        </el-form-item>
      </el-form>

      <div class="traffic-stats-preview">
        <div class="stat-item big">
          <span class="stat-label">大词</span>
          <span class="stat-value">{{ trafficStats.big_count }}</span>
        </div>
        <div class="stat-item medium">
          <span class="stat-label">中词</span>
          <span class="stat-value">{{ trafficStats.medium_count }}</span>
        </div>
        <div class="stat-item small">
          <span class="stat-label">小词</span>
          <span class="stat-value">{{ trafficStats.small_count }}</span>
        </div>
      </div>

      <div class="traffic-tips">
        <el-button type="info" plain size="small" @click="recommendBigThreshold">
          <el-icon><MagicStick /></el-icon>
          智能推荐（约20个大词）
        </el-button>
      </div>
    </div>
    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button type="primary" :loading="calculatingTraffic" @click="applyTrafficLevels">
        应用
      </el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.traffic-settings {
  padding: 0 10px;
}

.traffic-desc {
  color: var(--text-secondary);
  font-size: 14px;
  margin-bottom: 20px;
}

.threshold-hint {
  margin-left: 12px;
  font-size: 12px;
  color: var(--text-muted);
}

.traffic-stats-preview {
  display: flex;
  justify-content: space-around;
  margin: 24px 0;
  padding: 16px;
  background: var(--bg-primary);
  border-radius: 8px;
}

.stat-item {
  text-align: center;
}

.stat-item .stat-label {
  display: block;
  font-size: 14px;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.stat-item .stat-value {
  display: block;
  font-size: 28px;
  font-weight: 600;
}

.stat-item.big .stat-value {
  color: #f56c6c;
}

.stat-item.medium .stat-value {
  color: #e6a23c;
}

.stat-item.small .stat-value {
  color: #909399;
}

.traffic-tips {
  text-align: center;
  margin-top: 16px;
}
</style>

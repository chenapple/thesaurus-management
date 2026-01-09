<template>
  <div class="ad-world-map">
    <div class="map-container">
      <svg
        viewBox="0 0 1010 650"
        xmlns="http://www.w3.org/2000/svg"
        class="world-svg"
        @click="handleMapClick"
      >
        <defs>
          <filter id="glow" x="-20%" y="-20%" width="140%" height="140%">
            <feGaussianBlur stdDeviation="3" result="blur" />
            <feComposite in="SourceGraphic" in2="blur" operator="over" />
          </filter>
        </defs>

        <!-- 背景海洋 -->
        <rect width="100%" height="100%" fill="transparent" />

        <!-- 地图路径组 -->
        <g class="land-masses">
             <!-- 北美 -->
             <!-- CA -->
            <path
              d="M180,60 L380,60 L350,180 L140,150 Z" 
              class="country-path"
              :class="getCountryClass('CA')"
              data-code="CA"
              data-name="Canada"
              @mouseenter="showTooltip($event, 'CA')"
              @mouseleave="hideTooltip"
              @click.stop="selectCountry('CA')"
            />
             <!-- US -->
            <path
              d="M150,155 L350,185 L320,280 L120,240 Z" 
              class="country-path"
              :class="getCountryClass('US')"
              data-code="US"
              data-name="United States"
              @mouseenter="showTooltip($event, 'US')"
              @mouseleave="hideTooltip"
              @click.stop="selectCountry('US')"
            />
            <!-- MX -->
             <path
              d="M190,245 L300,285 L280,350 L220,330 Z" 
              class="country-path"
              :class="getCountryClass('MX')"
              data-code="MX"
              data-name="Mexico"
              @mouseenter="showTooltip($event, 'MX')"
              @mouseleave="hideTooltip"
              @click.stop="selectCountry('MX')"
            />
            
            <!-- 南美 (简略) -->
            <!-- BR -->
            <path
              d="M320,360 L450,380 L420,550 L300,480 Z" 
              class="country-path"
              :class="getCountryClass('BR')"
              data-code="BR"
              data-name="Brazil"
              @mouseenter="showTooltip($event, 'BR')"
              @mouseleave="hideTooltip"
              @click.stop="selectCountry('BR')"
            />

            <!-- 欧洲 (整体稍微放大以便点击) -->
            <!-- UK -->
            <path
              d="M480,130 L510,130 L505,160 L475,155 Z" 
              class="country-path"
              :class="getCountryClass('UK')"
              data-code="UK"
              data-name="United Kingdom"
              @mouseenter="showTooltip($event, 'UK')"
              @mouseleave="hideTooltip"
              @click.stop="selectCountry('UK')"
            />
            <!-- DE -->
            <path
              d="M515,145 L545,145 L540,175 L510,170 Z" 
              class="country-path"
              :class="getCountryClass('DE')"
              data-code="DE"
              data-name="Germany"
              @mouseenter="showTooltip($event, 'DE')"
              @mouseleave="hideTooltip"
              @click.stop="selectCountry('DE')"
            />
            <!-- FR -->
             <path
              d="M490,165 L520,165 L515,195 L485,190 Z" 
              class="country-path"
              :class="getCountryClass('FR')"
              data-code="FR"
              data-name="France"
              @mouseenter="showTooltip($event, 'FR')"
              @mouseleave="hideTooltip"
              @click.stop="selectCountry('FR')"
            />
             <!-- IT -->
             <path
              d="M525,185 L545,185 L550,225 L530,220 Z" 
              class="country-path"
              :class="getCountryClass('IT')"
              data-code="IT"
              data-name="Italy"
              @mouseenter="showTooltip($event, 'IT')"
              @mouseleave="hideTooltip"
              @click.stop="selectCountry('IT')"
            />
            <!-- ES -->
             <path
              d="M470,195 L500,195 L490,225 L460,215 Z" 
              class="country-path"
              :class="getCountryClass('ES')"
              data-code="ES"
              data-name="Spain"
              @mouseenter="showTooltip($event, 'ES')"
              @mouseleave="hideTooltip"
              @click.stop="selectCountry('ES')"
            />

            <!-- 亚洲/澳洲 -->
             <!-- IN -->
             <path
              d="M680,250 L750,250 L730,330 L690,310 Z" 
              class="country-path"
              :class="getCountryClass('IN')"
              data-code="IN"
              data-name="India"
              @mouseenter="showTooltip($event, 'IN')"
              @mouseleave="hideTooltip"
              @click.stop="selectCountry('IN')"
            />
            <!-- JP -->
             <path
              d="M860,180 L890,190 L870,230 L850,220 Z" 
              class="country-path"
              :class="getCountryClass('JP')"
              data-code="JP"
              data-name="Japan"
              @mouseenter="showTooltip($event, 'JP')"
              @mouseleave="hideTooltip"
              @click.stop="selectCountry('JP')"
            />
             <!-- AU -->
             <path
              d="M820,450 L950,450 L930,580 L800,550 Z" 
              class="country-path"
              :class="getCountryClass('AU')"
              data-code="AU"
              data-name="Australia"
              @mouseenter="showTooltip($event, 'AU')"
              @mouseleave="hideTooltip"
              @click.stop="selectCountry('AU')"
            />
        </g>
        
        <!-- 装饰性点阵 (示意全球覆盖) -->
        <circle cx="250" cy="150" r="1" class="map-dot" />
        <circle cx="600" cy="300" r="1" class="map-dot" />
        <circle cx="800" cy="150" r="1" class="map-dot" />
      </svg>
      
      <!-- Tooltip -->
      <div 
        v-if="tooltip.show" 
        class="map-tooltip" 
        :style="{ top: tooltip.y + 'px', left: tooltip.x + 'px' }"
      >
        <div class="tooltip-country">
            <span class="flag" v-html="getFlag(tooltip.code)"></span>
            {{ tooltip.name }}
        </div>
        <div v-if="tooltip.data" class="tooltip-data">
           <div>花费: {{ formatMoney(tooltip.data.total_spend, tooltip.code) }}</div>
           <div>ACOS: {{ tooltip.data.avg_acos.toFixed(1) }}%</div>
        </div>
        <div v-else class="tooltip-empty">暂无数据</div>
      </div>

    </div>
    
    <div class="map-legend">
        <div class="legend-item"><span class="color-dot active"></span> 有数据</div>
        <div class="legend-item"><span class="color-dot selected"></span> 当前选中</div>
        <div class="legend-item"><span class="color-dot empty"></span> 无数据</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { getCountryFlag, COUNTRY_CURRENCY_MAP, getCountryLabel } from '../../types';

const props = defineProps<{
  countryStats: any[]; // 既然使用了 map, 这里传入 stats.by_country 数组
  selectedCountry: string | null; // null 表示 GLOBAL
}>();

const emit = defineEmits<{
  (e: 'select', countryCode: string | null): void;
}>();

// Tooltip 状态
const tooltip = ref({
  show: false,
  x: 0,
  y: 0,
  code: '',
  name: '',
  data: null as any
});

// 处理点击空白处 -> 选择全球
function handleMapClick() {
  emit('select', null);
}

// 选择国家
function selectCountry(code: string) {
  emit('select', code);
}

// 获取样式类
function getCountryClass(code: string) {
  const hasData = props.countryStats.some(s => s.country === code);
  const isSelected = props.selectedCountry === code;
  return {
    'has-data': hasData,
    'is-selected': isSelected
  };
}

// 显示 Tooltip
function showTooltip(event: MouseEvent, code: string) {
  const data = props.countryStats.find(s => s.country === code);
  tooltip.value = {
    show: true,
    x: event.offsetX + 20, // 相对于容器
    y: event.offsetY - 20,
    code,
    name: getCountryLabel(code),
    data
  };
}

function hideTooltip() {
  tooltip.value.show = false;
}

function getFlag(code: string) {
  return getCountryFlag(code);
}

function formatMoney(amount: number, country: string) {
  const symbol = COUNTRY_CURRENCY_MAP[country]?.symbol || '';
  return `${symbol}${amount.toFixed(2)}`;
}
</script>

<style scoped>
.ad-world-map {
  position: relative;
  background: var(--el-fill-color-light); /* 海洋背景 */
  border-radius: 12px;
  overflow: hidden;
  height: 100%;
  min-height: 400px;
  border: 1px solid var(--el-border-color-lighter);
}

.map-container {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.world-svg {
  width: 100%;
  height: 100%;
  max-width: 900px; /* 限制最大宽度，防止拉伸太大 */
}

/* 国家路径默认样式 */
.country-path {
  fill: #e0e0e0; /* 默认灰色 */
  stroke: #ffffff;
  stroke-width: 1px;
  transition: all 0.3s ease;
  cursor: pointer;
}

.country-path:hover {
  fill: #d0d0d0;
  transform: translateY(-2px); /* 悬浮上浮效果 */ 
}

/* 有数据样式 */
.country-path.has-data {
  fill: var(--el-color-primary-light-5);
  filter: drop-shadow(0 4px 6px rgba(0, 0, 0, 0.1));
}

.country-path.has-data:hover {
  fill: var(--el-color-primary-light-3);
}

/* 选中样式 */
.country-path.is-selected {
  fill: var(--el-color-primary);
  stroke: var(--el-color-primary-dark-2);
  stroke-width: 2px;
  filter: url(#glow); /* 发光效果 */
}

.map-tooltip {
  position: absolute;
  background: rgba(0, 0, 0, 0.8);
  color: white;
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 13px;
  pointer-events: none;
  z-index: 10;
  white-space: nowrap;
  backdrop-filter: blur(4px);
}

.tooltip-country {
    display: flex;
    align-items: center;
    gap: 6px;
    font-weight: 600;
    margin-bottom: 4px;
}

.tooltip-country :deep(svg) {
    width: 20px;
    height: 14px;
    border-radius: 2px;
}

.tooltip-data {
    font-size: 12px;
    opacity: 0.9;
}

.map-legend {
    position: absolute;
    bottom: 16px;
    left: 16px;
    display: flex;
    gap: 16px;
    background: var(--el-bg-color);
    padding: 8px 12px;
    border-radius: 8px;
    box-shadow: 0 2px 12px rgba(0,0,0,0.1);
    font-size: 12px;
    color: var(--el-text-color-regular);
}

.legend-item {
    display: flex;
    align-items: center;
    gap: 6px;
}

.color-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
}
.color-dot.active { background: var(--el-color-primary-light-5); }
.color-dot.selected { background: var(--el-color-primary); }
.color-dot.empty { background: #e0e0e0; }

.map-dot {
    fill: var(--el-border-color);
}
</style>

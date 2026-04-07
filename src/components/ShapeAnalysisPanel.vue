<script setup lang="ts">
import { computed, ref } from "vue";
import type { Analysis, ShapeAnalysis } from "../types/analysis";
import {
  classifyColorRoles,
  COLOR_ROLE_LABEL,
  DEFAULT_COLOR_ROLE_THRESHOLDS,
  type ColorRole,
} from "../utils/colorRole";

const props = defineProps<{
  analysis: Analysis;
  shapeAnalysis: ShapeAnalysis | null;
  shapeLoading: boolean;
  shapeError: string;
}>();

const emit = defineEmits<{
  analyze: [];
}>();

// ---- ビュー切り替え ----
type ViewMode = "stark" | "overlay";
const viewMode = ref<ViewMode>("stark");

const imageSrc = computed(() => {
  const s = props.shapeAnalysis;
  if (!s) return "";
  return viewMode.value === "stark"
    ? `data:image/png;base64,${s.starkBase64}`
    : `data:image/png;base64,${s.overlayBase64}`;
});

// ---- 統計表示 ----
const edgeDensityPct = computed(() => {
  const s = props.shapeAnalysis;
  if (!s) return "—";
  return `${(s.edgeDensity * 100).toFixed(1)}%`;
});

// ---- カラーロール分類 ----
const baseCutoff = ref(DEFAULT_COLOR_ROLE_THRESHOLDS.baseCutoff);
const accentCutoff = ref(DEFAULT_COLOR_ROLE_THRESHOLDS.accentCutoff);

// baseCutoff と accentCutoff の整合性を保つ
function onBaseCutoffChange(val: number) {
  baseCutoff.value = val;
  if (accentCutoff.value <= val + 5) {
    accentCutoff.value = Math.min(99, val + 5);
  }
}
function onAccentCutoffChange(val: number) {
  accentCutoff.value = val;
  if (baseCutoff.value >= val - 5) {
    baseCutoff.value = Math.max(1, val - 5);
  }
}

function resetThresholds() {
  baseCutoff.value = DEFAULT_COLOR_ROLE_THRESHOLDS.baseCutoff;
  accentCutoff.value = DEFAULT_COLOR_ROLE_THRESHOLDS.accentCutoff;
}

const classifiedColors = computed(() =>
  classifyColorRoles(props.analysis.dominants, {
    baseCutoff: baseCutoff.value,
    accentCutoff: accentCutoff.value,
  }),
);

const ROLE_COLOR: Record<ColorRole, string> = {
  base: "#2563eb",
  assort: "#16a34a",
  accent: "#dc2626",
};
</script>

<template>
  <section class="block shape-panel">
    <h2 class="h">シェイプ分析</h2>

    <!-- 未実行状態 -->
    <template v-if="!shapeAnalysis && !shapeLoading">
      <p class="muted small shape-lead">
        エッジ密度から画像内の「モノがある領域（ポジティブ）」と「余白（ネガティブ）」を推定します。
      </p>
      <p v-if="shapeError" class="shape-error small">{{ shapeError }}</p>
      <button type="button" class="btn-analyze" @click="emit('analyze')">
        シェイプ分析を実行
      </button>
    </template>

    <!-- ローディング -->
    <template v-else-if="shapeLoading">
      <p class="muted small shape-lead">分析中…</p>
      <div class="shape-spinner" aria-label="分析中" />
    </template>

    <!-- 結果表示 -->
    <template v-else-if="shapeAnalysis">
      <!-- ビュー切り替えトグル -->
      <div class="shape-toggle-row">
        <div class="toggle" role="group" aria-label="表示モード">
          <button
            type="button"
            class="toggle-btn"
            :class="{ 'toggle-btn--on': viewMode === 'stark' }"
            :aria-pressed="viewMode === 'stark'"
            @click="viewMode = 'stark'"
          >
            スタークビュー
          </button>
          <button
            type="button"
            class="toggle-btn"
            :class="{ 'toggle-btn--on': viewMode === 'overlay' }"
            :aria-pressed="viewMode === 'overlay'"
            @click="viewMode = 'overlay'"
          >
            オーバーレイ
          </button>
        </div>
        <button
          type="button"
          class="linkish linkish-tiny shape-rerun"
          @click="emit('analyze')"
        >
          再実行
        </button>
      </div>

      <!-- シェイプ画像 -->
      <div class="shape-img-wrap">
        <img
          :src="imageSrc"
          :alt="viewMode === 'stark' ? 'ポジ/ネガ スタークビュー' : 'ポジ/ネガ オーバーレイ'"
          class="shape-img"
        />
        <div v-if="viewMode === 'overlay'" class="shape-legend" aria-hidden="true">
          <span class="shape-legend-dot shape-legend-dot--pos" />ポジティブ
          <span class="shape-legend-dot shape-legend-dot--neg" />ネガティブ
        </div>
      </div>

      <!-- 統計 -->
      <dl class="shape-stats">
        <div class="shape-stat-row">
          <dt>ポジティブ</dt>
          <dd>{{ shapeAnalysis.positiveAreaPct.toFixed(1) }}%</dd>
        </div>
        <div class="shape-stat-row">
          <dt>ネガティブ</dt>
          <dd>{{ shapeAnalysis.negativeAreaPct.toFixed(1) }}%</dd>
        </div>
        <div class="shape-stat-row">
          <dt>エッジ密度</dt>
          <dd>{{ edgeDensityPct }}（{{ shapeAnalysis.complexityJa }}）</dd>
        </div>
        <div class="shape-stat-row">
          <dt>シェイプ数（推定）</dt>
          <dd>{{ shapeAnalysis.regionCount }}</dd>
        </div>
      </dl>
    </template>

    <!-- ---- カラーロール分類（常時表示） ---- -->
    <div class="shape-role-section">
      <h3 class="h shape-role-h">配色の役割分類</h3>
      <p class="muted small shape-lead">
        支配色の面積比をもとに、ベース / アソート / アクセントを分類します。
      </p>

      <!-- スライダー -->
      <div class="role-sliders">
        <label class="role-slider-label">
          <span class="role-slider-name" :style="{ color: ROLE_COLOR.base }">ベース上限</span>
          <input
            type="range"
            min="5"
            max="90"
            step="1"
            :value="baseCutoff"
            class="role-slider"
            :aria-label="`ベース上限 ${baseCutoff}%`"
            @input="onBaseCutoffChange(Number(($event.target as HTMLInputElement).value))"
          />
          <span class="role-slider-val">{{ baseCutoff }}%</span>
        </label>
        <label class="role-slider-label">
          <span class="role-slider-name" :style="{ color: ROLE_COLOR.accent }">アクセント開始</span>
          <input
            type="range"
            min="10"
            max="99"
            step="1"
            :value="accentCutoff"
            class="role-slider"
            :aria-label="`アクセント開始 ${accentCutoff}%`"
            @input="onAccentCutoffChange(Number(($event.target as HTMLInputElement).value))"
          />
          <span class="role-slider-val">{{ accentCutoff }}%</span>
        </label>
        <button
          type="button"
          class="linkish linkish-tiny role-reset"
          :disabled="baseCutoff === DEFAULT_COLOR_ROLE_THRESHOLDS.baseCutoff && accentCutoff === DEFAULT_COLOR_ROLE_THRESHOLDS.accentCutoff"
          @click="resetThresholds"
        >
          デフォルト（70 / 95）に戻す
        </button>
      </div>

      <!-- 分類結果 -->
      <ul class="role-list" aria-label="配色役割一覧">
        <li
          v-for="d in classifiedColors"
          :key="d.hex"
          class="role-row"
        >
          <span
            class="swatch xs"
            aria-hidden="true"
            :style="{ backgroundColor: d.hex }"
          />
          <span
            class="role-badge"
            :style="{
              color: ROLE_COLOR[d.role],
              borderColor: ROLE_COLOR[d.role],
            }"
          >{{ COLOR_ROLE_LABEL[d.role] }}</span>
          <span class="role-hex mono small">{{ d.hex }}</span>
          <span class="role-pct muted small">{{ d.pct.toFixed(1) }}%</span>
        </li>
      </ul>
    </div>
  </section>
</template>

<style scoped>
.shape-panel {
  border-top: 1px solid var(--stroke);
  padding-top: 1.1rem;
  margin-top: 0.25rem;
}

.shape-lead {
  margin: 0 0 0.75rem;
  line-height: 1.55;
}

.shape-error {
  color: var(--danger);
  margin: 0 0 0.5rem;
}

.btn-analyze {
  font-size: 0.9375rem;
  font-weight: 600;
  padding: 0.42rem 1rem;
  border-radius: 8px;
  border: 1px solid var(--stroke);
  background: var(--card);
  color: var(--text);
  cursor: pointer;
  transition: background 0.12s ease;
}

.btn-analyze:hover {
  background: var(--surface);
}

.btn-analyze:focus-visible {
  outline: 2px solid var(--link);
  outline-offset: 2px;
}

.shape-spinner {
  width: 28px;
  height: 28px;
  border: 3px solid var(--stroke);
  border-top-color: var(--primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin: 0.5rem 0;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.shape-toggle-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 0.65rem;
}

.shape-rerun {
  margin-left: auto;
}

.shape-img-wrap {
  position: relative;
  margin-bottom: 0.75rem;
}

.shape-img {
  display: block;
  width: 100%;
  border-radius: 8px;
  border: 1px solid var(--stroke);
  background: #111;
}

.shape-legend {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-top: 0.35rem;
  font-size: 0.875rem;
  color: var(--muted);
}

.shape-legend-dot {
  display: inline-block;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  vertical-align: middle;
  margin-right: 0.25rem;
}

.shape-legend-dot--pos { background: #ff6b35; }
.shape-legend-dot--neg { background: #4a90d9; }

.shape-stats {
  margin: 0 0 0.85rem;
  padding: 0.6rem 0.75rem;
  background: var(--surface);
  border-radius: 8px;
  border: 1px solid var(--stroke);
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 0.2rem 0.75rem;
}

.shape-stat-row {
  display: contents;
}

.shape-stats dt {
  font-weight: 600;
  font-size: 0.9375rem;
  color: var(--muted);
  white-space: nowrap;
}

.shape-stats dd {
  margin: 0;
  font-size: 0.9375rem;
}

/* ---- カラーロール ---- */
.shape-role-section {
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 1px solid var(--stroke);
}

.shape-role-h {
  margin-bottom: 0.4rem;
  font-size: 1.0625rem;
}

.role-sliders {
  margin: 0.65rem 0 0.85rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.role-slider-label {
  display: grid;
  grid-template-columns: 6rem 1fr 2.5rem;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.875rem;
}

.role-slider-name {
  font-weight: 600;
  white-space: nowrap;
}

.role-slider {
  width: 100%;
  accent-color: var(--primary);
  cursor: pointer;
}

.role-slider-val {
  text-align: right;
  font-size: 0.875rem;
  color: var(--muted);
  font-family: ui-monospace, monospace;
}

.role-reset {
  align-self: flex-start;
  font-size: 0.8125rem;
}

.role-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.45rem;
}

.role-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.role-badge {
  font-size: 0.75rem;
  font-weight: 700;
  padding: 0.1rem 0.4rem;
  border-radius: 4px;
  border: 1px solid currentColor;
  white-space: nowrap;
  flex-shrink: 0;
}

.role-hex {
  flex: 1;
  min-width: 0;
}

.role-pct {
  flex-shrink: 0;
}
</style>

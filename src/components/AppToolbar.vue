<script setup lang="ts">
defineProps<{ loading: boolean }>();
const emit = defineEmits<{ open: [] }>();
</script>

<template>
  <nav class="toolbar" aria-label="画像の選択とメニュー案内">
    <button
      type="button"
      class="btn btn-primary btn-open"
      :disabled="loading"
      @click="emit('open')"
    >
      {{ loading ? "読み込み中…" : "開く…" }}
    </button>
    <p class="toolbar-hint">
      閉じる・書き出し・用語集は、<strong>メニュー</strong>の「ファイル」「ヘルプ」から
    </p>
  </nav>
</template>

<style scoped>
.toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.65rem 1rem;
  padding: 0.65rem 1.25rem;
  background: #f3f3f5;
  border-bottom: 1px solid #d8d8de;
  width: 100%;
  box-sizing: border-box;
  flex-shrink: 0;
}

.toolbar-hint {
  margin: 0;
  flex: 1 1 12rem;
  font-size: 0.8125rem;
  line-height: 1.45;
  color: #5c5c66;
  font-weight: 500;
}

.toolbar-hint strong {
  font-weight: 600;
  color: #3a3a42;
}

.btn {
  border-radius: 8px;
  padding: 0.5rem 1.1rem;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid transparent;
  transition:
    background 0.14s ease,
    border-color 0.14s ease,
    box-shadow 0.14s ease,
    transform 0.1s ease,
    filter 0.14s ease;
}

.btn:focus-visible {
  outline: 2px solid var(--link);
  outline-offset: 2px;
}

.btn:focus:not(:focus-visible) {
  outline: none;
}

.btn:disabled {
  opacity: 0.48;
  cursor: not-allowed;
  box-shadow: none;
  transform: none;
}

.btn-primary {
  color: #fff;
  background: linear-gradient(180deg, #0b84ff 0%, #0071e3 100%);
  border-color: rgba(0, 0, 0, 0.12);
  box-shadow:
    0 1px 0 rgba(255, 255, 255, 0.2) inset,
    0 1px 2px rgba(0, 50, 120, 0.22);
}

.btn-primary:hover:not(:disabled) {
  filter: brightness(1.06);
  box-shadow:
    0 1px 0 rgba(255, 255, 255, 0.22) inset,
    0 2px 8px rgba(0, 80, 180, 0.28);
}

.btn-primary:active:not(:disabled) {
  filter: brightness(0.96);
  box-shadow: 0 1px 1px rgba(0, 40, 100, 0.2) inset;
}

.btn-open {
  border-radius: 999px;
  padding: 0.42rem 1.35rem;
  font-size: 0.9375rem;
  font-weight: 600;
  letter-spacing: 0.01em;
}
</style>

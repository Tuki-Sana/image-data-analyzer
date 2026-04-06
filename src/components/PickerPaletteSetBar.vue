<script setup lang="ts">
import type { PickerPaletteSet } from "../utils/pickerPaletteStorage";
import {
  PICKER_SET_NAME_MAX,
  paletteSetDisplayName,
} from "../utils/pickerPaletteStorage";

defineProps<{
  palettes: PickerPaletteSet[];
  activeId: string;
  setName: string;
  canDeleteSet: boolean;
}>();

const emit = defineEmits<{
  "update:activeId": [id: string];
  "update:setName": [name: string];
  newSet: [];
  duplicate: [];
  deleteSet: [];
}>();

function onSelect(ev: Event) {
  emit("update:activeId", (ev.target as HTMLSelectElement).value);
}

function onNameInput(ev: Event) {
  emit("update:setName", (ev.target as HTMLInputElement).value);
}
</script>

<template>
  <div class="palette-set-bar">
    <label class="palette-set-field">
      <span class="palette-set-field-label">カラーセット</span>
      <select class="palette-set-select" :value="activeId" @change="onSelect">
        <option v-for="p in palettes" :key="p.id" :value="p.id">
          {{ paletteSetDisplayName(p) }}
        </option>
      </select>
    </label>
    <label class="palette-set-field palette-set-field--grow">
      <span class="palette-set-field-label">セット名</span>
      <input
        class="palette-set-name-input"
        type="text"
        :value="setName"
        :maxlength="PICKER_SET_NAME_MAX"
        placeholder="無題"
        autocomplete="off"
        @input="onNameInput"
      />
    </label>
    <div class="palette-set-actions">
      <button type="button" class="palette-set-btn" @click="emit('newSet')">
        新規
      </button>
      <button type="button" class="palette-set-btn" @click="emit('duplicate')">
        複製
      </button>
      <button
        type="button"
        class="palette-set-btn palette-set-btn--danger"
        :disabled="!canDeleteSet"
        title="このカラーセットごと削除します（中の色も消えます）"
        @click="emit('deleteSet')"
      >
        セット削除
      </button>
    </div>
  </div>
</template>

<style scoped>
.palette-set-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-end;
  gap: 0.65rem 0.85rem;
  margin-bottom: 0.85rem;
  padding-bottom: 0.85rem;
  border-bottom: 1px solid var(--stroke);
}

.palette-set-field {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  min-width: 0;
}

.palette-set-field--grow {
  flex: 1 1 10rem;
}

.palette-set-field-label {
  font-size: 0.75rem;
  font-weight: 700;
  color: #5a5a62;
  letter-spacing: 0.02em;
}

.palette-set-select,
.palette-set-name-input {
  font-size: 0.875rem;
  padding: 0.35rem 0.5rem;
  border-radius: 8px;
  border: 1px solid #c5c5d0;
  background: #fff;
  color: var(--text);
  min-width: 0;
}

.palette-set-select {
  min-width: 7.5rem;
  max-width: 100%;
}

.palette-set-name-input {
  width: 100%;
  box-sizing: border-box;
}

.palette-set-name-input:focus,
.palette-set-select:focus {
  outline: 2px solid rgba(59, 130, 246, 0.35);
  outline-offset: 1px;
}

.palette-set-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
  align-items: center;
}

.palette-set-btn {
  font-size: 0.8125rem;
  font-weight: 600;
  padding: 0.35rem 0.65rem;
  border-radius: 8px;
  border: 1px solid #c5c5d0;
  background: #fff;
  color: var(--text);
  cursor: pointer;
  transition:
    background 0.12s ease,
    border-color 0.12s ease;
}

.palette-set-btn:hover:not(:disabled) {
  background: #f4f4f8;
  border-color: #a8a8b8;
}

.palette-set-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.palette-set-btn--danger {
  border-color: #e8c4c4;
  color: #b91c1c;
}

.palette-set-btn--danger:hover:not(:disabled) {
  background: #fff5f5;
  border-color: #e08080;
}
</style>

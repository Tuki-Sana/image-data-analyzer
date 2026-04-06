<script setup lang="ts">
import { ref } from "vue";

defineProps<{
  previewSrc: string;
  previewImageAlt: string;
  previewBgDark: boolean;
}>();

const emit = defineEmits<{
  sample: [
    payload: {
      clientX: number;
      clientY: number;
      img: HTMLImageElement;
    },
  ];
}>();

const imgRef = ref<HTMLImageElement | null>(null);

function onClick(e: MouseEvent) {
  const img = imgRef.value;
  if (!img) return;
  emit("sample", { clientX: e.clientX, clientY: e.clientY, img });
}

function onKeydown(e: KeyboardEvent) {
  if (e.key !== "Enter" && e.key !== " ") return;
  e.preventDefault();
  const img = imgRef.value;
  if (!img) return;
  const r = img.getBoundingClientRect();
  emit("sample", {
    clientX: r.left + r.width / 2,
    clientY: r.top + r.height / 2,
    img,
  });
}
</script>

<template>
  <section
    class="preview-wrap"
    :class="previewBgDark ? 'canvas-dark' : 'canvas-light'"
    aria-label="プレビュー"
  >
    <div class="preview-stage">
      <img
        ref="imgRef"
        :src="previewSrc"
        class="preview-img"
        :alt="previewImageAlt"
        tabindex="0"
        aria-describedby="preview-hint"
        @click="onClick"
        @keydown="onKeydown"
      />
    </div>
    <p id="preview-hint" class="hint">
      クリック、または画像にフォーカスして Enter / Space
      でその位置（キーボードは中央）の色を取得（原画像座標でサンプル）
    </p>
  </section>
</template>

<style scoped>
.preview-wrap {
  border-radius: 12px;
  padding: 12px;
  border: 1px solid var(--stroke);
  min-height: 0;
  min-width: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.preview-stage {
  flex: 1 1 0;
  min-height: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.canvas-light {
  background: repeating-conic-gradient(#e8ecf2 0% 25%, #d8dee8 0% 50%) 50% /
    20px 20px;
}

.canvas-dark {
  background: repeating-conic-gradient(#34383e 0% 25%, #40444c 0% 50%) 50% /
    20px 20px;
}

.preview-img {
  max-width: 100%;
  max-height: 100%;
  width: auto;
  height: auto;
  object-fit: contain;
  display: block;
  border-radius: 8px;
  cursor: crosshair;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.12);
}

.preview-img:focus-visible {
  outline: 3px solid var(--link);
  outline-offset: 3px;
}

.hint {
  margin: 0.5rem 0 0;
  font-size: 1rem;
  line-height: 1.45;
  flex-shrink: 0;
}

.preview-wrap.canvas-light .hint {
  color: #3d3d45;
}

.preview-wrap.canvas-dark .hint {
  color: rgba(255, 255, 255, 0.92);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.45);
}
</style>

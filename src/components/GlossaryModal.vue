<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, watch } from "vue";
import { glossaryEntries } from "../constants/glossaryJa";

const props = withDefaults(
  defineProps<{ open: boolean; focusEntryId?: string | null }>(),
  { focusEntryId: null },
);
const emit = defineEmits<{ close: [] }>();

function scrollToId(id: string) {
  const el = document.getElementById(`glossary-${id}`);
  el?.scrollIntoView({ behavior: "smooth", block: "start" });
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape" && props.open) {
    e.preventDefault();
    emit("close");
  }
}

watch(
  () => props.open,
  (v) => {
    if (v) {
      document.body.style.overflow = "hidden";
    } else {
      document.body.style.overflow = "";
    }
  },
);

watch(
  () => [props.open, props.focusEntryId] as const,
  async ([isOpen, id]) => {
    if (!isOpen || !id) return;
    await nextTick();
    scrollToId(id);
  },
);

onMounted(() => window.addEventListener("keydown", onKeydown));
onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
  document.body.style.overflow = "";
});
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="glossary-backdrop"
      role="presentation"
      @click.self="emit('close')"
    >
      <div
        class="glossary-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="glossary-title"
      >
        <div class="glossary-head">
          <h2 id="glossary-title" class="glossary-title">用語集・測り方</h2>
          <button type="button" class="btn-close" @click="emit('close')">
            閉じる
          </button>
        </div>

        <details class="glossary-toc-fold">
          <summary class="glossary-toc-summary">
            目次（開いて各節へジャンプ）
          </summary>
          <nav class="glossary-toc" aria-label="目次">
            <ul class="glossary-toc-list">
              <li v-for="e in glossaryEntries" :key="e.id" class="glossary-toc-item">
                <a
                  href="#"
                  class="glossary-toc-item-link"
                  @click.prevent="scrollToId(e.id)"
                >
                  {{ e.title }}
                </a>
              </li>
            </ul>
          </nav>
        </details>

        <div class="glossary-body">
          <article
            v-for="e in glossaryEntries"
            :id="`glossary-${e.id}`"
            :key="e.id"
            class="glossary-article"
          >
            <h3 class="glossary-h3">{{ e.title }}</h3>
            <p
              v-for="(p, i) in e.paragraphs"
              :key="i"
              class="glossary-p"
            >
              {{ p }}
            </p>
          </article>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.glossary-backdrop {
  position: fixed;
  inset: 0;
  z-index: 200;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
  box-sizing: border-box;
}

/* 高さを確定させないと flex の本文が潰れて 1 行程度しか見えない */
/* 16px 基準：Web での本文可読性の一般的下限。本文は 1.125rem ≒ 18px（弱視・高齢視でも読みやすい目安） */
.glossary-dialog {
  width: min(820px, calc(100vw - 2rem));
  height: min(92vh, 920px);
  max-height: 92vh;
  min-height: min(400px, 55vh);
  display: flex;
  flex-direction: column;
  background: var(--card, #fff);
  color: var(--text, #18181c);
  border-radius: 12px;
  border: 1px solid var(--stroke, #babac2);
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.2);
  overflow: hidden;
  font-size: 16px;
  line-height: 1.5;
}

.glossary-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  padding: 0.85rem 1rem;
  border-bottom: 1px solid var(--stroke, #babac2);
  background: var(--surface, #ececf0);
  flex-shrink: 0;
}

.glossary-title {
  margin: 0;
  font-size: 1.375rem;
  font-weight: 700;
  line-height: 1.3;
}

.btn-close {
  flex-shrink: 0;
  min-height: 44px;
  min-width: 4.5rem;
  padding: 0.5rem 1rem;
  font-size: 1rem;
  font-weight: 600;
  border-radius: 8px;
  border: 1px solid var(--stroke, #babac2);
  background: var(--card, #fff);
  cursor: pointer;
  color: var(--text, #18181c);
}

.btn-close:hover {
  background: var(--bg, #f8f8fa);
}

.btn-close:focus-visible {
  outline: 2px solid var(--link, #2563eb);
  outline-offset: 2px;
}

.glossary-toc-fold {
  flex-shrink: 0;
  border-bottom: 1px solid var(--stroke, #e0e0e6);
  background: #fafafc;
}

.glossary-toc-summary {
  padding: 0.65rem 1rem;
  font-size: 1.0625rem;
  font-weight: 700;
  color: var(--text, #18181c);
  cursor: pointer;
  list-style: none;
  user-select: none;
}

.glossary-toc-summary::-webkit-details-marker {
  display: none;
}

.glossary-toc-summary::before {
  content: "▸ ";
  display: inline-block;
  transition: transform 0.1s ease;
}

.glossary-toc-fold[open] > .glossary-toc-summary::before {
  transform: rotate(90deg);
}

.glossary-toc-summary:focus-visible {
  outline: 2px solid var(--link, #2563eb);
  outline-offset: 2px;
  border-radius: 4px;
}

.glossary-toc {
  padding: 0;
}

.glossary-toc-list {
  list-style: none;
  margin: 0;
  padding: 0 1rem 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.45rem;
  max-height: min(38vh, 360px);
  overflow-y: auto;
}

.glossary-toc-item {
  margin: 0;
}

.glossary-toc-item-link {
  padding: 0.6rem 0.85rem;
  min-height: 44px;
  box-sizing: border-box;
  display: flex;
  align-items: center;
  font-size: 1rem;
  line-height: 1.45;
  color: #1e3a5f;
  text-decoration: none;
  background: #fff;
  border: 1px solid #d8dce8;
  border-radius: 8px;
  border-left: 4px solid var(--link, #2563eb);
  transition:
    background 0.12s ease,
    border-color 0.12s ease;
}

.glossary-toc-item-link:hover {
  background: #eef3ff;
  border-color: #b8c5e8;
}

.glossary-toc-item-link:focus-visible {
  outline: 3px solid rgba(37, 99, 235, 0.45);
  outline-offset: 2px;
}

.glossary-body {
  padding: 1.1rem 1.25rem 1.5rem;
  overflow-y: auto;
  flex: 1 1 auto;
  min-height: 200px;
  -webkit-overflow-scrolling: touch;
}

.glossary-article {
  margin-bottom: 1.5rem;
  scroll-margin-top: 0.85rem;
  max-width: 42rem;
  margin-left: auto;
  margin-right: auto;
}

.glossary-article:last-child {
  margin-bottom: 0;
}

.glossary-h3 {
  margin: 0 0 0.65rem;
  font-size: 1.3125rem;
  font-weight: 700;
  color: var(--text, #18181c);
  line-height: 1.35;
}

.glossary-p {
  margin: 0 0 0.75rem;
  font-size: 1.125rem;
  line-height: 1.7;
  color: #1f1f26;
}

.glossary-p:last-child {
  margin-bottom: 0;
}
</style>

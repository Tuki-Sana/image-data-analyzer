<script setup lang="ts">
defineProps<{ appDisplayName: string }>();
</script>

<template>
  <header class="header" role="banner">
    <div class="header-inner">
      <div class="header-brand">
        <span class="header-mark" aria-hidden="true" />
        <div class="header-titles">
          <h1 class="title">{{ appDisplayName }}</h1>
        </div>
      </div>
    </div>
    <div class="header-accent" aria-hidden="true" />
  </header>
</template>

<style scoped>
.header {
  position: relative;
  background: linear-gradient(180deg, #35353a 0%, var(--primary) 100%);
  color: #fff;
  padding: 0.85rem 1.25rem 0;
  border-bottom: none;
  flex-shrink: 0;
}

.header-inner {
  max-width: 1200px;
  margin: 0 auto;
  padding-bottom: 0.75rem;
}

.header-brand {
  display: flex;
  align-items: center;
  gap: 0.85rem;
}

.header-mark {
  width: 2.5rem;
  height: 2.5rem;
  flex-shrink: 0;
  border-radius: 12px;
  background: conic-gradient(
    from 200deg,
    #f87171,
    #fbbf24,
    #4ade80,
    #38bdf8,
    #818cf8,
    #e879f9,
    #f87171
  );
  box-shadow:
    0 2px 6px rgba(0, 0, 0, 0.35),
    inset 0 1px 0 rgba(255, 255, 255, 0.35);
}

.header-titles {
  flex: 1;
  min-width: 0;
}

.header-accent {
  height: 3px;
  width: 100%;
  max-width: 1200px;
  margin: 0 auto;
  border-radius: 2px 2px 0 0;
  background: linear-gradient(
    90deg,
    #6366f1,
    #8b5cf6,
    #ec4899,
    #f97316,
    #eab308,
    #22c55e,
    #0ea5e9
  );
  opacity: 0.92;
}

.title {
  margin: 0;
  font-size: 1.35rem;
  font-weight: 800;
  letter-spacing: -0.02em;
  line-height: 1.2;
  /* 太字のキャップ高でアイコンより視覚的に上に寄るのを抑える */
  transform: translateY(0.07em);
}
</style>

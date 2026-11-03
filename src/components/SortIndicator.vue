<template>
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 24 24"
    fill="none"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    class="size-3.5"
    aria-hidden="true"
    focusable="false"
  >
    <!-- 下折角 -->
    <path
      d="m7 15 5 5 5-5"
      stroke="currentColor"
      class="transition-colors"
      :class="active === 'desc' ? 'text-foreground' : 'text-muted-foreground/60'"
    />
    <!-- 上折角 -->
    <path
      d="m7 9 5-5 5 5"
      stroke="currentColor"
      class="transition-colors"
      :class="active === 'asc' ? 'text-foreground' : 'text-muted-foreground/60'"
    />
  </svg>
</template>

<script setup lang="ts">
/** ChevronsUpDown 图标：可单独高亮上/下折角，active 为当前排序方向 */
defineProps<{ active: "asc" | "desc" | null }>();
</script>

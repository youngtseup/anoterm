<template>
  <div class="flex shrink-0 items-center gap-3 border-b border-border bg-background px-3 py-2">
    <Loader2 class="size-4 shrink-0 animate-spin text-muted-foreground" />
    <div class="min-w-0 flex-1">
      <div class="flex items-center justify-between gap-3 text-[12px] leading-none">
        <span class="truncate" :title="current">{{ current || "准备中…" }}</span>
        <span class="shrink-0 text-muted-foreground">{{ detail }}</span>
      </div>
      <div class="mt-1.5 h-1.5 overflow-hidden rounded-full bg-muted">
        <div
          v-if="total > 0"
          class="h-full rounded-full bg-primary transition-[width] duration-150"
          :style="{ width: `${percent}%` }"
        />
        <div v-else class="h-full w-1/3 animate-pulse rounded-full bg-primary/60" />
      </div>
    </div>
    <Button variant="outline" size="sm" :disabled="cancelling" @click="emit('cancel')">
      <Loader2 v-if="cancelling" class="size-4 animate-spin" />
      <X v-else class="size-4" />
      {{ cancelling ? "取消中" : "取消" }}
    </Button>
  </div>
</template>

<script setup lang="ts">
/**
 * 传输进度条：当前文件、字节进度/百分比与取消按钮。
 * total 未知（统计中/目录预扫描失败）时显示不定长动画。
 */
import { computed } from "vue";
import { Loader2, X } from "@lucide/vue";
import { Button } from "./ui/button";

const props = defineProps<{
  current: string;
  transferred: number;
  total: number;
  cancelling?: boolean;
}>();

const emit = defineEmits<{ cancel: [] }>();

const percent = computed(() =>
  props.total > 0 ? Math.min(100, Math.floor((props.transferred / props.total) * 100)) : 0,
);

const detail = computed(() => {
  if (props.total > 0) {
    return `${percent.value}% · ${formatBytes(props.transferred)} / ${formatBytes(props.total)}`;
  }
  return formatBytes(props.transferred);
});

function formatBytes(size: number): string {
  if (size < 1024) return size + " B";
  if (size < 1024 * 1024) return (size / 1024).toFixed(1) + " KB";
  if (size < 1024 * 1024 * 1024) return (size / (1024 * 1024)).toFixed(1) + " MB";
  return (size / (1024 * 1024 * 1024)).toFixed(2) + " GB";
}
</script>

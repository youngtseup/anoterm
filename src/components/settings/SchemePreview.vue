<script setup lang="ts">
import type { HTMLAttributes } from "vue";
import { cn } from "@/lib/utils";
import type { SchemeColors } from "@/lib/theme";

const props = defineProps<{
  colors: SchemeColors;
  class?: HTMLAttributes["class"];
}>();
</script>

<template>
  <div
    :class="cn('flex overflow-hidden rounded-md border', props.class)"
    :style="{ background: props.colors.background, borderColor: props.colors.border }"
  >
    <div
      class="flex w-1/4 flex-col gap-1 border-r p-1.5"
      :style="{ background: props.colors.surface, borderColor: props.colors.border }"
    >
      <span class="h-1 w-6 rounded-full" :style="{ background: props.colors.primary }"></span>
      <span class="h-0.5 w-full rounded-full" :style="{ background: props.colors.muted }"></span>
      <span class="h-0.5 w-3/4 rounded-full" :style="{ background: props.colors.muted }"></span>
    </div>
    <div class="flex flex-1 flex-col justify-center gap-1 p-2">
      <span
        class="h-0.5 w-2/3 rounded-full"
        :style="{ background: props.colors.foreground }"
      ></span>
      <span
        class="h-0.5 w-1/2 rounded-full"
        :style="{ background: props.colors.foreground, opacity: 0.65 }"
      ></span>
      <div class="mt-1 flex items-center gap-1">
        <span class="h-1.5 w-7 rounded-sm" :style="{ background: props.colors.primary }"></span>
        <span
          class="size-1.5 rounded-full"
          :style="{ background: props.colors.destructive }"
        ></span>
      </div>
    </div>
  </div>
</template>

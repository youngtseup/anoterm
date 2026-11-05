<template>
  <Dialog v-model:open="open">
    <DialogContent
      class="inset-x-0 top-10 bottom-0 flex max-w-none translate-x-0 translate-y-0 flex-col gap-0 overflow-hidden rounded-none border-0 p-0 sm:max-w-none"
      :show-close-button="false"
      @pointer-down-outside.prevent
    >
      <DialogTitle class="sr-only">设置</DialogTitle>
      <DialogDescription class="sr-only">应用设置与版本信息</DialogDescription>

      <div class="flex min-h-0 flex-1 flex-col">
        <div class="flex min-h-0 flex-1">
          <aside class="flex w-44 shrink-0 flex-col border-r border-border bg-muted/30 p-3">
            <div class="px-2 pt-1 pb-3 text-base font-semibold">设置</div>
            <nav class="flex flex-col gap-1">
              <button
                v-for="section in sections"
                :key="section.id"
                type="button"
                class="flex cursor-pointer items-center gap-2.5 rounded-md px-2.5 py-2 text-left text-sm transition-colors"
                :class="
                  activeSectionId === section.id
                    ? 'bg-accent font-medium text-accent-foreground'
                    : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground'
                "
                @click="activeSectionId = section.id"
              >
                <component :is="section.icon" class="size-4" />
                {{ section.label }}
              </button>
            </nav>
          </aside>

          <section class="min-w-0 flex-1 overflow-y-auto p-6">
            <component :is="activeSection.component" :key="activeSection.id" />
          </section>
        </div>

        <footer
          class="flex items-center justify-end gap-2 border-t border-border bg-muted/20 px-4 py-3"
        >
          <Button variant="outline" @click="close">关闭</Button>
          <Button variant="outline" @click="apply">应用</Button>
          <Button @click="applyAndClose">应用并关闭</Button>
        </footer>
      </div>
    </DialogContent>
  </Dialog>
</template>

<script setup lang="ts">
/**
 * 设置弹窗：
 * - 左侧菜单与右侧内容由 sections 配置驱动，每个菜单对应独立组件
 * - 右下角操作：关闭、应用、应用并关闭
 */
import { computed, markRaw, ref } from "vue";
import { toast } from "vue-sonner";
import { Info } from "@lucide/vue";
import { Button } from "./ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "./ui/dialog";
import AboutSettings from "./settings/AboutSettings.vue";

const sections = [
  {
    id: "about",
    label: "关于我们",
    icon: markRaw(Info),
    component: markRaw(AboutSettings),
  },
] as const;

type SectionId = (typeof sections)[number]["id"];

const props = defineProps<{ modelValue: boolean }>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
}>();

const open = computed({
  get: () => props.modelValue,
  set: (v: boolean) => emit("update:modelValue", v),
});

const activeSectionId = ref<SectionId>("about");
const activeSection = computed(
  () => sections.find((section) => section.id === activeSectionId.value) ?? sections[0],
);

function close() {
  open.value = false;
}

function apply() {
  toast.success("设置已保存");
}

function applyAndClose() {
  apply();
  open.value = false;
}
</script>

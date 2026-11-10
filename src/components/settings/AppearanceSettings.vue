<script setup lang="ts">
import { ref } from "vue";
import { Check, Copy, Pencil, Plus, Trash2 } from "@lucide/vue";
import {
  resolveSchemeColors,
  schemeList,
  type SchemeColors,
  type SchemeTuning,
  type ThemeMode,
  type ThemeScheme,
} from "../../lib/theme";
import { useTheme } from "../../composables/useTheme";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "../ui/alert-dialog";
import { Button } from "../ui/button";
import SchemeEditorDialog from "./SchemeEditorDialog.vue";
import SchemePreview from "./SchemePreview.vue";

const { appearanceState, selectScheme, addCustomScheme, updateCustomScheme, deleteCustomScheme } =
  useTheme();

const modes: { mode: ThemeMode; label: string; description: string }[] = [
  { mode: "light", label: "亮色模式", description: "选择明亮模式使用的配色方案" },
  { mode: "dark", label: "暗色模式", description: "选择黑暗模式使用的配色方案" },
];

const editorOpen = ref(false);
const editorMode = ref<ThemeMode>("light");
const editingScheme = ref<ThemeScheme | null>(null);
const templateScheme = ref<ThemeScheme | null>(null);
const deleteTarget = ref<ThemeScheme | null>(null);
const deleteOpen = ref(false);

function schemesFor(mode: ThemeMode): ThemeScheme[] {
  return schemeList(mode, appearanceState.value.customSchemes);
}

function activeIdFor(mode: ThemeMode): string {
  return mode === "light"
    ? appearanceState.value.activeLightId
    : appearanceState.value.activeDarkId;
}

function openCreate(mode: ThemeMode) {
  editorMode.value = mode;
  editingScheme.value = null;
  templateScheme.value = null;
  editorOpen.value = true;
}

function openEdit(scheme: ThemeScheme) {
  editorMode.value = scheme.mode;
  editingScheme.value = scheme;
  templateScheme.value = null;
  editorOpen.value = true;
}

function requestDelete(scheme: ThemeScheme) {
  deleteTarget.value = scheme;
  deleteOpen.value = true;
}

function handleSave(payload: { name: string; colors: SchemeColors; tuning: SchemeTuning }) {
  if (editingScheme.value) {
    updateCustomScheme(editingScheme.value.id, payload);
  } else {
    addCustomScheme({ ...payload, mode: editorMode.value });
  }
  editorOpen.value = false;
}

function duplicateBuiltin(scheme: ThemeScheme) {
  editorMode.value = scheme.mode;
  editingScheme.value = null;
  templateScheme.value = scheme;
  editorOpen.value = true;
}

function confirmDelete() {
  const target = deleteTarget.value;
  if (target) deleteCustomScheme(target.id);
  deleteTarget.value = null;
  deleteOpen.value = false;
}
</script>

<template>
  <div>
    <h2 class="text-base font-semibold">外观</h2>
    <p class="mt-1 text-sm text-muted-foreground">
      选择亮色与暗色模式各自的配色方案，也可以创建自己的方案。
    </p>

    <section v-for="item in modes" :key="item.mode" class="mt-6">
      <div>
        <h3 class="text-sm font-semibold">{{ item.label }}</h3>
        <p class="mt-0.5 text-xs text-muted-foreground">{{ item.description }}</p>
      </div>

      <div class="mt-3 grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
        <div
          v-for="scheme in schemesFor(item.mode)"
          :key="scheme.id"
          role="button"
          tabindex="0"
          class="group relative cursor-pointer rounded-lg border p-2 transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring/50"
          :class="
            activeIdFor(item.mode) === scheme.id
              ? 'border-primary bg-primary/5 ring-1 ring-primary/30'
              : 'border-border hover:border-ring/50 hover:bg-muted/40'
          "
          :aria-pressed="activeIdFor(item.mode) === scheme.id"
          @click="selectScheme(item.mode, scheme.id)"
          @keydown.enter.prevent="selectScheme(item.mode, scheme.id)"
          @keydown.space.prevent="selectScheme(item.mode, scheme.id)"
        >
          <SchemePreview :colors="resolveSchemeColors(scheme)" class="h-16 w-full" />

          <div class="mt-2 flex items-center gap-1.5 px-0.5">
            <span class="truncate text-sm font-medium">{{ scheme.name }}</span>
            <span
              v-if="!scheme.builtin"
              class="shrink-0 rounded bg-muted px-1 py-0.5 text-[10px] leading-none text-muted-foreground"
            >
              自定义
            </span>
            <Check
              v-if="activeIdFor(item.mode) === scheme.id"
              class="ml-auto size-4 shrink-0 text-primary"
            />
          </div>

          <div
            class="pointer-events-none absolute top-1 right-1 flex gap-1 opacity-0 transition-opacity group-focus-within:pointer-events-auto group-focus-within:opacity-100 group-hover:pointer-events-auto group-hover:opacity-100"
          >
            <Button
              v-if="scheme.builtin"
              variant="secondary"
              size="icon-xs"
              title="复制为自定义方案"
              @click.stop="duplicateBuiltin(scheme)"
            >
              <Copy class="size-3" />
            </Button>
            <template v-else>
              <Button
                variant="secondary"
                size="icon-xs"
                title="编辑方案"
                @click.stop="openEdit(scheme)"
              >
                <Pencil class="size-3" />
              </Button>
              <Button
                variant="secondary"
                size="icon-xs"
                title="删除方案"
                @click.stop="requestDelete(scheme)"
              >
                <Trash2 class="size-3" />
              </Button>
            </template>
          </div>
        </div>

        <button
          type="button"
          class="flex min-h-[104px] cursor-pointer flex-col items-center justify-center gap-1.5 rounded-lg border border-dashed border-border text-muted-foreground transition-colors hover:border-primary/50 hover:bg-muted/40 hover:text-foreground"
          @click="openCreate(item.mode)"
        >
          <Plus class="size-5" />
          <span class="text-sm">新建{{ item.mode === "light" ? "亮色" : "暗色" }}方案</span>
        </button>
      </div>
    </section>

    <p
      class="mt-6 rounded-md border border-border/70 bg-muted/40 px-3 py-2 text-xs leading-relaxed text-muted-foreground"
    >
      提示：点击任意方案可实时预览对应亮/暗模式；标题栏切换亮/暗会直接生效并保存。关闭设置会放弃未应用的配色方案修改，点击“应用”后恢复全局亮/暗模式。内置方案可复制为自定义方案后再调节明度与对比度。
    </p>

    <SchemeEditorDialog
      v-model="editorOpen"
      :mode="editorMode"
      :scheme="editingScheme"
      :template="templateScheme"
      @save="handleSave"
    />

    <AlertDialog v-model:open="deleteOpen">
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>删除配色方案</AlertDialogTitle>
          <AlertDialogDescription>
            确定删除「{{
              deleteTarget?.name
            }}」吗？若该方案当前被选中，将回退到默认方案。删除会在点击“应用”后生效。
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel @click="deleteTarget = null">取消</AlertDialogCancel>
          <AlertDialogAction
            class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            @click="confirmDelete"
          >
            删除
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  </div>
</template>

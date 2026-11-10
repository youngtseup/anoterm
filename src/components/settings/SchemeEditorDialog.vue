<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  BUILTIN_DARK_SCHEMES,
  BUILTIN_LIGHT_SCHEMES,
  COLOR_FIELDS,
  normalizeHex,
  resolveSchemeColors,
  type SchemeColors,
  type SchemeTuning,
  type ThemeMode,
  type ThemeScheme,
} from "../../lib/theme";
import { Button } from "../ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "../ui/dialog";
import { Input } from "../ui/input";
import { Label } from "../ui/label";
import SchemePreview from "./SchemePreview.vue";

type SchemePayload = { name: string; colors: SchemeColors; tuning: SchemeTuning };
type ColorTexts = Record<keyof SchemeColors, string>;

const props = defineProps<{
  modelValue: boolean;
  mode: ThemeMode;
  scheme: ThemeScheme | null;
  template: ThemeScheme | null;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  save: [payload: SchemePayload];
}>();

const open = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit("update:modelValue", value),
});

const modeLabel = computed(() => (props.mode === "light" ? "亮色" : "暗色"));
const dialogTitle = computed(() =>
  props.scheme ? "编辑配色方案" : props.template ? "复制配色方案" : "新建配色方案",
);

const name = ref("");
const colors = ref<SchemeColors>({ ...BUILTIN_LIGHT_SCHEMES[0].colors });
const colorTexts = ref<ColorTexts>({} as ColorTexts);
const tuning = ref<SchemeTuning>({ brightness: 0, contrast: 0 });

const effectiveColors = computed(() =>
  resolveSchemeColors({ colors: colors.value, tuning: tuning.value }),
);

function baseColors(): SchemeColors {
  const source =
    props.scheme?.colors ??
    props.template?.colors ??
    (props.mode === "light" ? BUILTIN_LIGHT_SCHEMES[0] : BUILTIN_DARK_SCHEMES[0]).colors;
  return { ...source };
}

function reset() {
  const source = props.scheme ?? props.template;
  name.value = source
    ? `${source.name}${props.scheme ? "" : "（自定义）"}`.slice(0, 32)
    : props.mode === "light"
      ? "我的亮色方案"
      : "我的暗色方案";
  const next = baseColors();
  colors.value = next;
  tuning.value = { ...(source?.tuning ?? { brightness: 0, contrast: 0 }) };
  colorTexts.value = Object.fromEntries(
    COLOR_FIELDS.map((field) => [field.key, next[field.key]]),
  ) as ColorTexts;
}

function resetTuning() {
  tuning.value = { brightness: 0, contrast: 0 };
}

function signed(value: number): string {
  return value > 0 ? `+${value}` : String(value);
}

watch(
  () => props.modelValue,
  (value) => {
    if (value) reset();
  },
);

watch(
  () => props.scheme,
  () => {
    if (props.modelValue) reset();
  },
);

watch(
  () => props.template,
  () => {
    if (props.modelValue) reset();
  },
);

const nameError = computed(() => {
  const value = name.value.trim();
  if (!value) return "请输入方案名称";
  if (value.length > 32) return "名称不能超过 32 个字符";
  return "";
});

const invalidColor = computed(
  () => COLOR_FIELDS.find((field) => !normalizeHex(colorTexts.value[field.key])) ?? null,
);

const canSave = computed(() => !nameError.value && !invalidColor.value);

function setColorFromPicker(key: keyof SchemeColors, event: Event) {
  const value = (event.target as HTMLInputElement).value;
  colors.value[key] = value;
  colorTexts.value[key] = value;
}

function setColorFromText(key: keyof SchemeColors, value: string | number) {
  const text = String(value);
  colorTexts.value[key] = text;
  const normalized = normalizeHex(text);
  if (normalized) colors.value[key] = normalized;
}

function normalizeText(key: keyof SchemeColors) {
  const normalized = normalizeHex(colorTexts.value[key]);
  colorTexts.value[key] = normalized ?? colors.value[key];
}

function save() {
  if (!canSave.value) return;
  emit("save", {
    name: name.value.trim(),
    colors: { ...colors.value },
    tuning: { ...tuning.value },
  });
  open.value = false;
}
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent class="max-h-[85vh] overflow-y-auto sm:max-w-xl">
      <DialogHeader>
        <DialogTitle>{{ dialogTitle }}</DialogTitle>
        <DialogDescription>
          {{ modeLabel }}模式 ·
          {{
            props.template ? "点击“保存方案”后才会加入列表，" : ""
          }}保存后需点击设置页底部的“应用”才会持久化
        </DialogDescription>
      </DialogHeader>

      <div class="space-y-4">
        <div class="space-y-1.5">
          <Label for="scheme-name">方案名称</Label>
          <Input
            id="scheme-name"
            v-model="name"
            maxlength="32"
            placeholder="例如：我的终端配色"
            :aria-invalid="!!nameError"
          />
          <p v-if="nameError" class="text-xs text-destructive">{{ nameError }}</p>
        </div>

        <SchemePreview :colors="effectiveColors" class="h-24 w-full" />

        <div class="space-y-3 rounded-md border border-border/60 p-3">
          <div class="flex items-center justify-between gap-2">
            <span class="text-sm font-medium">明度与对比度</span>
            <Button variant="ghost" size="xs" @click="resetTuning">重置调节</Button>
          </div>

          <div class="space-y-1.5">
            <div class="flex items-center justify-between gap-2">
              <span class="text-xs text-muted-foreground">界面明度</span>
              <span class="font-mono text-xs text-muted-foreground">
                {{ signed(tuning.brightness) }}
              </span>
            </div>
            <input
              v-model.number="tuning.brightness"
              type="range"
              min="-40"
              max="40"
              step="1"
              class="w-full cursor-pointer accent-primary"
              aria-label="界面明度"
            />
            <p class="text-xs text-muted-foreground">调低可减少背景与面板的刺眼感。</p>
          </div>

          <div class="space-y-1.5">
            <div class="flex items-center justify-between gap-2">
              <span class="text-xs text-muted-foreground">文字对比度</span>
              <span class="font-mono text-xs text-muted-foreground">
                {{ signed(tuning.contrast) }}
              </span>
            </div>
            <input
              v-model.number="tuning.contrast"
              type="range"
              min="-50"
              max="50"
              step="1"
              class="w-full cursor-pointer accent-primary"
              aria-label="文字对比度"
            />
            <p class="text-xs text-muted-foreground">调低可让文字更柔和，最低保持可读对比度。</p>
          </div>
        </div>

        <div class="space-y-2">
          <div class="text-sm font-medium">基础颜色</div>
          <div
            v-for="field in COLOR_FIELDS"
            :key="field.key"
            class="flex items-center gap-3 rounded-md border border-border/60 px-3 py-2"
          >
            <input
              type="color"
              class="size-8 shrink-0 cursor-pointer rounded-md border border-border bg-transparent p-0"
              :value="colors[field.key]"
              :aria-label="field.label"
              @input="setColorFromPicker(field.key, $event)"
            />
            <div class="min-w-0 flex-1">
              <div class="text-sm font-medium">{{ field.label }}</div>
              <div class="text-xs text-muted-foreground">{{ field.description }}</div>
            </div>
            <Input
              class="h-8 w-28 shrink-0 font-mono text-xs"
              :model-value="colorTexts[field.key]"
              :aria-invalid="!normalizeHex(colorTexts[field.key])"
              @update:model-value="setColorFromText(field.key, $event)"
              @blur="normalizeText(field.key)"
            />
          </div>
        </div>

        <p v-if="invalidColor" class="text-xs text-destructive">
          “{{ invalidColor.label }}”不是有效的十六进制颜色，请使用 #RRGGBB 格式。
        </p>
      </div>

      <DialogFooter>
        <Button variant="outline" @click="open = false">取消</Button>
        <Button :disabled="!canSave" @click="save">保存方案</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>

/**
 * 全局主题状态（单例）。
 *
 * `.dark` 类挂在 <html> 上，同时驱动 shadcn 变量与终端配色，
 * 因此需要模块级 ref 供任意组件读取，而不是只在 App 内部持有。
 */
import { computed, ref, watch, type Ref } from "vue";
import type { ITheme } from "@xterm/xterm";
import { getSetting, setSetting } from "../api/settings";
import {
  cloneAppearance,
  defaultAppearance,
  findScheme,
  resolveSchemeTokens,
  sanitizeAppearance,
  type AppearanceState,
  type SchemeColors,
  type SchemeTuning,
  type ThemeMode,
  type ThemeScheme,
} from "../lib/theme";

const THEME_STORAGE_KEY = "anoterm:theme";
const APPEARANCE_CACHE_KEY = "anoterm.appearance.cache";
const APPEARANCE_SETTING_KEY = "appearance";

/** 读取持久化的主题偏好；存储不可用或值非法时返回 null */
function readStoredTheme(): boolean | null {
  try {
    const stored = localStorage.getItem(THEME_STORAGE_KEY);
    if (stored === "dark") return true;
    if (stored === "light") return false;
  } catch {
    /* ignore */
  }
  return null;
}

/** 将当前主题写入 localStorage */
function persistTheme(dark: boolean) {
  try {
    localStorage.setItem(THEME_STORAGE_KEY, dark ? "dark" : "light");
  } catch {
    /* ignore */
  }
}

function readAppearanceCache(): AppearanceState {
  try {
    const raw = localStorage.getItem(APPEARANCE_CACHE_KEY);
    if (!raw) return defaultAppearance();
    return sanitizeAppearance(JSON.parse(raw));
  } catch {
    return defaultAppearance();
  }
}

function writeAppearanceCache(state: AppearanceState) {
  try {
    localStorage.setItem(APPEARANCE_CACHE_KEY, JSON.stringify(state));
  } catch {
    /* ignore */
  }
}

const isDark: Ref<boolean> = ref(readStoredTheme() ?? false);
const savedAppearance = ref<AppearanceState>(readAppearanceCache());
const draftAppearance = ref<AppearanceState | null>(null);
const previewMode = ref<ThemeMode | null>(null);
const themeRevision = ref(0);

const appearanceState = computed(() => draftAppearance.value ?? savedAppearance.value);
const effectiveMode = computed<ThemeMode>(
  () => previewMode.value ?? (isDark.value ? "dark" : "light"),
);
const isEffectiveDark = computed(() => effectiveMode.value === "dark");
const currentScheme = computed(() => findScheme(appearanceState.value, effectiveMode.value));
const appearanceDirty = computed(
  () =>
    draftAppearance.value !== null &&
    JSON.stringify(draftAppearance.value) !== JSON.stringify(savedAppearance.value),
);

function applyTheme() {
  const root = document.documentElement;
  const scheme = currentScheme.value;
  for (const [name, value] of Object.entries(resolveSchemeTokens(scheme))) {
    root.style.setProperty(`--${name}`, value);
  }
  root.classList.toggle("dark", isEffectiveDark.value);
  root.style.colorScheme = isEffectiveDark.value ? "dark" : "light";
  themeRevision.value += 1;
  requestAnimationFrame(syncWindowBackground);
}

watch(
  [effectiveMode, currentScheme],
  () => {
    applyTheme();
  },
  { immediate: true, deep: true },
);

watch(isDark, persistTheme, { immediate: true });

/**
 * 原生窗口与 WebView 的底色跟随主题：快速拖动窗口缩放时，
 * WebView 尚未重绘的区域会露出自己的底色，若保持默认白底就会闪烁。
 */
function syncWindowBackground() {
  if (!("__TAURI_INTERNALS__" in window)) return;
  import("@tauri-apps/api/webviewWindow")
    .then(({ getCurrentWebviewWindow }) => {
      const [r, g, b] = readThemeRgb("--background");
      return getCurrentWebviewWindow().setBackgroundColor([r, g, b]);
    })
    .catch(() => {});
}

export async function initAppearance(): Promise<void> {
  try {
    const raw = await getSetting(APPEARANCE_SETTING_KEY);
    if (!raw) return;
    const parsed = sanitizeAppearance(JSON.parse(raw));
    savedAppearance.value = parsed;
    writeAppearanceCache(parsed);
  } catch {
    /* ignore */
  }
}

export function beginAppearanceDraft() {
  previewMode.value = null;
  if (draftAppearance.value) return;
  draftAppearance.value = cloneAppearance(savedAppearance.value);
}

export function discardAppearanceDraft() {
  draftAppearance.value = null;
  previewMode.value = null;
}

export async function commitAppearanceDraft(): Promise<void> {
  const next = draftAppearance.value;
  if (!next) {
    previewMode.value = null;
    return;
  }
  await setSetting(APPEARANCE_SETTING_KEY, JSON.stringify(next));
  const committed = cloneAppearance(next);
  savedAppearance.value = committed;
  writeAppearanceCache(committed);
  draftAppearance.value = null;
  previewMode.value = null;
}

function updateDraft(mutate: (state: AppearanceState) => void) {
  const next = cloneAppearance(appearanceState.value);
  mutate(next);
  draftAppearance.value = next;
}

function selectScheme(mode: ThemeMode, id: string) {
  updateDraft((state) => {
    if (mode === "light") state.activeLightId = id;
    else state.activeDarkId = id;
  });
  previewMode.value = mode;
}

function createSchemeId(): string {
  const uuid =
    globalThis.crypto?.randomUUID?.() ?? `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  return `custom-${uuid}`;
}

function addCustomScheme(input: {
  name: string;
  mode: ThemeMode;
  colors: SchemeColors;
  tuning?: SchemeTuning;
}): ThemeScheme {
  const scheme: ThemeScheme = {
    id: createSchemeId(),
    name: input.name.trim().slice(0, 32),
    mode: input.mode,
    colors: { ...input.colors },
    ...(input.tuning ? { tuning: { ...input.tuning } } : {}),
  };
  updateDraft((state) => {
    state.customSchemes.push(scheme);
    if (scheme.mode === "light") state.activeLightId = scheme.id;
    else state.activeDarkId = scheme.id;
  });
  previewMode.value = scheme.mode;
  return scheme;
}

function updateCustomScheme(
  id: string,
  patch: { name?: string; colors?: SchemeColors; tuning?: SchemeTuning },
) {
  updateDraft((state) => {
    const index = state.customSchemes.findIndex((scheme) => scheme.id === id);
    if (index < 0) return;
    const existing = state.customSchemes[index];
    const tuning = patch.tuning
      ? { brightness: patch.tuning.brightness, contrast: patch.tuning.contrast }
      : existing.tuning;
    const normalizedTuning =
      tuning && (tuning.brightness !== 0 || tuning.contrast !== 0) ? tuning : undefined;
    state.customSchemes[index] = {
      ...existing,
      name: patch.name?.trim().slice(0, 32) || existing.name,
      colors: patch.colors ? { ...patch.colors } : existing.colors,
      ...(normalizedTuning ? { tuning: normalizedTuning } : {}),
    };
    if (!normalizedTuning) delete state.customSchemes[index].tuning;
  });
}

function deleteCustomScheme(id: string) {
  updateDraft((state) => {
    state.customSchemes = state.customSchemes.filter((scheme) => scheme.id !== id);
    const light = findScheme(state, "light");
    const dark = findScheme(state, "dark");
    state.activeLightId = light.id;
    state.activeDarkId = dark.id;
  });
}

export function useTheme() {
  function toggleTheme() {
    previewMode.value = null;
    isDark.value = !isDark.value;
  }

  function saveTheme() {
    persistTheme(isDark.value);
  }

  return {
    isDark,
    isEffectiveDark,
    effectiveMode,
    toggleTheme,
    saveTheme,
    themeRevision,
    savedAppearance,
    draftAppearance,
    appearanceState,
    appearanceDirty,
    currentScheme,
    initAppearance,
    beginAppearanceDraft,
    discardAppearanceDraft,
    commitAppearanceDraft,
    selectScheme,
    addCustomScheme,
    updateCustomScheme,
    deleteCustomScheme,
  };
}

/**
 * 读取主题 token 的 rgb 分量。
 *
 * xterm 的解析器只原生认识 hex 与 rgb()，其余格式靠 canvas 回退且拒绝半透明色，
 * 故这里统一把 oklch 变量光栅化为不透明 rgb，再交给终端。
 */
export function readThemeRgb(name: string): [number, number, number] {
  const probe = document.createElement("span");
  probe.style.display = "none";
  probe.style.color = `var(${name})`;
  document.body.appendChild(probe);
  const css = getComputedStyle(probe).color;
  probe.remove();

  const rgb = css.match(/rgba?\((\d+(?:\.\d+)?)[\s,]+(\d+(?:\.\d+)?)[\s,]+(\d+(?:\.\d+)?)/);
  if (rgb) return [Number(rgb[1]), Number(rgb[2]), Number(rgb[3])];

  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = 1;
  const ctx = canvas.getContext("2d", { willReadFrequently: true })!;
  ctx.fillStyle = css;
  ctx.fillRect(0, 0, 1, 1);
  const [r, g, b] = ctx.getImageData(0, 0, 1, 1).data;
  return [r, g, b];
}

type Rgb = [number, number, number];

function rgbLuminance([r, g, b]: Rgb): number {
  const channel = (value: number) => {
    const v = value / 255;
    return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
}

function rgbContrast(a: Rgb, b: Rgb): number {
  const la = rgbLuminance(a);
  const lb = rgbLuminance(b);
  return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05);
}

/**
 * 由当前主题变量生成 xterm 配色：亮色白底黑字，暗色黑底白字。
 *
 * 返回的每次都是新对象——xterm 对 `options.theme` 做引用比较，原地改属性不生效。
 */
export function buildTerminalTheme(): ITheme {
  const css = ([r, g, b]: Rgb) => `rgb(${r}, ${g}, ${b})`;
  const bg = readThemeRgb("--background");
  const fg = readThemeRgb("--foreground");
  const accent = readThemeRgb("--primary");
  const cursor = rgbContrast(accent, bg) >= 3 ? accent : fg;
  const selection = rgbContrast(accent, bg) >= 3 ? accent : fg;
  const theme: ITheme = {
    background: css(bg),
    foreground: css(fg),
    cursor: css(cursor),
    cursorAccent: css(bg),
    selectionBackground: `rgba(${selection[0]}, ${selection[1]}, ${selection[2]}, 0.28)`,
  };
  if (!isEffectiveDark.value) {
    // 默认 ANSI 调色板的 white/brightWhite 是浅灰，铺在白底上会看不见文字
    theme.white = css(fg);
    theme.brightWhite = css(fg);
  }
  return theme;
}

/**
 * 文字与背景的最小对比度（WCAG AA）。
 * 换肤后原生命色容易撞色——黄字压白底、深灰字压黑底都近乎不可见，
 * 交给 xterm 动态抬前景色，而不是逐个改写调色板。
 */
export const TERMINAL_MIN_CONTRAST = 4.5;

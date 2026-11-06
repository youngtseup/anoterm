/**
 * 全局主题状态（单例）。
 *
 * `.dark` 类挂在 <html> 上，同时驱动 shadcn 变量与终端配色，
 * 因此需要模块级 ref 供任意组件读取，而不是只在 App 内部持有。
 */
import { ref, watch, type Ref } from "vue";
import type { ITheme } from "@xterm/xterm";

const THEME_STORAGE_KEY = "anoterm:theme";

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

const isDark: Ref<boolean> = ref(readStoredTheme() ?? false);

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

// 主题切换：shadcn 变量作用于 html 的 .dark 类，同时持久化偏好
watch(
  isDark,
  (v) => {
    document.documentElement.classList.toggle("dark", v);
    persistTheme(v);
    // 等样式（CSS 变量）生效后再读取背景色
    requestAnimationFrame(syncWindowBackground);
  },
  { immediate: true },
);

export function useTheme() {
  function toggleTheme() {
    isDark.value = !isDark.value;
  }
  /** 显式保存当前主题（设置弹窗「应用」使用） */
  function saveTheme() {
    persistTheme(isDark.value);
  }
  return { isDark, toggleTheme, saveTheme };
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

  // 现代色彩函数（oklch 等）浏览器会原样返回，需自行光栅化
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = 1;
  const ctx = canvas.getContext("2d", { willReadFrequently: true })!;
  ctx.fillStyle = css;
  ctx.fillRect(0, 0, 1, 1);
  const [r, g, b] = ctx.getImageData(0, 0, 1, 1).data;
  return [r, g, b];
}

type Rgb = [number, number, number];

/**
 * 由当前主题变量生成 xterm 配色：亮色白底黑字，暗色黑底白字。
 *
 * 返回的每次都是新对象——xterm 对 `options.theme` 做引用比较，原地改属性不生效。
 */
export function buildTerminalTheme(): ITheme {
  const css = ([r, g, b]: Rgb) => `rgb(${r}, ${g}, ${b})`;
  const bg = readThemeRgb("--background");
  const fg = readThemeRgb("--foreground");
  const theme: ITheme = {
    background: css(bg),
    foreground: css(fg),
    cursor: css(fg),
    cursorAccent: css(bg),
    selectionBackground: `rgba(${fg[0]}, ${fg[1]}, ${fg[2]}, 0.25)`,
  };
  if (!isDark.value) {
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

export type ThemeMode = "light" | "dark";

export interface SchemeColors {
  background: string;
  foreground: string;
  surface: string;
  muted: string;
  border: string;
  primary: string;
  destructive: string;
}

export interface SchemeTuning {
  brightness: number;
  contrast: number;
}

export interface ThemeScheme {
  id: string;
  name: string;
  mode: ThemeMode;
  colors: SchemeColors;
  builtin?: boolean;
  tokens?: Record<string, string>;
  tuning?: SchemeTuning;
}

export interface AppearanceState {
  activeLightId: string;
  activeDarkId: string;
  customSchemes: ThemeScheme[];
}

export const DEFAULT_LIGHT_SCHEME_ID = "builtin-light-default";
export const DEFAULT_DARK_SCHEME_ID = "builtin-dark-default";

export const COLOR_FIELDS: {
  key: keyof SchemeColors;
  label: string;
  description: string;
}[] = [
  { key: "background", label: "背景", description: "页面与终端的底色" },
  { key: "foreground", label: "前景", description: "正文与终端文字颜色" },
  { key: "surface", label: "面板", description: "卡片、弹层与侧栏背景" },
  { key: "muted", label: "次要背景", description: "悬浮、选中与分区底色" },
  { key: "border", label: "边框", description: "分隔线与控件描边" },
  { key: "primary", label: "主色", description: "选中态、进度与终端光标" },
  { key: "destructive", label: "危险色", description: "错误提示与删除操作" },
];

const DEFAULT_LIGHT_TOKENS: Record<string, string> = {
  background: "oklch(1 0 0)",
  foreground: "oklch(0.145 0 0)",
  card: "oklch(1 0 0)",
  "card-foreground": "oklch(0.145 0 0)",
  popover: "oklch(1 0 0)",
  "popover-foreground": "oklch(0.145 0 0)",
  primary: "oklch(0.205 0 0)",
  "primary-foreground": "oklch(0.985 0 0)",
  secondary: "oklch(0.97 0 0)",
  "secondary-foreground": "oklch(0.205 0 0)",
  muted: "oklch(0.97 0 0)",
  "muted-foreground": "oklch(0.556 0 0)",
  accent: "oklch(0.97 0 0)",
  "accent-foreground": "oklch(0.205 0 0)",
  destructive: "oklch(0.577 0.245 27.325)",
  "destructive-foreground": "oklch(0.985 0 0)",
  border: "oklch(0.922 0 0)",
  input: "oklch(0.922 0 0)",
  ring: "oklch(0.708 0 0)",
  "chart-1": "oklch(0.646 0.222 41.116)",
  "chart-2": "oklch(0.6 0.118 184.704)",
  "chart-3": "oklch(0.398 0.07 227.392)",
  "chart-4": "oklch(0.828 0.189 84.429)",
  "chart-5": "oklch(0.769 0.188 70.08)",
};

const DEFAULT_DARK_TOKENS: Record<string, string> = {
  background: "oklch(0.145 0 0)",
  foreground: "oklch(0.985 0 0)",
  card: "oklch(0.205 0 0)",
  "card-foreground": "oklch(0.985 0 0)",
  popover: "oklch(0.205 0 0)",
  "popover-foreground": "oklch(0.985 0 0)",
  primary: "oklch(0.922 0 0)",
  "primary-foreground": "oklch(0.205 0 0)",
  secondary: "oklch(0.269 0 0)",
  "secondary-foreground": "oklch(0.985 0 0)",
  muted: "oklch(0.269 0 0)",
  "muted-foreground": "oklch(0.708 0 0)",
  accent: "oklch(0.269 0 0)",
  "accent-foreground": "oklch(0.985 0 0)",
  destructive: "oklch(0.704 0.191 22.216)",
  "destructive-foreground": "oklch(0.985 0 0)",
  border: "oklch(1 0 0 / 10%)",
  input: "oklch(1 0 0 / 15%)",
  ring: "oklch(0.556 0 0)",
  "chart-1": "oklch(0.488 0.243 264.376)",
  "chart-2": "oklch(0.696 0.17 162.48)",
  "chart-3": "oklch(0.769 0.188 70.08)",
  "chart-4": "oklch(0.627 0.265 303.9)",
  "chart-5": "oklch(0.645 0.246 16.439)",
};

export const BUILTIN_LIGHT_SCHEMES: ThemeScheme[] = [
  {
    id: DEFAULT_LIGHT_SCHEME_ID,
    name: "明亮",
    mode: "light",
    builtin: true,
    colors: {
      background: "#ffffff",
      foreground: "#0a0a0a",
      surface: "#ffffff",
      muted: "#f5f5f5",
      border: "#e5e5e5",
      primary: "#171717",
      destructive: "#e7000b",
    },
    tokens: DEFAULT_LIGHT_TOKENS,
  },
  {
    id: "builtin-light-soft",
    name: "柔和亮色",
    mode: "light",
    builtin: true,
    colors: {
      background: "#f6f6f2",
      foreground: "#2b2b2b",
      surface: "#fbfbf8",
      muted: "#ecece6",
      border: "#d9d9d2",
      primary: "#3a3a3a",
      destructive: "#b91c1c",
    },
  },
  {
    id: "builtin-light-ocean",
    name: "海盐蓝",
    mode: "light",
    builtin: true,
    colors: {
      background: "#f4f8fc",
      foreground: "#16233a",
      surface: "#ffffff",
      muted: "#e6eef8",
      border: "#cfdcec",
      primary: "#2563eb",
      destructive: "#dc2626",
    },
  },
  {
    id: "builtin-light-solar",
    name: "暖阳",
    mode: "light",
    builtin: true,
    colors: {
      background: "#fdf6e3",
      foreground: "#3b3228",
      surface: "#fffdf5",
      muted: "#f3e9d2",
      border: "#e2d4b7",
      primary: "#b45309",
      destructive: "#dc2626",
    },
  },
  {
    id: "builtin-light-mint",
    name: "薄荷",
    mode: "light",
    builtin: true,
    colors: {
      background: "#f3faf6",
      foreground: "#132a22",
      surface: "#ffffff",
      muted: "#e1f2e8",
      border: "#c9e2d5",
      primary: "#0f766e",
      destructive: "#dc2626",
    },
  },
  {
    id: "builtin-light-sakura",
    name: "樱花",
    mode: "light",
    builtin: true,
    colors: {
      background: "#fbf4f7",
      foreground: "#33202a",
      surface: "#fffafc",
      muted: "#f5e5ec",
      border: "#ead0dc",
      primary: "#be185d",
      destructive: "#dc2626",
    },
  },
];

export const BUILTIN_DARK_SCHEMES: ThemeScheme[] = [
  {
    id: DEFAULT_DARK_SCHEME_ID,
    name: "深色",
    mode: "dark",
    builtin: true,
    colors: {
      background: "#171717",
      foreground: "#fafafa",
      surface: "#262626",
      muted: "#404040",
      border: "#333333",
      primary: "#e5e5e5",
      destructive: "#ff6467",
    },
    tokens: DEFAULT_DARK_TOKENS,
  },
  {
    id: "builtin-dark-soft",
    name: "柔和暗色",
    mode: "dark",
    builtin: true,
    colors: {
      background: "#141414",
      foreground: "#d4d4d4",
      surface: "#1e1e1e",
      muted: "#2a2a2a",
      border: "#363636",
      primary: "#c9c9c9",
      destructive: "#e57373",
    },
  },
  {
    id: "builtin-dark-midnight",
    name: "午夜",
    mode: "dark",
    builtin: true,
    colors: {
      background: "#0b1220",
      foreground: "#e6edf7",
      surface: "#131e33",
      muted: "#1b2942",
      border: "#2a3b58",
      primary: "#60a5fa",
      destructive: "#f87171",
    },
  },
  {
    id: "builtin-dark-forest",
    name: "森林",
    mode: "dark",
    builtin: true,
    colors: {
      background: "#0d1712",
      foreground: "#e3f2e9",
      surface: "#15241b",
      muted: "#1d3227",
      border: "#2c4a39",
      primary: "#4ade80",
      destructive: "#f87171",
    },
  },
  {
    id: "builtin-dark-violet",
    name: "幽紫",
    mode: "dark",
    builtin: true,
    colors: {
      background: "#120f1f",
      foreground: "#ece8f8",
      surface: "#1c1730",
      muted: "#272044",
      border: "#392d5c",
      primary: "#a78bfa",
      destructive: "#fb7185",
    },
  },
  {
    id: "builtin-dark-coffee",
    name: "暖褐",
    mode: "dark",
    builtin: true,
    colors: {
      background: "#191310",
      foreground: "#f2e8df",
      surface: "#241b16",
      muted: "#30241d",
      border: "#45352b",
      primary: "#e8b37a",
      destructive: "#ef6a5a",
    },
  },
];

const BUILTIN_SCHEMES = [...BUILTIN_LIGHT_SCHEMES, ...BUILTIN_DARK_SCHEMES];

export function defaultAppearance(): AppearanceState {
  return {
    activeLightId: DEFAULT_LIGHT_SCHEME_ID,
    activeDarkId: DEFAULT_DARK_SCHEME_ID,
    customSchemes: [],
  };
}

export function schemeList(mode: ThemeMode, customSchemes: ThemeScheme[]): ThemeScheme[] {
  const builtins = mode === "light" ? BUILTIN_LIGHT_SCHEMES : BUILTIN_DARK_SCHEMES;
  return [...builtins, ...customSchemes.filter((scheme) => scheme.mode === mode)];
}

export function findScheme(
  state: AppearanceState,
  mode: ThemeMode,
  customSchemes: ThemeScheme[] = state.customSchemes,
): ThemeScheme {
  const list = schemeList(mode, customSchemes);
  const id = mode === "light" ? state.activeLightId : state.activeDarkId;
  return list.find((scheme) => scheme.id === id) ?? list[0];
}

export function isBuiltinScheme(id: string): boolean {
  return BUILTIN_SCHEMES.some((scheme) => scheme.id === id);
}

export function normalizeHex(value: string): string | null {
  const input = value.trim().toLowerCase();
  const short = input.match(/^#([0-9a-f])([0-9a-f])([0-9a-f])$/);
  if (short) return `#${short[1]}${short[1]}${short[2]}${short[2]}${short[3]}${short[3]}`;
  const full = input.match(/^#[0-9a-f]{6}$/);
  return full ? input : null;
}

function hexToRgb(hex: string): [number, number, number] {
  const value = normalizeHex(hex) ?? "#000000";
  return [
    Number.parseInt(value.slice(1, 3), 16),
    Number.parseInt(value.slice(3, 5), 16),
    Number.parseInt(value.slice(5, 7), 16),
  ];
}

function rgbToHex(rgb: [number, number, number]): string {
  return `#${rgb.map((v) => Math.round(v).toString(16).padStart(2, "0")).join("")}`;
}

export function mixColors(a: string, b: string, amount: number): string {
  const from = hexToRgb(a);
  const to = hexToRgb(b);
  return rgbToHex([
    from[0] + (to[0] - from[0]) * amount,
    from[1] + (to[1] - from[1]) * amount,
    from[2] + (to[2] - from[2]) * amount,
  ]);
}

function relativeLuminance(hex: string): number {
  const [r, g, b] = hexToRgb(hex).map((v) => {
    const channel = v / 255;
    return channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

export function contrastRatio(a: string, b: string): number {
  const la = relativeLuminance(a);
  const lb = relativeLuminance(b);
  const light = Math.max(la, lb);
  const dark = Math.min(la, lb);
  return (light + 0.05) / (dark + 0.05);
}

function readableForeground(background: string): string {
  const black = "#000000";
  const white = "#ffffff";
  return contrastRatio(black, background) >= contrastRatio(white, background) ? black : white;
}

function ensureContrast(color: string, background: string, minimum: number): string {
  if (contrastRatio(color, background) >= minimum) return color;
  const target = relativeLuminance(background) > 0.5 ? "#000000" : "#ffffff";
  let current = color;
  for (let step = 1; step <= 10; step++) {
    current = mixColors(color, target, step / 10);
    if (contrastRatio(current, background) >= minimum) return current;
  }
  return current;
}

interface Oklch {
  l: number;
  c: number;
  h: number;
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(maximum, Math.max(minimum, value));
}

function srgbToLinear(value: number): number {
  const v = value / 255;
  return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
}

function linearToSrgb(value: number): number {
  const v = clamp(value, 0, 1);
  return 255 * (v <= 0.0031308 ? 12.92 * v : 1.055 * v ** (1 / 2.4) - 0.055);
}

function rgbToOklch(hex: string): Oklch {
  const [r, g, b] = hexToRgb(hex).map(srgbToLinear);
  const l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
  const m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
  const s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;
  const lRoot = Math.cbrt(l);
  const mRoot = Math.cbrt(m);
  const sRoot = Math.cbrt(s);
  const lightness = 0.2104542553 * lRoot + 0.793617785 * mRoot - 0.0040720468 * sRoot;
  const a = 1.9779984951 * lRoot - 2.428592205 * mRoot + 0.4505937099 * sRoot;
  const bAxis = 0.0259040371 * lRoot + 0.7827717662 * mRoot - 0.808675766 * sRoot;
  let hue = (Math.atan2(bAxis, a) * 180) / Math.PI;
  if (hue < 0) hue += 360;
  return { l: lightness, c: Math.hypot(a, bAxis), h: hue };
}

function oklchToHex({ l, c, h }: Oklch): string {
  const radians = (h * Math.PI) / 180;
  const a = c * Math.cos(radians);
  const b = c * Math.sin(radians);
  const lRoot = l + 0.3963377774 * a + 0.2158037573 * b;
  const mRoot = l - 0.1055613458 * a - 0.0638541728 * b;
  const sRoot = l - 0.0894841775 * a - 1.291485548 * b;
  const l3 = lRoot ** 3;
  const m3 = mRoot ** 3;
  const s3 = sRoot ** 3;
  const r = 4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3;
  const g = -1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3;
  const b3 = -0.0041960863 * l3 - 0.7034186147 * m3 + 1.707614701 * s3;
  return rgbToHex([linearToSrgb(r), linearToSrgb(g), linearToSrgb(b3)]);
}

function shiftLightness(hex: string, delta: number): string {
  const color = rgbToOklch(hex);
  return oklchToHex({ ...color, l: clamp(color.l + delta, 0, 1) });
}

function adjustForegroundContrast(foreground: string, background: string, target: number): string {
  const foregroundOklch = rgbToOklch(foreground);
  const backgroundOklch = rgbToOklch(background);
  const darkSide = foregroundOklch.l <= backgroundOklch.l;
  let low = darkSide ? 0 : backgroundOklch.l;
  let high = darkSide ? backgroundOklch.l : 1;
  let best = foreground;
  let bestDifference = Math.abs(contrastRatio(foreground, background) - target);
  for (let step = 0; step < 28; step++) {
    const mid = (low + high) / 2;
    const candidate = oklchToHex({ l: mid, c: foregroundOklch.c, h: foregroundOklch.h });
    const ratio = contrastRatio(candidate, background);
    const difference = Math.abs(ratio - target);
    if (difference < bestDifference) {
      best = candidate;
      bestDifference = difference;
    }
    if (darkSide) {
      if (ratio > target) low = mid;
      else high = mid;
    } else if (ratio > target) {
      high = mid;
    } else {
      low = mid;
    }
  }
  return best;
}

export function resolveSchemeColors(scheme: Pick<ThemeScheme, "colors" | "tuning">): SchemeColors {
  const { colors, tuning } = scheme;
  if (!tuning) return colors;
  const brightness = clamp(tuning.brightness, -40, 40);
  const contrast = clamp(tuning.contrast, -50, 50);
  if (brightness === 0 && contrast === 0) return colors;

  const delta = (brightness / 40) * 0.06;
  const background = shiftLightness(colors.background, delta);
  const surface = shiftLightness(colors.surface, delta);
  const muted = shiftLightness(colors.muted, delta);
  const border = shiftLightness(colors.border, delta);
  let foreground = colors.foreground;
  if (contrast !== 0) {
    const base = contrastRatio(foreground, background);
    const target = Math.min(21, Math.max(Math.min(base, 4.5), base * (1 + contrast / 100)));
    foreground = adjustForegroundContrast(foreground, background, target);
  }
  return { ...colors, background, surface, muted, border, foreground };
}

function deriveTokens(scheme: ThemeScheme, colors: SchemeColors): Record<string, string> {
  const { background, foreground, surface, muted, border, primary, destructive } = colors;
  const secondaryText = ensureContrast(
    mixColors(foreground, background, scheme.mode === "light" ? 0.45 : 0.4),
    background,
    4.5,
  );
  const chartBase = mixColors(primary, surface, 0.15);
  return {
    background,
    foreground,
    card: surface,
    "card-foreground": foreground,
    popover: surface,
    "popover-foreground": foreground,
    primary,
    "primary-foreground": readableForeground(primary),
    secondary: muted,
    "secondary-foreground": foreground,
    muted,
    "muted-foreground": secondaryText,
    accent: muted,
    "accent-foreground": foreground,
    destructive,
    "destructive-foreground": readableForeground(destructive),
    border,
    input: border,
    ring: primary,
    "chart-1": primary,
    "chart-2": chartBase,
    "chart-3": mixColors(primary, "#3b82f6", 0.5),
    "chart-4": mixColors(destructive, "#f59e0b", 0.45),
    "chart-5": mixColors(destructive, "#8b5cf6", 0.45),
  };
}

export function resolveSchemeTokens(scheme: ThemeScheme): Record<string, string> {
  const colors = resolveSchemeColors(scheme);
  return { ...deriveTokens(scheme, colors), ...(scheme.tokens ?? {}) };
}

function sanitizeColors(raw: unknown): SchemeColors | null {
  if (!raw || typeof raw !== "object") return null;
  const input = raw as Record<string, unknown>;
  const colors = {} as SchemeColors;
  for (const field of COLOR_FIELDS) {
    const value = input[field.key];
    if (typeof value !== "string") return null;
    const normalized = normalizeHex(value);
    if (!normalized) return null;
    colors[field.key] = normalized;
  }
  return colors;
}

function sanitizeTuning(raw: unknown): SchemeTuning | undefined {
  if (!raw || typeof raw !== "object") return undefined;
  const input = raw as Record<string, unknown>;
  const brightness =
    typeof input.brightness === "number" && Number.isFinite(input.brightness)
      ? Math.round(clamp(input.brightness, -40, 40))
      : 0;
  const contrast =
    typeof input.contrast === "number" && Number.isFinite(input.contrast)
      ? Math.round(clamp(input.contrast, -50, 50))
      : 0;
  if (brightness === 0 && contrast === 0) return undefined;
  return { brightness, contrast };
}

function sanitizeCustomScheme(raw: unknown): ThemeScheme | null {
  if (!raw || typeof raw !== "object") return null;
  const input = raw as Record<string, unknown>;
  if (typeof input.id !== "string" || !input.id || input.id.length > 64) return null;
  if (isBuiltinScheme(input.id)) return null;
  if (typeof input.name !== "string") return null;
  const name = input.name.trim().slice(0, 32);
  if (!name) return null;
  if (input.mode !== "light" && input.mode !== "dark") return null;
  const colors = sanitizeColors(input.colors);
  if (!colors) return null;
  const tuning = sanitizeTuning(input.tuning);
  return {
    id: input.id,
    name,
    mode: input.mode,
    colors,
    ...(tuning ? { tuning } : {}),
  };
}

export function sanitizeAppearance(raw: unknown): AppearanceState {
  const fallback = defaultAppearance();
  if (!raw || typeof raw !== "object") return fallback;
  const input = raw as Record<string, unknown>;
  const customSchemes: ThemeScheme[] = [];
  const ids = new Set<string>();
  if (Array.isArray(input.customSchemes)) {
    for (const item of input.customSchemes.slice(0, 100)) {
      const scheme = sanitizeCustomScheme(item);
      if (!scheme || ids.has(scheme.id)) continue;
      ids.add(scheme.id);
      customSchemes.push(scheme);
    }
  }
  const state: AppearanceState = {
    activeLightId: typeof input.activeLightId === "string" ? input.activeLightId : "",
    activeDarkId: typeof input.activeDarkId === "string" ? input.activeDarkId : "",
    customSchemes,
  };
  const lightExists = schemeList("light", customSchemes).some(
    (scheme) => scheme.id === state.activeLightId,
  );
  const darkExists = schemeList("dark", customSchemes).some(
    (scheme) => scheme.id === state.activeDarkId,
  );
  if (!lightExists) state.activeLightId = DEFAULT_LIGHT_SCHEME_ID;
  if (!darkExists) state.activeDarkId = DEFAULT_DARK_SCHEME_ID;
  return state;
}

export function cloneAppearance(state: AppearanceState): AppearanceState {
  return {
    activeLightId: state.activeLightId,
    activeDarkId: state.activeDarkId,
    customSchemes: state.customSchemes.map((scheme) => ({
      ...scheme,
      colors: { ...scheme.colors },
      ...(scheme.tuning ? { tuning: { ...scheme.tuning } } : {}),
    })),
  };
}

<template>
  <div>
    <h2 class="text-base font-semibold">关于我们</h2>

    <div class="mt-5 space-y-5">
      <div class="flex items-center gap-4">
        <img
          :src="appIcon"
          alt="Anoterm"
          class="size-14 shrink-0 rounded-xl border border-border/60 object-cover shadow-sm"
        />
        <div class="min-w-0">
          <div class="flex items-center gap-2">
            <span class="text-xl font-semibold">{{ info.name }}</span>
            <span
              class="rounded-full border border-border px-2 py-0.5 text-xs text-muted-foreground"
            ></span>
          </div>
        </div>
        <Button variant="outline" size="sm" class="ml-auto shrink-0" @click="copyAllInfo">
          <Copy class="size-3.5" />
          复制全部信息
        </Button>
      </div>

      <div class="overflow-hidden rounded-lg border border-border/60">
        <div
          v-for="(row, index) in infoRows"
          :key="row.label"
          class="flex items-start gap-4 px-4 py-3 text-sm"
          :class="index !== infoRows.length - 1 ? 'border-b border-border/60' : ''"
        >
          <span class="w-24 shrink-0 text-muted-foreground">{{ row.label }}</span>
          <span class="min-w-0 font-mono text-[13px] break-all">{{ row.value }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/** 关于我们：展示应用/平台/系统信息，并支持整体复制 */
import { computed, onMounted, ref } from "vue";
import { toast } from "vue-sonner";
import { Copy } from "@lucide/vue";
import { getName, getTauriVersion, getVersion } from "@tauri-apps/api/app";
import { arch, platform, version as osVersion } from "@tauri-apps/plugin-os";
import { Button } from "../ui/button";
import appIcon from "../../assets/app-icon.png";

interface AppInfo {
  name: string;
  version: string;
  tauriVersion: string;
  platform: string;
  osVersion: string;
  arch: string;
  webview: string;
}

const PLATFORM_NAMES: Record<string, string> = {
  windows: "Windows",
  macos: "macOS",
  linux: "Linux",
  ios: "iOS",
  android: "Android",
  freebsd: "FreeBSD",
  dragonfly: "DragonFly BSD",
  netbsd: "NetBSD",
  openbsd: "OpenBSD",
  solaris: "Solaris",
  illumos: "illumos",
};

const info = ref<AppInfo>({
  name: "Anoterm",
  version: "未知",
  tauriVersion: "未知",
  platform: "未知",
  osVersion: "未知",
  arch: "未知",
  webview: "未知",
});

const infoRows = computed(() => [
  { label: "应用版本", value: info.value.version },
  { label: "Tauri 版本", value: info.value.tauriVersion },
  { label: "平台", value: info.value.platform },
  { label: "系统版本", value: info.value.osVersion },
  { label: "架构", value: info.value.arch },
  { label: "WebView", value: info.value.webview },
]);

function hasTauri() {
  return "__TAURI_INTERNALS__" in window;
}

function detectPlatform(): string {
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("windows")) return "Windows";
  if (ua.includes("mac os") || ua.includes("macintosh")) return "macOS";
  if (ua.includes("android")) return "Android";
  if (ua.includes("iphone") || ua.includes("ipad")) return "iOS";
  if (ua.includes("linux")) return "Linux";
  return "未知";
}

function detectWebview(): string {
  const ua = navigator.userAgent;
  const edge = ua.match(/Edg\/([\d.]+)/);
  if (edge) return `Edge ${edge[1]}`;
  const chrome = ua.match(/Chrome\/([\d.]+)/);
  if (chrome) return `Chrome ${chrome[1]}`;
  const firefox = ua.match(/Firefox\/([\d.]+)/);
  if (firefox) return `Firefox ${firefox[1]}`;
  const safari = ua.match(/Version\/([\d.]+).*Safari/);
  if (safari) return `Safari ${safari[1]}`;
  return ua || "未知";
}

async function readValue<T>(read: () => T | Promise<T>, fallback = "未知"): Promise<string> {
  try {
    const value = await read();
    if (value === null || value === undefined || value === "") return fallback;
    return String(value);
  } catch {
    return fallback;
  }
}

async function loadInfo() {
  const next: AppInfo = {
    name: "Anoterm",
    version: "未知",
    tauriVersion: "未知",
    platform: detectPlatform(),
    osVersion: "未知",
    arch: "未知",
    webview: detectWebview(),
  };

  if (hasTauri()) {
    const [name, version, tauriVersion, platformName, systemVersion, cpuArch] = await Promise.all([
      readValue(getName),
      readValue(getVersion),
      readValue(getTauriVersion),
      readValue(platform),
      readValue(osVersion),
      readValue(arch),
    ]);
    next.name = name;
    next.version = version;
    next.tauriVersion = tauriVersion;
    next.platform = PLATFORM_NAMES[platformName] ?? platformName;
    next.osVersion = systemVersion;
    next.arch = cpuArch;
  }

  info.value = next;
}

onMounted(loadInfo);

async function copyAllInfo() {
  const text = [
    `${info.value.name} v${info.value.version}`,
    `应用版本：${info.value.version}`,
    `Tauri 版本：${info.value.tauriVersion}`,
    `平台：${info.value.platform}`,
    `系统版本：${info.value.osVersion}`,
    `架构：${info.value.arch}`,
    `WebView：${info.value.webview}`,
  ].join("\n");

  try {
    await navigator.clipboard.writeText(text);
    toast.success("关于信息已复制");
  } catch {
    toast.error("复制失败，请手动选择复制");
  }
}
</script>

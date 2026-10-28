/**
 * SFTP 布局状态（全局单例）：
 * - dual：左右双栏（左本地 / 右远端），默认
 * - single：单栏远端浏览器
 * 通过标签栏 SFTP 标签右键菜单切换，持久化到 localStorage。
 */

import { ref, watch } from "vue";

export type SftpLayout = "dual" | "single";

const STORAGE_KEY = "anoterm.sftp.layout";
const saved = localStorage.getItem(STORAGE_KEY);

const layout = ref<SftpLayout>(saved === "single" ? "single" : "dual");

watch(layout, (value) => localStorage.setItem(STORAGE_KEY, value));

export function useSftpLayout() {
  return { layout };
}

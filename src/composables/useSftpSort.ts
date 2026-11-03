/**
 * SFTP 文件列表排序状态（本地/远端各一份，全局单例）：
 * - 默认按修改时间降序（最新在前）
 * - 排序偏好持久化到 localStorage，所有标签页共享
 * - 文件夹置顶由 FilePane 的排序逻辑保证
 */

import { ref, watch } from "vue";

export type SortKey = "name" | "size" | "mtime";
export type SortDirection = "asc" | "desc";
export type SftpSide = "local" | "remote";

interface SortState {
  key: SortKey;
  dir: SortDirection;
}

const DEFAULT: SortState = { key: "mtime", dir: "desc" };
const SORT_KEYS: SortKey[] = ["name", "size", "mtime"];

function storageKey(side: SftpSide): string {
  return `anoterm.sftp.sort.${side}`;
}

function load(side: SftpSide): SortState {
  try {
    const raw = localStorage.getItem(storageKey(side));
    if (!raw) return { ...DEFAULT };
    const parsed = JSON.parse(raw) as Partial<SortState>;
    const key = parsed.key && SORT_KEYS.includes(parsed.key) ? parsed.key : DEFAULT.key;
    const dir = parsed.dir === "asc" || parsed.dir === "desc" ? parsed.dir : DEFAULT.dir;
    return { key, dir };
  } catch {
    return { ...DEFAULT };
  }
}

function createState(side: SftpSide) {
  const initial = load(side);
  const sortKey = ref<SortKey>(initial.key);
  const sortDir = ref<SortDirection>(initial.dir);

  watch([sortKey, sortDir], () => {
    localStorage.setItem(
      storageKey(side),
      JSON.stringify({ key: sortKey.value, dir: sortDir.value }),
    );
  });

  /** 同一字段切换升降序；切换到新字段时名称默认升序，大小/时间默认降序 */
  function toggleSort(key: SortKey) {
    if (sortKey.value === key) {
      sortDir.value = sortDir.value === "asc" ? "desc" : "asc";
    } else {
      sortKey.value = key;
      sortDir.value = key === "name" ? "asc" : "desc";
    }
  }

  return { sortKey, sortDir, toggleSort };
}

const states: Record<SftpSide, ReturnType<typeof createState>> = {
  local: createState("local"),
  remote: createState("remote"),
};

export function useSftpSort(side: SftpSide) {
  return states[side];
}

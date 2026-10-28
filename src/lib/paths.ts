/**
 * 文件面板路径工具：远端（POSIX）与本地（兼容 Windows 盘符/反斜杠）路径处理。
 */

/** 远端路径拼接（父目录 + 子项名） */
export function joinRemote(parent: string, name: string): string {
  if (parent === "/" || parent === "") return "/" + name;
  return parent.replace(/\/+$/, "") + "/" + name;
}

/** 远端路径的上级目录 */
export function remoteParent(path: string): string {
  const parts = path.split("/").filter(Boolean);
  if (parts.length <= 1) return "/";
  return "/" + parts.slice(0, -1).join("/");
}

/** 本地路径拼接（识别盘符，Windows 用反斜杠） */
export function joinLocal(parent: string, name: string): string {
  if (/^[A-Za-z]:[\\/]?$/.test(parent)) return parent.slice(0, 2) + "\\" + name;
  const sep = /^[A-Za-z]:/.test(parent) || parent.includes("\\") ? "\\" : "/";
  return parent.replace(/[\\/]+$/, "") + sep + name;
}

/** 本地路径的上级目录（盘符根或 / 返回自身） */
export function localParent(path: string): string {
  const win = /^[A-Za-z]:/.test(path) || path.includes("\\");
  const parts = path.split(/[\\/]+/).filter(Boolean);
  if (win) {
    const root = parts[0] && parts[0].endsWith(":") ? parts[0] + "\\" : "\\";
    if (parts.length <= 1) return root;
    const rest = parts.slice(1, -1);
    return rest.length ? root + rest.join("\\") : root;
  }
  if (parts.length <= 1) return "/";
  return "/" + parts.slice(0, -1).join("/");
}

/** 是否已处于根目录（上级按钮禁用判断） */
export function isRootPath(kind: "local" | "remote", path: string): boolean {
  if (!path) return true;
  if (kind === "remote") return path === "/";
  return path === "/" || /^[A-Za-z]:[\\/]?$/.test(path);
}

/** 规范化手输的本地路径（去引号，裸盘符补斜杠） */
export function normalizeLocalPath(input: string): string {
  let p = input.trim();
  if (/^".*"$/.test(p)) p = p.slice(1, -1);
  if (/^[A-Za-z]:$/.test(p)) p += "\\";
  return p;
}

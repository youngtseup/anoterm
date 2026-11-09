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

/** 虚拟「此电脑」段的路径标记（仅用于盘符切换菜单） */
export const DRIVES_PATH = "__drives__";

/** 地址栏面包屑的一段 */
export interface PathSegment {
  name: string;
  path: string;
  /** 是否为虚拟段（如「此电脑」，用于列出盘符而非目录） */
  drives?: boolean;
}

/** 在 Windows 路径最前面插入「此电脑」，用于切换盘符 */
function withDrivesRoot(segments: PathSegment[]): PathSegment[] {
  return [{ name: "此电脑", path: DRIVES_PATH, drives: true }, ...segments];
}

/** 远端路径 → 面包屑（根为 /，逐级累积） */
export function remoteBreadcrumbs(path: string): PathSegment[] {
  if (!path) return [];
  const segments: PathSegment[] = [{ name: "/", path: "/" }];
  let current = "";
  for (const part of path.split("/").filter(Boolean)) {
    current += "/" + part;
    segments.push({ name: part, path: current });
  }
  return segments;
}

/** 本地路径 → 面包屑（兼容 Windows 盘符、UNC 与 Unix 根） */
export function localBreadcrumbs(path: string): PathSegment[] {
  if (!path) return [];
  // UNC：\\server\share 作为根级条目
  if (/^[\\/]{2}/.test(path)) {
    const parts = path.split(/[\\/]+/).filter(Boolean);
    if (parts.length >= 2) {
      const root = "\\\\" + parts[0] + "\\" + parts[1];
      const segments: PathSegment[] = [{ name: root, path: root }];
      let current = root;
      for (const part of parts.slice(2)) {
        current += "\\" + part;
        segments.push({ name: part, path: current });
      }
      return withDrivesRoot(segments);
    }
    const root = parts.length === 1 ? "\\\\" + parts[0] : "\\\\";
    return withDrivesRoot([{ name: root, path: root }]);
  }
  // Windows 盘符
  const drive = /^([A-Za-z]:)[\\/]?/.exec(path);
  if (drive) {
    const root = drive[1] + "\\";
    const segments: PathSegment[] = [{ name: drive[1], path: root }];
    let current = root;
    for (const part of path
      .slice(drive[0].length)
      .split(/[\\/]+/)
      .filter(Boolean)) {
      current = current.endsWith("\\") ? current + part : current + "\\" + part;
      segments.push({ name: part, path: current });
    }
    return withDrivesRoot(segments);
  }
  // Unix 绝对路径
  if (path.startsWith("/")) {
    const segments: PathSegment[] = [{ name: "/", path: "/" }];
    let current = "";
    for (const part of path.split("/").filter(Boolean)) {
      current += "/" + part;
      segments.push({ name: part, path: current });
    }
    return segments;
  }
  // 相对路径等无法安全拆分的场景：降级为单段
  return [{ name: path, path }];
}

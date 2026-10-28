<div align="center">
  <img src="src-tauri/icons/icon.png" width="96" height="96" alt="Anoterm logo" />
  <h1>Anoterm</h1>
  <p><strong>轻量、跨平台的 SSH / SFTP 连接管理与终端客户端</strong></p>
  <p><sub>A lightweight cross-platform SSH/SFTP connection manager and terminal</sub></p>

  <p>
    <a href="https://github.com/youngtseup/anoterm/releases"><img src="https://img.shields.io/github/v/release/youngtseup/anoterm?include_prereleases" alt="Release" /></a>
    <a href="https://github.com/youngtseup/anoterm/issues"><img src="https://img.shields.io/github/issues/youngtseup/anoterm" alt="Issues" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue" alt="License: Apache-2.0" /></a>
    <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey" alt="Platform" />
    <img src="https://img.shields.io/badge/Tauri-2-24c8db" alt="Tauri 2" />
    <img src="https://img.shields.io/badge/Vue-3-42b883" alt="Vue 3" />
    <img src="https://img.shields.io/badge/Rust-dea584?logo=rust&logoColor=white" alt="Rust" />
    <img src="https://img.shields.io/badge/AI-assisted%20development-8A2BE2" alt="AI-assisted development" />
  </p>
</div>

Anoterm 是一款基于 **Tauri 2 + Vue 3 + Rust** 的跨平台 SSH/SFTP 客户端，提供多标签终端、SFTP 文件管理与加密的本地连接存储，适合日常服务器管理。

> **AI 开发声明**：本项目基于 AI 开发完成。

> 项目处于早期开发阶段，功能仍在快速迭代。欢迎通过 [Issues](https://github.com/youngtseup/anoterm/issues) 反馈问题或建议。

## 功能特性

- **连接管理**：分组树形展示，支持分组与连接的增删改查。
- **多种认证方式**：密码或私钥（OpenSSH / PEM / PPK，支持加密私钥口令），编辑连接时可一键测试连通性。
- **主机密钥校验**：TOFU 模式，首次连接确认并保存指纹，密钥变更时告警，降低中间人攻击风险。
- **多标签 SSH 终端**：同一连接可打开多个标签，基于 xterm.js 渲染，标签切换时终端会话保活。
- **SFTP 文件管理**：单栏 / 左右双栏两种视图（右键标签栏的 SFTP 标签切换），双栏可在本地与远端之间互传文件（含文件夹递归），传输带进度条且可随时取消；支持地址栏跳转、新建、重命名、删除与右键菜单。
- **三种会话类型**：SSH、SFTP、SSH + SFTP，可在同一标签内切换终端与文件视图。
- **仪表盘与最近连接**：展示分组数、连接数与已连接会话数，最近连接支持一键重连。
- **主题与布局**：亮色 / 暗色主题，侧边栏可收起。
- **本地持久化**：SQLite 存储分组、连接与最近记录；连接凭据经 AES-256-GCM 加密，私钥仅保存文件路径。
- **自动更新**：内置 Tauri Updater，更新包经 minisign 签名校验。

## 下载安装

前往 [Releases](https://github.com/youngtseup/anoterm/releases/latest) 下载对应平台的安装包：

| 平台 | 架构 | 安装包 |
| --- | --- | --- |
| Windows | x64 | `Anoterm_<版本>_x64-setup.exe`（NSIS） |
| macOS | Universal | `Anoterm_<版本>_universal.dmg`（Apple Silicon / Intel 通用） |
| Linux | x64 / arm64 | `.AppImage`（免安装）、`.deb` |

安装包暂未做代码签名：

- Windows 首次运行会出现 SmartScreen「未知发布者」提示，选择「更多信息 → 仍要运行」。
- macOS 首次打开需右键 → 打开，或执行：

  ```bash
  xattr -dr com.apple.quarantine /Applications/Anoterm.app
  ```

Linux 下 AppImage 需先赋可执行权限：

```bash
chmod +x Anoterm_*.AppImage
./Anoterm_*.AppImage
```

## 快速上手

1. 在标题栏创建分组，在分组右键菜单或标题栏中新增连接。
2. 填写主机、端口与用户名，选择密码或私钥认证。
3. 点击「测试连接」验证配置，保存后点击连接即可打开终端。
4. 在标签内切换到 SFTP 子视图，管理远端文件。

常用快捷键：

| 快捷键 | 功能 |
| --- | --- |
| `Ctrl/Cmd + C` | 有选区时复制选中文本 |
| `Ctrl + C` | 无选区时向远端发送中断信号（SIGINT） |
| `Ctrl/Cmd + V` | 粘贴 |
| `Ctrl + Tab` / `Ctrl + Shift + Tab` | 在 SSH / SFTP 子视图间切换 |

## 自动更新

应用内置 [Tauri Updater](https://v2.tauri.app/plugin/updater/)，点击标题栏「云下载」图标即可检查并安装更新。

- **更新源**：GitHub Releases 的 [`latest.json`](https://github.com/youngtseup/anoterm/releases/latest/download/latest.json)。
- **签名校验**：更新包使用 minisign（Ed25519）签名，公钥内置于应用配置。
- **普通用户**只需下载安装包；`.sig` 与 `.app.tar.gz` 为更新器使用的文件，无需手动安装。

## 技术栈

| 层 | 技术 |
| --- | --- |
| 桌面框架 | [Tauri 2](https://v2.tauri.app)（Rust） |
| 前端 | [Vue 3](https://vuejs.org)（`<script setup>`）+ TypeScript + Vite |
| UI | [shadcn-vue](https://www.shadcn-vue.com) + [Tailwind CSS](https://tailwindcss.com) |
| 终端 | [xterm.js](https://xtermjs.org) |
| SSH / SFTP | [russh](https://github.com/warp-tech/russh)（Rust 原生实现） |
| 存储 | SQLite（rusqlite，bundled） |

## 本地开发

### 环境要求

- Node.js 20+（推荐 22）与 [pnpm](https://pnpm.io) 10
- Rust 工具链（stable）
- Windows：[WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2)（Win10/11 通常已内置）
- macOS：Xcode Command Line Tools
- Linux：[Tauri 系统依赖](https://v2.tauri.app/start/prerequisites/)（WebKitGTK 等）

### 启动

```bash
pnpm install
pnpm tauri dev
```

> 开发模式使用独立 identifier `top.youngtse.anoterm.dev` 隔离数据，数据库位于
> `%APPDATA%\top.youngtse.anoterm.dev\anoterm.db`（macOS 为
> `~/Library/Application Support/top.youngtse.anoterm.dev/anoterm.db`），不会影响正式版数据。

### 常用命令

| 命令 | 说明 |
| --- | --- |
| `pnpm tauri dev` | 启动开发环境（前端热更新 + Rust 调试构建） |
| `pnpm build` | 前端类型检查与构建 |
| `pnpm run version <版本>` | 同步修改四处版本号（`package.json` / `tauri.conf.json` / `Cargo.toml` / `Cargo.lock`） |
| `pnpm format` / `pnpm format:check` | Prettier 格式化 / 格式校验 |
| `cargo fmt --manifest-path src-tauri/Cargo.toml` | Rust 格式化 |
| `cargo clippy --manifest-path src-tauri/Cargo.toml` | Rust 静态检查 |
| `pnpm tauri icon <图片路径>` | 由 1024×1024 PNG 生成全套应用图标 |

修改版本号只需执行 `pnpm run version 0.3.0`；不带参数时以 `package.json` 当前版本对齐其余三个文件。

修改代码后请确保 `pnpm format:check` 与 `cargo clippy` 通过（`src/components/ui` 为生成代码，不参与 Prettier 检查）。

### 项目结构

```text
├── src/                          # 前端
│   ├── api/                      # 后端命令封装（connections / sftp / local / index）
│   ├── components/               # 标题栏、侧边栏、标签栏、终端、SFTP、仪表盘、对话框
│   ├── composables/              # useTabs / useTheme 等状态逻辑
│   ├── assets/                   # 全局样式
│   └── types.ts                  # 全局类型定义
├── scripts/
│   ├── set-version.mjs           # 版本号设置（pnpm run version x.y.z，同步四处）
│   ├── make-latest-json.mjs      # 跨平台更新清单合成（CI 使用）
│   └── make-latest-json.ps1      # Windows 本地应急清单生成
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs                # 应用装配：插件、数据目录、命令注册
│   │   ├── db_commands.rs        # 分组 / 连接 / 最近连接命令
│   │   ├── storage.rs            # SQLite 数据层
│   │   ├── crypto.rs             # 凭据加解密
│   │   ├── local_fs.rs           # 本地文件系统命令（双栏 SFTP）
│   │   └── ssh/                  # SSH / SFTP 会话（connect / shell / sftp）
│   ├── tauri.conf.json           # Tauri 主配置
│   ├── tauri.windows.conf.json   # Windows 打包配置（NSIS）
│   ├── tauri.linux.conf.json     # Linux 打包配置（deb / AppImage）
│   └── tauri.macos.conf.json     # macOS 打包配置（使用加内边距的图标）
└── .github/workflows/release.yml # CI 多平台构建与发布
```

## 构建与发布

### 本地构建

```bash
# 仅构建前端（类型检查 + Vite 构建）
pnpm build

# 完整桌面打包（需要更新签名私钥，否则 updater 产物生成失败）
export TAURI_SIGNING_PRIVATE_KEY_PATH="$PWD/src-tauri/.tauri/anoterm.key"
pnpm tauri build
```

Windows PowerShell：

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY_PATH = "$PWD\src-tauri\.tauri\anoterm.key"
pnpm tauri build
```

### 产物位置

| 平台 | 安装包 | 更新包（自动更新使用） |
| --- | --- | --- |
| Windows | `src-tauri/target/release/bundle/nsis/Anoterm_<版本>_x64-setup.exe` | 同左（`.exe` 直接被 updater 复用）+ `.sig` |
| macOS | `.../bundle/dmg/Anoterm_<版本>_universal.dmg` | `.../bundle/macos/Anoterm.app.tar.gz` + `.sig` |
| Linux | `.../bundle/appimage/*.AppImage`、`.../bundle/deb/*.deb` | `.AppImage` 本身 + `.AppImage.sig` |

- Windows 使用 NSIS 中文安装向导，安装时可自定义目录。
- Linux 仅构建 `.deb` 与 `.AppImage`，配置见 `src-tauri/tauri.linux.conf.json`。
- macOS 与 Linux 产物需在对应系统上构建；正式多平台发布由 GitHub Actions 完成。

<details>
<summary><strong>维护者：发布新版本</strong></summary>

正式发布由 [`.github/workflows/release.yml`](.github/workflows/release.yml) 完成：推送 `v*` tag 后自动构建 Windows x64、macOS Universal、Linux x64 与 Linux arm64，上传安装包 / 更新包，创建 **draft Release** 并合成 `latest.json`。

1. 设置版本号：执行 `pnpm run version 0.3.0`，会同步更新 `package.json`、`src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml` 与 `src-tauri/Cargo.lock` 四处。
2. 提交并推送代码后打 tag：

   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

3. 等待 Actions 完成构建，检查 draft Release 中的产物与 `latest.json`。
4. 确认无误后点击 **Publish release**；发布后应用内「检查更新」才会生效。

首次发布需在仓库 `Settings → Secrets and variables → Actions` 配置：

| Secret | 值 |
| --- | --- |
| `TAURI_SIGNING_PRIVATE_KEY` | `src-tauri/.tauri/anoterm.key` 的完整内容 |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 生成密钥时设置的口令（未设置则无需配置） |

> **重要**：签名私钥位于 `src-tauri/.tauri/anoterm.key`（已 gitignore），请妥善备份；丢失后无法发布可被旧版本接受的更新。

<details>
<summary>本地应急发布（仅 Windows）</summary>

1. 执行 `pnpm run version 0.3.0` 更新版本号。
2. 设置 `TAURI_SIGNING_PRIVATE_KEY_PATH` 后执行 `pnpm tauri build`。
3. 生成更新清单：

   ```powershell
   .\scripts\make-latest-json.ps1 -Version 0.2.0
   ```

4. 在 GitHub Releases 创建或更新 Release（tag `v0.2.0`），上传 `Anoterm_0.2.0_x64-setup.exe`、对应 `.sig` 与 `latest.json`。

> 此方式仅覆盖 Windows 平台；macOS / Linux 请使用 CI 发布。

</details>

</details>

## 常见问题

<details>
<summary><b>Windows 提示「Windows 已保护你的电脑」/ macOS 提示无法验证开发者</b></summary>

安装包尚未做代码签名。Windows 请点击「更多信息 → 仍要运行」；macOS 请右键应用选择「打开」，或执行上文中的 `xattr` 命令解除隔离。

</details>

<details>
<summary><b>Linux AppImage 无法运行</b></summary>

先确认已赋予可执行权限：

```bash
chmod +x Anoterm_*.AppImage
```

若提示缺少 `libfuse.so.2`，安装 FUSE 2 兼容库：Ubuntu 22.04 使用 `sudo apt install libfuse2`，Ubuntu 24.04 使用 `sudo apt install libfuse2t64`。也可以改用 `.deb` 安装。

</details>

<details>
<summary><b>数据存储在哪里？</b></summary>

数据库固定存放在系统应用数据目录，与安装位置无关：

| 环境 | Windows | macOS | Linux |
| --- | --- | --- | --- |
| 正式版 | `%APPDATA%\top.youngtse.anoterm\anoterm.db` | `~/Library/Application Support/top.youngtse.anoterm/anoterm.db` | `~/.local/share/top.youngtse.anoterm/anoterm.db` |
| 开发模式 | `%APPDATA%\top.youngtse.anoterm.dev\anoterm.db` | `~/Library/Application Support/top.youngtse.anoterm.dev/anoterm.db` | `~/.local/share/top.youngtse.anoterm.dev/anoterm.db` |

</details>

<details>
<summary><b>如何卸载并清除数据？</b></summary>

**Windows**：通过「应用和功能」卸载。卸载向导提供「删除应用数据」复选框：勾选会同时删除数据库目录（`%APPDATA%\top.youngtse.anoterm`）与 WebView2 缓存目录（`%LOCALAPPDATA%\top.youngtse.anoterm`）；不勾选则保留数据，便于重装后继续使用。

**macOS**：把 `.app` 拖入废纸篓不会删除用户数据。退出应用后，按需清理以下目录（不存在的路径会自动跳过）：

```bash
rm -rf "$HOME/Library/Application Support/top.youngtse.anoterm" \
       "$HOME/Library/WebKit/top.youngtse.anoterm" \
       "$HOME/Library/HTTPStorages/top.youngtse.anoterm" \
       "$HOME/Library/Caches/top.youngtse.anoterm" \
       "$HOME/Library/Preferences/top.youngtse.anoterm.plist" \
       "$HOME/Library/Saved Application State/top.youngtse.anoterm.savedState"
```

也可以使用按 bundle id 清理的第三方工具，如 [Pearcleaner](https://github.com/alienator88/Pearcleaner) 或 AppCleaner。

**Linux**：删除 AppImage 文件，或通过包管理器卸载 `.deb`；本地数据目录需按上表手动清理。

</details>

## 安全说明

- 连接密码与私钥口令通过 AES-256-GCM 加密后存入 SQLite，使用连接时由后端解密认证；私钥本身不复制入库，只保存文件路径。
- 当前加密主密钥为内置常量，可防止直接读取数据库文件获取明文，但无法抵御二进制逆向提取。后续计划接入系统凭据库（keyring / DPAPI / Keychain）。
- 首次连接采用 TOFU 主机密钥校验，指纹变更时会要求用户确认，避免静默接受新密钥。
- 安装包暂未做代码签名，请从本仓库 Releases 页面下载。

## 反馈与贡献

欢迎通过 [Issues](https://github.com/youngtseup/anoterm/issues) 反馈 Bug、提出建议或讨论使用问题。

本项目主要用于个人使用，**暂不接受 Pull Request**。如果你需要额外功能或自行定制：

1. Fork 本仓库。
2. 参考[本地开发](#本地开发)搭建环境并修改代码。
3. 自行构建使用。二次开发或分发时请遵守[许可证](#许可证)，保留版权声明与 [NOTICE](NOTICE)，并标注修改内容。

较大的功能建议，欢迎先开 Issue 讨论。

## 许可证

本项目采用 [Apache License 2.0](LICENSE) 许可。

- **允许**：商业使用、修改、分发与私有使用。
- **署名与声明**：分发时需保留版权声明、许可证文本与 [NOTICE](NOTICE)，并在修改过的文件中标注修改。
- **专利授权**：贡献者授予专利许可；若发起专利诉讼，相关专利授权将终止。
- **不强制开源**：衍生作品可采用其他许可证（包括闭源发布），但须遵守上述声明要求。
- **商标**：不授予项目名称或商标使用权。
- 第三方依赖遵循其各自许可证。

完整条款见 [LICENSE](LICENSE) 与 [Apache 官方文本](https://www.apache.org/licenses/LICENSE-2.0)。

## 致谢

- [russh](https://github.com/warp-tech/russh)：Rust 原生 SSH / SFTP 实现
- [xterm.js](https://xtermjs.org)：终端渲染
- [Tauri](https://v2.tauri.app) / [Vue](https://vuejs.org) / [shadcn-vue](https://www.shadcn-vue.com)：应用框架与 UI 体系
- [Tabby](https://tabby.sh) / [SecureCRT](https://www.vandyke.com/products/securecrt)：界面与交互参考

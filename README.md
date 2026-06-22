<div align="center">

# RustPad

**A modern, cross-platform code editor built with Rust — inspired by Notepad++, with integrated Beyond Compare–style file diff.**

**用 Rust 打造的现代化跨平台代码编辑器 —— 灵感来自 Notepad++，并集成了类 Beyond Compare 的文件对比能力。**

[![CI](https://github.com/rustpad/rustpad/actions/workflows/ci.yml/badge.svg)](https://github.com/rustpad/rustpad/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

[English](#english) · [中文](#中文) · [Architecture / 架构](docs/ARCHITECTURE.md)

</div>

---

<a name="english"></a>

## English

RustPad is a lightweight, fast, native desktop code editor written in pure Rust on top of
[`egui`](https://github.com/emilk/egui) / [`eframe`](https://github.com/emilk/egui/tree/master/crates/eframe).
It aims to combine the familiar ergonomics of Notepad++ with a powerful side-by-side diff tool,
all in a single self-contained binary that runs on macOS, Windows, and Linux.

### Features

#### Core editing
- Multi-tab editing with unlimited undo/redo (command-pattern history)
- Syntax highlighting via `syntect` (100+ languages, TextMate grammars)
- Manual language override per tab (View → Language)
- Smart auto-indentation and configurable tab size
- Line numbers, current-line highlight, and minimap overview
- Code folding, autocomplete, and macro recording/playback
- Split view (horizontal / vertical) and configurable word wrap
- Full clipboard support (Cut / Copy / Paste / Select All) and free text selection

#### Find & Replace
- Floating, draggable Find / Replace dialog (Notepad++ style)
- Literal, whole-word, case-sensitive, and regular-expression modes
- Forward / backward direction and wrap-around
- Live highlighting of **all** matches plus the current match
- A dockable **Search results** panel listing every hit with its line number — click to jump
- Count, "Find All in Current Document", and "Find All in All Opened Documents"
- Cross-file search (Find in Files) over a directory with glob filters

#### File diff (Beyond Compare style)
- Side-by-side aligned diff view with synchronized scrolling
- Line-, word-, and character-level highlighting (inline diffs)
- Three algorithms: Myers, Patience, LCS
- Ignore options: whitespace, case, line endings
- Change navigation (previous / next, F7 / F8), merge left/right, save either side
- Folder comparison and three-way merge with conflict detection
- Export an HTML diff report

#### Files & sessions
- File-explorer sidebar (tree view) with full paths
- Encoding detection & conversion (UTF-8 / UTF-16 LE-BE / Latin-1) and BOM handling
- Line-ending normalization (LF / CRLF / CR)
- Save-format / encoding selection on save
- "Save changes?" prompt before closing a tab or quitting (no silent data loss)
- Auto-save with crash recovery and full session persistence (reopens last files)

#### Customization
- Built-in themes: Dark, Light, Monokai, Solarized Dark, plus custom JSON themes
- Configurable keybindings with Notepad++ and VS Code presets
- Font family / size, all persisted to `~/.config/rustpad/config.toml`

#### Cross-platform
- macOS: custom menu handling so `Cmd+Q` prompts to save; `.app` / `.dmg` packaging
- Windows: high-DPI support and file-association registration
- Linux: full support via `egui` / `eframe`

### Building

Prerequisites: Rust 1.75+ ([rustup](https://rustup.rs/)). On Linux you also need GTK3 dev
packages (for native file dialogs), e.g. `sudo apt install libgtk-3-dev`.

```bash
git clone https://github.com/rustpad/rustpad.git
cd rustpad

cargo build              # debug build
cargo build --release    # optimized build
cargo run --release      # build & run
```

#### Packaging

```bash
# macOS: build .app + .dmg (signs locally)
./scripts/build_macos.sh

# Generic Unix
./scripts/build.sh

# Windows
.\scripts\build.ps1
```

### Keyboard shortcuts (Notepad++ preset)

| Action | Shortcut | Action | Shortcut |
|--------|----------|--------|----------|
| New tab | `Ctrl/Cmd+N` | Find | `Ctrl/Cmd+F` |
| Open | `Ctrl/Cmd+O` | Replace | `Ctrl/Cmd+H` |
| Save | `Ctrl/Cmd+S` | Find in Files | `Ctrl/Cmd+Shift+F` |
| Save As | `Ctrl/Cmd+Shift+S` | Find Next / Prev | `F3` / `F4` |
| Close tab | `Ctrl/Cmd+W` | Go to Line | `Ctrl/Cmd+G` |
| Quit | `Ctrl/Cmd+Q` | Compare Files | `Ctrl/Cmd+D` |
| Undo / Redo | `Ctrl/Cmd+Z` / `Ctrl/Cmd+Y` | Diff Prev / Next | `F7` / `F8` |
| Cut / Copy / Paste | `Ctrl/Cmd+X/C/V` | Toggle Sidebar | `Ctrl/Cmd+B` |
| Select All | `Ctrl/Cmd+A` | Command Palette | `Ctrl/Cmd+Shift+P` |

Customize via `~/.config/rustpad/keybindings.json` (see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)).

### Configuration

All settings live in `~/.config/rustpad/config.toml`; custom themes go in
`~/.config/rustpad/themes/*.json`. Set `RUSTPAD_LOG=debug` for verbose logging;
crash logs are written to `~/.config/rustpad/crash.log`.

### Contributing

1. Fork & branch (`git checkout -b feature/my-feature`)
2. Make sure `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` all pass
3. Open a pull request

### License

[MIT](LICENSE).

---

<a name="中文"></a>

## 中文

RustPad 是一个用纯 Rust 编写、基于
[`egui`](https://github.com/emilk/egui) / [`eframe`](https://github.com/emilk/egui/tree/master/crates/eframe)
的轻量、快速的原生桌面代码编辑器。它的目标是把 Notepad++ 熟悉的使用体验与强大的并排文件对比工具
结合起来，并打包成一个可在 macOS、Windows、Linux 上运行的自包含可执行文件。

### 功能特性

#### 核心编辑
- 多标签页编辑，无限撤销/重做（基于命令模式的历史记录）
- 通过 `syntect` 实现语法高亮（支持 100+ 种语言，TextMate 语法）
- 每个标签页可手动指定语言（视图 → Language）
- 智能自动缩进，可配置 Tab 宽度
- 行号、当前行高亮、缩略图（Minimap）总览
- 代码折叠、自动补全、宏录制/回放
- 分屏视图（水平/垂直），可配置自动换行
- 完整剪贴板支持（剪切/复制/粘贴/全选）与自由文本选择

#### 查找与替换
- 浮动、可拖动的查找/替换对话框（Notepad++ 风格）
- 支持普通文本、全字匹配、区分大小写、正则表达式模式
- 支持向前/向后方向与循环查找（wrap-around）
- 实时高亮**所有**匹配项并突出显示当前匹配
- 可停靠的 **查找结果面板**，列出每条命中及其行号 —— 点击即可跳转
- 计数、"在当前文档查找全部"、"在所有打开文档查找全部"
- 跨文件搜索（在文件夹中查找），支持通配符过滤

#### 文件对比（Beyond Compare 风格）
- 并排对齐的差异视图，滚动同步
- 行级、词级、字符级高亮（行内差异）
- 三种算法：Myers、Patience、LCS
- 忽略选项：空白字符、大小写、行尾符
- 差异导航（上一处/下一处，F7/F8）、左右合并、分别保存两侧
- 文件夹对比与带冲突检测的三方合并
- 导出 HTML 差异报告

#### 文件与会话
- 文件浏览器侧边栏（树状视图），显示完整路径
- 编码检测与转换（UTF-8 / UTF-16 LE-BE / Latin-1）及 BOM 处理
- 行尾符规范化（LF / CRLF / CR）
- 保存时可选择保存格式/编码
- 关闭标签页或退出前弹出"是否保存"提示（避免静默丢失数据）
- 自动保存 + 崩溃恢复，完整的会话持久化（重新打开上次的文件）

#### 自定义
- 内置主题：Dark、Light、Monokai、Solarized Dark，并支持自定义 JSON 主题
- 可配置快捷键，内置 Notepad++ 与 VS Code 两套预设
- 字体族/字号设置，全部持久化到 `~/.config/rustpad/config.toml`

#### 跨平台
- macOS：自定义菜单处理，使 `Cmd+Q` 也能弹出保存提示；支持 `.app` / `.dmg` 打包
- Windows：高 DPI 支持与文件关联注册
- Linux：通过 `egui` / `eframe` 完整支持

### 构建

环境要求：Rust 1.75+（[rustup](https://rustup.rs/)）。Linux 上还需安装 GTK3 开发包
（用于原生文件对话框），例如 `sudo apt install libgtk-3-dev`。

```bash
git clone https://github.com/rustpad/rustpad.git
cd rustpad

cargo build              # 调试构建
cargo build --release    # 优化构建
cargo run --release      # 构建并运行
```

#### 打包

```bash
# macOS：构建 .app + .dmg（本地签名）
./scripts/build_macos.sh

# 通用 Unix
./scripts/build.sh

# Windows
.\scripts\build.ps1
```

### 快捷键（Notepad++ 预设）

| 操作 | 快捷键 | 操作 | 快捷键 |
|------|--------|------|--------|
| 新建标签 | `Ctrl/Cmd+N` | 查找 | `Ctrl/Cmd+F` |
| 打开 | `Ctrl/Cmd+O` | 替换 | `Ctrl/Cmd+H` |
| 保存 | `Ctrl/Cmd+S` | 文件中查找 | `Ctrl/Cmd+Shift+F` |
| 另存为 | `Ctrl/Cmd+Shift+S` | 查找下/上一个 | `F3` / `F4` |
| 关闭标签 | `Ctrl/Cmd+W` | 跳转到行 | `Ctrl/Cmd+G` |
| 退出 | `Ctrl/Cmd+Q` | 文件对比 | `Ctrl/Cmd+D` |
| 撤销/重做 | `Ctrl/Cmd+Z` / `Ctrl/Cmd+Y` | 上/下一处差异 | `F7` / `F8` |
| 剪切/复制/粘贴 | `Ctrl/Cmd+X/C/V` | 切换侧边栏 | `Ctrl/Cmd+B` |
| 全选 | `Ctrl/Cmd+A` | 命令面板 | `Ctrl/Cmd+Shift+P` |

可通过 `~/.config/rustpad/keybindings.json` 自定义（详见 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)）。

### 配置

所有设置保存在 `~/.config/rustpad/config.toml`；自定义主题放在
`~/.config/rustpad/themes/*.json`。设置 `RUSTPAD_LOG=debug` 可开启详细日志；
崩溃日志写入 `~/.config/rustpad/crash.log`。

### 参与贡献

1. Fork 并新建分支（`git checkout -b feature/my-feature`）
2. 确保 `cargo fmt --check`、`cargo clippy -- -D warnings`、`cargo test` 全部通过
3. 提交 Pull Request

### 许可证

[MIT](LICENSE)。

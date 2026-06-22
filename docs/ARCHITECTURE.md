# RustPad — Architecture & Source Guide / 架构与源码功能介绍

This document describes the codebase module by module, in English and 中文.
本文件逐模块介绍代码库，提供中英文对照说明。

---

## 1. High-level overview / 总体概览

**EN.** RustPad is a single-binary native desktop application. The UI is built with the
immediate-mode GUI library `egui` (windowed by `eframe`). The architecture follows a clear
separation between **pure data/logic** (editor buffer, diff engine, search engine, config) and
the **UI layer** (egui widgets). A single central state object, `RustpadApp`, owns everything and
is driven once per frame by `eframe::App::update`.

**中文.** RustPad 是一个单可执行文件的原生桌面应用。界面基于即时模式 GUI 库 `egui`
（由 `eframe` 提供窗口）。架构上明确区分**纯数据/逻辑层**（编辑缓冲、差异引擎、搜索引擎、配置）
与 **UI 层**（egui 控件）。一个中心状态对象 `RustpadApp` 持有全部状态，由 `eframe::App::update`
每帧驱动一次。

```
main.rs ─┬─> platform::init (logging, panic hook, native window options)
         └─> RustpadApp::new ──> eframe event loop
                                   └─ update(ctx) each frame:
                                        handle_close_request → sync_theme
                                        → collect_shortcuts → process_pending_actions
                                        → render menu/toolbar/tabs/sidebar/editor/
                                          search/status/diff/dialogs
```

### Data flow / 数据流

**EN.** Keyboard/mouse events come from egui each frame. `collect_shortcuts` maps them to a
`Command` (via the keybinding table), `process_pending_actions` mutates the active `TextBuffer` /
tabs / search / diff state, and the UI modules re-render from that state. File I/O, search and
diff are plain Rust modules with unit tests and no UI dependency.

**中文.** 每帧从 egui 获取键鼠事件。`collect_shortcuts` 通过快捷键表将其映射为 `Command`，
`process_pending_actions` 据此修改当前的 `TextBuffer`/标签页/搜索/差异状态，UI 模块再依据状态重绘。
文件读写、搜索、差异都是带单元测试、不依赖 UI 的纯 Rust 模块。

---

## 2. Module reference / 模块说明

### `main.rs` — Entry point / 程序入口
**EN.** Initializes the tokio runtime, platform logging and panic hook, configures the native
window (size, Glow/OpenGL renderer), applies platform-specific tweaks (notably disabling winit's
default macOS menu so `Cmd+Q` reaches the app), and launches `RustpadApp`.
**中文.** 初始化 tokio 运行时、平台日志与崩溃钩子，配置原生窗口（尺寸、Glow/OpenGL 渲染器），
应用平台特定调整（重点：关闭 winit 默认 macOS 菜单，使 `Cmd+Q` 能传递到应用），并启动 `RustpadApp`。

### `app.rs` — Central application state / 中心应用状态
**EN.** Defines `RustpadApp`, the hub that owns the tab manager, config, highlighter, search and
diff state, dialog flags and all pending actions. Implements `eframe::App::update` (the per-frame
loop), command dispatch, file open/save, theme application, the save-on-quit flow
(`handle_close_request` / `confirm_quit_*` with the `force_quit` guard), and all find/replace and
diff orchestration methods.
**中文.** 定义 `RustpadApp`，作为持有标签管理器、配置、高亮器、搜索与差异状态、对话框标志及所有待处理动作
的中枢。实现 `eframe::App::update`（每帧循环）、命令分发、文件打开/保存、主题应用、退出前保存流程
（`handle_close_request` / `confirm_quit_*`，配合 `force_quit` 保护标志），以及全部查找/替换与差异的编排方法。

### `config/` — Configuration & themes / 配置与主题
**EN.** `mod.rs` defines `AppConfig` (editor / UI / window / auto-save sections) persisted as TOML
under `~/.config/rustpad/`. `theme.rs` defines the color theme system: 4 built-in themes plus
user JSON themes, mapping to both egui visuals and the editor's syntax colors.
**中文.** `mod.rs` 定义 `AppConfig`（编辑器/界面/窗口/自动保存等分区），以 TOML 形式持久化到
`~/.config/rustpad/`。`theme.rs` 定义配色主题系统：4 个内置主题加用户 JSON 主题，
同时映射到 egui 视觉样式与编辑器语法配色。

### `editor/` — Editor core (pure logic) / 编辑器核心（纯逻辑）
- **`mod.rs`** — Encoding/line-ending detection, `read_file_to_string`, `EditAction` undo enum.
  编码/行尾检测、`read_file_to_string`、撤销动作枚举 `EditAction`。
- **`buffer.rs`** — `TextBuffer`: text storage on a `SimpleRope` with char-based positions and
  undo/redo history. `TextBuffer`：基于 `SimpleRope` 的文本存储，使用字符位置并维护撤销/重做历史。
- **`cursor.rs`** — `Cursor` and `Selection` (line/column model). 光标与选区（行/列模型）。
- **`tab.rs`** — `TabManager` and per-tab state (path, dirty flag, cursor, syntax override).
  `TabManager` 与每个标签的状态（路径、脏标记、光标、语言覆盖）。
- **`file_io.rs`** — File reading/writing with encoding & BOM handling. 带编码与 BOM 处理的文件读写。
- **`fold.rs` / `indent.rs` / `autocomplete.rs` / `macro_recorder.rs`** — Code folding,
  auto-indent rules, completion engine, and macro record/replay. 代码折叠、自动缩进规则、补全引擎、宏录制回放。
- **`view.rs`** — Editor view helper data. 编辑器视图辅助数据。

### `highlight/` — Syntax highlighting / 语法高亮
**EN.** Wraps `syntect`: detects syntax by extension or explicit name, caches highlighted spans
per line, and exposes the list of available languages (used by View → Language).
**中文.** 封装 `syntect`：按扩展名或显式名称识别语法，按行缓存高亮片段，并暴露可用语言列表
（供 视图 → Language 使用）。

### `search/` — Search engine / 搜索引擎
**EN.** `SearchEngine` finds all matches (literal / whole-word / case / regex) using **character
offsets** consistent with `TextBuffer`, supports next/prev navigation with wrap-around, and
performs single and bulk replace. Fully unit-tested, no UI dependency.
**中文.** `SearchEngine` 使用与 `TextBuffer` 一致的**字符偏移**查找全部匹配（普通/全字/大小写/正则），
支持带循环的上一处/下一处导航，并执行单个与批量替换。完整单元测试，不依赖 UI。

### `diff/` — Diff engine / 差异引擎
- **`mod.rs`** — Data types: `DiffTag`, `DiffLine`, `DiffHunk`, and the side-by-side aligned
  `DiffRow` (pairs left/right content, inline char spans, change grouping).
  数据类型：`DiffTag`、`DiffLine`、`DiffHunk` 及并排对齐的 `DiffRow`（配对左右内容、行内字符片段、变更分组）。
- **`engine.rs`** — `DiffEngine` with Myers / Patience / LCS algorithms, ignore options, and
  `build_aligned_rows` + `inline_char_spans` for the side-by-side view.
  `DiffEngine`，含 Myers/Patience/LCS 算法、忽略选项，以及用于并排视图的 `build_aligned_rows` 与 `inline_char_spans`。
- **`folder_diff.rs`** — Recursive directory comparison with status. 递归文件夹对比及状态。
- **`three_way.rs`** — Three-way merge with conflict detection. 带冲突检测的三方合并。

### `session/` — Session & auto-save / 会话与自动保存
**EN.** `Session` persists open files, active tab, per-file cursor/scroll, workspace root and
recent files, so the app reopens exactly where you left off. Includes the `AutoSaveManager`.
**中文.** `Session` 持久化打开的文件、当前标签、每个文件的光标/滚动位置、工作区根目录与最近文件，
使应用重开时恢复到上次状态。包含 `AutoSaveManager`。

### `platform/` — Platform glue / 平台适配
**EN.** `mod.rs` provides cross-platform logging init and a panic hook (writes `crash.log`).
`macos.rs` and `windows.rs` configure native options and OS-specific behaviors.
**中文.** `mod.rs` 提供跨平台日志初始化与崩溃钩子（写入 `crash.log`）。
`macos.rs` 与 `windows.rs` 配置原生选项与各操作系统的特定行为。

### `plugin/` — Plugin API (skeleton) / 插件接口（雏形）
**EN.** Defines the `PluginApi` trait (open/save/key hooks, text transform) for future extension.
**中文.** 定义 `PluginApi` trait（打开/保存/按键钩子、文本转换），为后续扩展预留。

### `ui/` — egui UI layer / egui 界面层
**EN.** Each file renders one part of the interface from `RustpadApp` state:
**中文.** 每个文件依据 `RustpadApp` 状态渲染界面的一部分：

| File / 文件 | Responsibility / 职责 |
|-------------|------------------------|
| `menu.rs` | Top menu bar (File/Edit/View/Tools…) / 顶部菜单栏 |
| `toolbar.rs` | Icon toolbar / 工具栏 |
| `tab_bar.rs` | Open-document tabs / 文档标签栏 |
| `sidebar.rs` + `file_tree.rs` | File explorer & directory search / 文件浏览器与目录搜索 |
| `editor_view.rs` / `editor_widget.rs` | Main text editor: rendering, selection, cursor, match highlight / 主编辑器：渲染、选区、光标、匹配高亮 |
| `search_panel.rs` | Find/Replace dialog + dockable search-results panel / 查找替换对话框 + 可停靠结果面板 |
| `diff_view.rs` / `diff_toolbar.rs` / `diff_navigator.rs` | Side-by-side diff view & controls / 并排差异视图与控件 |
| `minimap.rs` | Code minimap / 代码缩略图 |
| `status_bar.rs` | Bottom status bar / 底部状态栏 |
| `dialogs.rs` | Modal dialogs: unsaved/quit prompts, goto-line, preferences, about / 模态对话框 |
| `command_palette.rs` | Quick command palette / 命令面板 |
| `keybindings.rs` | Configurable keybindings (Notepad++ / VS Code presets) / 可配置快捷键 |
| `layout.rs` / `split_view.rs` | Overall layout & split view / 整体布局与分屏 |

---

## 3. Key design decisions / 关键设计决策

**EN.**
- **Character offsets everywhere.** Buffer, search and diff all use char (not byte) positions, so
  multi-byte/CJK text is handled correctly.
- **One frame, one state.** All mutation happens in `process_pending_actions`; UI modules are
  pure renderers reading `RustpadApp`, which keeps the immediate-mode loop predictable.
- **Save safety.** Closing a tab or the app always checks for unsaved changes; the `force_quit`
  flag prevents the close-request guard from looping after the user chooses "Don't Save".
- **Logic is testable.** `editor`, `search`, `diff`, `config` carry their own unit tests and have
  no `egui` dependency.

**中文.**
- **全程字符偏移。** 缓冲、搜索、差异均使用字符（而非字节）位置，正确处理多字节/中日韩文本。
- **单帧单状态。** 所有修改集中在 `process_pending_actions`；UI 模块是只读 `RustpadApp` 的纯渲染器，
  使即时模式循环行为可预测。
- **保存安全。** 关闭标签或应用时始终检查未保存更改；`force_quit` 标志可避免用户选择"不保存"后
  关闭请求守卫陷入循环。
- **逻辑可测试。** `editor`、`search`、`diff`、`config` 各自带单元测试，且不依赖 `egui`。

---

## 4. Build, test, lint / 构建、测试、检查

```bash
cargo build            # / 构建
cargo test             # run unit tests / 运行单元测试
cargo clippy -- -D warnings   # lint / 静态检查
cargo fmt --check      # formatting / 格式检查
```

**EN.** The CI workflow (`.github/workflows/ci.yml`) runs fmt + clippy + test + release build on
Linux, macOS and Windows. On Linux, install GTK3 dev packages for `rfd` native dialogs.
**中文.** CI 工作流（`.github/workflows/ci.yml`）在 Linux/macOS/Windows 上运行
fmt + clippy + test + release 构建。Linux 上需安装 GTK3 开发包以支持 `rfd` 原生对话框。

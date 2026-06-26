# RustPad

[English](#english) · [中文](#中文)

[![Release](https://img.shields.io/github/v/release/joyoki/rustpad)](https://github.com/joyoki/rustpad/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

A modern, cross-platform code editor built with Rust, inspired by Notepad++ with integrated Beyond Compare-style file diff capabilities.

基于 Rust 的现代化跨平台文本/代码编辑器，灵感来自 Notepad++，并集成 Beyond Compare 风格的文件对比能力。

**Latest release / 最新版本:** [v0.1.5](https://github.com/joyoki/rustpad/releases/tag/v0.1.5)

---

## English

### Download

Pre-built binaries are available on [GitHub Releases](https://github.com/joyoki/rustpad/releases/latest):

| Platform | File |
|----------|------|
| Linux x86_64 | `rustpad-v0.1.5-linux-x86_64.tar.gz` |
| Windows x86_64 | `rustpad-v0.1.5-windows-x86_64.zip` |
| macOS | `rustpad-v0.1.5-macos.app.zip`, `rustpad-v0.1.5-macos.dmg` |

Extract and run the `rustpad` binary (or open `RustPad.app` on macOS).

### Features

#### Core Editing
- Multi-tab text editing with unlimited undo/redo
- Syntax highlighting via syntect (supports 100+ languages)
- Smart auto-indentation
- Code folding with gutter icons and scope guides
- Autocomplete engine
- Macro recording and playback
- Line numbers with current line highlight
- Minimap overview
- Split view (horizontal/vertical)
- Configurable word wrap
- Right-click color marks (background highlights)

#### File Diff (Beyond Compare / Notepad-- style)
- **Detached compare windows** for file, folder, and binary comparison
- Side-by-side diff view with synchronized scrolling and virtualized rows
- Line-level and character-level diff highlighting with merge arrows
- Three diff algorithms: Myers, Patience, LCS
- Ignore options: whitespace, case, line endings; strict mode
- Folder comparison with deep content mode and sync utilities
- Binary byte-level comparison view
- Export HTML diff reports; diff map strip and change navigation
- Per-line inline editing with undo in compare view

#### Search
- Find and Replace with regex support
- Cross-file search (Find in Files)
- Command palette (Ctrl+Shift+P)

#### File System
- File explorer sidebar (tree view)
- Encoding detection and conversion (UTF-8, UTF-16, Latin-1, etc.)
- BOM handling
- Line ending normalization (LF/CRLF/CR)
- Auto-save with crash recovery
- Session persistence

#### Customization
- 4 built-in themes: Dark, Light, Monokai, Solarized Dark
- Custom theme support (JSON format in `~/.config/rustpad/themes/`)
- Configurable keybindings with Notepad++ and VS Code presets
- Font family and size settings
- All settings persisted to `~/.config/rustpad/config.toml`

#### Cross-Platform
- macOS: native menu bar, .app bundle support
- Windows: high DPI support, file association registration
- Linux: full support via egui/eframe

### Building

#### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- macOS: Xcode Command Line Tools
- Windows: Visual Studio Build Tools

#### Quick Build

```bash
git clone https://github.com/joyoki/rustpad.git
cd rustpad
cargo build --release
cargo run --release
```

#### macOS packaging

```bash
chmod +x scripts/build.sh
./scripts/build.sh
```

#### Windows packaging

```powershell
.\scripts\build.ps1
```

### Keyboard Shortcuts (default)

| Action | Shortcut |
|--------|----------|
| New Tab | Ctrl+N |
| Open File | Ctrl+O |
| Save | Ctrl+S |
| Find | Ctrl+F |
| Replace | Ctrl+H |
| Command Palette | Ctrl+Shift+P |

See the full table in the [中文](#键盘快捷键默认) section below or source docs.

### Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Run `cargo clippy -- -D warnings` and `cargo test`
4. Submit a pull request

### License

MIT License. See [LICENSE](LICENSE) for details.

---

## 中文

### 下载

预编译包见 [GitHub Releases](https://github.com/joyoki/rustpad/releases/latest)：

| 平台 | 文件 |
|------|------|
| Linux x86_64 | `rustpad-v0.1.5-linux-x86_64.tar.gz` |
| Windows x86_64 | `rustpad-v0.1.5-windows-x86_64.zip` |
| macOS | `rustpad-v0.1.5-macos.app.zip`、`rustpad-v0.1.5-macos.dmg` |

解压后运行 `rustpad`（macOS 可打开 `RustPad.app`）。

### 功能特性

#### 核心编辑
- 多标签页编辑，支持无限撤销/重做
- 基于 syntect 的语法高亮（100+ 语言）
- 智能自动缩进
- 代码折叠（折叠栏图标 + 作用域竖线）
- 自动补全引擎
- 宏录制与回放
- 行号与当前行高亮
- 小地图（Minimap）概览
- 分屏视图（水平/垂直）
- 可配置自动换行
- 右键颜色标记（文字背景色）

#### 文件对比（Beyond Compare / Notepad-- 风格）
- **独立对比窗口**：文件、文件夹、二进制对比
- 左右并排对比，同步滚动，大文件虚拟化行渲染
- 行级与字符级差异高亮，支持合并按钮（▶/◀）
- 三种 Diff 算法：Myers、Patience、LCS
- 可忽略空白、大小写、换行符；严格模式
- 文件夹深度对比与同步工具
- 二进制字节级对比视图
- 导出 HTML 对比报告；差异图条与变更导航
- 对比视图中支持行内编辑与撤销

#### 搜索
- 查找与替换（支持正则）
- 跨文件搜索（在文件中查找）
- 命令面板（Ctrl+Shift+P）

#### 文件系统
- 侧边栏文件树
- 编码检测与转换（UTF-8、UTF-16、Latin-1 等）
- BOM 处理
- 换行符规范化（LF/CRLF/CR）
- 自动保存与崩溃恢复
- 会话持久化

#### 个性化
- 4 套内置主题：Dark、Light、Monokai、Solarized Dark
- 自定义主题（`~/.config/rustpad/themes/` 下的 JSON）
- 可配置快捷键（Notepad++ / VS Code 预设）
- 字体与字号设置
- 配置保存在 `~/.config/rustpad/config.toml`

#### 跨平台
- macOS：原生菜单栏、`.app` 打包
- Windows：高 DPI、文件关联
- Linux：通过 egui/eframe 完整支持

### 构建

#### 环境要求

- Rust 1.75+（[rustup](https://rustup.rs/)）
- macOS：Xcode Command Line Tools
- Windows：Visual Studio Build Tools

#### 快速构建

```bash
git clone https://github.com/joyoki/rustpad.git
cd rustpad
cargo build --release
cargo run --release
```

#### macOS 打包

```bash
chmod +x scripts/build.sh
./scripts/build.sh
```

#### Windows 打包

```powershell
.\scripts\build.ps1
```

### 键盘快捷键（默认）

| 操作 | 快捷键 |
|------|--------|
| 新建标签 | Ctrl+N |
| 打开文件 | Ctrl+O |
| 保存 | Ctrl+S |
| 另存为 | Ctrl+Shift+S |
| 关闭标签 | Ctrl+W |
| 撤销 | Ctrl+Z |
| 重做 | Ctrl+Y |
| 剪切 | Ctrl+X |
| 复制 | Ctrl+C |
| 粘贴 | Ctrl+V |
| 全选 | Ctrl+A |
| 查找 | Ctrl+F |
| 替换 | Ctrl+H |
| 转到行 | Ctrl+G |
| 在文件中查找 | Ctrl+Shift+F |
| 对比文件 | Ctrl+D |
| 切换侧边栏 | Ctrl+B |
| 命令面板 | Ctrl+Shift+P |
| 下一标签 | Ctrl+Tab |

### 自定义主题

在 `~/.config/rustpad/themes/` 创建 JSON 主题文件，颜色格式为 `[R, G, B, A]`（0–255）。示例：

```json
{
  "name": "My Theme",
  "background": [30, 30, 30, 255],
  "foreground": [212, 212, 212, 255],
  "selection_bg": [38, 79, 120, 200],
  "cursor_color": [212, 212, 212, 255]
}
```

### 配置

主配置文件：`~/.config/rustpad/config.toml`

```toml
[editor]
font_size = 14.0
tab_size = 4
show_line_numbers = true
word_wrap = false
auto_indent = true
highlight_current_line = true

[ui]
theme = "Dark"
show_minimap = true
keybinding_scheme = "NotepadPP"
```

### 调试

```bash
RUSTPAD_LOG=debug cargo run
```

崩溃日志：`~/.config/rustpad/crash.log`

### 参与贡献

1. Fork 本仓库
2. 创建功能分支（`git checkout -b feature/my-feature`）
3. 确保 `cargo clippy -- -D warnings` 与 `cargo test` 通过
4. 提交 Pull Request

### 许可证

MIT License，详见 [LICENSE](LICENSE)。

---

## Architecture / 架构

```
src/
├── main.rs              # Entry point / 入口
├── app.rs               # Application state / 应用状态
├── config/              # Config & themes / 配置与主题
├── diff/                # Diff engine / 对比引擎
├── editor/              # Editor core / 编辑器核心
├── highlight/           # Syntax highlighting / 语法高亮
├── platform/            # Platform-specific / 平台相关
├── search/              # Search / 搜索
├── session/             # Session & auto-save / 会话与自动保存
└── ui/                  # egui UI components / 界面组件
    ├── compare_session.rs   # Compare state / 对比会话
    ├── compare_window.rs    # Compare window UI / 对比窗口
    └── compare_viewport.rs  # Detached viewports / 独立视口
```

## Releases / 发布说明

See [CHANGELOG.md](CHANGELOG.md) and [GitHub Releases](https://github.com/joyoki/rustpad/releases).

变更记录见 [CHANGELOG.md](CHANGELOG.md) 与 [GitHub Releases](https://github.com/joyoki/rustpad/releases)。

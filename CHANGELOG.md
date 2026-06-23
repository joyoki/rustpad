# Changelog

[English](#english) · [中文](#中文)

All notable changes to RustPad are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

本文件记录 RustPad 的所有重要变更。

---

## English

### [0.1.2] - 2026-06-23

#### Added

- **Code folding gutter** (Notepad++ / IDE style): yellow fold icons with `−` / `+`, vertical scope guides, and click-to-toggle in the gutter between line numbers and the editor.
- **Automatic fold region detection** for braced blocks (`enum`, `struct`, `fn`, `impl`, etc.), with indent-based folding for languages without braces (e.g. Python).
- **Text background color marks** via the right-click menu; gutter color stripes on all lines covered by a mark.
- **Clipboard mark preservation**: copy/cut/paste keeps and remaps color marks with the text.
- **Keyboard edit mark tracking**: typing, Enter, Tab, Backspace, and Delete update color marks in sync with buffer edits.

#### Fixed

- **Code folding** no longer fails silently: folded lines use a visible-line layout so blocks actually collapse; right-click fold uses the click position.
- **Fold detection** ignores `{` / `}` inside strings and line comments; line indices align with the text buffer (trailing newlines no longer desync folds).
- **Syntax highlighting** always advances the syntect parser state when using the line cache, fixing broken function/keyword coloring on later lines.
- **Highlight/fold line splitting** matches `TextBuffer` line boundaries.
- **Right-click context menu**: copy/cut actions dismiss the menu correctly; selection is snapshotted for menu operations.
- **Color mark alignment**: mark backgrounds and line-number gutter stripes share the same Y-axis layout.
- **Log branding** displays `RustPad` (capital P) instead of `rustpad`.

#### Changed

- Context menu **Fold Current** toggles the block at the cursor (consistent with gutter clicks).
- Open-sourced on GitHub: [joyoki/rustpad](https://github.com/joyoki/rustpad).

### [0.1.1] - 2026-06-23

#### Added

- Initial public release baseline: multi-tab editor, syntax highlighting, diff view, search, themes, and cross-platform packaging scripts.

---

## 中文

### [0.1.2] - 2026-06-23

#### 新增

- **代码折叠栏**（Notepad++ / IDE 风格）：行号与编辑区之间的折叠栏，黄色 `−` / `+` 图标、竖向作用域线，点击即可折叠/展开。
- **自动识别可折叠区域**：支持 `{ ... }` 代码块（`enum`、`struct`、`fn`、`impl` 等）；无花括号语言支持缩进折叠（如 Python）。
- **文字背景色标记**：右键菜单设置颜色标记；行号区在标记覆盖的每一行显示色条。
- **剪贴板标记保留**：复制/剪切/粘贴时颜色标记随文本一起保留并重映射。
- **键盘编辑标记跟踪**：输入、回车、Tab、退格、删除时同步更新颜色标记。

#### 修复

- **代码折叠**不再“假折叠”：折叠后按可见行布局渲染，块内行真正收起；右键折叠使用点击位置。
- **折叠检测**忽略字符串与行注释中的 `{` / `}`；行号与文本缓冲区对齐（末尾换行不再导致错位）。
- **语法高亮**使用行缓存时仍推进 syntect 解析状态，修复后续行函数名/关键字着色错误。
- **高亮/折叠行切分**与 `TextBuffer` 行边界一致。
- **右键菜单**：复制/剪切后菜单正常关闭；操作时快照选区。
- **颜色标记对齐**：标记背景与行号色条使用同一 Y 轴布局。
- **日志品牌**显示为 `RustPad`（大写 P），而非 `rustpad`。

#### 变更

- 右键菜单 **折叠当前** 改为切换当前块折叠状态（与点击折叠栏一致）。
- 已在 GitHub 开源：[joyoki/rustpad](https://github.com/joyoki/rustpad)。

### [0.1.1] - 2026-06-23

#### 新增

- 首个公开发布基线：多标签编辑、语法高亮、Diff 视图、搜索、主题与跨平台打包脚本。

---

[0.1.2]: https://github.com/joyoki/rustpad/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/joyoki/rustpad/releases/tag/v0.1.1

# Changelog

All notable changes to RustPad are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-06-23

### Added

- **Code folding gutter** (Notepad++ / IDE style): yellow fold icons with `−` / `+`, vertical scope guides, and click-to-toggle in the gutter between line numbers and the editor.
- **Automatic fold region detection** for braced blocks (`enum`, `struct`, `fn`, `impl`, etc.), with indent-based folding for languages without braces (e.g. Python).
- **Text background color marks** via the right-click menu; gutter color stripes on all lines covered by a mark.
- **Clipboard mark preservation**: copy/cut/paste keeps and remaps color marks with the text.
- **Keyboard edit mark tracking**: typing, Enter, Tab, Backspace, and Delete update color marks in sync with buffer edits.

### Fixed

- **Code folding** no longer fails silently: folded lines use a visible-line layout so blocks actually collapse; right-click fold uses the click position.
- **Fold detection** ignores `{` / `}` inside strings and line comments; line indices align with the text buffer (trailing newlines no longer desync folds).
- **Syntax highlighting** always advances the syntect parser state when using the line cache, fixing broken function/keyword coloring on later lines.
- **Highlight/fold line splitting** matches `TextBuffer` line boundaries.
- **Right-click context menu**: copy/cut actions dismiss the menu correctly; selection is snapshotted for menu operations.
- **Color mark alignment**: mark backgrounds and line-number gutter stripes share the same Y-axis layout.
- **Log branding** displays `RustPad` (capital P) instead of `rustpad`.

### Changed

- Context menu **Fold Current** toggles the block at the cursor (consistent with gutter clicks).
- Open-sourced on GitHub: [joyoki/rustpad](https://github.com/joyoki/rustpad).

---

## [0.1.1] - 2026-06-23

### Added

- Initial public release baseline: multi-tab editor, syntax highlighting, diff view, search, themes, and cross-platform packaging scripts.

[0.1.2]: https://github.com/joyoki/rustpad/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/joyoki/rustpad/releases/tag/v0.1.1

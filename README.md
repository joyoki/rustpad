# RustPad

A modern, cross-platform code editor built with Rust, inspired by Notepad++ with integrated Beyond Compare-style file diff capabilities.

## Features

### Core Editing
- Multi-tab text editing with unlimited undo/redo
- Syntax highlighting via syntect (supports 100+ languages)
- Smart auto-indentation
- Code folding
- Autocomplete engine
- Macro recording and playback
- Line numbers with current line highlight
- Minimap overview
- Split view (horizontal/vertical)
- Configurable word wrap

### File Diff (Beyond Compare style)
- Side-by-side diff view with synchronized scrolling
- Line-level, word-level, and character-level diff highlighting
- Three diff algorithms: Myers, Patience, LCS
- Ignore options: whitespace, case, line endings
- Folder comparison with status indicators
- Three-way merge with conflict detection
- Export HTML diff reports
- Diff navigation (heatmap, prev/next, jump to Nth)

### Search
- Find and Replace with regex support
- Cross-file search (Find in Files)
- Command palette (Ctrl+Shift+P)

### File System
- File explorer sidebar (tree view)
- Encoding detection and conversion (UTF-8, UTF-16, Latin-1, etc.)
- BOM handling
- Line ending normalization (LF/CRLF/CR)
- Auto-save with crash recovery
- Session persistence

### Customization
- 4 built-in themes: Dark, Light, Monokai, Solarized Dark
- Custom theme support (JSON format in `~/.config/rustpad/themes/`)
- Configurable keybindings with Notepad++ and VS Code presets
- Font family and size settings
- All settings persisted to `~/.config/rustpad/config.toml`

### Cross-Platform
- macOS: native menu bar, .app bundle support
- Windows: high DPI support, file association registration
- Linux: full support via egui/eframe

## Building

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- macOS: Xcode Command Line Tools
- Windows: Visual Studio Build Tools

### Quick Build

```bash
# Clone
git clone https://github.com/rustpad/rustpad.git
cd rustpad

# Debug build
cargo build

# Release build
cargo build --release

# Run
cargo run --release
```

### Platform-Specific Packaging

**macOS:**
```bash
# Install cargo-bundle (optional, for .app bundle)
cargo install cargo-bundle

# Build script (creates .app and .dmg)
chmod +x scripts/build.sh
./scripts/build.sh
```

**Windows:**
```powershell
# Install cargo-bundle (optional, for MSI)
cargo install cargo-bundle

# Build script
.\scripts\build.ps1
```

### Using cargo-bundle directly

```bash
cargo bundle --release
```

Output:
- macOS: `target/release/bundle/osx/RustPad.app`
- Windows: `target/release/bundle/msi/`

## Keyboard Shortcuts

### Default (Notepad++ Compatible)

| Action | Shortcut |
|--------|----------|
| New Tab | Ctrl+N |
| Open File | Ctrl+O |
| Save | Ctrl+S |
| Save As | Ctrl+Shift+S |
| Close Tab | Ctrl+W |
| Undo | Ctrl+Z |
| Redo | Ctrl+Y |
| Cut | Ctrl+X |
| Copy | Ctrl+C |
| Paste | Ctrl+V |
| Select All | Ctrl+A |
| Find | Ctrl+F |
| Replace | Ctrl+H |
| Go to Line | Ctrl+G |
| Find in Files | Ctrl+Shift+F |
| Compare Files | Ctrl+D |
| Toggle Sidebar | Ctrl+B |
| Command Palette | Ctrl+Shift+P |
| Next Tab | Ctrl+Tab |

### Custom Keybindings

Create `~/.config/rustpad/keybindings.json` to customize shortcuts:

```json
{
  "scheme": "VSCode",
  "bindings": {
    "NewTab": [{"ctrl": true, "shift": false, "alt": false, "key": "N"}],
    "Save": [{"ctrl": true, "shift": false, "alt": false, "key": "S"}]
  }
}
```

## Custom Themes

Create JSON files in `~/.config/rustpad/themes/`:

```json
{
  "name": "My Theme",
  "background": [30, 30, 30, 255],
  "foreground": [212, 212, 212, 255],
  "line_number_bg": [30, 30, 30, 255],
  "line_number_fg": [128, 128, 128, 255],
  "current_line_bg": [40, 40, 40, 255],
  "selection_bg": [38, 79, 120, 200],
  "cursor_color": [212, 212, 212, 255],
  "gutter_bg": [30, 30, 30, 255],
  "sidebar_bg": [37, 37, 38, 255],
  "sidebar_fg": [204, 204, 204, 255],
  "status_bar_bg": [0, 122, 204, 255],
  "status_bar_fg": [255, 255, 255, 255],
  "tab_bar_bg": [45, 45, 48, 255],
  "tab_active_bg": [30, 30, 30, 255],
  "tab_active_fg": [255, 255, 255, 255],
  "tab_inactive_bg": [45, 45, 48, 255],
  "tab_inactive_fg": [160, 160, 160, 255],
  "diff_insert_bg": [228, 255, 228, 255],
  "diff_delete_bg": [255, 228, 228, 255],
  "diff_replace_bg": [255, 251, 228, 255],
  "search_highlight_bg": [255, 255, 0, 128],
  "minimap_bg": [30, 30, 30, 255],
  "scroll_bar_bg": [30, 30, 30, 255],
  "scroll_bar_fg": [121, 121, 121, 255]
}
```

Each color is `[R, G, B, A]` (0-255, RGBA).

## Configuration

All settings are stored in `~/.config/rustpad/config.toml`:

```toml
[editor]
font_size = 14.0
tab_size = 4
show_line_numbers = true
word_wrap = false
auto_indent = true
highlight_current_line = true
font_family = "JetBrains Mono"

[ui]
theme = "Dark"
show_minimap = true
sidebar_width = 220.0
keybinding_scheme = "NotepadPP"

[window]
width = 1280.0
height = 720.0
maximized = false

[auto_save]
enabled = true
interval_seconds = 300
```

## Debugging

Set `RUSTPAD_LOG` environment variable for verbose logging:

```bash
RUSTPAD_LOG=debug cargo run
```

Crash logs are written to `~/.config/rustpad/crash.log`.

## Architecture

```
src/
├── main.rs              # Entry point, window setup
├── app.rs               # Top-level application state
├── config/              # Configuration management + themes
│   ├── mod.rs           # AppConfig (TOML persistence)
│   └── theme.rs         # Theme system (4 built-in + custom)
├── diff/                # Diff engine
│   ├── engine.rs        # Myers/Patience/LCS algorithms
│   ├── folder_diff.rs   # Directory comparison
│   └── three_way.rs     # Three-way merge
├── editor/              # Editor core (pure data logic)
│   ├── buffer.rs        # TextBuffer with SimpleRope + undo
│   ├── cursor.rs        # Cursor and Selection
│   ├── tab.rs           # Tab management
│   ├── file_io.rs       # File I/O with encoding detection
│   ├── fold.rs          # Code folding
│   ├── indent.rs        # Auto-indentation
│   ├── autocomplete.rs  # Autocomplete engine
│   └── macro_recorder.rs # Macro recording
├── highlight/           # Syntax highlighting (syntect)
├── platform/            # Platform-specific code
│   ├── mod.rs           # Panic hook, logging init
│   ├── macos.rs         # macOS specifics
│   └── windows.rs       # Windows specifics
├── plugin/              # Plugin API (skeleton)
├── search/              # Search engine
├── session/             # Session persistence + auto-save
└── ui/                  # UI components (egui)
    ├── keybindings.rs   # Configurable keybinding system
    ├── menu.rs          # Menu bar
    ├── toolbar.rs       # Toolbar
    ├── editor_widget.rs # Core editor widget
    ├── diff_view.rs     # Diff view
    ├── file_tree.rs     # File explorer
    ├── command_palette.rs # Command palette
    └── ...
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Ensure `cargo clippy -- -D warnings` passes
4. Ensure `cargo test` passes
5. Submit a pull request

## License

MIT License. See [LICENSE](LICENSE) for details.

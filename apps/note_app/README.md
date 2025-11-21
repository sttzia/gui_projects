# Note App - Rich Text Editor

A feature-rich note-taking application built with **egui** framework, supporting inline text formatting and style preservation.

## Features

### Text Editing

- **Multiline text editor** with full editing capabilities
- **Mouse text selection** - drag to select text
- **Real-time formatting** - see styles applied directly in the editor
- **Dynamic font sizing** (8-72px) with + and - buttons

### Text Formatting

Apply formatting to selected text:

- **Bold** - Makes text bold with monospace font
- **Italic** - Italicizes text
- **Bold+Italic** - Combines both styles
- **Regular** - Removes all formatting

### File Operations

- **📂 Open** - Open existing `.rtxt` (rich text) or `.txt` (plain text) files
- **💾 Save** - Save to current file, or prompt for location if new
- **💾 Save As...** - Always prompt to save with a new name/location

### Rich Text Format (.rtxt)

- **Custom format** that preserves text and formatting
- **Backward compatible** - can open plain `.txt` files (without formatting)
- Stores styled ranges with position information
- Format structure:
  ```
  TEXT:<your text content>
  STYLES:
  start..end:StyleName
  ```

### User Interface

- **Top menu bar** - File operations and formatting buttons
- **Central editor** - Main text editing area with formatting preview
- **Status bar** - Shows current file path and character count
- **800×600 window** - Comfortable editing space

## Usage

### Running the Application

```bash
cargo run --package note_app
```

### Creating Formatted Notes

1. Type your text in the editor
2. Select text with your mouse (click and drag)
3. Click a formatting button (Bold, Italic, Bold+Italic, or Regular)
4. The formatting is applied immediately to the selected text
5. Click "💾 Save As..." and save as `.rtxt` to preserve formatting

### Opening Files

- Click "📂 Open" to browse for files
- Select `.rtxt` files to open with formatting preserved
- Select `.txt` files to open as plain text (no formatting)

### Adjusting Font Size

- Click "🔍+ Larger" to increase font size
- Click "🔍− Smaller" to decrease font size
- Current font size is displayed in the menu bar

## File Format

### Rich Text (.rtxt)

Custom format that stores both text content and formatting information:

- Line 1: `TEXT:` followed by the text content
- Line 2: `STYLES:` header
- Following lines: Range and style information (`start..end:StyleName`)

Example:

```
TEXT:Hello World! This is formatted text.
STYLES:
0..5:Bold
13..15:Italic
```

### Plain Text (.txt)

Standard text files are supported for opening but will not preserve formatting when saved.

## Technical Details

- **Framework**: egui 0.29 with eframe
- **File Dialogs**: rfd 0.14 for native file picker
- **Language**: Rust (Edition 2021)
- **Formatting System**: Custom styled ranges with position tracking

## Building from Source

```bash
cd apps/note_app
cargo build --release
```

## Workspace Integration

This app is part of the `gui_projects` workspace and uses shared dependencies defined in the root `Cargo.toml`:

- `eframe.workspace = true`
- `egui.workspace = true`
- `rfd.workspace = true`

## License

Part of the gui_projects workspace.

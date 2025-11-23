# Note App - Rich Text Editor

A feature-rich note-taking application built with **egui** framework, supporting inline text formatting, custom colors, and style preservation.

## Features

### Text Editing

- **Multiline text editor** with full editing capabilities
- **Mouse text selection** - drag to select text
- **Real-time formatting** - see styles applied directly in the editor
- **Dynamic font sizing** (8-72px) with + and - buttons
- **Font family selection** - Choose between Monospace, Proportional, or Emoji fonts
- **Undo/Redo** - Track edit history with Ctrl+Z and Ctrl+Y
- **Tab support** - Press **Ctrl+[** to insert 4 spaces for indentation
- **Find & Replace** - Search and replace text with multiple options
  - Find next/previous occurrence
  - Replace current match
  - Replace all occurrences
  - Keyboard shortcut: Ctrl+F to toggle find panel
- **Line numbers** - Optional display on the left margin (toggle with 🔢 button)

### Text Formatting

Apply formatting to selected text:

- **Bold** - Makes text 1.3x larger for emphasis
- **Italic** - Italicizes text
- **Bold+Italic** - Combines both styles
- **Regular** - Removes all formatting

### Color Features ✨

**Special feature for enhanced note-taking:**

- **Custom Text Colors** 🎨

  - Click the color picker button next to "Text Color:" in the toolbar
  - Choose any color from the full RGB spectrum
  - Select text and apply the color instantly
  - Perfect for color-coding notes, highlighting important information, or organizing content by topic

- **Text Highlighting (Background Color)** 🖍️

  - Enable highlighting with the checkbox next to "Highlight:"
  - Click the color button to select a highlight color (defaults to yellow)
  - Apply highlighting to selected text like a real highlighter marker
  - Uncheck the checkbox to disable highlighting
  - Great for marking key passages, creating visual emphasis, or studying

- **Color Persistence**
  - All colors (text and highlights) are saved in `.rtxt` files
  - Colors are preserved when reopening files
  - Each text range can have its own unique color combination

### File Operations

- **📂 Open** - Open existing `.rtxt` (rich text) or `.txt` (plain text) files
- **💾 Save** - Save to current file, or prompt for location if new
- **💾 Save As...** - Always prompt to save with a new name/location

### Rich Text Format (.rtxt)

- **Custom format** that preserves text, formatting, and colors
- **Backward compatible** - can open plain `.txt` files (without formatting)
- Stores styled ranges with position and color information
- Format structure:
  ```
  TEXT:<your text content>
  ---STYLES---
  start..end:StyleName:text_color:bg_color
  ```
  - Colors are stored as `R_G_B_A` format (e.g., `255_0_0_255` for red)
  - `none` indicates no color applied

### User Interface

- **Top menu bar** - File operations, formatting buttons, color pickers, and view options
- **Find & Replace panel** - Appears when activated with 🔍 Find button or Ctrl+F
- **Central editor** - Main text editing area with formatting and color preview
- **Line numbers** (optional) - Displayed on the left when enabled
- **Status bar** - Shows current file path, line count, character count, and tab shortcut
- **1200×1024 window** - Spacious editing area for comfortable note-taking

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

### Using Colors

**Applying Text Colors:**

1. Select text with your mouse
2. Click the color picker button next to "Text Color:" in the toolbar
3. Choose your desired color from the picker
4. The selected text will instantly change to the new color

**Applying Highlights:**

1. Check the checkbox next to "Highlight:" to enable highlighting
2. Click the color button to choose a highlight color (yellow by default)
3. Select text with your mouse
4. The selected text will have the highlight background applied

**Tips:**

- You can combine text colors, highlights, and formatting (Bold/Italic) on the same text
- Colors are only saved in `.rtxt` files - use this format to preserve your color choices
- Reset to default black text by choosing black from the text color picker
- Disable highlighting by unchecking the Highlight checkbox

### Font Selection

- **Font dropdown menu** in the toolbar
- Choose from three font families:
  - **Monospace** - Fixed-width font (default), perfect for code or aligned text
  - **Proportional** - Variable-width font, natural for reading
  - **Emoji** - Proportional font with full emoji support 😊

### Using Undo/Redo

- **Undo**: Click "↶ Undo" button or press **Ctrl+Z**
- **Redo**: Click "↷ Redo" button or press **Ctrl+Y**
- Up to 100 undo states are saved

### Finding and Replacing Text

1. Click "🔍 Find" button or press **Ctrl+F** to open the Find & Replace panel
2. Type text to find in the "Find:" field
3. Click "⬇ Next" to find next occurrence or "⬆ Prev" for previous
4. To replace:
   - Type replacement text in the "Replace:" field
   - Click "Replace" to replace current match
   - Click "Replace All" to replace all occurrences
5. Click "✖ Close" to hide the panel

### View Options

- **Line numbers**: Click "🔢 Show Lines" / "🔢 Hide Lines" to toggle
- **Tab insertion**: Use **Ctrl+[** to insert 4 spaces (displayed in status bar)

### Opening Files

- Click "📂 Open" to browse for files
- Select `.rtxt` files to open with formatting preserved
- Select `.txt` files to open as plain text (no formatting)

### Adjusting Font Size

- Click "🔍+ Larger" to increase font size
- Click "🔍− Smaller" to decrease font size
- Current font size is displayed in the menu bar

### Keyboard Shortcuts

- **Ctrl+Z**: Undo last change
- **Ctrl+Y**: Redo last undone change
- **Ctrl+F**: Toggle Find & Replace panel
- **Ctrl+[**: Insert 4 spaces for indentation
- **Tab**: Navigate between UI controls

## File Format

### Rich Text (.rtxt)

Custom format that stores text content, formatting, and color information:

- Line 1: `TEXT:` followed by the text content
- Line 2: `---STYLES---` separator
- Following lines: Range, style, and color information

Format: `start..end:StyleName:text_color:bg_color`

Where:

- `start..end` - Character position range
- `StyleName` - Bold, Italic, BoldItalic, or Regular
- `text_color` - RGB+Alpha as `R_G_B_A` or `none`
- `bg_color` - RGB+Alpha as `R_G_B_A` or `none`

Example:

```
TEXT:Hello World! This is formatted text.
---STYLES---
0..5:Bold:255_0_0_255:none
13..15:Italic:none:255_255_0_200
```

This example shows "Hello" in red bold text, and "is" in italic with yellow highlighting.

### Plain Text (.txt)

Standard text files are supported for opening but will not preserve formatting when saved.

## Technical Details

- **Framework**: egui 0.29 with eframe
- **File Dialogs**: rfd 0.14 for native file picker
- **Language**: Rust (Edition 2021)
- **Formatting System**: Custom styled ranges with position tracking
- **Color System**: Full RGB color support with alpha channel for text and backgrounds
- **Font System**: Three built-in font families (Monospace, Proportional, Emoji)

## Building from Source

```bash
cd apps/note_app
cargo build --release
```

### Windows Console Behavior

The application is configured to hide the console window in release builds on Windows:

- **Debug builds**: Console window appears for debugging (shows errors and `println!` output)
- **Release builds**: No console window, clean GUI-only launch

This is controlled by `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` in `main.rs`. Linux and macOS are not affected.

## Workspace Integration

This app is part of the `gui_projects` workspace and uses shared dependencies defined in the root `Cargo.toml`:

- `eframe.workspace = true`
- `egui.workspace = true`
- `rfd.workspace = true`

## License

Part of the gui_projects workspace.

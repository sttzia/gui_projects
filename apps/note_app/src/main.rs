#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use egui::{Color32, FontId, TextEdit};
use std::ops::Range;
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 1024.0])
            .with_title("Note App - Rich Text Editor"),
        ..Default::default()
    };
    eframe::run_native(
        "Note App",
        options,
        Box::new(|_cc| Ok(Box::<NoteApp>::default())),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TextFormatting {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

#[derive(Clone, Debug)]
struct StyledRange {
    range: Range<usize>,
    style: TextFormatting,
    text_color: Option<Color32>,
    bg_color: Option<Color32>,
}

#[derive(Clone, Debug)]
struct EditorState {
    text_content: String,
    styled_ranges: Vec<StyledRange>,
}

struct NoteApp {
    text_content: String,
    styled_ranges: Vec<StyledRange>,
    file_path: Option<PathBuf>,
    error_message: Option<String>,
    current_style: TextFormatting,
    cursor_range: Option<Range<usize>>,
    font_size: f32,
    // Undo/Redo
    undo_stack: Vec<EditorState>,
    redo_stack: Vec<EditorState>,
    // Find & Replace
    find_text: String,
    replace_text: String,
    show_find_replace: bool,
    last_find_position: usize,
    // Display options
    show_line_numbers: bool,
    tab_width: usize,
    font_family: String,
    // Color options
    current_text_color: Color32,
    current_bg_color: Option<Color32>,
    // Pending cursor position after programmatic text changes
    pending_cursor_pos: Option<usize>,
    // Flag to prevent cursor capture when programmatically setting selection
    skip_cursor_capture: bool,
}

impl Default for NoteApp {
    fn default() -> Self {
        Self {
            text_content: String::new(),
            styled_ranges: Vec::new(),
            file_path: None,
            error_message: None,
            current_style: TextFormatting::Regular,
            cursor_range: None,
            font_size: 16.0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            find_text: String::new(),
            replace_text: String::new(),
            show_find_replace: false,
            last_find_position: 0,
            show_line_numbers: true,
            tab_width: 4,
            font_family: "Monospace".to_string(),
            current_text_color: Color32::BLACK,
            current_bg_color: None,
            pending_cursor_pos: None,
            skip_cursor_capture: false,
        }
    }
}

impl NoteApp {
    fn save_with_formatting(&self, path: &PathBuf) -> Result<(), String> {
        // Check file extension
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        if extension == "rtxt" {
            // Save with formatting metadata for .rtxt files
            let mut content = String::new();
            content.push_str("TEXT:\n");
            content.push_str(&self.text_content);
            content.push_str("\n---STYLES---\n");
            for styled_range in &self.styled_ranges {
                let style_name = match styled_range.style {
                    TextFormatting::Regular => "Regular",
                    TextFormatting::Bold => "Bold",
                    TextFormatting::Italic => "Italic",
                    TextFormatting::BoldItalic => "BoldItalic",
                };

                // Format: start..end:style:text_color:bg_color
                let text_color_str = if let Some(color) = styled_range.text_color {
                    format!("{}_{}_{}_{}", color.r(), color.g(), color.b(), color.a())
                } else {
                    "none".to_string()
                };

                let bg_color_str = if let Some(color) = styled_range.bg_color {
                    format!("{}_{}_{}_{}", color.r(), color.g(), color.b(), color.a())
                } else {
                    "none".to_string()
                };

                content.push_str(&format!(
                    "{}..{}:{}:{}:{}\n",
                    styled_range.range.start,
                    styled_range.range.end,
                    style_name,
                    text_color_str,
                    bg_color_str
                ));
            }
            std::fs::write(path, content).map_err(|e| format!("Error saving file: {}", e))
        } else {
            // Save plain text exactly as-is for .txt and other files
            std::fs::write(path, &self.text_content)
                .map_err(|e| format!("Error saving file: {}", e))
        }
    }

    fn load_with_formatting(&mut self, path: &PathBuf) -> Result<(), String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Error reading file: {}", e))?;

        // Check if it's the new format with TEXT: header
        if content.starts_with("TEXT:\n") {
            // New format - find the separator
            if let Some(separator_pos) = content.find("\n---STYLES---\n") {
                // Extract text content (skip "TEXT:\n")
                self.text_content = content[6..separator_pos].to_string();

                // Extract styles section
                let styles_section = &content[separator_pos + 14..];
                self.styled_ranges.clear();

                for line in styles_section.lines() {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 2 {
                        let range_part = parts[0];
                        let style_part = parts[1];
                        let text_color_part = parts.get(2).copied();
                        let bg_color_part = parts.get(3).copied();

                        if let Some((start_str, end_str)) = range_part.split_once("..") {
                            if let (Ok(start), Ok(end)) =
                                (start_str.parse::<usize>(), end_str.parse::<usize>())
                            {
                                let style = match style_part {
                                    "Bold" => TextFormatting::Bold,
                                    "Italic" => TextFormatting::Italic,
                                    "BoldItalic" => TextFormatting::BoldItalic,
                                    _ => TextFormatting::Regular,
                                };

                                // Parse text color
                                let text_color = if let Some(color_str) = text_color_part {
                                    if color_str != "none" {
                                        let rgba: Vec<&str> = color_str.split('_').collect();
                                        if rgba.len() == 4 {
                                            if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
                                                rgba[0].parse::<u8>(),
                                                rgba[1].parse::<u8>(),
                                                rgba[2].parse::<u8>(),
                                                rgba[3].parse::<u8>(),
                                            ) {
                                                Some(Color32::from_rgba_unmultiplied(r, g, b, a))
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };

                                // Parse background color
                                let bg_color = if let Some(color_str) = bg_color_part {
                                    if color_str != "none" {
                                        let rgba: Vec<&str> = color_str.split('_').collect();
                                        if rgba.len() == 4 {
                                            if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
                                                rgba[0].parse::<u8>(),
                                                rgba[1].parse::<u8>(),
                                                rgba[2].parse::<u8>(),
                                                rgba[3].parse::<u8>(),
                                            ) {
                                                Some(Color32::from_rgba_unmultiplied(r, g, b, a))
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };

                                self.styled_ranges.push(StyledRange {
                                    range: start..end,
                                    style,
                                    text_color,
                                    bg_color,
                                });
                            }
                        }
                    }
                }
            } else {
                // No separator found, just use the text
                self.text_content = content[6..].to_string();
                self.styled_ranges.clear();
            }
        } else {
            // Old format - plain text file
            self.text_content = content;
            self.styled_ranges.clear();
        }

        Ok(())
    }

    fn apply_style_to_selection(&mut self) {
        if let Some(range) = self.cursor_range.clone() {
            if range.start < range.end {
                // Save state before modification
                self.save_state_for_undo();

                // Remove overlapping ranges
                self.styled_ranges
                    .retain(|r| r.range.end <= range.start || r.range.start >= range.end);

                // Add new styled range
                self.styled_ranges.push(StyledRange {
                    range: range.clone(),
                    style: self.current_style,
                    text_color: if self.current_text_color != Color32::BLACK {
                        Some(self.current_text_color)
                    } else {
                        None
                    },
                    bg_color: self.current_bg_color,
                });

                // Sort ranges by start position
                self.styled_ranges.sort_by_key(|r| r.range.start);
            }
        }
    }

    // Undo/Redo functionality
    fn save_state_for_undo(&mut self) {
        let state = EditorState {
            text_content: self.text_content.clone(),
            styled_ranges: self.styled_ranges.clone(),
        };
        self.undo_stack.push(state);
        self.redo_stack.clear(); // Clear redo stack when new change is made

        // Limit undo stack to 100 states
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
    }

    fn undo(&mut self) {
        if let Some(state) = self.undo_stack.pop() {
            // Save current state to redo stack
            let current = EditorState {
                text_content: self.text_content.clone(),
                styled_ranges: self.styled_ranges.clone(),
            };
            self.redo_stack.push(current);

            // Restore previous state
            self.text_content = state.text_content;
            self.styled_ranges = state.styled_ranges;
        }
    }

    fn redo(&mut self) {
        if let Some(state) = self.redo_stack.pop() {
            // Save current state to undo stack
            let current = EditorState {
                text_content: self.text_content.clone(),
                styled_ranges: self.styled_ranges.clone(),
            };
            self.undo_stack.push(current);

            // Restore redone state
            self.text_content = state.text_content;
            self.styled_ranges = state.styled_ranges;
        }
    }

    // Find & Replace functionality
    fn find_next(&mut self) {
        if self.find_text.is_empty() {
            return;
        }

        if let Some(pos) = self.text_content[self.last_find_position..].find(&self.find_text) {
            let actual_pos = self.last_find_position + pos;
            self.cursor_range = Some(actual_pos..actual_pos + self.find_text.len());
            self.last_find_position = actual_pos + 1;
            // Set pending cursor to the end of found text for visual feedback
            self.pending_cursor_pos = Some(actual_pos + self.find_text.len());
            self.skip_cursor_capture = true;
        } else {
            // Wrap around to beginning
            self.last_find_position = 0;
            if let Some(pos) = self.text_content.find(&self.find_text) {
                self.cursor_range = Some(pos..pos + self.find_text.len());
                self.last_find_position = pos + 1;
                self.pending_cursor_pos = Some(pos + self.find_text.len());
                self.skip_cursor_capture = true;
            }
        }
    }

    fn find_previous(&mut self) {
        if self.find_text.is_empty() {
            return;
        }

        let search_end = if self.last_find_position > 0 {
            self.last_find_position - 1
        } else {
            self.text_content.len()
        };

        if let Some(pos) = self.text_content[..search_end].rfind(&self.find_text) {
            self.cursor_range = Some(pos..pos + self.find_text.len());
            self.last_find_position = pos;
            self.pending_cursor_pos = Some(pos + self.find_text.len());
            self.skip_cursor_capture = true;
        } else {
            // Wrap around to end
            if let Some(pos) = self.text_content.rfind(&self.find_text) {
                self.cursor_range = Some(pos..pos + self.find_text.len());
                self.last_find_position = pos;
                self.pending_cursor_pos = Some(pos + self.find_text.len());
                self.skip_cursor_capture = true;
            }
        }
    }

    fn replace_current(&mut self) {
        let range = self.cursor_range.clone();
        if let Some(range) = range {
            if range.start < range.end && range.end <= self.text_content.len() {
                self.save_state_for_undo();

                let selected_text = &self.text_content[range.clone()];
                if selected_text == self.find_text {
                    self.text_content
                        .replace_range(range.clone(), &self.replace_text);

                    // Adjust styled ranges
                    let diff = self.replace_text.len() as i32 - self.find_text.len() as i32;
                    for styled_range in &mut self.styled_ranges {
                        if styled_range.range.start >= range.end {
                            styled_range.range.start =
                                (styled_range.range.start as i32 + diff).max(0) as usize;
                            styled_range.range.end =
                                (styled_range.range.end as i32 + diff).max(0) as usize;
                        }
                    }

                    self.find_next();
                }
            }
        }
    }

    fn replace_all(&mut self) {
        if self.find_text.is_empty() {
            return;
        }

        self.save_state_for_undo();

        let mut count = 0;
        while self.text_content.contains(&self.find_text) {
            self.text_content = self
                .text_content
                .replacen(&self.find_text, &self.replace_text, 1);
            count += 1;
        }

        if count > 0 {
            // Clear styled ranges when replacing all (simpler than adjusting all)
            self.styled_ranges.clear();
            self.error_message = Some(format!("Replaced {} occurrence(s)", count));
        }
    }

    fn render_rich_text_editable(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let styled_ranges = self.styled_ranges.clone();
        let font_size = self.font_size;
        let font_family = self.font_family.clone();

        let mut layouter = move |ui: &egui::Ui, text: &str, wrap_width: f32| {
            let mut layout_job = egui::text::LayoutJob::default();
            layout_job.wrap.max_width = wrap_width;

            let mut current_pos = 0;
            while current_pos < text.len() {
                // Find next style change
                let mut next_change = text.len();
                for styled_range in &styled_ranges {
                    if styled_range.range.start > current_pos
                        && styled_range.range.start < next_change
                    {
                        next_change = styled_range.range.start;
                    }
                    if styled_range.range.end > current_pos && styled_range.range.end < next_change
                    {
                        next_change = styled_range.range.end;
                    }
                }

                let end = next_change.min(text.len());
                let segment = &text[current_pos..end];

                // Determine style for this position
                let mut style = TextFormatting::Regular;
                let mut text_color: Option<Color32> = None;
                let mut bg_color: Option<Color32> = None;

                for styled_range in &styled_ranges {
                    if styled_range.range.contains(&current_pos) {
                        style = styled_range.style;
                        text_color = styled_range.text_color;
                        bg_color = styled_range.bg_color;
                        break;
                    }
                }

                // Create text format based on style
                let get_font_id = |size: f32| {
                    if font_family == "Proportional" || font_family == "Emoji" {
                        FontId::proportional(size)
                    } else {
                        FontId::monospace(size)
                    }
                };

                let base_color = text_color.unwrap_or(Color32::BLACK);

                let mut format = match style {
                    TextFormatting::Regular => egui::TextFormat {
                        font_id: get_font_id(font_size),
                        color: base_color,
                        ..Default::default()
                    },
                    TextFormatting::Bold => egui::TextFormat {
                        font_id: get_font_id(font_size * 1.3),
                        color: base_color,
                        ..Default::default()
                    },
                    TextFormatting::Italic => egui::TextFormat {
                        font_id: get_font_id(font_size),
                        color: base_color,
                        italics: true,
                        ..Default::default()
                    },
                    TextFormatting::BoldItalic => egui::TextFormat {
                        font_id: get_font_id(font_size * 1.3),
                        color: base_color,
                        italics: true,
                        ..Default::default()
                    },
                };

                // Apply background color if specified
                if let Some(bg) = bg_color {
                    format.background = bg;
                }

                layout_job.append(segment, 0.0, format);
                current_pos = end;
            }

            ui.fonts(|f| f.layout_job(layout_job))
        };

        let response = ui.add(
            TextEdit::multiline(&mut self.text_content)
                .desired_width(f32::INFINITY)
                .desired_rows(10)
                .layouter(&mut layouter),
        );

        // Capture cursor selection
        if let Some(mut state) = TextEdit::load_state(ui.ctx(), response.id) {
            // If we have a pending cursor position, set it now
            if let Some(pending_pos) = self.pending_cursor_pos.take() {
                use egui::text::{CCursor, CCursorRange};
                // Check if we have a selection range (from Find operation)
                if let Some(range) = &self.cursor_range {
                    if range.start < range.end {
                        // Set selection from start to end
                        let start_cursor = CCursor::new(range.start);
                        let end_cursor = CCursor::new(range.end);
                        state
                            .cursor
                            .set_char_range(Some(CCursorRange::two(start_cursor, end_cursor)));
                    } else {
                        // Single cursor position
                        let ccursor = CCursor::new(pending_pos);
                        state
                            .cursor
                            .set_char_range(Some(CCursorRange::one(ccursor)));
                    }
                } else {
                    // Single cursor position
                    let ccursor = CCursor::new(pending_pos);
                    state
                        .cursor
                        .set_char_range(Some(CCursorRange::one(ccursor)));
                }

                state.store(ui.ctx(), response.id);
                // Request focus on the text editor to ensure selection is visible
                response.request_focus();
                return response;
            }

            // Only capture cursor position if we're not programmatically setting it
            if !self.skip_cursor_capture {
                let cursor_range = state.cursor.char_range();
                if let Some(range) = cursor_range {
                    let start = range.primary.index.min(range.secondary.index);
                    let end = range.primary.index.max(range.secondary.index);
                    self.cursor_range = Some(start..end);
                }
            }

            // Reset flag after applying
            self.skip_cursor_capture = false;

            state.store(ui.ctx(), response.id);
        }

        response
    }
}

impl eframe::App for NoteApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard shortcuts
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Z)) {
            self.undo();
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Y)) {
            self.redo();
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::F)) {
            self.show_find_replace = !self.show_find_replace;
        }

        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                // File operations
                if ui.button("📂 Open").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Rich Text", &["rtxt"])
                        .add_filter("Plain Text", &["txt"])
                        .pick_file()
                    {
                        match self.load_with_formatting(&path) {
                            Ok(_) => {
                                self.file_path = Some(path);
                                self.error_message = None;
                            }
                            Err(e) => self.error_message = Some(e),
                        }
                    }
                }

                if ui.button("💾 Save").clicked() {
                    let path_option = if let Some(path) = &self.file_path {
                        Some(path.clone())
                    } else {
                        rfd::FileDialog::new()
                            .add_filter("Rich Text", &["rtxt"])
                            .set_file_name("untitled.rtxt")
                            .save_file()
                    };

                    if let Some(path) = path_option {
                        match self.save_with_formatting(&path) {
                            Ok(_) => {
                                self.file_path = Some(path);
                                self.error_message = None;
                            }
                            Err(e) => self.error_message = Some(e),
                        }
                    }
                }

                if ui.button("💾 Save As...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Rich Text", &["rtxt"])
                        .set_file_name("untitled.rtxt")
                        .save_file()
                    {
                        match self.save_with_formatting(&path) {
                            Ok(_) => {
                                self.file_path = Some(path);
                                self.error_message = None;
                            }
                            Err(e) => self.error_message = Some(e),
                        }
                    }
                }

                ui.separator();

                // Undo/Redo
                if ui.button("↶ Undo").clicked()
                    || ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Z))
                {
                    self.undo();
                }
                if ui.button("↷ Redo").clicked()
                    || ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Y))
                {
                    self.redo();
                }

                ui.separator();

                // Formatting
                if ui.button("Bold").clicked() {
                    self.current_style = TextFormatting::Bold;
                    self.apply_style_to_selection();
                }
                if ui.button("Italic").clicked() {
                    self.current_style = TextFormatting::Italic;
                    self.apply_style_to_selection();
                }
                if ui.button("Bold+Italic").clicked() {
                    self.current_style = TextFormatting::BoldItalic;
                    self.apply_style_to_selection();
                }
                if ui.button("Regular").clicked() {
                    self.current_style = TextFormatting::Regular;
                    self.apply_style_to_selection();
                }

                ui.separator();

                // Font size
                if ui.button("🔍+ Larger").clicked() {
                    self.font_size = (self.font_size + 2.0).min(72.0);
                }
                if ui.button("🔍− Smaller").clicked() {
                    self.font_size = (self.font_size - 2.0).max(8.0);
                }
                ui.label(format!("{:.0}px", self.font_size));

                ui.separator();

                // Font family
                egui::ComboBox::from_label("Font")
                    .selected_text(&self.font_family)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.font_family,
                            "Monospace".to_string(),
                            "Monospace",
                        );
                        ui.selectable_value(
                            &mut self.font_family,
                            "Proportional".to_string(),
                            "Proportional",
                        );
                        ui.selectable_value(
                            &mut self.font_family,
                            "Emoji".to_string(),
                            "Emoji (Proportional + Emoji)",
                        );
                    });

                ui.separator();

                // Color options
                ui.horizontal(|ui| {
                    ui.label("Text Color:");
                    if ui
                        .color_edit_button_srgba(&mut self.current_text_color)
                        .changed()
                    {
                        self.apply_style_to_selection();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Highlight:");
                    let mut has_bg = self.current_bg_color.is_some();
                    let mut bg_color = self.current_bg_color.unwrap_or(Color32::YELLOW);

                    if ui.checkbox(&mut has_bg, "").changed() {
                        self.current_bg_color = if has_bg { Some(bg_color) } else { None };
                    }

                    if has_bg {
                        if ui.color_edit_button_srgba(&mut bg_color).changed() {
                            self.current_bg_color = Some(bg_color);
                            self.apply_style_to_selection();
                        }
                    }
                });

                ui.separator();

                // View options
                if ui
                    .button(if self.show_line_numbers {
                        "🔢 Hide Lines"
                    } else {
                        "🔢 Show Lines"
                    })
                    .clicked()
                {
                    self.show_line_numbers = !self.show_line_numbers;
                }

                ui.separator();

                // Find & Replace
                if ui.button("🔍 Find").clicked() {
                    self.show_find_replace = !self.show_find_replace;
                }
            });
        });

        // Find & Replace panel
        if self.show_find_replace {
            egui::TopBottomPanel::top("find_replace").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Find:");
                    ui.text_edit_singleline(&mut self.find_text);

                    if ui.button("⬇ Next").clicked() {
                        self.find_next();
                    }
                    if ui.button("⬆ Prev").clicked() {
                        self.find_previous();
                    }

                    ui.separator();

                    ui.label("Replace:");
                    ui.text_edit_singleline(&mut self.replace_text);

                    if ui.button("Replace").clicked() {
                        self.replace_current();
                    }
                    if ui.button("Replace All").clicked() {
                        self.replace_all();
                    }

                    if ui.button("✖ Close").clicked() {
                        self.show_find_replace = false;
                    }
                });
            });
        }

        // Status bar at bottom
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(err) = &self.error_message {
                    ui.colored_label(Color32::RED, err);
                } else if let Some(path) = &self.file_path {
                    // Show just the filename, not the full path
                    let filename = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown");
                    ui.label(format!("📄 {}", filename));
                } else {
                    ui.label("📄 Untitled");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!(
                        "Lines: {} | Chars: {} | Tab: CTRL+[",
                        self.text_content.lines().count(),
                        self.text_content.len()
                    ));
                });
            });
        });

        // Central text editor panel
        egui::CentralPanel::default().show(ctx, |ui| {
            // Create a scroll area that fills the entire central panel
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        // Line numbers column
                        if self.show_line_numbers {
                            let line_count = self.text_content.lines().count().max(1);

                            // Use the same font family as the text editor
                            let font_id = if self.font_family == "Proportional"
                                || self.font_family == "Emoji"
                            {
                                FontId::proportional(self.font_size)
                            } else {
                                FontId::monospace(self.font_size)
                            };
                            ui.vertical(|ui| {
                                // Set spacing to match text editor line height exactly
                                ui.spacing_mut().item_spacing.y = 0.0;

                                for i in 1..=line_count {
                                    ui.add(egui::Label::new(
                                        egui::RichText::new(format!("{:4}", i))
                                            .font(font_id.clone())
                                            .color(Color32::DARK_GRAY),
                                    ));
                                }
                            });
                            ui.separator();
                        }

                        // Text editor column - use Ctrl+[ to insert 4 spaces
                        ui.vertical(|ui| {
                            // Check if Ctrl+[ was pressed for inserting spaces (indent)
                            let indent_pressed = ui.input(|i| {
                                i.key_pressed(egui::Key::OpenBracket) && i.modifiers.ctrl
                            });

                            // Handle Ctrl+[ to insert 4 spaces BEFORE rendering
                            if indent_pressed {
                                self.save_state_for_undo();
                                if let Some(range) = &self.cursor_range {
                                    let spaces = " ".repeat(self.tab_width);
                                    let cursor_pos = range.start;
                                    self.text_content.insert_str(cursor_pos, &spaces);
                                    // Set pending cursor position for next frame
                                    let new_cursor_pos = cursor_pos + spaces.len();
                                    self.cursor_range = Some(new_cursor_pos..new_cursor_pos);
                                    self.pending_cursor_pos = Some(new_cursor_pos);
                                }
                            }

                            self.render_rich_text_editable(ui);
                        });
                    });
                });
        });
    }
}

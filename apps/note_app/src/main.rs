use eframe::egui;
use egui::{Color32, FontId, TextEdit};
use std::ops::Range;
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
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
}

struct NoteApp {
    text_content: String,
    styled_ranges: Vec<StyledRange>,
    file_path: Option<PathBuf>,
    error_message: Option<String>,
    current_style: TextFormatting,
    cursor_range: Option<Range<usize>>,
    font_size: f32,
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
        }
    }
}

impl NoteApp {
    fn save_with_formatting(&self, path: &PathBuf) -> Result<(), String> {
        let mut content = String::new();
        content.push_str(&format!("TEXT:{}\n", self.text_content));
        content.push_str("STYLES:\n");
        for styled_range in &self.styled_ranges {
            let style_name = match styled_range.style {
                TextFormatting::Regular => "Regular",
                TextFormatting::Bold => "Bold",
                TextFormatting::Italic => "Italic",
                TextFormatting::BoldItalic => "BoldItalic",
            };
            content.push_str(&format!(
                "{}..{}:{}\n",
                styled_range.range.start, styled_range.range.end, style_name
            ));
        }

        std::fs::write(path, content).map_err(|e| format!("Error saving file: {}", e))
    }

    fn load_with_formatting(&mut self, path: &PathBuf) -> Result<(), String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Error reading file: {}", e))?;

        let mut lines = content.lines();

        // Read text content
        if let Some(first_line) = lines.next() {
            if let Some(text) = first_line.strip_prefix("TEXT:") {
                self.text_content = text.to_string();
            } else {
                // Old format - plain text
                self.text_content = content;
                self.styled_ranges.clear();
                return Ok(());
            }
        }

        // Skip "STYLES:" line
        if lines.next().is_none() {
            return Ok(());
        }

        // Read styled ranges
        self.styled_ranges.clear();
        for line in lines {
            if let Some((range_part, style_part)) = line.split_once(':') {
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
                        self.styled_ranges.push(StyledRange {
                            range: start..end,
                            style,
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn apply_style_to_selection(&mut self) {
        if let Some(range) = &self.cursor_range {
            if range.start < range.end {
                // Remove overlapping ranges
                self.styled_ranges
                    .retain(|r| r.range.end <= range.start || r.range.start >= range.end);

                // Add new styled range
                self.styled_ranges.push(StyledRange {
                    range: range.clone(),
                    style: self.current_style,
                });

                // Sort ranges by start position
                self.styled_ranges.sort_by_key(|r| r.range.start);
            }
        }
    }

    fn render_rich_text_editable(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let styled_ranges = self.styled_ranges.clone();
        let font_size = self.font_size;

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
                for styled_range in &styled_ranges {
                    if styled_range.range.contains(&current_pos) {
                        style = styled_range.style;
                        break;
                    }
                }

                // Create text format based on style
                let format = match style {
                    TextFormatting::Regular => egui::TextFormat {
                        font_id: FontId::proportional(font_size),
                        color: Color32::BLACK,
                        ..Default::default()
                    },
                    TextFormatting::Bold => egui::TextFormat {
                        font_id: FontId::monospace(font_size),
                        color: Color32::from_rgb(0, 0, 0),
                        background: Color32::from_rgba_premultiplied(0, 0, 0, 8),
                        ..Default::default()
                    },
                    TextFormatting::Italic => egui::TextFormat {
                        font_id: FontId::proportional(font_size),
                        color: Color32::BLACK,
                        italics: true,
                        ..Default::default()
                    },
                    TextFormatting::BoldItalic => egui::TextFormat {
                        font_id: FontId::monospace(font_size),
                        color: Color32::from_rgb(0, 0, 0),
                        background: Color32::from_rgba_premultiplied(0, 0, 0, 8),
                        italics: true,
                        ..Default::default()
                    },
                };

                layout_job.append(segment, 0.0, format);
                current_pos = end;
            }

            ui.fonts(|f| f.layout_job(layout_job))
        };

        let response = ui.add(
            TextEdit::multiline(&mut self.text_content)
                .desired_width(f32::INFINITY)
                .desired_rows(20)
                .layouter(&mut layouter),
        );

        // Capture cursor selection
        if let Some(state) = TextEdit::load_state(ui.ctx(), response.id) {
            let cursor_range = state.cursor.char_range();
            if let Some(range) = cursor_range {
                let start = range.primary.index.min(range.secondary.index);
                let end = range.primary.index.max(range.secondary.index);
                self.cursor_range = Some(start..end);
            }
        }

        response
    }
}

impl eframe::App for NoteApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
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
                        // First time saving - show save dialog
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
                ui.label(format!("Current style: {:?}", self.current_style));

                ui.separator();
                if ui.button("🔍+ Larger").clicked() {
                    self.font_size = (self.font_size + 2.0).min(72.0);
                }
                if ui.button("🔍− Smaller").clicked() {
                    self.font_size = (self.font_size - 2.0).max(8.0);
                }
                ui.label(format!("Font: {:.0}px", self.font_size));
            });
        });

        // Status bar at bottom
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(err) = &self.error_message {
                    ui.colored_label(Color32::RED, err);
                } else if let Some(path) = &self.file_path {
                    ui.label(format!("File: {}", path.display()));
                } else {
                    ui.label("Untitled");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Characters: {}", self.text_content.len()));
                });
            });
        });

        // Central text editor panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rich Text Note Editor");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label("Type and format your text below:");
                ui.label("(Select text with mouse, then click Bold/Italic/Regular)");
                ui.separator();

                // Rich text editor with inline formatting
                self.render_rich_text_editable(ui);

                ui.separator();
                ui.label("📝 Tips:");
                ui.label("• Drag to select text");
                ui.label("• Click formatting buttons to apply to selection");
                ui.label("• Formatting appears directly in the editor");
            });
        });
    }
}

use crate::dashboard::basic_components;
use alvr_gui_common::theme::log_colors::INFO_LIGHT;
use eframe::egui::{vec2, Align, Grid, Layout, TextEdit, Ui};

use super::{reset, INDENTATION_STEP};

pub struct SearchControl {
    pub query: String,
}

impl SearchControl {
    pub fn new() -> Self {
        Self {
            query: String::new(),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        Grid::new("search_grid")
            .striped(true)
            .num_columns(3)
            .show(ui, |ui| {
                // Row 1: Search label and textedit
                ui.horizontal(|ui| {
                    ui.label("Search Settings");

                    if ui.colored_label(INFO_LIGHT, "❓").hovered() {
                        basic_components::tooltip(ui, "search_help_tooltip", "BLA BLA BLA");
                    }
                });

                ui.allocate_ui_with_layout(
                    vec2(250.0, ui.available_height()),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.add(TextEdit::singleline(&mut self.query));
                    },
                );

                if reset::reset_button(ui, !self.query.is_empty(), "").clicked() {
                    self.query.clear();
                }
                ui.end_row();

                // Row 2: Display the search query
                ui.horizontal(|ui| {
                    ui.add_space(INDENTATION_STEP);
                    ui.label("Search query:");
                });
                ui.label(&self.query); // Display the query
                ui.end_row();
            });
    }
}

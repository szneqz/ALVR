use alvr_common::info;
use alvr_gui_common::{theme::log_colors::INFO_LIGHT, tooltip};
use eframe::egui::{vec2, Align, Grid, Layout, TextEdit, Ui};
use serde_json::Value;

use super::{reset, section, SettingControl, INDENTATION_STEP};

pub struct SearchControl {
    pub query: String,
}

impl SearchControl {
    pub fn new() -> Self {
        Self {
            query: String::new(),
        }
    }

    fn find_keys_of_leaves_with_substring(
        value: &Value,
        substring: &str,
        results: &mut Vec<String>,
        current_key: Option<String>,
    ) {
        match value {
            // If the value is an object, iterate through its entries
            Value::Object(map) => {
                for (key, v) in map {
                    let new_key = if let Some(ref parent_key) = current_key {
                        format!("{}.{}", parent_key, key)
                    } else {
                        key.clone()
                    };
                    Self::find_keys_of_leaves_with_substring(v, substring, results, Some(new_key));
                }
            }
            // If the value is an array, iterate through its elements
            Value::Array(array) => {
                for (index, item) in array.iter().enumerate() {
                    let new_key = if let Some(ref parent_key) = current_key {
                        format!("{}[{}]", parent_key, index)
                    } else {
                        format!("[{}]", index)
                    };
                    Self::find_keys_of_leaves_with_substring(
                        item,
                        substring,
                        results,
                        Some(new_key),
                    );
                }
            }
            // If the value is neither an object nor an array, it's a leaf
            _ => {
                if let Some(key) = &current_key {
                    if key.contains(substring) {
                        results.push(key.clone());
                    }
                }
            }
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, settings_json: &mut Option<Value>) {
        // Row 1: Search label and textedit
        ui.horizontal(|ui| {
            ui.label("Search Settings");

            if ui.colored_label(INFO_LIGHT, "❓").hovered() {
                tooltip(ui, "search_help_tooltip", "BLA BLA BLA");
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

        if !self.query.is_empty() {
            if let Some(session_fragment) = settings_json {
                let session_fragments_mut = session_fragment.as_object_mut().unwrap();

                // Vector to hold the matching keys
                let mut matching_keys = Vec::new();

                // Start the recursive search
                Self::find_keys_of_leaves_with_substring(
                    &session_fragment,
                    &self.query,
                    &mut matching_keys,
                    None,
                );

                // Print all matching keys
                let keys_string = matching_keys.join(", ");

                ui.label(keys_string); // Display the query
            }
        }

        ui.end_row();
    }

    pub fn get_found_labels(
        &mut self,
        entry_control: &mut SettingControl,
        result: &mut Vec<(String, String)>,
    ) {
        if !self.query.is_empty() {
            entry_control.get_display_name_structure(result);

            result.retain(|x| x.1.to_lowercase().contains(&self.query.to_lowercase()));

            for entry in result {
                info!("{} - {}", entry.0, entry.1);
            }
        }
    }
}

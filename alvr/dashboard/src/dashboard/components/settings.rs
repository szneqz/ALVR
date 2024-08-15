use super::{
    presets::{builtin_schema, PresetControl},
    search::SearchControl,
    NestingInfo, SettingControl,
};
use crate::dashboard::ServerRequest;
use alvr_common::info;
use alvr_gui_common::{theme, DisplayString};
use alvr_packets::AudioDevicesList;
use alvr_session::{SessionSettings, Settings};
use eframe::egui::{self, Align, Frame, Grid, Layout, RichText, ScrollArea, Ui};
#[cfg(target_arch = "wasm32")]
use instant::Instant;
use serde_json::{self as json, Value};
use settings_schema::SchemaNode;
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

const DATA_UPDATE_INTERVAL: Duration = Duration::from_secs(1);

struct TopLevelEntry {
    id: DisplayString,
    control: SettingControl,
}

pub struct SettingsTab {
    selected_top_tab_id: String,
    resolution_preset: PresetControl,
    framerate_preset: PresetControl,
    encoder_preset: PresetControl,
    game_audio_preset: Option<PresetControl>,
    microphone_preset: Option<PresetControl>,
    hand_tracking_interaction_preset: PresetControl,
    eye_face_tracking_preset: PresetControl,
    top_level_entries: Vec<TopLevelEntry>,
    session_settings_json: Option<json::Value>,
    last_update_instant: Instant,
    search_bar: SearchControl,
}

impl SettingsTab {
    pub fn new() -> Self {
        let nesting_info = NestingInfo {
            path: vec!["session_settings".into()],
            indentation_level: 0,
        };
        let schema = Settings::schema(alvr_session::session_settings_default());

        // Top level node must be a section
        let SchemaNode::Section { entries, .. } = schema else {
            unreachable!();
        };

        let top_level_entries = entries
            .into_iter()
            .map(|entry| {
                let id = entry.name;
                let display = super::get_display_name(&id, &entry.strings);

                let mut nesting_info = nesting_info.clone();
                nesting_info.path.push(id.clone().into());

                TopLevelEntry {
                    id: DisplayString { id, display },
                    control: SettingControl::new(nesting_info, entry.content),
                }
            })
            .collect();

        Self {
            selected_top_tab_id: "presets".into(),
            resolution_preset: PresetControl::new(builtin_schema::resolution_schema()),
            framerate_preset: PresetControl::new(builtin_schema::framerate_schema()),
            encoder_preset: PresetControl::new(builtin_schema::encoder_preset_schema()),
            game_audio_preset: None,
            microphone_preset: None,
            hand_tracking_interaction_preset: PresetControl::new(
                builtin_schema::hand_tracking_interaction_schema(),
            ),
            eye_face_tracking_preset: PresetControl::new(builtin_schema::eye_face_tracking_schema()),
            top_level_entries,
            session_settings_json: None,
            last_update_instant: Instant::now(),
            search_bar: SearchControl::new(),
        }
    }

    pub fn update_session(&mut self, session_settings: &SessionSettings) {
        let settings_json = json::to_value(session_settings).unwrap();

        self.resolution_preset
            .update_session_settings(&settings_json);
        self.framerate_preset
            .update_session_settings(&settings_json);
        self.encoder_preset.update_session_settings(&settings_json);
        if let Some(preset) = self.game_audio_preset.as_mut() {
            preset.update_session_settings(&settings_json)
        }
        if let Some(preset) = self.microphone_preset.as_mut() {
            preset.update_session_settings(&settings_json)
        }
        self.hand_tracking_interaction_preset
            .update_session_settings(&settings_json);
        self.eye_face_tracking_preset
            .update_session_settings(&settings_json);

        self.session_settings_json = Some(settings_json);
    }

    pub fn update_audio_devices(&mut self, list: AudioDevicesList) {
        let mut all_devices = list.output.clone();
        all_devices.extend(list.input);

        if let Some(json) = &self.session_settings_json {
            let mut preset = PresetControl::new(builtin_schema::game_audio_schema(all_devices));
            preset.update_session_settings(json);
            self.game_audio_preset = Some(preset);

            let mut preset = PresetControl::new(builtin_schema::microphone_schema(list.output));
            preset.update_session_settings(json);
            self.microphone_preset = Some(preset);
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Vec<ServerRequest> {
        let mut requests = vec![];

        let now = Instant::now();
        if now > self.last_update_instant + DATA_UPDATE_INTERVAL {
            if self.session_settings_json.is_none() {
                requests.push(ServerRequest::GetSession);
            }

            if self.game_audio_preset.is_none() {
                requests.push(ServerRequest::GetAudioDevices);
            }

            self.last_update_instant = now;
        }

        let mut path_value_pairs = vec![];
        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            Frame::group(ui.style())
                .fill(theme::DARKER_BG)
                .inner_margin(egui::vec2(15.0, 12.0))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.selectable_value(
                            &mut self.selected_top_tab_id,
                            "presets".into(),
                            RichText::new("Presets").raised().size(15.0),
                        );
                        for entry in &mut self.top_level_entries {
                            ui.selectable_value(
                                &mut self.selected_top_tab_id,
                                entry.id.id.clone(),
                                RichText::new(entry.id.display.clone()).raised().size(15.0),
                            );
                        }
                        ui.selectable_value(
                            &mut self.selected_top_tab_id,
                            "search".into(),
                            RichText::new("🔍 Search").raised().size(15.0),
                        );
                    })
                })
        });

        if self.selected_top_tab_id == "presets" {
            ScrollArea::new([false, true])
                .id_source("presets_scroll")
                .show(ui, |ui| {
                    Grid::new("presets_grid")
                        .striped(true)
                        .num_columns(2)
                        .show(ui, |ui| {
                            path_value_pairs.extend(self.resolution_preset.ui(ui));
                            ui.end_row();

                            path_value_pairs.extend(self.framerate_preset.ui(ui));
                            ui.end_row();

                            path_value_pairs.extend(self.encoder_preset.ui(ui));
                            ui.end_row();

                            if let Some(preset) = &mut self.game_audio_preset {
                                path_value_pairs.extend(preset.ui(ui));
                                ui.end_row();
                            }

                            if let Some(preset) = &mut self.microphone_preset {
                                path_value_pairs.extend(preset.ui(ui));
                                ui.end_row();
                            }

                            path_value_pairs.extend(self.hand_tracking_interaction_preset.ui(ui));
                            ui.end_row();

                            path_value_pairs.extend(self.eye_face_tracking_preset.ui(ui));
                            ui.end_row();
                        })
                });
        } else if self.selected_top_tab_id == "search" {
            ScrollArea::new([false, true])
                .id_source("search_scroll")
                .show(ui, |ui| {
                    Grid::new("search_grid")
                        .striped(true)
                        .num_columns(3)
                        .show(ui, |ui| {
                            self.search_bar.ui(ui, &mut self.session_settings_json);

                            if !self.search_bar.query.is_empty() {
                                if let Some(session_fragment) = &mut self.session_settings_json {
                                    let session_fragments_mut =
                                        session_fragment.as_object_mut().unwrap();

                                    for entry in &mut self.top_level_entries {
                                        let mut result: Vec<(String, String)> = Vec::new();
                                        let mut result_all: Vec<(String, String)> = Vec::new();
                                        self.search_bar.get_found_labels(
                                            &mut entry.control,
                                            &mut result,
                                            &mut result_all,
                                        );

                                        let mut result2: Vec<String> =
                                            result.into_iter().map(|f| (f.0)).collect();

                                        let mut result_all2: Vec<String> =
                                            result_all.into_iter().map(|f| (f.0)).collect();

                                        //result2.push("gui_collapsed".to_owned());
                                        //result2.push("variant".to_owned());
                                        result2.push("content".to_owned());
                                        //result2.push("enabled".to_owned());

                                        for res_entry in result2.clone() {
                                            info!("{}", res_entry);
                                        }

                                        info!(
                                            "{}",
                                            session_fragments_mut[&entry.id.id].to_string()
                                        );

                                        Self::remove_unmatched_branches(
                                            &mut session_fragments_mut[&entry.id.id],
                                            result2,
                                            result_all2,
                                        );

                                        info!(
                                            "{}",
                                            session_fragments_mut[&entry.id.id].to_string()
                                        );

                                        let response = entry.control.ui(
                                            ui,
                                            &mut session_fragments_mut[&entry.id.id],
                                            false,
                                        );

                                        if let Some(response) = response {
                                            path_value_pairs.push(response);
                                        }

                                        ui.end_row();
                                    }
                                }
                            }
                        });
                });
        } else {
            ScrollArea::new([false, true])
                .id_source(format!("{}_scroll", self.selected_top_tab_id))
                .show(ui, |ui| {
                    Grid::new(format!("{}_grid", self.selected_top_tab_id))
                        .striped(true)
                        .num_columns(2)
                        .show(ui, |ui| {
                            if let Some(session_fragment) = &mut self.session_settings_json {
                                let session_fragments_mut =
                                    session_fragment.as_object_mut().unwrap();

                                let entry = self
                                    .top_level_entries
                                    .iter_mut()
                                    .find(|entry: &&mut TopLevelEntry| {
                                        entry.id.id == self.selected_top_tab_id
                                    })
                                    .unwrap();

                                let response = entry.control.ui(
                                    ui,
                                    &mut session_fragments_mut[&entry.id.id],
                                    false,
                                );

                                if let Some(response) = response {
                                    path_value_pairs.push(response);
                                }

                                ui.end_row();
                            }
                        })
                });
        }

        if !path_value_pairs.is_empty() {
            requests.push(ServerRequest::SetValues(path_value_pairs));
        }

        requests
    }

    fn remove_unmatched_branches(
        value: &mut Value,
        key_names: Vec<String>,
        not_ignore_names: Vec<String>,
    ) -> bool {
        match value {
            Value::Object(map) => {
                let mut keep = false;
                let keys_to_remove: Vec<String> = map
                    .iter_mut()
                    .filter_map(|(k, v)| {
                        let stay = key_names.contains(k);
                        if Self::remove_unmatched_branches(
                            v,
                            key_names.clone(),
                            not_ignore_names.clone(),
                        ) || stay
                        {
                            keep = true; // Mark to keep this branch if any child matches
                            info!("{} - {}", keep, v.to_string());
                            None
                        } else if not_ignore_names.contains(k) {
                            // If it's a leaf node (not an object or array), keep it
                            keep = true;
                            None
                        } else {
                            info!("{} - {}", false, v.to_string());
                            Some(k.clone())
                        }
                    })
                    .collect();

                for keymap in map.clone() {
                    info!(
                        "{} <- map value = {} {}",
                        keymap.0,
                        keep,
                        map.keys().any(|k| key_names.contains(k))
                    );
                }

                for key in keys_to_remove {
                    map.remove(&key);
                }

                // Keep this object if it has matching keys or any of its children were kept
                keep || map.keys().any(|k| key_names.contains(k))
            }
            Value::Array(array) => {
                let mut i = 0;
                let mut keep = false;

                while i < array.len() {
                    if Self::remove_unmatched_branches(
                        &mut array[i],
                        key_names.clone(),
                        not_ignore_names.clone(),
                    ) {
                        keep = true;
                        i += 1; // Move to the next element
                    } else if !array[i].is_object() && !array[i].is_array() {
                        // If it's a leaf node (not an object or array), keep it
                        //keep = true;
                        i += 1;
                    } else {
                        array.remove(i); // Remove the element at index i
                    }
                }

                keep // Keep this array if any of its elements were kept
            }
            _ => false, // Primitive values are only kept if they are within objects or arrays with matching keys
        }
    }
}

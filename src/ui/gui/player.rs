use egui::{DragValue, Ui};
use strum::IntoEnumIterator as _;

use crate::{key_codes::KeyCode, ui::{app::App, gui::collapsing_open}};

impl App {
    pub fn player_settings(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .id_salt("player")
            .show(ui, |ui| {
                ui.columns(2, |cols| {
                    let left = &mut cols[0];
                    self.player_left(left);
                    let right = &mut cols[1];
                    self.player_right(right);
                });

                collapsing_open(ui, "Colors", |ui| {
                    if let Some(color) = self.color_picker(
                        ui,
                        &self.config.player.box_visible_color,
                        "Box (visible)",
                    ) {
                        self.config.player.box_visible_color = color;
                        self.send_config();
                    }

                    if let Some(color) = self.color_picker(
                        ui,
                        &self.config.player.box_invisible_color,
                        "Box (invisible)",
                    ) {
                        self.config.player.box_invisible_color = color;
                        self.send_config();
                    }

                    if let Some(color) =
                        self.color_picker(ui, &self.config.player.skeleton_color, "Skeleton")
                    {
                        self.config.player.skeleton_color = color;
                        self.send_config();
                    }
                    if let Some(color) = self.color_picker(ui, &self.config.player.sound.color, "Sound ESP") {
                        self.config.player.sound.color = color;
                        self.send_config();
                    }

                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                DragValue::new(&mut self.config.player.alpha)
                                    .range(0.0..=1.0)
                                    .speed(0.01)
                                    .max_decimals(2),
                            )
                            .changed()
                        {
                            self.send_config();
                        }
                        ui.label("Alpha (0=transparent, 1=opaque)");
                    });
                });
            });
    }

    fn player_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Players", |ui| {
            if ui
                .checkbox(&mut self.config.player.enabled, "Enable")
                .changed()
            {
                self.send_config();
            }

            egui::ComboBox::new("esp_hotkey", "ESP Hotkey")
                .selected_text(format!("{:?}", self.config.player.esp_hotkey))
                .show_ui(ui, |ui| {
                    for key_code in KeyCode::iter() {
                        let text = format!("{:?}", &key_code);
                        if ui
                            .selectable_value(&mut self.config.player.esp_hotkey, key_code, text)
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });

            if ui
                .checkbox(&mut self.config.player.show_friendlies, "Show Friendlies")
                .on_hover_text("Only active in custom game modes (workshop/custom maps)")
                .changed()
            {
                self.send_config();
            }

            egui::ComboBox::new("draw_box", "Box")
                .selected_text(format!("{:?}", self.config.player.draw_box))
                .show_ui(ui, |ui| {
                    use crate::config::DrawMode;

                    for mode in DrawMode::iter() {
                        let text = format!("{:?}", &mode);
                        if ui
                            .selectable_value(&mut self.config.player.draw_box, mode, text)
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });

            egui::ComboBox::new("box_mode", "Box Mode")
                .selected_text(format!("{:?}", self.config.player.box_mode))
                .show_ui(ui, |ui| {
                    use crate::config::BoxMode;

                    for mode in BoxMode::iter() {
                        let text = format!("{:?}", &mode);
                        if ui
                            .selectable_value(&mut self.config.player.box_mode, mode, text)
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });

            egui::ComboBox::new("draw_skeleton", "Skeleton")
                .selected_text(format!("{:?}", self.config.player.draw_skeleton))
                .show_ui(ui, |ui| {
                    use crate::config::DrawMode;

                    for mode in DrawMode::iter() {
                        let text = format!("{:?}", &mode);
                        if ui
                            .selectable_value(&mut self.config.player.draw_skeleton, mode, text)
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });

            if ui
                .checkbox(&mut self.config.player.head_circle, "Head Circle")
                .changed()
            {
                self.send_config();
            }
        });
        collapsing_open(ui, "Sound ESP", |ui| {
                if ui
                    .checkbox(&mut self.config.player.sound.enabled, "Enabled")
                    .on_hover_text("Show a circle under players when they make sound")
                    .changed()
                {
                    self.send_config();
                }
                
            if self.config.player.sound.enabled {
                ui.horizontal(|ui| {
                    ui.label("Footstep Range");
                    
                    let response = ui.add(
                        egui::DragValue::new(&mut self.config.player.sound.footstep_radius)
                            .speed(10.0)
                            .range(100.0..=3000.0)
                            .suffix(" units")
                    );
                    
                    if ui.button("↺").on_hover_text("Reset").clicked() {
                        self.config.player.sound.footstep_radius = crate::constants::cs2::SOUND_ESP_FOOTSTEP_RADIUS_DEFAULT;
                        self.send_config();
                    }
                    
                    if response.changed() {
                        self.send_config();
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Gunshot Range");
                    
                    let response = ui.add(
                        egui::DragValue::new(&mut self.config.player.sound.gunshot_radius)
                            .speed(10.0)
                            .range(100.0..=5000.0)
                            .suffix(" units")
                    );
                    
                    if ui.button("↺").on_hover_text("Reset").clicked() {
                        self.config.player.sound.gunshot_radius = crate::constants::cs2::SOUND_ESP_GUNSHOT_RADIUS_DEFAULT;
                        self.send_config();
                    }
                    
                    if response.changed() {
                        self.send_config();
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Weapon Range");
                    
                    let response = ui.add(
                        egui::DragValue::new(&mut self.config.player.sound.weapon_radius)
                            .speed(10.0)
                            .range(100.0..=3000.0)
                            .suffix(" units")
                    );
                    
                    if ui.button("↺").on_hover_text("Reset").clicked() {
                        self.config.player.sound.weapon_radius = crate::constants::cs2::SOUND_ESP_WEAPON_RADIUS_DEFAULT;
                        self.send_config();
                    }
                    
                    if response.changed() {
                        self.send_config();
                    }
                });
            }
        });
    }

    fn player_right(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Info", |ui| {
            if ui
                .checkbox(&mut self.config.player.health_bar, "Health Bar")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.config.player.armor_bar, "Armor Bar")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.config.player.player_name, "Player Name")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.config.player.weapon_icon, "Weapon Icon")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.config.player.tags, "Show Tags")
                .changed()
            {
                self.send_config();
            }
        });
    }
}

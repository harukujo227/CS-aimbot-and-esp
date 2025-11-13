use egui::{DragValue, Ui};

use crate::ui::{app::App, gui::collapsing_open};

impl App {
    pub fn hud_settings(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .id_salt("hud")
            .show(ui, |ui| {
                ui.columns(2, |cols| {
                    let left = &mut cols[0];
                    self.hud_left(left);
                    let right = &mut cols[1];
                    self.hud_right(right);
                });

                collapsing_open(ui, "Colors", |ui| {
                    if let Some(color) =
                        self.color_picker(ui, &self.config.hud.text_color, "Text Color")
                    {
                        self.config.hud.text_color = color;
                        self.send_config();
                    }

                    if let Some(color) =
                        self.color_picker(ui, &self.config.hud.crosshair_color, "Crosshair Color")
                    {
                        self.config.hud.crosshair_color = color;
                        self.send_config();
                    }
                });

                ui.collapsing("Grenade Trails", |ui| {
                    if ui
                        .checkbox(&mut self.config.hud.grenade_trails, "Grenade Trails")
                        .changed()
                    {
                        self.send_config();
                    }

                    if let Some(color) = self.color_picker(
                        ui,
                        &self.config.hud.smoke_trail_color,
                        "Smoke Trail Color",
                    ) {
                        self.config.hud.smoke_trail_color = color;
                        self.send_config();
                    }

                    if let Some(color) = self.color_picker(
                        ui,
                        &self.config.hud.molotov_trail_color,
                        "Molotov Trail Color",
                    ) {
                        self.config.hud.molotov_trail_color = color;
                        self.send_config();
                    }

                    if let Some(color) = self.color_picker(
                        ui,
                        &self.config.hud.incendiary_trail_color,
                        "Incendiary Trail Color",
                    ) {
                        self.config.hud.incendiary_trail_color = color;
                        self.send_config();
                    }

                    if let Some(color) = self.color_picker(
                        ui,
                        &self.config.hud.flash_trail_color,
                        "Flash Trail Color",
                    ) {
                        self.config.hud.flash_trail_color = color;
                        self.send_config();
                    }

                    if let Some(color) = self.color_picker(
                        ui,
                        &self.config.hud.he_trail_color,
                        "HE Grenade Trail Color",
                    ) {
                        self.config.hud.he_trail_color = color;
                        self.send_config();
                    }

                    if let Some(color) = self.color_picker(
                        ui,
                        &self.config.hud.decoy_trail_color,
                        "Decoy Trail Color",
                    ) {
                        self.config.hud.decoy_trail_color = color;
                        self.send_config();
                    }
                });
            });
    }

    fn hud_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "HUD", |ui| {
            if ui
                .checkbox(&mut self.config.hud.bomb_timer, "Bomb Timer")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.config.hud.fov_circle, "FOV Circle")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.config.hud.sniper_crosshair, "Sniper Crosshair")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.config.hud.dropped_weapons, "Dropped Weapons")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.config.hud.spectators, "Spectator List")
                .changed()
            {
                self.send_config();
            }
        });
    }

    fn hud_right(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Appearance", |ui| {
            if ui
                .checkbox(&mut self.config.hud.text_outline, "Text Outline")
                .changed()
            {
                self.send_config();
            }

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.config.hud.line_width)
                            .range(0.1..=8.0)
                            .speed(0.02)
                            .max_decimals(1),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Line Width");
            });

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.config.hud.font_size)
                            .range(1.0..=99.0)
                            .speed(0.2)
                            .max_decimals(1),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Font Size");
            });

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.config.hud.icon_size)
                            .range(1.0..=99.0)
                            .speed(0.2)
                            .max_decimals(1),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Icon Size");
            });
        });

        ui.collapsing("Advanced", |ui| {
            if ui
                .checkbox(&mut self.config.hud.debug, "Debug Overlay")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.config.hud.debug_bvh, "Debug BVH")
                .changed()
            {
                self.send_config();
            }

            if self.config.hud.debug_bvh {
                if ui
                    .checkbox(&mut self.config.hud.bvh_aabbs, "Show AABBs")
                    .changed()
                {
                    self.send_config();
                }
                if ui
                    .checkbox(&mut self.config.hud.bvh_triangles, "Show Triangles")
                    .changed()
                {
                    self.send_config();
                }
            }
        });
    }
}

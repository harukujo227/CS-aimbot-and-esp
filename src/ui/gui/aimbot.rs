use egui::{DragValue, Ui};
use strum::IntoEnumIterator as _;

use crate::{
    config::{KeyMode, TargetingMode},
    cs2::{bones::Bones, entity::weapon::Weapon, key_codes::KeyCode},
    ui::{app::App, drag_range::DragRange, gui::collapsing_open},
};

#[derive(PartialEq)]
pub enum AimbotTab {
    Global,
    Weapon,
}

impl App {
    pub fn aimbot_settings(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.aimbot_tab, AimbotTab::Global, "Global");
            ui.selectable_value(&mut self.aimbot_tab, AimbotTab::Weapon, "Weapon");
            if self.aimbot_tab == AimbotTab::Weapon {
                egui::ComboBox::new("aimbot_weapon", "Weapon")
                    .selected_text(format!("{:?}", self.aimbot_weapon))
                    .show_ui(ui, |ui| {
                        for weapon in Weapon::iter() {
                            if weapon == Weapon::Unknown {
                                continue;
                            }
                            let text = format!("{:?}", weapon);
                            ui.selectable_value(&mut self.aimbot_weapon, weapon, text);
                        }
                    });
            }
        });
        ui.separator();
        ui.columns(2, |cols| {
            let left = &mut cols[0];
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .id_salt("aimbot_left")
                .show(left, |left| {
                    self.aimbot_left(left);
                });

            let right = &mut cols[1];
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .id_salt("aimbot_right")
                .show(right, |right| {
                    self.aimbot_right(right);
                });
        });
    }

    fn aimbot_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Aimbot", |ui| {
            egui::ComboBox::new("aimbot_hotkey", "Hotkey")
                .selected_text(format!("{:?}", self.config.aim.aimbot_hotkey))
                .show_ui(ui, |ui| {
                    for key_code in KeyCode::iter() {
                        let text = format!("{:?}", &key_code);
                        if ui
                            .selectable_value(&mut self.config.aim.aimbot_hotkey, key_code, text)
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });

            if self.aimbot_tab == AimbotTab::Weapon
                && ui
                    .checkbox(
                        &mut self.weapon_config().aimbot.enable_override,
                        "Enable Override",
                    )
                    .on_hover_text("Enable aimbot settings override for a specific weapon")
                    .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.weapon_config().aimbot.enabled, "Enable Aimbot")
                .changed()
            {
                self.send_config();
            }

            egui::ComboBox::new("aimbot_mode", "Mode")
                .selected_text(format!("{:?}", self.weapon_config().aimbot.mode))
                .show_ui(ui, |ui| {
                    for mode in KeyMode::iter() {
                        let text = format!("{:?}", &mode);
                        if ui
                            .selectable_value(&mut self.weapon_config().aimbot.mode, mode, text)
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });

            if ui
                .checkbox(
                    &mut self.weapon_config().aimbot.target_friendlies,
                    "Target Friendlies",
                )
                .on_hover_text("Only active in custom game modes (workshop/custom maps)")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(
                    &mut self.weapon_config().aimbot.distance_adjusted_fov,
                    "Distance-Adjusted FOV",
                )
                .on_hover_text("Adjusts FOV based on target distance")
                .changed()
            {
                self.send_config();
            }

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.weapon_config().aimbot.fov)
                            .range(0.1..=360.0)
                            .suffix("°")
                            .speed(0.02)
                            .max_decimals(1),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("FOV");
            });

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.weapon_config().aimbot.smooth)
                            .range(0.0..=20.0)
                            .speed(0.02)
                            .max_decimals(1),
                    )
                    .on_hover_text("Smooth the aimbot movements\nSetting it to 0 will instantly snap to the target")
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Smooth");
            });

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.weapon_config().aimbot.start_bullet)
                            .range(0..=10)
                            .speed(0.05),
                    )
                    .on_hover_text(
                        "How many bullets you need to shoot\nbefore the aimbot starts aiming",
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Start Bullet");
            });

            egui::ComboBox::new("targeting_mode", "Targeting Mode")
                .selected_text(format!("{:?}", self.weapon_config().aimbot.targeting_mode))
                .show_ui(ui, |ui| {
                    for mode in TargetingMode::iter() {
                        let text = format!("{:?}", &mode);
                        if ui
                            .selectable_value(
                                &mut self.weapon_config().aimbot.targeting_mode,
                                mode,
                                text,
                            )
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });
        });

        ui.collapsing("Checks", |ui| {
            if ui
                .checkbox(
                    &mut self.weapon_config().aimbot.visibility_check,
                    "Visibility Check",
                )
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.weapon_config().aimbot.flash_check, "Flash Check")
                .changed()
            {
                self.send_config();
            }
        });

        ui.collapsing("Bones", |ui| {
            for bone in Bones::iter() {
                let text = format!("{:?}", bone);
                let index = self
                    .weapon_config()
                    .aimbot
                    .bones
                    .iter()
                    .position(|b| *b == bone);
                if ui.selectable_label(index.is_some(), text).clicked() {
                    if let Some(index) = index {
                        self.weapon_config().aimbot.bones.remove(index);
                    } else {
                        self.weapon_config().aimbot.bones.push(bone);
                    }
                    self.send_config();
                }
            }
        });
    }

    fn aimbot_right(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Triggerbot", |ui| {
            if self.aimbot_tab == AimbotTab::Weapon
                && ui
                    .checkbox(
                        &mut self.weapon_config().triggerbot.enable_override,
                        "Enable Override",
                    )
                    .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(
                    &mut self.weapon_config().triggerbot.enabled,
                    "Enable Triggerbot",
                )
                .changed()
            {
                self.send_config();
            }

            egui::ComboBox::new("triggerbot_hotkey", "Hotkey")
                .selected_text(format!("{:?}", self.config.aim.triggerbot_hotkey))
                .show_ui(ui, |ui| {
                    for key_code in KeyCode::iter() {
                        let text = format!("{:?}", &key_code);
                        if ui
                            .selectable_value(
                                &mut self.config.aim.triggerbot_hotkey,
                                key_code,
                                text,
                            )
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });

            ui.horizontal(|ui| {
                if ui
                    .add(DragRange::new(
                        &mut self.weapon_config().triggerbot.delay,
                        0..=999,
                    ))
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Delay (ms)");
            });

            egui::ComboBox::new("triggerbot_mode", "Mode")
                .selected_text(format!("{:?}", self.weapon_config().triggerbot.mode))
                .show_ui(ui, |ui| {
                    for mode in KeyMode::iter() {
                        let text = format!("{:?}", &mode);
                        if ui
                            .selectable_value(&mut self.weapon_config().triggerbot.mode, mode, text)
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });

            if ui
                .checkbox(&mut self.weapon_config().triggerbot.head_only, "Head Only")
                .changed()
            {
                self.send_config();
            }

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.weapon_config().triggerbot.shot_duration)
                            .range(0..=2000)
                            .speed(10.0),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Additional Duration (ms)");
            });
        });

        ui.collapsing("Checks\u{200b}", |ui| {
            if ui
                .checkbox(
                    &mut self.weapon_config().triggerbot.flash_check,
                    "Flash Check",
                )
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(
                    &mut self.weapon_config().triggerbot.scope_check,
                    "Scope Check",
                )
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(
                    &mut self.weapon_config().triggerbot.velocity_check,
                    "Velocity Check",
                )
                .on_hover_text("Only shoot if the player moves slower than the specified threshold")
                .changed()
            {
                self.send_config();
            }

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.weapon_config().triggerbot.velocity_threshold)
                            .range(0..=5000),
                    )
                    .on_hover_text(
                        "Maximum velocity at which the triggerbot can shoot (in CS2 Units)",
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Velocity Threshold");
            });
        });

        collapsing_open(ui, "RCS", |ui| {
            if self.aimbot_tab == AimbotTab::Weapon
                && ui
                    .checkbox(
                        &mut self.weapon_config().rcs.enable_override,
                        "Enable Override",
                    )
                    .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.weapon_config().rcs.enabled, "Enable RCS")
                .changed()
            {
                self.send_config();
            }

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.weapon_config().rcs.smooth)
                            .range(0.0..=1.0)
                            .speed(0.02),
                    )
                    .on_hover_text("Lower values mean more direct recoil control")
                    .changed()
                {
                    self.send_config();
                }
                ui.label("RCS Smooth");
            });
        });
    }
}

use egui::Ui;
use utils::log;

use crate::{
    message::{Message, RadarStatus, Target},
    ui::{app::App, color::Colors, gui::helpers::collapsing_open},
};

impl App {
    pub fn radar_settings(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .id_salt("hud_left")
            .show(ui, |ui| {
                collapsing_open(ui, "Radar", |ui| {
                    ui.label(egui::RichText::new(format!("{}", self.radar_status)).color(
                        match self.radar_status {
                            RadarStatus::Connected(_) => Colors::GREEN,
                            RadarStatus::Disconnected => Colors::YELLOW,
                        },
                    ));

                    if ui
                        .checkbox(&mut self.config.radar.enabled, "Enable Radar")
                        .changed()
                    {
                        self.send_message(
                            Message::RadarSetEnabled(self.config.radar.enabled),
                            Target::Radar,
                        );
                        self.save();
                    }

                    if ui
                        .text_edit_singleline(&mut self.config.radar.url)
                        .changed()
                    {
                        self.send_message(
                            Message::ChangeRadarUrl(self.config.radar.url.clone()),
                            Target::Radar,
                        );
                        self.save();
                    }

                    if let RadarStatus::Connected(uuid) = &self.radar_status {
                        ui.horizontal(|ui| {
                            if ui.button("Open").clicked() {
                                let link =
                                    format!("http://{}/?uuid={}", self.config.radar.url, uuid);
                                std::process::Command::new("xdg-open")
                                    .arg(&link)
                                    .status()
                                    .unwrap();
                                log::info!("opened link ({link})");
                            }

                            if ui.button("Copy Link").clicked() {
                                let link =
                                    format!("http://{}/?uuid={}", self.config.radar.url, uuid);
                                log::info!("copied link ({link})");
                                // ctx.copy_text(link);
                                self.clipboard.set_text(link).unwrap();
                            }
                        });
                    }
                });
            });
    }
}

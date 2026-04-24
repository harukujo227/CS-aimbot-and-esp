use std::sync::atomic::Ordering;

use egui::Ui;

use crate::{
    config::write_app_config,
    os::crash::STACKTRACE_SENT,
    ui::{app::App, gui::helpers::checkbox},
};

impl App {
    pub fn application_settings(&mut self, ui: &mut Ui) {
        if checkbox(
            ui,
            "Send Stacktraces",
            &mut self.app_config.send_stacktraces,
        ) {
            write_app_config(&self.app_config);
            STACKTRACE_SENT.store(!self.app_config.send_stacktraces, Ordering::Relaxed);
        }
    }

    pub fn stacktrace_popup(&mut self, ctx: &egui::Context) {
        egui::Window::new("Send Crash Reports?")
            .resizable([false, false])
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("This helps me fix bugs, and only includes application stack traces.");
                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Yes").clicked() {
                        self.app_config.first_launch = false;
                        self.app_config.send_stacktraces = true;
                        write_app_config(&self.app_config);
                    }

                    if ui.button("No").clicked() {
                        self.app_config.first_launch = false;
                        self.app_config.send_stacktraces = false;
                        write_app_config(&self.app_config);
                        STACKTRACE_SENT.store(true, Ordering::Relaxed);
                    }
                });
            });
    }
}

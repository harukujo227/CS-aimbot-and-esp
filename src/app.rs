use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex, mpsc},
    time::{Duration, Instant},
};

use egui::{FontData, FontDefinitions, Stroke, Style};
use egui_glow::glow;
use winit::{
    application::ApplicationHandler,
    event::{StartCause, WindowEvent},
};

use crate::{
    bvh::Bvh,
    color::Colors,
    config::{
        Config, DEFAULT_CONFIG_NAME, GameStatus, available_configs, exe_path, parse_config,
        write_config,
    },
    cs2::weapon::Weapon,
    data::Data,
    gui::{AimbotTab, Tab},
    message::Message,
    mouse::DeviceStatus,
    window_context::WindowContext,
};

const FRAME_RATE: u64 = 120;
const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / FRAME_RATE);

pub struct App {
    pub window: Option<WindowContext>,
    pub gl: Option<Arc<glow::Context>>,
    pub gui_glow: Option<egui_glow::EguiGlow>,
    pub overlay_glow: Option<egui_glow::EguiGlow>,
    next_frame_time: Instant,
    pub should_close: bool,

    pub tx: mpsc::Sender<Message>,
    pub rx: mpsc::Receiver<Message>,
    pub data: Arc<Mutex<Data>>,
    #[allow(unused)]
    pub bvh: Arc<Mutex<HashMap<String, Bvh>>>,

    pub status: GameStatus,
    pub mouse_status: DeviceStatus,
    pub menu_open: bool,

    pub config: Config,
    pub current_config: PathBuf,
    pub available_configs: Vec<PathBuf>,
    pub new_config_name: String,

    pub current_tab: Tab,
    pub aimbot_tab: AimbotTab,
    pub aimbot_weapon: Weapon,
}

impl App {
    pub fn new(
        tx: mpsc::Sender<Message>,
        rx: mpsc::Receiver<Message>,
        data: Arc<Mutex<Data>>,
        bvh: Arc<Mutex<HashMap<String, Bvh>>>,
    ) -> Self {
        // read config
        let config = parse_config(&exe_path().join(DEFAULT_CONFIG_NAME));
        // override config if invalid
        write_config(&config, &exe_path().join(DEFAULT_CONFIG_NAME));

        let ret = Self {
            window: None,
            gl: None,
            gui_glow: None,
            overlay_glow: None,
            next_frame_time: Instant::now() + FRAME_DURATION,
            should_close: false,

            tx,
            rx,
            data,
            bvh,
            config,
            current_config: exe_path().join(DEFAULT_CONFIG_NAME),
            available_configs: available_configs(),
            new_config_name: String::new(),

            status: GameStatus::GameNotStarted,
            mouse_status: DeviceStatus::Disconnected,
            menu_open: false,

            current_tab: Tab::Aimbot,
            aimbot_tab: AimbotTab::Global,
            aimbot_weapon: Weapon::Ak47,
        };
        ret.send_config();
        ret
    }
}

impl ApplicationHandler for App {
    fn new_events(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, cause: StartCause) {
        if let StartCause::ResumeTimeReached { .. } = cause {
            if let Some(window) = &self.window {
                window.window().request_redraw();
            }
            self.next_frame_time += FRAME_DURATION;
        }
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let (window, gl) = create_display(event_loop);
        let gl = Arc::new(gl);
        let mut gui_glow = egui_glow::EguiGlow::new(event_loop, gl.clone(), None, None, true);
        let overlay_glow = egui_glow::EguiGlow::new(event_loop, gl.clone(), None, None, true);
        prep_ctx(&mut gui_glow.egui_ctx);
        gui_glow.egui_ctx.set_pixels_per_point(1.2);
        overlay_glow.egui_ctx.set_pixels_per_point(1.0);

        self.window = Some(window);
        self.gl = Some(gl);
        self.gui_glow = Some(gui_glow);
        self.overlay_glow = Some(overlay_glow);

        self.next_frame_time = Instant::now() + FRAME_DURATION;
        event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
            self.next_frame_time,
        ));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = &self.window else {
            return;
        };

        if self.should_close {
            event_loop.exit();
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                window.resize(new_size);
            }
            WindowEvent::RedrawRequested => {
                event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                    self.next_frame_time,
                ));
                self.render();
            }
            _ => {
                let window = self.window.as_ref().unwrap().window();
                let gui_response = self
                    .gui_glow
                    .as_mut()
                    .unwrap()
                    .on_window_event(window, &event);

                if gui_response.repaint {
                    window.request_redraw();
                }
            }
        }
    }
}

fn create_display(
    event_loop: &winit::event_loop::ActiveEventLoop,
) -> (WindowContext, glow::Context) {
    let glutin_window_context = WindowContext::new(event_loop);
    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            let s = std::ffi::CString::new(s)
                .expect("failed to construct C string from string for gl proc address");

            glutin_window_context.get_proc_address(&s)
        })
    };

    (glutin_window_context, gl)
}

fn prep_ctx(ctx: &mut egui::Context) {
    // add font
    let fira_sans = include_bytes!("../resources/FiraSansIcons.ttf");
    let mut font_definitions = FontDefinitions::default();
    font_definitions.font_data.insert(
        String::from("fira_sans"),
        Arc::new(FontData::from_static(fira_sans)),
    );

    // insert into font definitions, so it gets chosen as default
    font_definitions
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, String::from("fira_sans"));

    ctx.set_fonts(font_definitions);

    ctx.style_mut_of(egui::Theme::Dark, gui_style);
}

fn gui_style(style: &mut Style) {
    style.interaction.selectable_labels = false;
    for font in style.text_styles.iter_mut() {
        font.1.size = 16.0;
    }
    //style.visuals.override_text_color = Some(Color32::WHITE);

    style.visuals.window_fill = Colors::BASE;
    style.visuals.panel_fill = Colors::BASE;
    style.visuals.extreme_bg_color = Colors::BACKDROP;

    let bg_stroke = Stroke::new(1.0, Colors::SUBTEXT);
    let fg_stroke = Stroke::new(1.0, Colors::TEXT);
    let dark_stroke = Stroke::new(1.0, Colors::BASE);

    style.visuals.selection.bg_fill = Colors::BLUE;
    style.visuals.selection.stroke = dark_stroke;

    style.visuals.widgets.active.bg_fill = Colors::HIGHLIGHT;
    style.visuals.widgets.active.bg_stroke = bg_stroke;
    style.visuals.widgets.active.fg_stroke = fg_stroke;
    style.visuals.widgets.active.weak_bg_fill = Colors::HIGHLIGHT;

    style.visuals.widgets.hovered.bg_fill = Colors::HIGHLIGHT;
    style.visuals.widgets.hovered.bg_stroke = bg_stroke;
    style.visuals.widgets.hovered.fg_stroke = fg_stroke;
    style.visuals.widgets.hovered.weak_bg_fill = Colors::HIGHLIGHT;

    style.visuals.widgets.inactive.bg_fill = Colors::HIGHLIGHT;
    style.visuals.widgets.inactive.fg_stroke = fg_stroke;
    style.visuals.widgets.inactive.weak_bg_fill = Colors::HIGHLIGHT;

    style.visuals.widgets.noninteractive.bg_fill = Colors::HIGHLIGHT;
    style.visuals.widgets.noninteractive.fg_stroke = fg_stroke;
    style.visuals.widgets.noninteractive.weak_bg_fill = Colors::HIGHLIGHT;

    style.visuals.widgets.open.bg_fill = Colors::HIGHLIGHT;
    style.visuals.widgets.open.bg_stroke = bg_stroke;
    style.visuals.widgets.open.fg_stroke = fg_stroke;
    style.visuals.widgets.open.weak_bg_fill = Colors::HIGHLIGHT;
}

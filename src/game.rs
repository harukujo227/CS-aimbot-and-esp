use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Instant,
};

use crossbeam::channel::{Receiver, Sender};

use crate::{
    bvh::Bvh,
    config::{Config, LOOP_DURATION, SLEEP_DURATION, exe_path, DEFAULT_CONFIG_NAME, parse_config, write_config},
    cs2::CS2,
    data::Data,
    message::{Envelope, GameStatus, Message, Target},
    mouse::{DeviceStatus, Mouse, discover_mice},
};

pub trait Game: std::fmt::Debug {
    fn is_valid(&self) -> bool;
    fn setup(&mut self);
    fn run(&mut self, config: &Config, mouse: &mut Mouse);
    fn data(&self, config: &Config, data: &mut Data);
}

pub struct GameManager {
    tx: Sender<Envelope>,
    rx: Receiver<Message>,
    data: Arc<Mutex<Data>>,
    config: Config,
    mouse: Mouse,
    aimbot: CS2,
    manual_mouse: bool,
    preferred_event: Option<String>,
    failed_preferred_attempts: usize,
}

impl GameManager {
    pub fn new(
        tx: Sender<Envelope>,
        rx: Receiver<Message>,
        data: Arc<Mutex<Data>>,
        bvh: Arc<Mutex<HashMap<String, Bvh>>>,
    ) -> Self {
        let mouse = Mouse::open();

        let mut game = Self {
            tx,
            rx,
            data,
            config: Config::default(),
            mouse,
            aimbot: CS2::new(bvh),
            manual_mouse: false,
            preferred_event: None,
            failed_preferred_attempts: 0,
        };

        let config_path = exe_path().join(DEFAULT_CONFIG_NAME);
        if config_path.exists() {
            game.config = parse_config(&config_path);
        }

        if let Some(ref name) = game.config.preferred_mouse {
            if let Some(device) = crate::mouse::get_mouse_by_name(name) {
                let candidate = device.try_open();
                if let DeviceStatus::Working(_) = candidate.status {
                    log::info!("Using saved preferred mouse: {}", name);
                    game.mouse = candidate;
                    game.manual_mouse = true;
                    game.preferred_event = Some(device.event_name.clone());
                }
            }
        }

        game.send_game_message(Message::MouseStatus(game.mouse.status.clone()));

        game
    }

    fn send_game_message(&self, message: Message) {
        let envelope = Envelope {
            target: Target::Gui,
            message,
        };
        if self.tx.send(envelope).is_err() {
            std::process::exit(0);
        }
    }

    pub fn run(&mut self) {
        self.send_game_message(Message::GameStatus(GameStatus::GameNotStarted));
        let mut previous_status = GameStatus::GameNotStarted;
        loop {
            let start = Instant::now();
            while let Ok(message) = self.rx.try_recv() {
                self.parse_message(message);
            }

            let mut mouse_valid = self.mouse.valid();
            if !mouse_valid || self.mouse.status == DeviceStatus::NotFound {
                mouse_valid = self.find_mouse();
            }

            if !self.aimbot.is_valid() {
                if previous_status == GameStatus::Working {
                    self.send_game_message(Message::GameStatus(GameStatus::GameNotStarted));
                    previous_status = GameStatus::GameNotStarted;
                }
                self.aimbot.setup();
            }

            if mouse_valid && self.aimbot.is_valid() {
                if previous_status == GameStatus::GameNotStarted {
                    self.send_game_message(Message::GameStatus(GameStatus::Working));
                    previous_status = GameStatus::Working;
                }
                self.aimbot.run(&self.config, &mut self.mouse);
                let mut data = self.data.lock().unwrap();
                self.aimbot.data(&self.config, &mut data);
            }

            if self.aimbot.is_valid() && mouse_valid {
                let elapsed = start.elapsed();
                if elapsed < LOOP_DURATION {
                    sleep(LOOP_DURATION - elapsed);
                } else {
                    log::debug!("aimbot loop took {}ms", elapsed.as_millis());
                    sleep(LOOP_DURATION);
                }
            } else {
                sleep(SLEEP_DURATION);
            }
        }
    }

    fn parse_message(&mut self, message: Message) {
        match message {
            Message::Config(config) => {
                self.config = *config;
            }
            Message::SelectMouse(event_name) => {
                log::info!("User selected mouse: {}", event_name);

                if let Some(device) = discover_mice().into_iter().find(|d| d.event_name == event_name) {
                    let new_mouse = device.try_open();
                    if let DeviceStatus::Working(_) = new_mouse.status {
                        log::info!("Mouse successfully applied: {}", device.name);
                        self.mouse = new_mouse;
                        self.manual_mouse = true;
                        self.preferred_event = Some(device.event_name.clone());
                        
                        self.config.preferred_mouse = Some(device.name.clone());
                        let config_path = exe_path().join(DEFAULT_CONFIG_NAME);
                        write_config(&self.config, &config_path);
                        log::info!("Saved preferred mouse '{}' to config", device.name);
                        
                        self.failed_preferred_attempts = 0;
                        self.send_game_message(Message::MouseStatus(self.mouse.status.clone()));
                    } else {
                        log::warn!("Failed to apply mouse {}", event_name);
                        self.send_game_message(Message::MouseStatus(DeviceStatus::NotFound));
                    }
                } else {
                    log::warn!("Mouse {} not found in discover_mice()", event_name);
                    self.send_game_message(Message::MouseStatus(DeviceStatus::NotFound));
                }
            }
            _ => {}
        }
    }

    fn find_mouse(&mut self) -> bool {
        self.send_game_message(Message::MouseStatus(DeviceStatus::Disconnected));
        log::info!("mouse disconnected");
        self.mouse.status = DeviceStatus::Disconnected;

        if let Some(ref event_name) = self.preferred_event {
            if let Some(device) = discover_mice().into_iter().find(|d| &d.event_name == event_name) {
                let candidate = device.try_open();
                if let DeviceStatus::Working(_) = candidate.status {
                    let status = candidate.status.clone();
                    self.mouse = candidate;
                    self.failed_preferred_attempts = 0;
                    self.send_game_message(Message::MouseStatus(status));
                    log::info!("manual mouse {} reconnected", device.path);
                    return true;
                }
            }
            
            self.failed_preferred_attempts += 1;
            const MAX_ATTEMPTS: usize = 2;
            
            if self.failed_preferred_attempts >= MAX_ATTEMPTS {
                log::warn!("preferred mouse {} unavailable after {} attempts, falling back to any available mouse", 
                          event_name, MAX_ATTEMPTS);
                self.preferred_event = None;
                self.manual_mouse = false;
                self.failed_preferred_attempts = 0;
            } else {
                log::warn!("manual mouse {} still unavailable (attempt {}/{})", 
                          event_name, self.failed_preferred_attempts, MAX_ATTEMPTS);
                self.send_game_message(Message::MouseStatus(DeviceStatus::NotFound));
                return false;
            }
        }

        for device in discover_mice() {
            let candidate = device.try_open();
            if let DeviceStatus::Working(_) = candidate.status {
                let status = candidate.status.clone();
                self.mouse = candidate;
                self.send_game_message(Message::MouseStatus(status));
                log::info!("switched to fallback mouse {}", device.path);
                return true;
            }
        }

        log::warn!("no mice available");
        self.send_game_message(Message::MouseStatus(DeviceStatus::NotFound));
        false
    }
}
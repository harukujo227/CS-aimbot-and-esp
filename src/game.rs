use std::{
    collections::HashMap,
    sync::{Arc, Mutex, mpsc},
    thread::sleep,
    time::Instant,
};

use log::{debug, info};

use crate::{
    bvh::Bvh,
    config::{Config, GameStatus, LOOP_DURATION, SLEEP_DURATION},
    cs2::CS2,
    data::Data,
    message::Message,
    mouse::{DeviceStatus, Mouse},
};

pub trait Game: std::fmt::Debug {
    fn is_valid(&self) -> bool;
    fn setup(&mut self);
    fn run(&mut self, config: &Config, mouse: &mut Mouse);
    fn data(&self, config: &Config, data: &mut Data);
}

pub struct GameManager {
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
    data: Arc<Mutex<Data>>,
    config: Config,
    mouse: Mouse,
    aimbot: CS2,
    previous_menu_key_state: bool,
}

impl GameManager {
    pub fn new(
        tx: mpsc::Sender<Message>,
        rx: mpsc::Receiver<Message>,
        data: Arc<Mutex<Data>>,
        bvh: Arc<Mutex<HashMap<String, Bvh>>>,
    ) -> Self {
        let mouse = Mouse::open();

        let mut aimbot = Self {
            tx,
            rx,
            data,
            config: Config::default(),
            mouse,
            aimbot: CS2::new(bvh),
            previous_menu_key_state: false,
        };

        aimbot.send_message(Message::MouseStatus(aimbot.mouse.status.clone()));

        aimbot
    }

    fn send_message(&mut self, message: Message) {
        if self.tx.send(message).is_err() {
            std::process::exit(0);
        }
    }

    pub fn run(&mut self) {
        self.send_message(Message::Status(GameStatus::GameNotStarted));
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
                    self.send_message(Message::Status(GameStatus::GameNotStarted));
                    previous_status = GameStatus::GameNotStarted;
                }
                self.aimbot.setup();
            }

            if mouse_valid && self.aimbot.is_valid() {
                if previous_status == GameStatus::GameNotStarted {
                    self.send_message(Message::Status(GameStatus::Working));
                    previous_status = GameStatus::Working;
                }
                self.aimbot.run(&self.config, &mut self.mouse);
                let mut data = self.data.lock().unwrap();
                self.aimbot.data(&self.config, &mut data);
                drop(data);
                let menu_key_state = self.aimbot.is_button_down(&self.config.menu_hotkey);
                if menu_key_state && !self.previous_menu_key_state {
                    self.send_message(Message::ToggleMenu);
                }
                self.previous_menu_key_state = menu_key_state;
            }

            if self.aimbot.is_valid() && mouse_valid {
                let elapsed = start.elapsed();
                if elapsed < LOOP_DURATION {
                    sleep(LOOP_DURATION - elapsed);
                } else {
                    debug!("aimbot loop took {}ms", elapsed.as_millis());
                    sleep(LOOP_DURATION);
                }
            } else {
                sleep(SLEEP_DURATION);
            }
        }
    }

    fn parse_message(&mut self, message: Message) {
        if let Message::Config(config) = message {
            self.config = config
        }
    }

    fn find_mouse(&mut self) -> bool {
        let mut mouse_valid = false;
        self.send_message(Message::MouseStatus(DeviceStatus::Disconnected));
        info!("mouse disconnected");
        self.mouse.status = DeviceStatus::Disconnected;
        let mouse = Mouse::open();
        if let DeviceStatus::Working(_) = mouse.status {
            info!("mouse reconnected");
            mouse_valid = true;
        }
        self.send_message(Message::MouseStatus(mouse.status.clone()));
        self.mouse = mouse;
        mouse_valid
    }
}

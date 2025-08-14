use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Instant,
};

use crossbeam::channel::{Receiver, Sender};

use crate::{
    bvh::Bvh,
    config::{Config, LOOP_DURATION, SLEEP_DURATION},
    cs2::CS2,
    data::Data,
    message::{Envelope, GameStatus, Message, Target},
    mouse::{DeviceStatus, Mouse},
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
}

impl GameManager {
    pub fn new(
        tx: Sender<Envelope>,
        rx: Receiver<Message>,
        data: Arc<Mutex<Data>>,
        bvh: Arc<Mutex<HashMap<String, Bvh>>>,
    ) -> Self {
        let mouse = Mouse::open();

        let game = Self {
            tx,
            rx,
            data,
            config: Config::default(),
            mouse,
            aimbot: CS2::new(bvh),
        };

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
        if let Message::Config(config) = message {
            self.config = *config
        }
    }

    fn find_mouse(&mut self) -> bool {
        let mut mouse_valid = false;
        self.send_game_message(Message::MouseStatus(DeviceStatus::Disconnected));
        log::info!("mouse disconnected");
        self.mouse.status = DeviceStatus::Disconnected;
        let mouse = Mouse::open();
        if let DeviceStatus::Working(_) = mouse.status {
            log::info!("mouse reconnected");
            mouse_valid = true;
        }
        self.send_game_message(Message::MouseStatus(mouse.status.clone()));
        self.mouse = mouse;
        mouse_valid
    }
}

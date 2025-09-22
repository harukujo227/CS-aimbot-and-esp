use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
    time::Duration,
};

use crossbeam::channel::{Receiver, Sender};
use serde::Deserialize;
use tungstenite::{WebSocket, client};

use crate::{
    config::DEFAULT_URL,
    data::Data,
    message::{Envelope, Message, RadarStatus, Target},
};

pub struct Radar {
    websocket: Option<WebSocket<TcpStream>>,
    uuid: Option<String>,

    enabled: bool,
    url: String,

    tx: Sender<Envelope>,
    rx: Receiver<Message>,
    data: Arc<Mutex<Data>>,
}

impl Radar {
    pub fn new(tx: Sender<Envelope>, rx: Receiver<Message>, data: Arc<Mutex<Data>>) -> Self {
        Self {
            websocket: None,
            uuid: None,

            enabled: false,
            url: DEFAULT_URL.to_string(),

            tx,
            rx,
            data,
        }
    }

    pub fn run(&mut self) {
        loop {
            self.run_tick();
        }
    }

    fn run_tick(&mut self) {
        let mut should_reconnect = false;
        while let Ok(message) = self.rx.try_recv() {
            match message {
                Message::ChangeRadarUrl(url) => {
                    self.url = url;
                    should_reconnect = true;
                }
                Message::RadarSetEnabled(enabled) => self.enabled = enabled,
                _ => {}
            }
        }

        if !self.enabled {
            self.websocket = None;
            self.send_message(Message::RadarStatus(RadarStatus::Disconnected));
            std::thread::sleep(Duration::from_secs(1));
            return;
        }

        if let Some(websocket) = &mut self.websocket
            && let Some(uuid) = &self.uuid
            && !should_reconnect
        {
            let data = self.data.lock().unwrap();

            if data.in_game {
                let message_string = message(&data, uuid);

                if !message_string.is_empty() && message_string.len() > 50 {
                    let ws_message = tungstenite::Message::text(message_string);
                    let res = websocket.send(ws_message);
                    if let Err(error) = res {
                        log::warn!("could not send radar message: {error}");
                        let _ = websocket.close(None);
                    }
                } else {
                    println!("[ERROR] Message too short or empty, not sending");
                }
            }
        } else if !self.connect() {
            std::thread::sleep(Duration::from_secs(1));
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    fn connect(&mut self) -> bool {
        self.send_message(Message::RadarStatus(RadarStatus::Disconnected));
        let (url, url_full) = {
            if self.url.starts_with("ws://") {
                (self.url.chars().skip(5).collect(), self.url.clone())
            } else {
                (self.url.clone(), format!("ws://{}", self.url))
            }
        };

        let Ok(parsed_address) = url::Url::parse(&url_full) else {
            log::debug!("{url} is not a valid address");
            return false;
        };
        let Ok(address) = parsed_address.socket_addrs(|| Some(6346)) else {
            log::debug!("{url} is not reachable");
            return false;
        };
        let Ok(stream) =
            TcpStream::connect_timeout(address.first().unwrap(), Duration::from_secs(5))
        else {
            log::debug!("could not connect to {url}");
            self.websocket = None;
            return false;
        };
        let Ok((mut websocket, _)) = client(&url_full, stream) else {
            log::debug!("could not connect to {url}");
            return false;
        };

        // send handshake, get uuid
        let message =
            tungstenite::Message::text(serde_json::json!({"kind":"connect_server"}).to_string());
        websocket.send(message).unwrap();

        loop {
            if websocket.can_read() {
                let reply = websocket.read().unwrap();
                let json: UuidReply =
                    serde_json::from_str(reply.into_text().unwrap().as_str()).unwrap();
                self.uuid = Some(json.uuid);
                break;
            }
        }

        self.websocket = Some(websocket);
        let uuid = self.uuid.clone().unwrap();
        self.send_message(Message::RadarStatus(RadarStatus::Connected(uuid)));

        true
    }

    pub fn send_message(&self, message: Message) {
        let envelope = Envelope {
            target: Target::Gui,
            message,
        };
        if self.tx.send(envelope).is_err() {
            std::process::exit(0);
        }
    }
}

fn message(data: &Data, uuid: &str) -> String {
    let json_obj = serde_json::json!({
        "kind": "update_data",
        "uuid": uuid,
        "players": data.players,
        "friendlies": data.friendlies,
        "bomb": data.bomb,
        "map_name": data.map_name,
        "in_game": data.in_game,
    });

    let result = json_obj.to_string();

    if result.len() < 100 {
        println!("[WARN] Very short message: {}", result);
    }
    result
}

#[allow(unused)]
#[derive(Deserialize)]
struct UuidReply {
    pub kind: String,
    pub uuid: String,
}

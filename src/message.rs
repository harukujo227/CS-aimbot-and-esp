use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{config::Config, mouse::DeviceStatus};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum GameStatus {
    Working,
    GameNotStarted,
}

impl Display for GameStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameStatus::Working => write!(f, "Working"),
            GameStatus::GameNotStarted => write!(f, "Game Not Started"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RadarStatus {
    Disconnected,
    Connected,
}

#[derive(Clone, Debug)]
pub enum Message {
    Config(Box<Config>),
    GameStatus(GameStatus),
    MouseStatus(DeviceStatus),
    RadarStatus(RadarStatus),
    ChangeRadarUrl(String),
    RadarSetEnabled(bool),
}

#[derive(Clone, Debug)]
pub enum Target {
    Gui,
    Game,
    Radar,
}

#[derive(Clone, Debug)]
pub struct Envelope {
    pub target: Target,
    pub message: Message,
}

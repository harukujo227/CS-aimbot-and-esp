use std::fmt::Display;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GameStatus {
    Working,
    NotStarted,
}

impl Display for GameStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameStatus::Working => write!(f, "Working"),
            GameStatus::NotStarted => write!(f, "Not Started"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RadarStatus {
    Disconnected,
    Connected(Uuid),
}

impl Display for RadarStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RadarStatus::Connected(_) => write!(f, "Connected"),
            RadarStatus::Disconnected => write!(f, "Disconnected"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Config(Box<Config>),
    GameStatus(GameStatus),
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

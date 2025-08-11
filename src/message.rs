use crate::{
    config::{Config, GameStatus},
    mouse::DeviceStatus,
};

#[derive(Clone, Debug)]
pub enum Message {
    Config(Config),
    Status(GameStatus),
    MouseStatus(DeviceStatus),
    ToggleMenu,
}

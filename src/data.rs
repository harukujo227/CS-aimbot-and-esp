use std::collections::HashMap;

use glam::{Mat4, Vec2, Vec3};

use crate::cs2::{bones::Bones, weapon::Weapon};

#[derive(Debug, Default)]
pub struct Data {
    pub in_game: bool,
    pub is_ffa: bool,
    pub weapon: Weapon,
    pub players: Vec<PlayerData>,
    pub local_player: PlayerData,
    pub weapons: Vec<(Weapon, Vec3)>,
    pub bomb: BombData,
    pub view_matrix: Mat4,
    pub window_position: Vec2,
    pub window_size: Vec2,
    pub triggerbot_active: bool,
}

#[derive(Debug, Default)]
pub struct PlayerData {
    pub health: i32,
    pub armor: i32,
    pub position: Vec3,
    pub head: Vec3,
    pub name: String,
    pub weapon: Weapon,
    pub bones: HashMap<Bones, Vec3>,
    pub has_defuser: bool,
    pub has_helmet: bool,
    pub has_bomb: bool,
    pub visible: bool,
}

#[derive(Debug, Default)]
pub struct BombData {
    pub planted: bool,
    pub timer: f32,
    pub being_defused: bool,
    pub position: Vec3,
}

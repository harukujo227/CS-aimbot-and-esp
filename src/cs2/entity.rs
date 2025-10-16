use glam::Vec3;
use serde::Serialize;

use crate::cs2::{
    CS2,
    inferno::{Inferno, InfernoInfo},
    molotov::{Molotov, MolotovInfo},
    player::Player,
    smoke::{Smoke, SmokeInfo},
    weapon::Weapon,
};

#[derive(Debug, Clone)]
pub enum Entity {
    Weapon { weapon: Weapon, entity: u64 },
    Inferno(Inferno),
    Smoke(Smoke),
    Molotov(Molotov),
    Flashbang(u64),
    HeGrenade(u64),
    Decoy(u64),
}

#[derive(Debug, Clone, Serialize)]
pub enum EntityInfo {
    Weapon { weapon: Weapon, position: Vec3 },
    Inferno(InfernoInfo),
    Smoke(SmokeInfo),
    Molotov(MolotovInfo),
    Flashbang(GrenadeInfo),
    HeGrenade(GrenadeInfo),
    Decoy(GrenadeInfo),
}

#[derive(Debug, Clone, Serialize)]
pub struct GrenadeInfo {
    pub entity: u64,
    pub position: Vec3,
    pub name: &'static str,
}

impl GrenadeInfo {
    pub fn new(entity: u64, name: &'static str, cs2: &CS2) -> Self {
        Self {
            entity,
            position: Player::entity(entity).position(cs2),
            name,
        }
    }
}

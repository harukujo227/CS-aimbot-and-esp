use std::time::Instant;

use glam::Vec3;

#[derive(Debug)]
#[allow(unused)]
pub struct Trail {
    pub trail: Vec<Vec3>,
    pub last_update: Instant,
}

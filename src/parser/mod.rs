use crate::parser::bvh::{Bvh, Triangle};

pub mod bvh;

pub fn read_map(pid: i32, vphys_world: u64) -> Option<Bvh> {
    let triangles = unsafe {read_bvh(pid, vphys_world as usize)}?;
    let mut bvh = Bvh::new();
    bvh.set(triangles);
    bvh.build();
    Some(bvh)
}

unsafe extern "Rust" {
    fn read_bvh(pid: i32, vphys_world: usize) -> Option<Vec<Triangle>>;
}

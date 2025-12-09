#![cfg_attr(target_arch = "spirv", no_std)]
#![deny(warnings)]

use spirv_std::glam::{UVec3};
use spirv_std::spirv;

#[spirv(compute(threads(1024)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] out_pos: &mut [u32],
) {
    let idx = id.x;
    out_pos[idx as usize] = idx;
}

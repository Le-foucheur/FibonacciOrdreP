#![feature(array_windows, likely_unlikely, iter_array_chunks)]
#![allow(unexpected_cfgs)]
#![cfg_attr(target_arch = "spirv", no_std)]
#![deny(warnings)]

use spirv_std::glam::UVec3;
use spirv_std::spirv;
/// DATA repr in the_big_buffer:
/// work_buffer_size | out_buffer_size | valid | n_sign | step_num | [steps: step_num/32] | ( p | [work_buffers:2*work_buffer_size]):..
/// repr in output
/// [output_buffer:out_buffer_size]:..
#[spirv(compute(threads(64)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] the_big_buffer: &mut [u32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] output_buffer: &mut [u32],
) {
    output_buffer[id.x as usize] = the_big_buffer[id.x as usize];
}

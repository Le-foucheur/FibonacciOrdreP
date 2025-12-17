#![feature(array_windows, likely_unlikely, iter_array_chunks)]
#![allow(unexpected_cfgs)]
#![cfg_attr(target_arch = "spirv", no_std)]
#![deny(warnings)]

use spirv_std::glam::UVec3;
use spirv_std::spirv;

mod algo;
mod bit_iterator;
use algo::*;

/// DATA repr in the_big_buffer:
/// work_buffer_size | out_buffer_size | valid | n_sign | step_num | [steps: step_num/32] | ( p | [work_buffers:2*work_buffer_size]):..
/// repr in output
/// [output_buffer:out_buffer_size]:..
#[spirv(compute(threads(1024)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] params: &mut [u32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] scratch1: &mut [u32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] scratch2: &mut [u32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] output_buffer: &mut [u32],
) {
    let idx = id.x as usize;

    
    
    let work_buffer_size = params[0] as usize;
    let out_buffer_size = params[1] as usize;
    let valid = params[2] as usize;
    let n_sign = params[3] as i32;
    let step_num = params[4] as usize;
    let step_num_u32 = step_num.div_ceil(32);
    let personnal_space_index = work_buffer_size * idx;

    let max_invoc = params.len() - 5 - step_num_u32;
    if idx >= max_invoc {
        return;
    }
    
    let p = params[5+step_num_u32+idx] as usize;

    
    
    calculator(
        params,
        scratch1,
        scratch2,
        personnal_space_index,
        work_buffer_size,
        output_buffer,
        out_buffer_size*idx,
        out_buffer_size,
        step_num,
        n_sign,
        p,
        valid,
    );
}

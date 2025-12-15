#![feature(array_windows, likely_unlikely, iter_array_chunks)]
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
#[spirv(compute(threads(64)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] the_big_buffer: &mut [u32],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] output_buffer: &mut [u32],
) {
    fn work<'a>(idx: usize, mut the_big_buffer: &'a mut [u32], mut output_buffer: &'a mut [u32]) {
        let work_buffer_size = the_big_buffer[0] as usize;
        let out_buffer_size = the_big_buffer[1] as usize;
        let valid = the_big_buffer[2] as usize;
        let n_sign = the_big_buffer[3] as i32;
        let step_num = the_big_buffer[4] as usize;
        let step_num_u32 = step_num.div_ceil(32);
        let personnal_space_size = work_buffer_size * 2 + 1;
        let personnal_space_index = 5 + step_num_u32 + (personnal_space_size * idx);
        let p = the_big_buffer[personnal_space_index] as usize;
        let steps_as_u32 = unsafe {
            Slice::new(&raw mut the_big_buffer, 5, step_num_u32)
        };
        let buf1 = unsafe {
            Slice::new(
                &raw mut the_big_buffer,
                personnal_space_index + 1,
                work_buffer_size,
            )
        };
        let buf2 = unsafe {
            Slice::new(
                &raw mut the_big_buffer,
                personnal_space_index + 1 + work_buffer_size,
                work_buffer_size,
            )
        };
        let output = unsafe {
            Slice::new(
                &raw mut output_buffer,
                out_buffer_size * idx,
                out_buffer_size,
            )
        };
        let params = Parametters { p, valid };

        calculator(buf1, buf2, output, params, step_num, steps_as_u32, n_sign);
    }
    work(id.x as usize, the_big_buffer, output_buffer)
}

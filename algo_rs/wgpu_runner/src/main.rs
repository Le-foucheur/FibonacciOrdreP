#![feature(iter_array_chunks)]

const NUM_THREADS: u32 = 1024;

use num::{BigInt, Integer, Signed, bigint::Sign};
use std::{
    iter::{repeat, repeat_n},
    str::FromStr,
    time::Instant,
};
use tokio;
use wgpu::{
    Backends, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BufferAddress, BufferDescriptor,
    BufferUsages, COPY_BUFFER_ALIGNMENT, CommandEncoder, ComputePipeline, Device, InstanceFlags,
    PollType, ShaderStages, util::DeviceExt,
};
mod bit_iterator;
use bit_iterator::BitIterable;

mod shader_spirv {
    use wgpu::{Device, ShaderModule};

    const SHADER: wgpu::ShaderModuleDescriptorPassthrough =
        wgpu::include_spirv_raw!(env!("shader.spv"));
    pub fn load(device: &Device) -> ShaderModule {
        println!("loading shader located at {}", env!("shader.spv"));
        unsafe { device.create_shader_module_passthrough(SHADER) }
    }
}

use bytemuck;
use cpualgo::{calculator, setup};

#[tokio::main]
async fn main() {
    let p_values = 100_000..100_000 + 4096;
    let n = BigInt::from_str("3216843213215654646513213216549874654132132165498765431321").unwrap();

    let bloat = WgpuBloat::init(false).await;
    let test_buffer =
        bloat.allocate_out_buffer(p_values.len() as u32, 1920, BufferUsages::COPY_SRC);
    let final_buffer = bloat.device.create_buffer(&BufferDescriptor {
        label: label("out_buffer", false),
        size: test_buffer.get().2.size(),
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    println!("setup finished, calculating ...");
    for _ in 0..100 {
        let time = Instant::now();
        bloat
            .compute(true, &test_buffer, n.clone(), p_values.clone(), |encoder| {
                encoder.copy_buffer_to_buffer(
                    &test_buffer.buffer,
                    0,
                    &final_buffer,
                    0,
                    final_buffer.size(),
                );
                encoder.map_buffer_on_submit(&final_buffer, wgpu::MapMode::Read, .., |result| {
                    result.unwrap();
                })
            })
            .await;
        final_buffer.unmap();
        println!(
            "gpu compute finished in {}s",
            (Instant::now() - time).as_secs_f64()
        );
    }
    let binding = final_buffer.get_mapped_range(..);
    let data = binding.chunks_exact(1920.div_ceil(&32) * 4);
    for (chunk, p) in data.zip(p_values.clone()) {

        /* let n = n.clone();
        let n_as_bits = n
            .iter_u32_digits()
            .rev()
            .flat_map(|limb| limb.iter_bits().rev());
        let params = setup(p as usize);
        let mut scratch1 = Vec::from_iter(repeat_n(0, params.ranges_size));
        let mut scratch2 = Vec::from_iter(repeat_n(0, params.ranges_size));
        let mut output = Vec::from_iter(repeat_n(0, 1920.div_ceil(&32).next_multiple_of(&2)));
        calculator(
            scratch1.as_mut_slice(),
            scratch2.as_mut_slice(),
            output.as_mut_slice(),
            params,
            n_as_bits,
            false,
        );

        let eq = *bytemuck::cast_slice::<_, u8>(output.as_slice()) == *chunk;
        if !eq {
            println!(
                "{p}",
                // bytemuck::cast_slice::<_, u32>(chunk),
                // output.as_slice()
            );
        } */
    }
}

struct WgpuBloat {
    device: Device,
    fibo_pipeline: ComputePipeline,
    queue: wgpu::Queue,
}

pub struct StructuredBuffer {
    num_p_values: u32,
    size_u32_per_p: u32,
    buffer: wgpu::Buffer,
}

impl StructuredBuffer {
    //un getter pour pas qu'on puisse modifier les valeurs histoire d'avoir un truc un minimum safe.
    /// Renvoie self.num_p_values, self.size_u32_per_p, &self.buffer
    pub fn get(&self) -> (u32, u32, &wgpu::Buffer) {
        (self.num_p_values, self.size_u32_per_p, &self.buffer)
    }
}

fn label(s: &str, debug: bool) -> Option<&str> {
    match debug {
        true => Some(s),
        false => None,
    }
}

impl WgpuBloat {
    pub fn device(&self) -> &Device {
        &self.device
    }
    pub fn pipeline(&self) -> &ComputePipeline {
        &self.fibo_pipeline
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
    async fn init(debug: bool) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: Backends::VULKAN,
            flags: if debug {
                InstanceFlags::VALIDATION | InstanceFlags::GPU_BASED_VALIDATION
            } else {
                InstanceFlags::empty()
            },
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds {
                for_resource_creation: None,
                for_device_loss: None,
            },
            backend_options: Default::default(),
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: label("Main fibo device", debug),
                required_features: wgpu::Features::SHADER_INT64
                    | wgpu::Features::EXPERIMENTAL_PASSTHROUGH_SHADERS,
                required_limits: wgpu::Limits::defaults(),
                experimental_features: unsafe { wgpu::ExperimentalFeatures::enabled() },
                trace: wgpu::Trace::Off,
                memory_hints: wgpu::MemoryHints::Performance,
            })
            .await
            .expect("failed to find device");
        let shader = shader_spirv::load(&device);

        let buffer_descriptor = (0..4)
            .map(|i| BindGroupLayoutEntry {
                binding: i,
                visibility: ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: i == 0 },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect::<Vec<_>>();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: label("Fibo pipeline layout", debug),
            bind_group_layouts: &[
                &device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: label("fibo group layout", debug),
                    entries: buffer_descriptor.as_slice(),
                }),
            ],
            immediates_ranges: &[],
        });

        let fibo_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: label("Fibo compute pipeline", debug),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main_cs"),
            compilation_options: Default::default(),
            cache: Default::default(),
        });
        WgpuBloat {
            device,
            fibo_pipeline,
            queue,
        }
    }
    pub fn allocate_out_buffer(
        &self,
        num_p_val: u32,
        buffer_nbbits: u32,
        suplemental_usage: BufferUsages,
    ) -> StructuredBuffer {
        let individual_buffer_size = buffer_nbbits.div_ceil(32).next_multiple_of(2);
        let size = individual_buffer_size * num_p_val;
        StructuredBuffer {
            num_p_values: num_p_val,
            size_u32_per_p: individual_buffer_size,
            buffer: self.device.create_buffer(&BufferDescriptor {
                label: None,
                mapped_at_creation: false,
                size: ((size as usize * size_of::<u32>()) as wgpu::BufferAddress)
                    .next_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT),
                usage: suplemental_usage | BufferUsages::STORAGE,
            }),
        }
    }

    pub async fn compute(
        &self,
        debug: bool,
        out_buffer: &StructuredBuffer,
        mut n: BigInt,
        p_values: impl Iterator<Item = u32> + ExactSizeIterator + Clone,
        additional_command: impl Fn(&mut CommandEncoder),
    ) -> () {
        let p_max = p_values.clone().max().unwrap();
        let valid = p_max.div_ceil(32) + 2;
        let work_buffer_size = (valid + p_max.div_ceil(64) + 2).next_multiple_of(2);

        let (n_sign, num_step) = match n.sign() {
            Sign::Plus => (1_i32 as u32, n.bits() - 1),
            Sign::NoSign => (0_i32 as u32, 0),
            Sign::Minus => {
                n = n.abs();
                let ret = (-1_i32 as u32, n.bits() - 1);
                n -= 1;
                ret
            }
        };

        let needed_limbs = num_step.div_ceil(32) as usize;
        let skip_in_first_limb = (32 - (num_step as i32)).rem_euclid(32) as usize;

        let steps_bits = n
            .iter_u32_digits()
            .take(needed_limbs)
            .rev()
            .flat_map(|limb| limb.iter_bits().rev().map(|x| x ^ (n.is_negative())))
            .skip(skip_in_first_limb);
        let steps_u32 = steps_bits
            .chain(repeat(false))
            .array_chunks::<32>()
            .take(needed_limbs)
            .map(|values| {
                let mut res = 0;
                for (index, bool) in values.into_iter().enumerate() {
                    res |= (bool as u32) << index;
                }
                res
            });

        let entry_size =
            (((5 + needed_limbs + out_buffer.num_p_values as usize) * size_of::<u32>())
                as wgpu::BufferAddress)
                .next_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT);
        let entry_data = [
            work_buffer_size,
            out_buffer.size_u32_per_p as u32,
            valid,
            n_sign,
            num_step as u32,
        ]
        .into_iter()
        .chain(steps_u32)
        .chain(p_values)
        .chain(repeat(p_max))
        .flat_map(|word| word.to_ne_bytes().into_iter())
        .take(entry_size as usize)
        .collect::<Vec<_>>();
        let entry_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: label("Entry buffer", debug),
                usage: BufferUsages::STORAGE,
                contents: entry_data.as_slice(),
            });
        let work_buffer_descriptor = BufferDescriptor {
            label: label("Scratchpad buffer", debug),
            size:
                ((work_buffer_size as usize * out_buffer.num_p_values as usize * size_of::<u32>())
                    as BufferAddress)
                    .next_multiple_of(COPY_BUFFER_ALIGNMENT),
            mapped_at_creation: false,
            usage: BufferUsages::STORAGE,
        };
        let work_buffer1 = self.device.create_buffer(&work_buffer_descriptor);
        let work_buffer2 = self.device.create_buffer(&work_buffer_descriptor);

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: label("Temporary fibo work_group", debug),
            layout: &self.fibo_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: entry_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: work_buffer1.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: work_buffer2.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: out_buffer.buffer.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: label("Fibo compute pass", debug),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.fibo_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(out_buffer.num_p_values.div_ceil(NUM_THREADS as u32), 1, 1);
        }
        additional_command(&mut encoder);
        let finished = encoder.finish();
        println!("submiting work ...");
        let submission = self.queue.submit([finished]);
        self.device
            .poll(PollType::Wait {
                submission_index: Some(submission),
                timeout: None,
            })
            .unwrap();
    }
}

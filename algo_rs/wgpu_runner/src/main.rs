#![feature(iter_array_chunks)]

const NUM_THREADS: usize = 1024;

use num::{BigInt, Integer, Signed, bigint::Sign};
use std::iter::{repeat, repeat_n};
use tokio;
use wgpu::{BufferAddress, BufferDescriptor, BufferUsages, COPY_BUFFER_ALIGNMENT, util::DeviceExt};
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

#[tokio::main]
async fn main() {
    let bloat = wgpu_inital_setup(true).await;
    let p_values = (10..(4096 * 16 + 10)).step_by(16);
    let test_buffer = allocate_out_buffer(&bloat, p_values.len(), 10_000);

    println!("out buffer allocated");
    wgpu_compute(
        true,
        &bloat,
        test_buffer,
        32,
        1,
        [654065456].into_iter(),
        p_values,
    );
    println!("computation success");
}

struct WgpuBloat {
    device: wgpu::Device,
}
struct TrustedLenIterator<I> {
    iter: I,
    len: usize,
}
impl<I: Iterator> Iterator for TrustedLenIterator<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.len -= 1;
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}
impl<I: Iterator> ExactSizeIterator for TrustedLenIterator<I> {}

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
        false => None
    }
}

impl WgpuBloat {
    async fn init(debug: bool) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::TIMESTAMP_QUERY
                    | wgpu::Features::SHADER_INT64
                    | wgpu::Features::EXPERIMENTAL_PASSTHROUGH_SHADERS,
                required_limits: wgpu::Limits::defaults(),
                experimental_features: unsafe { wgpu::ExperimentalFeatures::enabled() },
                trace: wgpu::Trace::Off,
                memory_hints: wgpu::MemoryHints::Performance,
            })
            .await
            .expect("failed to find device");
        let shader = shader_spirv::load(&device);
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: label("Fibo compute pipeline", debug),
            layout: None,
            module: &shader,
            entry_point: None,
            compilation_options: Default::default(),
            cache: Default::default(),
        });
        ()
    }
    pub fn allocate_out_buffer(
        bloat: &WgpuBloat,
        num_p_val: u32,
        buffer_nbbits: u32,
    ) -> StructuredBuffer {
        let individual_buffer_size = buffer_nbbits.div_ceil(32);
        let size = individual_buffer_size * num_p_val;
        StructuredBuffer {
            num_p_values: num_p_val,
            size_u32_per_p: individual_buffer_size,
            buffer: bloat.device.create_buffer(&BufferDescriptor {
                label: None,
                mapped_at_creation: false,
                size: ((size as usize * size_of::<u32>()) as wgpu::BufferAddress)
                    .next_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT),
                usage: BufferUsages::COPY_SRC | BufferUsages::STORAGE,
            }),
        }
    }

    pub async fn wgpu_compute(
        &self,
        debug: bool,
        out_buffer: StructuredBuffer,
        mut n: BigInt,
        p_values: impl Iterator<Item = u32> + ExactSizeIterator + Clone,
    ) -> () {
        let p_max = p_values.clone().max().unwrap();
        let valid = p_max.div_ceil(32) + 2;
        let work_buffer_size = valid + p_max.div_ceil(64) + 2;

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
                label: None,
                usage: BufferUsages::STORAGE,
                contents: entry_data.as_slice(),
            });
        let work_buffer_descriptor = BufferDescriptor {
            label: None,
            size:
                ((work_buffer_size as usize * out_buffer.num_p_values as usize * size_of::<u32>())
                    as BufferAddress)
                    .next_multiple_of(COPY_BUFFER_ALIGNMENT),
            mapped_at_creation: false,
            usage: BufferUsages::STORAGE,
        };

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: entry_buffer.as_entire_binding(),
            }],
        });

        /* let descriptor_set = DescriptorSet::new(
            vulkan.descriptor_set_allocator.clone(),
            vulkan.descriptor_set_layout.clone(),
            [WriteDescriptorSet::buffer(0, data_buffer.clone()),
                                WriteDescriptorSet::buffer( 1, out_buffer.clone()),],
            [],
        )
        .unwrap();

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            vulkan.command_buffer_allocator.clone(),
            vulkan.queue_family_index,
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let work_group_counts = [num_worker, 1, 1];


        let command_buffer = command_buffer_builder.build().unwrap();
        if debug { println!("setup sucess, launching calculations");}
        let future = sync::now(vulkan.device.clone())
            .then_execute(vulkan.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap(); */
    }
}

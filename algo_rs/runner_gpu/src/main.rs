use std::iter::repeat_n;
use std::sync::Arc;
use std::time::{self, Instant};
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::layout::DescriptorSetLayout;
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceFeatures, Queue, QueueCreateInfo, QueueFlags,
};
use vulkano::instance::debug::ValidationFeatureEnable;
use vulkano::instance::{Instance, InstanceCreateInfo, InstanceExtensions};
use vulkano::memory::allocator::{
    AllocationCreateInfo, FreeListAllocator, GenericMemoryAllocator, MemoryTypeFilter,
    StandardMemoryAllocator,
};
use vulkano::pipeline::compute::ComputePipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::Pipeline;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::pipeline::{ComputePipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::sync::{self, GpuFuture, Sharing};
use vulkano::VulkanLibrary;

mod shader_spirv {
    use vulkano::shader::{spirv, ShaderModule};

    use super::*;

    const BYTES: &[u8] = include_bytes!(env!("shader.spv")).as_slice();
    pub fn load(device: Arc<Device>) -> Arc<ShaderModule> {
        println!("loading shader located at {}", env!("shader.spv"));
        unsafe {
            ShaderModule::new(
                device,
                vulkano::shader::ShaderModuleCreateInfo::new(
                    spirv::bytes_to_words(BYTES)
                        .unwrap()
                        .into_owned()
                        .as_slice(),
                ),
            )
            .unwrap()
        }
    }
}

const THREADS: usize = 1024;

fn main() {
    let vulkanbloat = vulkan_inital_setup(true);
    let min = 800_000;
    let max = 800_000 + (33 * 1024 * 8);
    let mut p_values = (min..max).step_by(33);

    let time_start = Instant::now();
    let test_buffer = allocate_out_buffer(&vulkanbloat, p_values.len(), 10_000);

    vulkan_compute(
        true,
        &vulkanbloat,
        test_buffer,
        10_000,
        32,
        1,
        [654065456].into_iter(),
        p_values.clone(),
    );
    let finish = Instant::now();
    println!(
        "computation success for {} different p starting at {} in {} seconds",
        p_values.len(),
        p_values.next().unwrap(),
        (finish - time_start).as_secs_f32()
    );
}

struct VulkanBloat {
    memory_allocator: Arc<GenericMemoryAllocator<FreeListAllocator>>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    queue_family_index: u32,
    device: Arc<Device>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    queue: Arc<Queue>,
    compute_pipeline: Arc<ComputePipeline>,
    descriptor_set_layout: Arc<DescriptorSetLayout>,
}
fn vulkan_inital_setup(debug: bool) -> VulkanBloat {
    let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
    if debug {
        println!(
            "foufd library with max available version {}",
            library.api_version()
        );
    }
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_layers: vec!["VK_LAYER_KHRONOS_validation".to_string()],
            enabled_extensions: InstanceExtensions {
                ext_validation_features: true,
                ..Default::default()
            },
            enabled_validation_features: vec![ValidationFeatureEnable::BestPractices],
            ..Default::default()
        },
    )
    .expect("failed to create a vulkan instance");

    let physical_devices = instance
        .enumerate_physical_devices()
        .expect("could not enumerate devices");

    if debug {
        println!(
            "Number of available vulkan devices: {}",
            physical_devices.len()
        );
    }
    let requested_features = DeviceFeatures {
        variable_pointers: true,
        vulkan_memory_model: true,
        shader_int64: true,
        ..Default::default()
    };
    let mut filtered_phy_dev =
        physical_devices.filter(|dev| dev.supported_features().contains(&requested_features));

    let primary_device = filtered_phy_dev
        .next()
        .expect("No vulkan-capable device, aborting");
    println!(
        "Found {} queue(s) family on primary device",
        primary_device.queue_family_properties().len()
    );
    let queue_family_index = primary_device
        .queue_family_properties()
        .iter()
        .position(|queue_family_properties| {
            queue_family_properties
                .queue_flags
                .contains(QueueFlags::COMPUTE)
        })
        .expect("couldn't find a compute queue family") as u32;

    let (device, mut queues) = Device::new(
        primary_device,
        DeviceCreateInfo {
            // here we pass the desired queue family to use by index
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_features: requested_features,
            ..Default::default()
        },
    )
    .expect("failed to create device");
    let queue = queues.next().unwrap();
    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        device.clone(),
        StandardCommandBufferAllocatorCreateInfo::default(),
    ));
    if debug {
        println!("Loading shader ...");
    }
    let shader = shader_spirv::load(device.clone());
    let shader_entry = shader.entry_point("main_cs").unwrap();
    if debug {
        println!("Done")
    }

    if debug {
        println!("Making descriptor set");
    }
    let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
        device.clone(),
        Default::default(),
    ));
    if debug {
        println!("Creating stage");
    }
    let stage = PipelineShaderStageCreateInfo::new(shader_entry.clone());
    if debug {
        println!("Creating pipeline layout")
    }
    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
            .into_pipeline_layout_create_info(device.clone())
            .unwrap(),
    )
    .expect("failed to create layout");
    if debug {
        println!("Creating compute pipeline")
    }
    let compute_pipeline = ComputePipeline::new(
        device.clone(),
        None,
        ComputePipelineCreateInfo::stage_layout(stage, layout),
    )
    .expect("failed to create compute pipeline");
    if debug {
        println!("getting descriptor set layout")
    }
    let pipeline_layout = compute_pipeline.layout();
    let descriptor_set_layouts = pipeline_layout.set_layouts();

    let descriptor_set_layout = descriptor_set_layouts[0].clone();
    if debug {
        println!("vulkan sucessfully initialized");
    }
    VulkanBloat {
        memory_allocator,
        command_buffer_allocator,
        queue_family_index,
        device,
        descriptor_set_allocator,
        queue,
        descriptor_set_layout,
        compute_pipeline,
    }
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
fn allocate_out_buffer(
    vulkan: &VulkanBloat,
    num_buffer: usize,
    buffer_nbbits: usize,
) -> Subbuffer<[u32]> {
    let buffer_size = buffer_nbbits.div_ceil(32);
    let num_buffer = num_buffer.next_multiple_of(THREADS);
    Buffer::from_iter(
        vulkan.memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        repeat_n(0_u32, num_buffer * buffer_size),
    )
    .expect("failed to create buffer")
}
fn vulkan_compute(
    debug: bool,
    vulkan: &VulkanBloat,
    out_buffer: Subbuffer<[u32]>,
    out_size: u32,
    num_step: u32,
    n_sign: u32,
    n_steps: impl ExactSizeIterator<Item = u32>,
    p_values: impl ExactSizeIterator<Item = u32> + Clone,
) -> () {
    let p_max = p_values.clone().max().unwrap();
    let num_worker = p_values.len().div_ceil(THREADS) as u32;
    let num_spaces = p_values.len().next_multiple_of(THREADS);
    let valid = p_max.div_ceil(32) + 2;
    let work_buffer_size = valid + p_max.div_ceil(64) + 2;

    let entry_data = TrustedLenIterator {
        iter: [work_buffer_size, out_size, valid, n_sign, num_step]
            .into_iter()
            .chain(n_steps)
            .chain(p_values.flat_map(|p| {
                repeat_n(p, 1).chain(repeat_n(0_u32, 2 * work_buffer_size as usize))
            })),
        len: 5
            + (num_step as usize).div_ceil(32)
            + (1 + 2 * work_buffer_size as usize) * num_spaces as usize,
    };

    let data_buffer = Buffer::from_iter(
        vulkan.memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        entry_data,
    )
    .expect("failed to create buffer");

    let descriptor_set = DescriptorSet::new(
        vulkan.descriptor_set_allocator.clone(),
        vulkan.descriptor_set_layout.clone(),
        [
            WriteDescriptorSet::buffer(0, data_buffer.clone()),
            WriteDescriptorSet::buffer(1, out_buffer.clone()),
        ],
        [],
    )
    .unwrap();

    let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
        vulkan.command_buffer_allocator.clone(),
        vulkan.queue_family_index,
        CommandBufferUsage::SimultaneousUse,
    )
    .unwrap();
    if debug {
        println!("submitting {num_worker} worker group")
    }
    let work_group_counts = [num_worker, 1, 1];

    unsafe {
        command_buffer_builder
            .bind_pipeline_compute(vulkan.compute_pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                vulkan.compute_pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .unwrap()
            .dispatch(work_group_counts)
            .unwrap();
    }

    let command_buffer = command_buffer_builder.build().unwrap();

    let future = sync::now(vulkan.device.clone())
        .then_execute(vulkan.queue.clone(), command_buffer)
        .unwrap();

    future
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();
}

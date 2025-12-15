use std::iter::{repeat_n, TrustedLen};
use std::sync::Arc;
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
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
use vulkano::memory::allocator::{
    AllocationCreateInfo, FreeListAllocator, GenericMemoryAllocator, MemoryTypeFilter,
    StandardMemoryAllocator,
};
use vulkano::pipeline::compute::ComputePipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::Pipeline;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::pipeline::{ComputePipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::shader::EntryPoint;
use vulkano::sync::{self, GpuFuture};
use vulkano::VulkanLibrary;

mod shader_spirv {
    use eager2::lazy;

    lazy!(vulkano_shaders::shader! {

        bytes: eager!(env!("shader.spv"))
    });
}

fn main() {
    let mut vulkanbloat = vulkan_inital_setup(true);

    let (future, buffer) = vulkan_compute(true, &mut vulkanbloat, data, 1024);
}

struct VulkanBloat {
    memory_allocator: Arc<GenericMemoryAllocator<FreeListAllocator>>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    queue_family_index: u32,
    device: Arc<Device>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    queue: Arc<Queue>,
    input_descriptor_set_layout: Arc<DescriptorSetLayout>,
    output_descriptor_set_layout: Arc<DescriptorSetLayout>,
}
fn vulkan_inital_setup(debug: bool) -> VulkanBloat {
    let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");

    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
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
        vulkan_memory_model: true,
        shader_int64: true,
        shader_int8: true,
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
            enabled_features: DeviceFeatures {
                vulkan_memory_model: true,
                ..Default::default()
            },
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

    let shader = shader_spirv::load(device.clone()).expect("failed to create shader module");

    let shader_entry = shader.entry_point("main_cs").unwrap();
    let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
        device.clone(),
        Default::default(),
    ));
    let stage = PipelineShaderStageCreateInfo::new(shader_entry.clone());
    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
            .into_pipeline_layout_create_info(device.clone())
            .unwrap(),
    )
    .unwrap();

    let compute_pipeline = ComputePipeline::new(
        device.clone(),
        None,
        ComputePipelineCreateInfo::stage_layout(stage, layout),
    )
    .expect("failed to create compute pipeline");

    let pipeline_layout = compute_pipeline.layout();
    let descriptor_set_layouts = pipeline_layout.set_layouts();

    let input_descriptor_set_layout = descriptor_set_layouts[0];
    let output_descriptor_set_layout = descriptor_set_layouts[1];
    VulkanBloat {
        memory_allocator,
        command_buffer_allocator,
        queue_family_index,
        device,
        descriptor_set_allocator,
        queue,
        input_descriptor_set_layout,
        output_descriptor_set_layout,
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

fn vulkan_compute<T: Iterator<Item = u32> + ExactSizeIterator>(
    _debug: bool,
    vulkan: &mut VulkanBloat,
    out_buffer: Arc<DescriptorSet>,
    out_size: u32,
    num_step: u32,
    n_sign: u32,
    n_steps: impl Iterator<Item = u32> + ExactSizeIterator,
    p_values: impl Iterator<Item = u32> + ExactSizeIterator + Clone,
) -> () {
    let p_max = p_values.clone().max().unwrap();
    let num_worker = p_values.len().div_ceil(64) as u32;
    let num_spaces = num_worker * 64;
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

    let input_descriptor_set = DescriptorSet::new(
        vulkan.descriptor_set_allocator.clone(),
        vulkan.input_descriptor_set_layout.clone(),
        [WriteDescriptorSet::buffer(0, data_buffer.clone())], // 0 is the binding
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

    unsafe {
        command_buffer_builder
            .bind_pipeline_compute(compute_pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                compute_pipeline.layout().clone(),
                descriptor_set_layout_index as u32,
                (input_descriptor_set, out_buffer),
            )
            .unwrap()
            .dispatch(work_group_counts)
            .unwrap();
    }

    let command_buffer = command_buffer_builder.build().unwrap();

    let future = sync::now(vulkan.device.clone())
        .then_execute(vulkan.queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None);
}

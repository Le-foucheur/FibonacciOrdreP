use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::device::{Device, DeviceCreateInfo, DeviceFeatures, Queue, QueueCreateInfo, QueueFlags};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
use vulkano::memory::allocator::{AllocationCreateInfo, FreeListAllocator, GenericMemoryAllocator, MemoryTypeFilter, StandardMemoryAllocator};
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


    data =
        
     
    let (future,buffer)= vulkan_compute(true,&mut vulkanbloat,data,1024);
}

struct VulkanBloat{
    memory_allocator: Arc<GenericMemoryAllocator<FreeListAllocator>>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,queue_family_index:u32,
    shader_entry:EntryPoint,
    device:Arc<Device>,
    descriptor_set_allocator:Arc<StandardDescriptorSetAllocator>,
    queue:Arc<Queue>,
}
fn vulkan_inital_setup(debug:bool)->VulkanBloat{
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

    let cs = shader.entry_point("main_cs").unwrap();
    let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
        device.clone(),
        Default::default(),
    ));
    VulkanBloat { memory_allocator,command_buffer_allocator,queue_family_index,shader_entry:cs,device,descriptor_set_allocator,queue }
}



fn vulkan_compute<T:Iterator<Item = u32>+ExactSizeIterator>(_debug: bool,vulkan:&mut VulkanBloat,entry_data:T,invocation_num:u32) -> (impl Future,Subbuffer<[u32]>){



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


    let stage = PipelineShaderStageCreateInfo::new(vulkan.shader_entry.clone());
    let layout = PipelineLayout::new(
        vulkan.device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
            .into_pipeline_layout_create_info(vulkan.device.clone())
            .unwrap(),
    )
    .unwrap();

    let compute_pipeline = ComputePipeline::new(
        vulkan.device.clone(),
        None,
        ComputePipelineCreateInfo::stage_layout(stage, layout),
    )
    .expect("failed to create compute pipeline");

    let pipeline_layout = compute_pipeline.layout();
    let descriptor_set_layouts = pipeline_layout.set_layouts();

    let descriptor_set_layout_index = 0;
    let descriptor_set_layout = descriptor_set_layouts
        .get(descriptor_set_layout_index)
        .unwrap();
    let descriptor_set = DescriptorSet::new(
        vulkan.descriptor_set_allocator.clone(),
        descriptor_set_layout.clone(),
        [WriteDescriptorSet::buffer(0, data_buffer.clone())], // 0 is the binding
        [],
    )
    .unwrap();

    

    let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
        vulkan.command_buffer_allocator.clone(),
        vulkan.queue_family_index
            ,
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    let work_group_counts = [invocation_num, 1, 1];

    unsafe {
        command_buffer_builder
            .bind_pipeline_compute(compute_pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                compute_pipeline.layout().clone(),
                descriptor_set_layout_index as u32,
                descriptor_set,
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

    (future,data_buffer)
}

use std::error::Error;

use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo, QueueFlags};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryUsage, StandardMemoryAllocator};
use vulkano::pipeline::{ComputePipeline, Pipeline, PipelineBindPoint};
use vulkano::sync::{self, GpuFuture};
use vulkano::VulkanLibrary;
use vulkano_shaders::*;

mod gpu_constructer;
mod deploy_shader;

#[derive(BufferContents)]
#[repr(C)]
struct TestStruct {
    first: i32,
    second: i32,
    res: i32,
}

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/test.frag",
    }
}

// device, queues, 

fn main() {
    let (device, mut queues) = gpu_constructer::construct_gpu();

    // -=-=-=-=-=

    let queue = queues.next().unwrap();

    let command_buffer_allocator = StandardCommandBufferAllocator::new(
        device.clone(),
        StandardCommandBufferAllocatorCreateInfo::default(),
    );
    let memory_allocator = StandardMemoryAllocator::new_default(device.clone());



    // let data: TestStruct = TestStruct {
    //     first: 5,
    //     second: 7,
    //     res: 10,
    // };
    let mut data = Vec::new();
    for a in 1..=20 {
        for b in 1..=20 {
            data.push(TestStruct {
                first: a.clone(),
                second: b.clone(),
                res: 0,
            });
        }
        // data.push(a);
    }



    let data2 = 0..64; //staging, gpu 1, gpu 2, download
    let buffer = Buffer::from_iter(
        &memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        data2
    )
    .expect("failed to create buffer");
    println!("buffer (pogger)");

    let shader = cs::load(device.clone()).expect("failed to create shader module");

    let future = deploy_shader::deploy(shader,device,queue,[1,1,1]);

    future.wait(None).unwrap();
    let binding = buffer.read().unwrap();
    println!("{binding:?}");
    // let content = binding.iter();
    // for i in content {
    //     println!("{i}");
    // }
}

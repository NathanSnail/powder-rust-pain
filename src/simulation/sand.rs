use std::sync::Arc;

use vulkano::buffer::allocator::SubbufferAllocator;
use vulkano::buffer::subbuffer::BufferWriteGuard;
use vulkano::buffer::Subbuffer;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferUsage, CopyBufferInfo,
    PrimaryAutoCommandBuffer, PrimaryCommandBufferAbstract,
};
use vulkano::device::{Device, Queue};
use vulkano::padded::Padded;
use vulkano::sync::future::{FenceSignalFuture, NowFuture};
use vulkano::sync::GpuFuture;

use crate::deploy_shader;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryUsage};

pub mod sand_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/sand_particle.glsl",
        custom_derives: [Debug,Clone,Copy,],
    }
}

pub const PADDING: usize = 4;

impl Default for sand_shader::Material {
    fn default() -> sand_shader::Material {
        sand_shader::Material {
            id: 0,
            colour: [1f32, 0f32, 1f32],
            pos: [0f32, 0f32],
            vel: [0f32, 0f32],
            mass: 1f32,
            target: [0f32, 0f32],
            force: 0f32,
            stable: 0f32,
            tags: 0,
            gas: 0,
        }
    }
}

pub fn tick(
    device: &Arc<Device>,
    queue: &Arc<Queue>,
    command: Arc<PrimaryAutoCommandBuffer>,
) -> FenceSignalFuture<CommandBufferExecFuture<NowFuture>> {
    deploy_shader::deploy(device.clone(), queue.clone(), command)
}

pub fn upload_device_buffer(
    memory_allocator: &(impl MemoryAllocator + ?Sized),
    size: u64,
) -> Subbuffer<[Padded<sand_shader::Material, PADDING>]> {
    Buffer::new_slice(
        memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_DST, // you need to be able to copy to a device only buffer so this is fine
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::DeviceOnly,
            ..Default::default()
        },
        size,
    )
    .expect("failed to create buffer")
}
///! Slow and generally shouldn't be used, use a device and transfer buffer with download.
// pub fn upload_standard_buffer(
//     data: Vec<Padded<sand_shader::Material, PADDING>>,
//     memory_allocator: &(impl MemoryAllocator + ?Sized),
// ) -> Subbuffer<[Padded<sand_shader::Material, PADDING>]> {
//     Buffer::from_iter(
//         memory_allocator,
//         BufferCreateInfo {
//             usage: BufferUsage::STORAGE_BUFFER,
//             ..Default::default()
//         },
//         AllocationCreateInfo {
//             usage: MemoryUsage::Upload,
//             ..Default::default()
//         },
//         data,
//     )
//     .expect("failed to create buffer")
// }

pub fn upload_transfer_source_buffer(
    data: Vec<[Padded<sand_shader::Material, PADDING>; 2]>,
    memory_allocator: &(impl MemoryAllocator + ?Sized),
) -> Subbuffer<sand_shader::Materials> {
    let new_buf = Buffer::new_unsized::<sand_shader::Materials>(
        memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        data.len() as u64,
    )
    .unwrap();
    {
        let mut buffer_writer = new_buf.write().unwrap();
        for (key, value) in data.iter().enumerate() {
            buffer_writer.mat[key] = *value;
        }
    }
    new_buf
    // Buffer::from_iter(
    //     memory_allocator,
    //     BufferCreateInfo {
    //         usage: BufferUsage::TRANSFER_SRC,
    //         ..Default::default()
    //     },
    //     AllocationCreateInfo {
    //         usage: MemoryUsage::Upload,
    //         ..Default::default()
    //     },
    //     data,
    // )
    // .expect("failed to create buffer")
}

pub fn make_innacessible_buffer(
    data: Vec<[Padded<sand_shader::Material, PADDING>; 2]>,
    memory_allocator: &(impl MemoryAllocator + ?Sized),
    work_groups: &[u32; 3],
    device: &Arc<Device>,
    queue: &Arc<Queue>,
) -> vulkano::buffer::Subbuffer<[vulkano::padded::Padded<sand_shader::Material, 4>]> {
    let buffer_accessible = upload_transfer_source_buffer(data, memory_allocator);
    let buffer_inaccessible = upload_device_buffer(memory_allocator, (2 * work_groups[0] * 64) as u64);

    // Create one-time command to copy between the buffers.
    let command_buffer_allocator =
        StandardCommandBufferAllocator::new(device.clone(), Default::default());
    let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
        &command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();
    command_buffer_builder
        .copy_buffer(CopyBufferInfo::buffers(
            buffer_accessible,
            buffer_inaccessible.clone(),
        ))
        .unwrap();
    let command_buffer = command_buffer_builder.build().unwrap();

    // Execute copy and wait for copy to complete before proceeding.
    command_buffer
        .execute(queue.clone())
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();
    buffer_inaccessible
}

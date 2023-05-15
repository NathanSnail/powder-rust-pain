use std::collections::btree_map::Iter;
use std::sync::Arc;

use vulkano::buffer::Subbuffer;
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage};
use vulkano::device::{Device, Queue};

use crate::deploy_shader;
use crate::pass_structs::Material;
use vulkano::memory::allocator::{
    AllocationCreateInfo, FreeListAllocator, GenericMemoryAllocator, MemoryAllocator, MemoryUsage,
    StandardMemoryAllocator,
};
use vulkano::sync::{self};

mod sand_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/sand_particle.glsl",
    }
}

pub fn tick(
    memory_allocator:&'static GenericMemoryAllocator<std::sync::Arc<vulkano::memory::allocator::FreeListAllocator>>,
    device: &Arc<Device>,
    queue: &Arc<Queue>,
) {
    let mat = Material {
        colour: [1f32, 0.5, 0f32],
        pos: [100f32, 100f32],
        ..Default::default()
    };
    let mats = (0..64).map(|_| mat.clone()).collect();
    let buf = upload_buffer(mats, memory_allocator);
    deploy_shader::deploy(
        sand_shader::load(device.clone()).expect("Failed to create compute shader."),
        device.clone(),
        queue.clone(),
        &buf,
        [1, 1, 1],
    );
}

pub fn upload_buffer(
    data: Vec<Material>,
    memory_allocator: &(impl MemoryAllocator + ?Sized),
) -> Subbuffer<[Material]> {
    Buffer::from_iter(
        memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        data,
    )
    .expect("failed to create buffer")
}

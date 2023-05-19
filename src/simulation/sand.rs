use std::sync::Arc;

use vulkano::buffer::Subbuffer;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::device::{Device, Queue};
use vulkano::padded::Padded;

use crate::deploy_shader;
use vulkano::memory::allocator::{
    AllocationCreateInfo, GenericMemoryAllocator, MemoryAllocator, MemoryUsage,
};

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
    // world: Vec<sand_shader::Material>,
    buffer: &Subbuffer<[Padded<sand_shader::Material, PADDING>]>,
) {
    // let buffer = upload_buffer(world, memory_allocator);
    let future = deploy_shader::deploy(
        sand_shader::load(device.clone()).expect("Failed to create compute shader."),
        device.clone(),
        queue.clone(),
        buffer,
        [1, 1, 1],
    );
    future.wait(None).unwrap();
    let binding = buffer.read().unwrap();
    // let mut new: Vec<sand_shader::Material> = Vec::new();
    for (key, val) in binding.iter().enumerate() {
        if key <= 1 {
            // let out = val.pos;
            println!("{val:?}");
        }
        // new.push(deref);
    }
    // new
}

pub fn upload_buffer(
    data: Vec<Padded<sand_shader::Material, PADDING>>,
    memory_allocator: &(impl MemoryAllocator + ?Sized),
) -> Subbuffer<[Padded<sand_shader::Material, PADDING>]> {
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

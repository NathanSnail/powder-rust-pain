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
    memory_allocator: &GenericMemoryAllocator<std::sync::Arc<vulkano::memory::allocator::FreeListAllocator>>,
    device: &Arc<Device>,
    queue: &Arc<Queue>,
	world: &mut Vec<Material>,
) {
    let material = Material {
        colour: [1f32, 0.5, 0f32],
        pos: [100f32, 100f32],
		id: 1,
        ..Default::default()
    };
    let materials = (0..64).map(|_| material.clone()).collect();
    let buffer = upload_buffer(materials, memory_allocator);
    let future = deploy_shader::deploy(
        sand_shader::load(device.clone()).expect("Failed to create compute shader."),
        device.clone(),
        queue.clone(),
        &buffer,
        [1, 1, 1],
    );
	future.wait(None).unwrap();
    // let binding = buffer.read().unwrap();
    // for val in binding.iter() {
    //     let id = val.id;
	// 	println!("{id:?}");
    // }
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

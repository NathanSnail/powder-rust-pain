use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage};

use vulkano::memory::allocator::{
    AllocationCreateInfo, GenericMemoryAllocator, MemoryUsage, StandardMemoryAllocator,
};
use vulkano::padded::Padded;
use vulkano::sync::{self};

mod deploy_shader;
mod gpu_constructor;
mod pass_structs;
mod simulation;
mod window;

use simulation::sand::{PADDING,sand_shader::Material};

#[derive(BufferContents)]
#[repr(C)]
struct TestStruct {
    first: i32,
    second: i32,
    res: i32,
}

// device, queues,

fn main() {
    let mut world: Vec<Padded<Material, PADDING>> = Vec::new();
	let work_groups = [4,1,1];
    for i in 1..(64 * work_groups[0]) {
        let i_f = i as f32;
        world.push(Padded
			(Material {
                id: i,
                colour: [i_f / 100f32, i_f / 100f32, i_f / 100f32],
                pos: [i_f, 100f32],
                ..Default::default()
            })
        )
    }

    let (library, _physical_device, _queue_family_index, _instance, device, mut queues) =
        gpu_constructor::construct_gpu();

    // -=-=-=-=-=

    let queue = queues.next().unwrap();

    // let command_buffer_allocator = StandardCommandBufferAllocator::new(
    //     device.clone(),
    //     StandardCommandBufferAllocatorCreateInfo::default(),
    // );
    let memory_allocator: GenericMemoryAllocator<
        std::sync::Arc<vulkano::memory::allocator::FreeListAllocator>,
    > = StandardMemoryAllocator::new_default(device.clone());

    // let data2 = 0..64; //staging, gpu 1, gpu 2, download (eventually)

    window::make_window(library, memory_allocator, device, queue, world,work_groups);
    //main.rs is done now as window now has control
}

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/test/test.glsl",
    }
}

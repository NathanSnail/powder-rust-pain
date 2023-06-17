use simulation::ecs::{Entity, Script};
use vulkano::buffer::BufferContents;

use vulkano::memory::allocator::{GenericMemoryAllocator, StandardMemoryAllocator};
use vulkano::padded::Padded;
use vulkano::sync::{self};

mod deploy_shader;
mod gpu_constructor;
mod pass_structs;
mod simulation;
mod window;

use simulation::sand::{sand_shader::Material, PADDING};
use window::init::fragment_shader::Sprite;

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
    let work_groups = [2usize.pow(6) as u32, 1, 1]; //2^4*64 points
    for i in 0..(64 * work_groups[0]) {
        let i_f = i as f32;
        world.push(Padded(Material {
            id: i,
            colour: [
                i_f / (64.0 * work_groups[0] as f32),
                i_f / (64.0 * work_groups[0] as f32),
                i_f / (64.0 * work_groups[0] as f32),
            ],
            pos: [
                i_f / (64.0 * work_groups[0] as f32),
                i_f / (64.0 * work_groups[0] as f32),
            ],
            ..Default::default()
        }));
    }

    let (
        library,
        physical_device,
        _queue_family_index,
        _instance,
        device,
        mut queues,
        window,
        surface,
        event_loop,
        window_size,
    ) = gpu_constructor::construct_gpu();
    // -=-=-=-=-=

    let queue = queues.next().unwrap();

    let memory_allocator: GenericMemoryAllocator<
        std::sync::Arc<vulkano::memory::allocator::FreeListAllocator>,
    > = StandardMemoryAllocator::new_default(device.clone());

    let mut entities = vec![Entity {
        pos: [0f32, 0f32],
        sprite: Sprite {
            pos: [0f32, 0f32],
            offset: [0f32, 0f32],
        },
        scripts: vec![Script {}],
    }];
    // let data2 = 0..64; //staging, gpu 1, gpu 2, download (eventually)

    window::make_window(
        library,
        memory_allocator,
        device,
        queue,
        world,
        work_groups,
        physical_device,
        window,
        surface,
        event_loop,
        window_size,
        entities,
    );
    //main.rs is done now as window now has control
}

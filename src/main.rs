use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage};

use vulkano::memory::allocator::{AllocationCreateInfo, MemoryUsage, StandardMemoryAllocator};
use vulkano::sync::{self};

mod deploy_shader;
mod gpu_constructor;
mod window;
mod pass_structs;

#[derive(BufferContents)]
#[repr(C)]
struct TestStruct {
    first: i32,
    second: i32,
    res: i32,
}

// device, queues,

fn main() {
    let (library, _physical_device, _queue_family_index, _instance, device, mut queues) =
        gpu_constructor::construct_gpu();

    // -=-=-=-=-=

    let queue = queues.next().unwrap();

    // let command_buffer_allocator = StandardCommandBufferAllocator::new(
    //     device.clone(),
    //     StandardCommandBufferAllocatorCreateInfo::default(),
    // );
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
                first: a,
                second: b,
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
        data2,
    )
    .expect("failed to create buffer");
    println!("buffer (pogger)");

    mod cs {
        vulkano_shaders::shader! {
            ty: "compute",
            path: "src/shaders/test.frag",
        }
    }

    let shader = cs::load(device.clone()).expect("failed to create shader module");

    let future = deploy_shader::deploy(shader, device, queue, &buffer, [1, 1, 1]);

    future.wait(None).unwrap();
    let binding = buffer.read().unwrap();
    for _val in binding.iter() {
        // println!("{val}");
    }

    window::window(library);

    // println!("{binding:?}");
    // let content = binding.iter();
    // for i in content {
    //     println!("{i}");
    // }
}

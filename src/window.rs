use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};



use vulkano::image::ImageUsage;

use vulkano::memory::allocator::{AllocationCreateInfo, MemoryUsage, StandardMemoryAllocator};

use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::swapchain::Swapchain;
use vulkano::swapchain::SwapchainCreateInfo;
use vulkano::VulkanLibrary;

use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow};


mod utils;
mod init;

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path:"src/shaders/test_vert.vert"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path:"src/shaders/test_frag.frag"
    }
}

pub fn window(library: Arc<VulkanLibrary>) {
    // let required_extensions = vulkano_win::required_extensions(&library);
    // let instance = Instance::new(
    //     library,
    //     InstanceCreateInfo {
    //         enabled_extensions: required_extensions,
    //         ..Default::default()
    //     },
    // )
    // .expect("failed to create instance");

    // let event_loop = EventLoop::new();

    // let surface = WindowBuilder::new()
    //     .build_vk_surface(&event_loop, instance.clone())
    //     .unwrap();

    // let window = surface
    //     .object()
    //     .unwrap()
    //     .clone()
    //     .downcast::<Window>()
    //     .unwrap();

    // let device_extensions = DeviceExtensions {
    //     khr_swapchain: true,
    //     ..DeviceExtensions::empty()
    // };

    // let (physical_device, queue_family_index) = instance
    //     .enumerate_physical_devices()
    //     .expect("failed to get devices")
    //     .filter(|p| p.supported_extensions().contains(&device_extensions))
    //     .filter_map(|p| {
    //         p.queue_family_properties()
    //             .iter()
    //             .enumerate()
    //             .position(|(i, q)| {
    //                 q.queue_flags.contains(QueueFlags::GRAPHICS)
    //                     && p.surface_support(i as u32, &surface).unwrap_or(false)
    //             })
    //             .map(|q| (p, q as u32))
    //     })
    //     .min_by_key(|(p, _)| match p.properties().device_type {
    //         PhysicalDeviceType::DiscreteGpu => 0,
    //         PhysicalDeviceType::IntegratedGpu => 1,
    //         PhysicalDeviceType::VirtualGpu => 2,
    //         PhysicalDeviceType::Cpu => 3,
    //         _ => 4,
    //     })
    //     .expect("no device available");

    // let (device, mut queues) = Device::new(
    //     physical_device.clone(),
    //     DeviceCreateInfo {
    //         queue_create_infos: vec![QueueCreateInfo {
    //             queue_family_index,
    //             ..Default::default()
    //         }],
    //         enabled_extensions: device_extensions, // new
    //         ..Default::default()
    //     },
    // ).expect("failed to create window device? how could the buffer succeed and this fail, gpu isn't plugged in?");

    // let queue = queues.next().unwrap();

    // let caps = physical_device
    //     .surface_capabilities(&surface, Default::default())
    //     .expect("failed to get surface capabilities");

    // let window_size = window.inner_size();
    // let _composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
    // let _image_format = Some(
    //     physical_device
    //         .surface_formats(&surface, Default::default())
    //         .unwrap()[0]
    //         .0,
    // );
	let (physical_device,surface,device,window,window_size,event_loop,queue) = init::initialize_window(&library);

    let (swapchain, images) = {
        let caps = physical_device
            .surface_capabilities(&surface, Default::default())
            .expect("failed to get surface capabilities");

        let dimensions = window.inner_size();
        let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
        let image_format = Some(
            physical_device
                .surface_formats(&surface, Default::default())
                .unwrap()[0]
                .0,
        );

        Swapchain::new(
            device.clone(),
            surface,
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count,
                image_format,
                image_extent: dimensions.into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha,
                ..Default::default()
            },
        )
        .unwrap()
    };

    let render_pass = utils::get_render_pass(device.clone(), swapchain);
    let framebuffers = utils::get_framebuffers(&images, render_pass.clone());

    let memory_allocator = StandardMemoryAllocator::new_default(device.clone());

    let vertex1 = utils::CPUVertex {
        position: [-0.5, -0.5],
    };
    let vertex2 = utils::CPUVertex {
        position: [0.0, 0.5],
    };
    let vertex3 = utils::CPUVertex {
        position: [0.5, -0.25],
    };
    let vertex_buffer = Buffer::from_iter(
        &memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        vec![vertex1, vertex2, vertex3],
    )
    .unwrap();

    let vs = vs::load(device.clone()).expect("failed to create shader module");
    let fs = fs::load(device.clone()).expect("failed to create shader module");

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: window_size.into(),
        depth_range: 0.0..1.0,
    };

    let pipeline = utils::get_pipeline(device.clone(), vs, fs, render_pass, viewport);

    let _command_buffers =
        utils::get_command_buffers(&device, &queue, &pipeline, &framebuffers, &vertex_buffer);

    event_loop.run(|event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event:
                WindowEvent::CursorMoved {
                    device_id: _,
                    position,
                    ..
                },
            ..
        } => {
            println!("{position:?}");
        }
        _ => (),
    });
}

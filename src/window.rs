use std::sync::Arc;
use vulkano::device::{
    physical::PhysicalDevice, Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo,
};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::VulkanLibrary;
use vulkano_win::VkSurfaceBuild;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

pub fn window(
    physical_device: &Arc<PhysicalDevice>,
    queue_family_index: u32,
    library: Arc<VulkanLibrary>,
) {
    let required_extensions = vulkano_win::required_extensions(&library);
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: required_extensions,
            ..Default::default()
        },
    )
    .expect("failed to create instance");

    let event_loop = EventLoop::new();

    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance)
        .unwrap();

    let window = surface
        .object()
        .unwrap()
        .clone()
        .downcast::<Window>()
        .unwrap();

    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    let (device, mut queues) = Device::new(
        physical_device.clone(),
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_extensions: device_extensions, // new
            ..Default::default()
        },
    ).expect("failed to create window device? how could the buffer succeed and this fail, gpu isn't plugged in?");

    let queue = queues.next().unwrap();
    
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

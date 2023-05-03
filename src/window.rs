use std::sync::Arc;
use vulkano::instance::Instance;
use vulkano::instance::InstanceCreateInfo;
use vulkano::VulkanLibrary;
use vulkano_win::VkSurfaceBuild;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::event_loop::ControlFlow;
use winit::window::WindowBuilder;

pub fn window(library: Arc<VulkanLibrary>) {
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
    let _surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance)
        .unwrap();
    event_loop.run(|event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        _ => (),
    });
}

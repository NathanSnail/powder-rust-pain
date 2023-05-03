use std::sync::Arc;
use vulkano::instance::Instance;
use vulkano::{VulkanLibrary};
use vulkano_win::VkSurfaceBuild;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use vulkano::instance::InstanceCreateInfo;

pub fn window(instance: Arc<Instance>, library: Arc<VulkanLibrary>) {
	let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
    let instance =
        Instance::new(library.clone(), InstanceCreateInfo::default()).expect("failed to create instance");
    let _required_extensions = vulkano_win::required_extensions(&library);
    let event_loop = EventLoop::new(); // ignore this for now
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

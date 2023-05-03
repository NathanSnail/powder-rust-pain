use std::sync::Arc;
use vulkano::instance::Instance;
use vulkano::{VulkanLibrary};
use vulkano_win::VkSurfaceBuild;

use winit::event_loop::{EventLoop};
use winit::window::WindowBuilder;
use vulkano::instance::InstanceCreateInfo;


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

}

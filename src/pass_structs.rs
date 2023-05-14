use std::sync::Arc;

use vulkano::device::physical::PhysicalDevice;
use vulkano::device::Queue;
use vulkano::device::{
    Device,
};

use vulkano::swapchain::Surface;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window};

pub struct WindowInitialized {
    pub physical_device: Arc<PhysicalDevice>,
    pub surface: Arc<Surface>,
    pub device: Arc<Device>,
    pub window: Arc<Window>,
    pub window_size: PhysicalSize<u32>,
    pub event_loop: EventLoop<()>,
    pub queue: Arc<Queue>,
}

// pub struct GpuConstructed {
// 	pub vulkan_library: Arc<VulkanLibrary>,
//     pub physical_device: Arc<PhysicalDevice>,
// 	pub queue_family_index: u32,
//     pub instance: Arc<Instance>,
//     pub device: Arc<Device>,
//     pub queues: dyn Iterator<Item = Arc<Queue>>,
// }
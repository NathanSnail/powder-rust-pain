use std::fs::{self, File};
use std::io::{Cursor, Read};
use std::sync::Arc;

use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::{CommandBufferAllocator, StandardCommandBufferAllocator};
use vulkano::command_buffer::{
    self, AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferUsage,
    PrimaryAutoCommandBuffer,
};
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::Device;
use vulkano::device::Queue;

use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{ImageDimensions, ImmutableImage, MipmapsCount, SwapchainImage};
use vulkano::memory::allocator::{
    AllocationCreateInfo, FreeListAllocator, GenericMemoryAllocator, MemoryUsage,
    StandardMemoryAllocator,
};
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::RenderPass;
use vulkano::sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo};
use vulkano::shader::ShaderModule;
use vulkano::swapchain::{PresentFuture, Surface, SwapchainAcquireFuture};
use vulkano::sync::future::{FenceSignalFuture, JoinFuture};
use vulkano::sync::GpuFuture;

use winit::dpi::PhysicalSize;

use winit::window::Window;

use super::utils::{self, CPUVertex};

type FenceExpanded = Option<
    Arc<
        FenceSignalFuture<
            PresentFuture<
                CommandBufferExecFuture<JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture>>,
            >,
        >,
    >,
>;

pub fn initialize_swapchain_screen<T, U>(
    render_physical_device: Arc<PhysicalDevice>,
    render_device: Arc<Device>,
    window: Arc<Window>,
    surface: Arc<Surface>,
    window_size: PhysicalSize<u32>,
    render_queue: Arc<Queue>,
    world_buffer: &Subbuffer<[T]>,
    sprite_buffer: &Subbuffer<[U]>,
    command_buffer_allocator: &StandardCommandBufferAllocator,
    memory_allocator: &GenericMemoryAllocator<Arc<FreeListAllocator>>,
    device: &Arc<Device>,
) -> (
    std::sync::Arc<vulkano::swapchain::Swapchain>,
    bool,
    std::vec::Vec<std::sync::Arc<vulkano::command_buffer::PrimaryAutoCommandBuffer>>,
    Viewport,
    Arc<RenderPass>,
    Arc<ShaderModule>,
    Arc<ShaderModule>,
    Subbuffer<[CPUVertex]>,
    Vec<FenceExpanded>,
    u32,
    Vec<Arc<SwapchainImage>>,
    Arc<GraphicsPipeline>,
    Arc<Queue>,
    Arc<ImageView<ImmutableImage>>,
    Arc<Sampler>,
    AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
) {
    let (swapchain, images) =
        utils::get_swapchain(&render_physical_device, &render_device, &window, surface);
    let render_pass = utils::get_render_pass(render_device.clone(), swapchain.clone());
    let frame_buffers = utils::get_framebuffers(&images, render_pass.clone());

    let render_memory_allocator = StandardMemoryAllocator::new_default(render_device.clone());

    let vertex1 = utils::CPUVertex {
        position: [-1.0, -1.0],
    };
    let vertex2 = utils::CPUVertex {
        position: [3.0, -1.0], // 3 because -1 -> 1 => width = 2, 1 + 2 = 3
    };
    let vertex3 = utils::CPUVertex {
        position: [-1.0, 3.0],
    };
    // let vertex4 = utils::CPUVertex {
    //     position: [0.5, 0.5],
    // }; Clipping makes this useless, see https://www.saschawillems.de/blog/2016/08/13/vulkan-tutorial-on-rendering-a-fullscreen-quad-without-buffers/
    let vertex_buffer = Buffer::from_iter(
        &render_memory_allocator,
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

    let vs_loaded =
        vertex_shader::load(render_device.clone()).expect("failed to create shader module");
    let fs_loaded =
        fragment_shader::load(render_device.clone()).expect("failed to create shader module");

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: window_size.into(),
        depth_range: 0.0..1.0,
    };

    let recreate_swapchain = false;
    let frames_in_flight = images.len();
    let fences: Vec<FenceExpanded> = vec![None; frames_in_flight];
    let previous_fence_i = 0;

    let render_pipeline = utils::get_pipeline(
        render_device.clone(),
        vs_loaded.clone(),
        fs_loaded.clone(),
        render_pass.clone(),
        viewport.clone(),
    );
    let push_constants = fragment_shader::PushType {
        dims: [window_size.width as f32, window_size.height as f32],
    };

    let mut uploads = AutoCommandBufferBuilder::primary(
        command_buffer_allocator,
        render_queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    let (texture, sampler) = get_image(memory_allocator, device, &mut uploads);

    let command_buffers = utils::get_command_buffers(
        &render_device,
        &render_queue,
        &render_pipeline,
        &frame_buffers,
        &vertex_buffer,
        push_constants,
        world_buffer,
        sprite_buffer,
        &texture,
        sampler.clone(),
    );

    (
        swapchain,
        recreate_swapchain,
        command_buffers,
        viewport,
        render_pass,
        vs_loaded,
        fs_loaded,
        vertex_buffer,
        fences,
        previous_fence_i,
        images,
        render_pipeline,
        render_queue,
        texture,
        sampler,
        uploads,
    )
}

pub fn get_image(
    memory_allocator: &GenericMemoryAllocator<Arc<FreeListAllocator>>,
    device: &Arc<Device>,
    uploads: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
) -> (Arc<ImageView<ImmutableImage>>, Arc<Sampler>) {
    let texture = {
        let png_bytes = { fs::read("./data/atlas.png").expect("Can't read atlas") };
        let cursor = Cursor::new(png_bytes);
        let decoder = png::Decoder::new(cursor);
        let mut reader = decoder.read_info().unwrap();
        let info = reader.info();
        let dimensions = ImageDimensions::Dim2d {
            width: info.width,
            height: info.height,
            array_layers: 1,
        };
        let mut image_data = Vec::new();
        image_data.resize((info.width * info.height * 4) as usize, 0);
        reader.next_frame(&mut image_data).unwrap();

        let image = ImmutableImage::from_iter(
            memory_allocator,
            image_data,
            dimensions,
            MipmapsCount::One,
            Format::R8G8B8A8_SRGB,
            uploads,
        )
        .unwrap();
        ImageView::new_default(image).unwrap()
    };

    let sampler = Sampler::new(
        device.clone(),
        SamplerCreateInfo {
            mag_filter: Filter::Nearest,
            min_filter: Filter::Nearest,
            address_mode: [SamplerAddressMode::Repeat; 3],
            ..Default::default()
        },
    )
    .unwrap();

    (texture, sampler)
}

pub mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path:"src/shaders/test/test_vert.vert"
    }
}

pub mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path:"src/shaders/test/test_frag.frag"
    }
}

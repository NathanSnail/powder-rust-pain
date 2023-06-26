use std::io::Cursor;
use std::sync::Arc;

use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferUsage,
    PrimaryCommandBufferAbstract,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::layout::DescriptorSetLayout;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::Device;
use vulkano::device::Queue;

use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{ImageDimensions, ImmutableImage, MipmapsCount, SwapchainImage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryUsage, StandardMemoryAllocator};
use vulkano::pipeline::graphics::color_blend::ColorBlendState;
use vulkano::pipeline::graphics::input_assembly::{InputAssemblyState, PrimitiveTopology};
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::{GraphicsPipeline, Pipeline};
use vulkano::render_pass::{RenderPass, Subpass};
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
    Arc<Sampler>,
    Option<Box<dyn GpuFuture>>,
    Arc<GraphicsPipeline>,
    Arc<PersistentDescriptorSet>,
    Vec<Arc<SwapchainImage>>,
    utils::Atlas,
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

    let descriptor_set_allocator = StandardDescriptorSetAllocator::new(render_device.clone());
    let command_buffer_allocator =
        StandardCommandBufferAllocator::new(render_device.clone(), Default::default());
    let uploads = AutoCommandBufferBuilder::primary(
        &command_buffer_allocator,
        render_queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    let texture = {
        let png_bytes = include_bytes!("../atlas.png").to_vec();
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

        let mut uploads = AutoCommandBufferBuilder::primary(
            &command_buffer_allocator,
            render_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap(); // upload our image once

        let image = ImmutableImage::from_iter(
            &render_memory_allocator,
            image_data,
            dimensions,
            MipmapsCount::One,
            Format::R8G8B8A8_SRGB,
            &mut uploads,
        )
        .unwrap();
        ImageView::new_default(image).unwrap()
    };

    let sampler = Sampler::new(
        render_device.clone(),
        SamplerCreateInfo {
            mag_filter: Filter::Nearest,
            min_filter: Filter::Nearest,
            address_mode: [SamplerAddressMode::Repeat; 3],
            ..Default::default()
        },
    )
    .unwrap();

    let render_pass = vulkano::single_pass_renderpass!(
        render_device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.image_format(),
                samples: 1,
            },
        },
        pass: {
            color: [color],
            depth_stencil: {},
        },
    )
    .unwrap();

    let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

    let pipeline = GraphicsPipeline::start()
        .vertex_input_state(utils::CPUVertex::per_vertex())
        .vertex_shader(vs_loaded.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new().topology(PrimitiveTopology::TriangleStrip))
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fs_loaded.entry_point("main").unwrap(), ())
        .color_blend_state(ColorBlendState::new(subpass.num_color_attachments()).blend_alpha())
        .render_pass(subpass)
        .with_auto_layout(render_device.clone(), |layout_create_infos| {
            // Modify the auto-generated layout by setting an immutable sampler to set 0 binding 2.
            let binding = layout_create_infos[0].bindings.get_mut(&2).unwrap();
            binding.immutable_samplers = vec![sampler.clone()];
        })
        .unwrap();

    let layout = pipeline.layout().set_layouts().get(0).unwrap();

    // Use `image_view` instead of `image_view_sampler`, since the sampler is already in the
    // layout.
    let set = PersistentDescriptorSet::new(
        &descriptor_set_allocator,
        layout.clone(),
        [
            WriteDescriptorSet::buffer(0, world_buffer.clone()),
            WriteDescriptorSet::buffer(1, sprite_buffer.clone()),
            WriteDescriptorSet::image_view(2, texture.clone()),
        ],
    )
    .unwrap();

    let previous_frame_end = Some(
        uploads
            .build()
            .unwrap()
            .execute(render_queue.clone())
            .unwrap()
            .boxed(),
    );

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
        sampler.clone(),
    );
    let push_constants = fragment_shader::PushType {
        dims: [window_size.width as f32, window_size.height as f32],
    };
    let command_buffers = utils::get_command_buffers(
        &render_device,
        &render_queue,
        &render_pipeline,
        &frame_buffers,
        &vertex_buffer,
        push_constants,
        world_buffer,
        sprite_buffer,
        texture.clone(),
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
        sampler,
        previous_frame_end,
        pipeline,
        set,
        images,
        texture,
    )
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

use std::mem::swap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};

use vulkano::image::ImageUsage;

use vulkano::memory::allocator::{AllocationCreateInfo, MemoryUsage, StandardMemoryAllocator, MemoryAllocator};

use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::swapchain::{
    acquire_next_image, Swapchain, SwapchainCreationError, SwapchainPresentInfo,
};
use vulkano::swapchain::{AcquireError, SwapchainCreateInfo};
use vulkano::VulkanLibrary;

use vulkano::sync::future::FenceSignalFuture;
use vulkano::sync::{self, FlushError, GpuFuture};
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

use crate::pass_structs::WindowInitialized;
use crate::simulation::sand;

mod init;
mod utils;

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path:"src/shaders/test/test_vert.vert"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path:"src/shaders/test/test_frag.frag"
    }
}

pub fn make_window(library: Arc<VulkanLibrary>, memory_allocater: &(impl MemoryAllocator + ?Sized)) {
    let WindowInitialized {
        physical_device,
        surface,
        device,
        window,
        mut window_size,
        event_loop,
        queue,
    } = init::initialize_window(&library);

    let (mut swapchain, images) = {
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

    let render_pass = utils::get_render_pass(device.clone(), swapchain.clone());
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

    let mut viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: window_size.into(),
        depth_range: 0.0..1.0,
    };

    let mut recreate_swapchain = false;
    let frames_in_flight = images.len();
    let mut fences: Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; frames_in_flight];
    let mut previous_fence_i = 0;

    let pipeline = utils::get_pipeline(
        device.clone(),
        vs.clone(),
        fs.clone(),
        render_pass.clone(),
        viewport.clone(),
    );

    let mut command_buffers =
        utils::get_command_buffers(&device, &queue, &pipeline, &framebuffers, &vertex_buffer);

    //fps
    let mut frames = [0f64; 60];
    let mut cur_frame = 0;
    let mut time = 0f64;
    event_loop.run(move |event, _, control_flow| match event {
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
            // println!("{position:?}");
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(_),
            ..
        } => {
            recreate_swapchain = true;
        }
        Event::RedrawEventsCleared => {
            if recreate_swapchain {
                // println!("recreating swapchain (slow)");
                recreate_swapchain = false;

                let new_dimensions = window.inner_size();
                window_size = new_dimensions;

                let (new_swapchain, new_images) = match swapchain.recreate(SwapchainCreateInfo {
                    image_extent: new_dimensions.into(), // here, "image_extend" will correspond to the window dimensions
                    ..swapchain.create_info()
                }) {
                    Ok(r) => r,
                    // This error tends to happen when the user is manually resizing the window.
                    // Simply restarting the loop is the easiest way to fix this issue.
                    Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                    Err(e) => panic!("failed to recreate swapchain: {e}"),
                };
                swapchain = new_swapchain;

                let new_framebuffers = utils::get_framebuffers(&new_images, render_pass.clone());
                viewport.dimensions = window_size.into();
                let new_pipeline = utils::get_pipeline(
                    device.clone(),
                    vs.clone(),
                    fs.clone(),
                    render_pass.clone(),
                    viewport.clone(),
                );
                command_buffers = utils::get_command_buffers(
                    &device,
                    &queue,
                    &new_pipeline,
                    &new_framebuffers,
                    &vertex_buffer,
                );
            }

            let (image_i, suboptimal, acquire_future) =
                match acquire_next_image(swapchain.clone(), None) {
                    Ok(r) => r,
                    Err(AcquireError::OutOfDate) => {
                        recreate_swapchain = true;
                        return;
                    }
                    Err(e) => panic!("failed to acquire next image: {e}"),
                };
            if suboptimal {
                recreate_swapchain = true;
            }

            // wait for the fence related to this image to finish (normally this would be the oldest fence)
            if let Some(image_fence) = &fences[image_i as usize] {
                image_fence.wait(None).unwrap();
            }

            let previous_future = match fences[previous_fence_i as usize].clone() {
                // Create a NowFuture
                None => {
                    let mut now = sync::now(device.clone());
                    now.cleanup_finished();

                    now.boxed()
                }
                // Use the existing FenceSignalFuture
                Some(fence) => fence.boxed(),
            };

            let future = previous_future
                .join(acquire_future)
                .then_execute(queue.clone(), command_buffers[image_i as usize].clone())
                .unwrap()
                .then_swapchain_present(
                    queue.clone(),
                    SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_i),
                )
                .then_signal_fence_and_flush();

            fences[image_i as usize] = match future {
                Ok(value) => Some(Arc::new(value)),
                Err(FlushError::OutOfDate) => {
                    recreate_swapchain = true;
                    None
                }
                Err(e) => {
                    println!("failed to flush future: {e}");
                    None
                }
            };
            previous_fence_i = image_i;

            cur_frame += 1;
            cur_frame %= 60;
            let start = SystemTime::now();
            let since_the_epoch = start
                .duration_since(UNIX_EPOCH)
                .expect("time went backwards")
                .as_millis() as f64;
            // println!("{since_the_epoch:?}");
            frames[cur_frame] = since_the_epoch / 1000f64 - time;
            time = since_the_epoch / 1000f64;
            let mut sum_time = 0f64;
            for frame_time in frames.iter() {
                sum_time += frame_time;
            }
            sum_time /= 60f64;
            let fps = 1f64 / sum_time + 0.5;
            print!("\rFPS: {fps:.0?}   ");
			sand::tick(&memory_allocator);
        }
        _ => (),
    });
}

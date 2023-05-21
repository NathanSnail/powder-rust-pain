use std::sync::Arc;

use crate::deploy_shader;
use crate::pass_structs::WindowInitialized;
use crate::simulation::sand::{self, sand_shader::Material, PADDING};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferUsage, CopyBufferInfo,
    PrimaryCommandBufferAbstract,
};
use vulkano::device::{Device, Queue};
use vulkano::memory::allocator::GenericMemoryAllocator;
use vulkano::padded::Padded;
use vulkano::swapchain::AcquireError;
use vulkano::swapchain::{acquire_next_image, SwapchainPresentInfo};
use vulkano::sync::future::{FenceSignalFuture, NowFuture};
use vulkano::sync::{self, FlushError, GpuFuture};
use vulkano::VulkanLibrary;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

mod fps;
mod init;
mod utils;

const FPS_DISPLAY: bool = true;

pub fn make_window(
    library: Arc<VulkanLibrary>,
    compute_memory_allocator: GenericMemoryAllocator<
        std::sync::Arc<vulkano::memory::allocator::FreeListAllocator>,
    >,
    compute_device: Arc<Device>,
    compute_queue: Arc<Queue>,
    world: Vec<Padded<Material, PADDING>>,
    work_groups: [u32; 3],
) {
    let WindowInitialized {
        physical_device: render_physical_device,
        surface,
        device: _render_device,
        window,
        window_size,
        event_loop,
        queue: _render_queue,
    } = init::initialize_window(&library);

    let (
        mut swapchain,
        mut recreate_swapchain,
        mut command_buffers,
        mut viewport,
        render_pass,
        vs,
        fs,
        vertex_buffer,
        mut fences,
        mut previous_fence_i,
    ) = init::initialize_swapchain_screen(
        render_physical_device,
        render_device.clone(),
        window.clone(),
        surface,
        window_size,
        render_queue.clone(),
    );

    //fps
    let mut frames = [0f64; 15];
    let mut cur_frame = 0;
    let mut time = 0f64;
    //compute
    let world_buffer_accessible =
        sand::upload_transfer_source_buffer(world, &compute_memory_allocator);
    let world_buffer_inaccessible =
        sand::upload_device_buffer(&compute_memory_allocator, (work_groups[0] * 64) as u64);

    // Create one-time command to copy between the buffers.
    let command_buffer_allocator =
        StandardCommandBufferAllocator::new(compute_device.clone(), Default::default());
    let mut cbb = AutoCommandBufferBuilder::primary(
        &command_buffer_allocator,
        compute_queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();
    cbb.copy_buffer(CopyBufferInfo::buffers(
        world_buffer_accessible,
        world_buffer_inaccessible.clone(),
    ))
    .unwrap();
    let cb = cbb.build().unwrap();

    // Execute copy and wait for copy to complete before proceeding.
    cb.execute(compute_queue.clone())
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();
    // Transfer complete
    let compute_shader_loaded =
        sand::sand_shader::load(compute_device.clone()).expect("Failed to create compute shader.");
    let deploy_command = Arc::new(deploy_shader::get_deploy_command(
        &compute_shader_loaded,
        &compute_device,
        &compute_queue,
        &world_buffer_inaccessible,
        work_groups,
    ));
    let mut next_future: Option<FenceSignalFuture<CommandBufferExecFuture<NowFuture>>> = None;

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
                    position: _,
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
                utils::recreate_swapchain(
                    &window,
                    &render_pass,
                    &mut swapchain,
                    &mut viewport,
                    &render_device,
                    &render_queue,
                    &vertex_buffer,
                    &mut command_buffers,
                    &vs,
                    &fs,
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
                    let mut now = sync::now(render_device.clone());
                    now.cleanup_finished();

                    now.boxed()
                }
                // Use the existing FenceSignalFuture
                Some(fence) => fence.boxed(),
            };

            let future = previous_future
                .join(acquire_future)
                .then_execute(
                    render_queue.clone(),
                    command_buffers[image_i as usize].clone(),
                )
                .unwrap()
                .then_swapchain_present(
                    render_queue.clone(),
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
            if FPS_DISPLAY {
                fps::do_fps(&mut frames, &mut cur_frame, &mut time);
            }
            if next_future.is_some() {
                match next_future.as_ref().unwrap().wait(None) {
                    Ok(_) => {}
                    Err(err) => {
                        panic!("{err:?}")
                    }
                }
                // let _binding = world_buffer_inaccessible.read().unwrap();
                // for (key, val) in binding.iter().enumerate() {
                //     if key <= 1 {
                //         println!("{val:?}");
                //     }
                // }
            }

            next_future = Option::from(sand::tick(
                // 1 frame of lag
                &compute_device.clone(),
                &compute_queue.clone(),
                deploy_command.clone(),
            ));
        }
        _ => (),
    });
}

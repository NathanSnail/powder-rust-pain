use std::sync::Arc;

use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};

use vulkano::command_buffer::CommandBufferExecFuture;

use vulkano::device::{Device, Queue};

use vulkano::memory::allocator::{
    AllocationCreateInfo, GenericMemoryAllocator, MemoryUsage, StandardMemoryAllocator,
};

use vulkano::padded::Padded;
use vulkano::pipeline::graphics::viewport::Viewport;

use vulkano::swapchain::{acquire_next_image, SwapchainCreationError, SwapchainPresentInfo};
use vulkano::swapchain::{AcquireError, SwapchainCreateInfo};
use vulkano::VulkanLibrary;

use vulkano::sync::future::{FenceSignalFuture, NowFuture};
use vulkano::sync::{self, FlushError, GpuFuture};
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

use crate::deploy_shader;
use crate::pass_structs::WindowInitialized;
use crate::simulation::sand::{self, sand_shader::Material, PADDING};

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
        device: render_device,
        window,
        window_size,
        event_loop,
        queue: render_queue,
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
    let world_buffer = sand::upload_buffer(world, &compute_memory_allocator);
    let compute_shader_loaded =
        sand::sand_shader::load(compute_device.clone()).expect("Failed to create compute shader.");
    let deploy_command = Arc::new(deploy_shader::get_deploy_command(
        &compute_shader_loaded,
        &compute_device,
        &compute_queue,
        &world_buffer,
        work_groups,
    ));
    let mut next_future: Option<FenceSignalFuture<CommandBufferExecFuture<NowFuture>>> = None;
    // let frames_r = &mut frames; winit static garbo or smth, idk why this does not work.
    // let cur_frame_r = &mut cur_frame;
    // let time_r = &mut time;
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
                let _binding = world_buffer.read().unwrap();
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

use std::fs;

use simulation::ecs::Entity;
use simulation::sand::sand_shader::Hitbox;
use vulkano::buffer::BufferContents;

use vulkano::memory::allocator::{GenericMemoryAllocator, StandardMemoryAllocator};
use vulkano::padded::Padded;
use vulkano::sync::{self};

use rlua::{Lua, Table};

mod deploy_shader;
mod gpu_constructor;
mod lua_funcs;
mod pass_structs;
mod simulation;
mod window;

use simulation::sand::{sand_shader::Material, PADDING};
use window::init::fragment_shader::Sprite;

#[derive(BufferContents)]
#[repr(C)]
struct TestStruct {
    first: i32,
    second: i32,
    res: i32,
}

// device, queues,

macro_rules! handle_lua_elem {
    ($type_of:ty, $name:expr, $dest:ident, $value:expr, $building_mat:expr) => {
        let cv = $value.get::<&str, $type_of>($name);
        if cv.is_ok() {
            $building_mat.$dest = cv.unwrap();
        }
    };
}

macro_rules! handle_lua_vec {
    ($name:expr, $dest:ident, $count:expr, $value:expr, $building_mat:expr) => {
        let cv = $value.get::<&str, Table>($name);
        if cv.is_ok() {
            let mut vals = [0f32; $count];
            for elem in cv.unwrap().pairs::<usize, f32>() {
                let v = elem.unwrap();
                vals[v.0 - 1] = v.1;
            }
            $building_mat.$dest = vals;
        }
    };
}

fn main() {
    let lua_obj = Lua::new();
    let mut world: Vec<Padded<Material, PADDING>> = Vec::new();

    lua_obj.context(|ctx| {
        let content = fs::read_to_string("./init.lua").unwrap(); // load init func
        let data = ctx.load(&content[..]).eval::<Table>().unwrap();

        for elem in data.pairs::<usize, Table>() {
            let (_, value) = elem.unwrap();
            let mut building_mat = Material {
                ..Default::default()
            };
            // let cv = value.get::<&str, u32>("id");
            // if cv.is_ok() {
            //     building_mat.id = cv.unwrap();
            // }
            handle_lua_elem!(u32, "id", id, value, building_mat);
            handle_lua_elem!(f32, "mass", mass, value, building_mat);
            handle_lua_elem!(f32, "force", force, value, building_mat);
            handle_lua_elem!(f32, "stable", stable, value, building_mat);
            handle_lua_elem!(u32, "tags", tags, value, building_mat);
            handle_lua_elem!(u32, "gas", gas, value, building_mat);
            handle_lua_vec!("colour", colour, 3, value, building_mat);
            handle_lua_vec!("pos", pos, 2, value, building_mat);
            handle_lua_vec!("vel", vel, 2, value, building_mat);
            handle_lua_vec!("target", target, 2, value, building_mat);

            world.push(Padded::<Material, 4>(building_mat))
        }
    });

    let work_groups = [world.len() as u32 / 64u32, 1, 1]; // autocalc workgroups
    let (
        library,
        physical_device,
        _queue_family_index,
        _instance,
        device,
        mut queues,
        window,
        surface,
        event_loop,
        window_size,
    ) = gpu_constructor::construct_gpu();
    // -=-=-=-=-=

    let queue = queues.next().unwrap();

    let memory_allocator: GenericMemoryAllocator<
        std::sync::Arc<vulkano::memory::allocator::FreeListAllocator>,
    > = StandardMemoryAllocator::new_default(device.clone());

    let entities = vec![Entity {
        hitbox: Hitbox {
            pos: [0f32, 0f32],
            size: [0f32, 0f32],
            mass: 0f32,
            simulate: 0, // 0 && 1 for true and false because of shader weirdness.
        },
        sprite: Sprite {
            pos: [0.3f32, 0.1f32],
            size: [0.2f32, 0.5f32],
            offset: [0.3f32, 0.3f32],
            scale: [3.0f32, 3.0f32],
        },
        data: "".to_owned(),
    }];
    // let data2 = 0..64; //staging, gpu 1, gpu 2, download (eventually)

    // lua.context(|lua_ctx| {
    //     let globals = lua_ctx.globals();

    //     globals.set("string_var", "hello").unwrap();
    // });

    // lua.context(|lua_ctx| {
    //     let globals = lua_ctx.globals();
    // 	let val: String = globals.get("string_var").unwrap();
    // 	println!("{val:?}");
    // });

    window::make_window(
        library,
        memory_allocator,
        device,
        queue,
        world,
        work_groups,
        physical_device,
        window,
        surface,
        event_loop,
        window_size,
        entities,
        lua_obj,
    );
    //main.rs is done now as window now has control
}

pub fn test() {
    println!("luad");
}

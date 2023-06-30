use rlua::Context;

use crate::simulation::ecs::Entity;

pub fn create(lua_ctx: Context, entities: &mut Vec<Entity>) {
    let globals = lua_ctx.globals();
    let equality_lua = lua_ctx
        .create_function(|_, (a, b): (i32, i32)| Result::Ok(equality_rs(a, b)))
        .unwrap();
    globals.set("equality", equality_lua).unwrap();
}

fn equality_rs(a: i32, b: i32) -> bool {
    a == b
}

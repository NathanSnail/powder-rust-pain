use rlua::{Lua, Context};

pub fn create(lua_ctx: Context) {
    let globals = lua_ctx.globals();
    let equality = lua_ctx
        .create_function(|_, (a, v): (i32, i32)| Result::Ok(a == v))
        .unwrap();
    globals.set("equality", equality).unwrap();
}

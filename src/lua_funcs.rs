use rlua::{Context, Table, ToLuaMulti, Value::Nil};

use crate::simulation::ecs::Entity;

pub fn create(lua_ctx: Context, entities: Vec<Entity>) {
    let globals = lua_ctx.globals();

    let equality_lua = lua_ctx
        .create_function(|_, (a, b): (i32, i32)| Result::Ok(equality_rs(a, b)))
        .unwrap();
    globals.set("equality", equality_lua).unwrap();

    let get_entities_lua = lua_ctx
        .create_function(move |lua_ctx, _: rlua::Value| {
            Result::Ok(get_entities_rs(&lua_ctx, entities.clone()))
        })
        .unwrap();
    globals.set("GetEntities", get_entities_lua).unwrap();
}

fn equality_rs(a: i32, b: i32) -> bool {
    a == b
}

fn get_entities_rs<'a>(lua_ctx: &Context<'a>, entities: Vec<Entity>) -> Table<'a> {
    let table = lua_ctx.create_table().unwrap();
    let mut counter = 1;
    for (key, elem) in entities.iter().enumerate() {
        if elem.deleted {
            continue;
        }
        table.set(counter, key).unwrap();
		counter += 1;
    }
    table
}

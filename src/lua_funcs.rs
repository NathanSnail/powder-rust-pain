use rlua::{Context, Table, ToLuaMulti, Value::Nil};

use crate::simulation::ecs::Entity;

pub fn create(lua_ctx: Context, entities: Vec<Entity>, frame: usize, time: u128) {
    let globals = lua_ctx.globals();

    let e_1 = entities.clone();
    let get_entities_lua = lua_ctx
        .create_function(move |lua_ctx, _: rlua::Value| Result::Ok(get_entities(&lua_ctx, &e_1)))
        .unwrap();
    globals.set("GetEntities", get_entities_lua).unwrap();

    let e_2 = entities.clone();
    let get_entities_lua = lua_ctx
        .create_function(move |lua_ctx, id: usize| Result::Ok(get_entity_pos(&lua_ctx, &e_2, id)))
        .unwrap();
    globals.set("GetEntityPosition", get_entities_lua).unwrap();

    let e_3 = entities.clone();
    let get_entities_lua = lua_ctx
        .create_function(move |lua_ctx, id: usize| Result::Ok(get_entity_vel(&lua_ctx, &e_3, id)))
        .unwrap();
    globals.set("GetEntityBelocity", get_entities_lua).unwrap();

    let e_4 = entities.clone();
    let get_entities_lua = lua_ctx
        .create_function(move |_, id: usize| Result::Ok(get_entity_data(&e_4, id)))
        .unwrap();
    globals.set("GetEntityData", get_entities_lua).unwrap();

    let get_entities_lua = lua_ctx
        .create_function(move |_, _: rlua::Value| Result::Ok(get_cur_frame(frame)))
        .unwrap();
    globals.set("GetFrame", get_entities_lua).unwrap();

    let get_entities_lua = lua_ctx
        .create_function(move |_, _: rlua::Value| Result::Ok(get_cur_time(time)))
        .unwrap();
    globals.set("GetTime", get_entities_lua).unwrap();
}

fn get_entities<'a>(lua_ctx: &Context<'a>, entities: &[Entity]) -> Table<'a> {
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

fn get_entity_pos<'a>(lua_ctx: &Context<'a>, entities: &[Entity], id: usize) -> Table<'a> {
    let table = lua_ctx.create_table().unwrap();
    let pos = entities[id].hitbox.pos;
    table.set("x", pos[0]).unwrap();
    table.set("y", pos[1]).unwrap();
    table
}

fn get_entity_vel<'a>(lua_ctx: &Context<'a>, entities: &[Entity], id: usize) -> Table<'a> {
    let table = lua_ctx.create_table().unwrap();
    let vel = entities[id].hitbox.vel;
    table.set("x", vel[0]).unwrap();
    table.set("y", vel[1]).unwrap();
    table
}

fn get_entity_data<'a>(entities: &[Entity], id: usize) -> String {
    entities[id].data.clone()
}

fn get_cur_frame<'a>(frame: usize) -> usize {
    frame
}

fn get_cur_time<'a>(time: u128) -> u128 {
    time
}

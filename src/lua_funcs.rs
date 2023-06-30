use rlua::{
    Context, MultiValue, Table, ToLuaMulti,
    Value::{self, Nil},
};

use crate::simulation::ecs::Entity;

pub fn create(lua_ctx: Context, entities: Vec<Entity>, frame: usize, time: u128) {
    let globals = lua_ctx.globals();

    let e_1 = entities.clone();
    let temp_fn = lua_ctx
        .create_function(move |lua_ctx, _: rlua::Value| Result::Ok(get_entities(&lua_ctx, &e_1)))
        .unwrap();
    globals.set("GetEntities", temp_fn).unwrap();

    let e_2 = entities.clone();
    let temp_fn = lua_ctx
        .create_function(move |lua_ctx, id: usize| Result::Ok(get_entity_pos(&lua_ctx, &e_2, id)))
        .unwrap();
    globals.set("GetEntityPosition", temp_fn).unwrap();

    let e_3 = entities.clone();
    let temp_fn = lua_ctx
        .create_function(move |lua_ctx, id: usize| Result::Ok(get_entity_vel(&lua_ctx, &e_3, id)))
        .unwrap();
    globals.set("GetEntityVelocity", temp_fn).unwrap();

    let entities_clone = entities.clone();
    let temp_fn = lua_ctx
        .create_function(move |_, id: usize| Result::Ok(get_entity_data(&entities_clone, id)))
        .unwrap();
    globals.set("GetEntityData", temp_fn).unwrap();

    let temp_fn = lua_ctx
        .create_function(move |_, _: Value| Result::Ok(get_cur_frame(frame)))
        .unwrap();
    globals.set("GetFrame", temp_fn).unwrap();

    let temp_fn = lua_ctx
        .create_function(move |_, _: Value| Result::Ok(get_cur_time(time)))
        .unwrap();
    globals.set("GetTime", temp_fn).unwrap();

    let temp_fn = lua_ctx
        .create_function(move |lua_ctx, (id, path, value): (usize, String, Table)| {
            set_entity_value(lua_ctx, id, path, value);
            Result::Ok(())
        })
        .unwrap();
    globals.set("EntitySetComponentValue", temp_fn).unwrap();

    let temp_fn = lua_ctx
        .create_function(move |lua_ctx, (id, path): (usize, String)| {
            get_entity_value(lua_ctx, &entities, id, path).to_lua_multi(lua_ctx)
        })
        .unwrap();
    globals.set("EntityGetComponentValue", temp_fn).unwrap();
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

fn get_entity_data(entities: &[Entity], id: usize) -> String {
    entities[id].data.clone()
}

fn get_cur_frame(frame: usize) -> usize {
    frame
}

fn get_cur_time(time: u128) -> u128 {
    time
}

fn set_entity_value(lua_ctx: Context, id: usize, path: String, values: Table) {
    let mut cmd = "table.insert(deltas,{".to_owned();
    let value1 = &values.get::<usize, String>(1).unwrap()[..];
    let value2 = &(if values.len().unwrap() == 2 {
        values.get::<usize, String>(1).unwrap()
    } else {
        "0".to_owned()
    })[..];
    cmd.push_str(&id.to_string()[..]);
    cmd.push(',');
    cmd.push_str(&path[..]);
    cmd.push(',');
    cmd.push_str(&value1[..]);
    cmd.push(',');
    cmd.push_str(&value2[..]);
    cmd.push_str("})");
    lua_ctx.load(&cmd[..]).exec().unwrap();
}

enum EntityData<'a> {
    Vec2(Table<'a>),
    Float(f32),
    Bool(bool),
    String(String),
    Nil(),
}

impl ToLuaMulti<'_> for EntityData<'_> {
    fn to_lua_multi(self, lua: Context<'_>) -> rlua::Result<rlua::MultiValue<'_>> {
        match self {
            EntityData::Vec2(v) => {
                let table_n = lua.create_table().unwrap();
                table_n.set("x", v.get::<String, f32>("x".to_string()).unwrap()).unwrap();
                table_n.set("y", v.get::<String, f32>("y".to_string()).unwrap()).unwrap();
                rlua::Result::Ok(table_n.to_lua_multi(lua).unwrap())
            }
            EntityData::Float(v) => rlua::Result::Ok(v.to_lua_multi(lua).unwrap()),
            EntityData::Bool(v) => rlua::Result::Ok(v.to_lua_multi(lua).unwrap()),
            EntityData::String(v) => rlua::Result::Ok(v.to_lua_multi(lua).unwrap()),
            EntityData::Nil() => rlua::Result::Ok(Nil.to_lua_multi(lua).unwrap()),
        }
    }
}

fn get_entity_value<'a>(
    lua_ctx: Context<'a>,
    entities: &[Entity],
    id: usize,
    path: String,
) -> EntityData<'a> {
    let deltas = lua_ctx.globals().get("deltas");
    let mut val = EntityData::Nil();
    match &path[..] {
        // code duplication with minor differences, ideally id use macros but no time
        "sprite.pos" => {
            let [data1, data2] = entities[id].sprite.pos;
            let table = lua_ctx.create_table().unwrap();
            table.set("x", data1).unwrap();
            table.set("y", data2).unwrap();
            val = EntityData::Vec2(table);
        }
        "sprite.size" => {
            let [data1, data2] = entities[id].sprite.size;
            let table = lua_ctx.create_table().unwrap();
            table.set("x", data1).unwrap();
            table.set("y", data2).unwrap();
            val = EntityData::Vec2(table);
        }
        "sprite.offset" => {
            let [data1, data2] = entities[id].sprite.offset;
            let table = lua_ctx.create_table().unwrap();
            table.set("x", data1).unwrap();
            table.set("y", data2).unwrap();
            val = EntityData::Vec2(table);
        }
        "sprite.scale" => {
            let [data1, data2] = entities[id].sprite.scale;
            let table = lua_ctx.create_table().unwrap();
            table.set("x", data1).unwrap();
            table.set("y", data2).unwrap();
            val = EntityData::Vec2(table);
        }
        "hitbox.pos" => {
            let [data1, data2] = entities[id].hitbox.pos;
            let table = lua_ctx.create_table().unwrap();
            table.set("x", data1).unwrap();
            table.set("y", data2).unwrap();
            val = EntityData::Vec2(table);
        }
        "hitbox.size" => {
            let [data1, data2] = entities[id].hitbox.size;
            let table = lua_ctx.create_table().unwrap();
            table.set("x", data1).unwrap();
            table.set("y", data2).unwrap();
            val = EntityData::Vec2(table);
        }
        "hitbox.mass" => {
            val = EntityData::Float(entities[id].hitbox.mass);
        }
        "hitbox.simulate" => {
            val = EntityData::Bool(entities[id].hitbox.simulate == 1);
        }
        "data" => {
            val = EntityData::String(entities[id].data.clone());
        }
        "deleted" => {
            val = EntityData::Bool(entities[id].deleted);
        }
        _ => (panic!("invalid path")),
    }
    if deltas.is_ok() {
        let deltas: Table = deltas.unwrap();
        for elem in deltas.pairs::<usize, Table>() {
            let (_, value) = elem.unwrap();
            let eid: usize = value.get(1).unwrap();
            let cid: String = value.get(2).unwrap();
            if eid == id && cid == path {
                match &cid[..] {
                    "sprite.pos" => {
                        let data1: f32 = value.get(3).unwrap();
                        let data2: f32 = value.get(4).unwrap();
                        let table = lua_ctx.create_table().unwrap();
                        table.set("x", data1).unwrap();
                        table.set("y", data2).unwrap();
                        val = EntityData::Vec2(table);
                    }
                    "sprite.size" => {
                        let data1: f32 = value.get(3).unwrap();
                        let data2: f32 = value.get(4).unwrap();
                        let table = lua_ctx.create_table().unwrap();
                        table.set("x", data1).unwrap();
                        table.set("y", data2).unwrap();
                        val = EntityData::Vec2(table);
                    }
                    "sprite.offset" => {
                        let data1: f32 = value.get(3).unwrap();
                        let data2: f32 = value.get(4).unwrap();
                        let table = lua_ctx.create_table().unwrap();
                        table.set("x", data1).unwrap();
                        table.set("y", data2).unwrap();
                        val = EntityData::Vec2(table);
                    }
                    "sprite.scale" => {
                        let data1: f32 = value.get(3).unwrap();
                        let data2: f32 = value.get(4).unwrap();
                        let table = lua_ctx.create_table().unwrap();
                        table.set("x", data1).unwrap();
                        table.set("y", data2).unwrap();
                        val = EntityData::Vec2(table);
                    }
                    "hitbox.pos" => {
                        let data1: f32 = value.get(3).unwrap();
                        let data2: f32 = value.get(4).unwrap();
                        let table = lua_ctx.create_table().unwrap();
                        table.set("x", data1).unwrap();
                        table.set("y", data2).unwrap();
                        val = EntityData::Vec2(table);
                    }
                    "hitbox.size" => {
                        let data1: f32 = value.get(3).unwrap();
                        let data2: f32 = value.get(4).unwrap();
                        let table = lua_ctx.create_table().unwrap();
                        table.set("x", data1).unwrap();
                        table.set("y", data2).unwrap();
                        val = EntityData::Vec2(table);
                    }
                    "hitbox.mass" => {
                        let data1: f32 = value.get(3).unwrap();
                        let data2: f32 = value.get(4).unwrap();
                        let table = lua_ctx.create_table().unwrap();
                        table.set("x", data1).unwrap();
                        table.set("y", data2).unwrap();
                        val = EntityData::Vec2(table);
                    }
                    "hitbox.simulate" => {
                        let data1 = value.get(3).unwrap();
                        val = EntityData::Bool(data1);
                    }
                    "data" => {
                        let data1 = value.get(3).unwrap();
                        val = EntityData::String(data1);
                    }
                    "deleted" => {
                        let data1 = value.get(3).unwrap();
                        val = EntityData::Bool(data1);
                    }
                    _ => {
                        panic!("invalid path")
                    }
                }
            }
        }
    }
    val
}

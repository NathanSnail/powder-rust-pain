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

    let entities_clone = entities.clone(); // hackery to move to two different closures.
    let temp_fn = lua_ctx
        .create_function(move |lua_ctx, (id, path): (usize, String)| {
            get_entity_value(lua_ctx, &entities_clone, id, path).to_lua_multi(lua_ctx)
        })
        .unwrap();
    globals.set("EntityGetComponentValue", temp_fn).unwrap();

    let temp_fn = lua_ctx
        .create_function(move |lua_ctx, _: Value| {
            create_entity(lua_ctx, &entities);
            Result::Ok(())
        })
        .unwrap();
    globals.set("CreateEntity", temp_fn).unwrap();
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

fn get_cur_frame(frame: usize) -> usize {
    frame
}

fn get_cur_time(time: u128) -> u128 {
    time
}

fn set_entity_value(lua_ctx: Context, id: usize, path: String, values: Table) {
    let mut cmd = "RS_deltas = RS_deltas or {};table.insert(RS_deltas,{".to_owned();
    let v_typer = values.get::<usize, String>(1);
    let value1 = &if v_typer.is_err() {
        if values.get::<usize, bool>(1).unwrap() {
            "true".to_owned()
        } else {
            "false".to_owned()
        } // others get type coerced
    } else {
        values.get::<usize, String>(1).unwrap()
    }[..];
    let value2 = &(if values.len().unwrap() == 2 {
        values.get::<usize, String>(2).unwrap()
    } else {
        "0".to_owned()
    })[..];
    cmd.push_str(&id.to_string()[..]);
    cmd.push_str(",\"");
    cmd.push_str(&path[..]);
    cmd.push_str("\",");
    cmd.push_str(value1);
    cmd.push(',');
    cmd.push_str(value2);
    cmd.push_str("})");
    // println!("cmd");
    // println!("{cmd:?}");
    lua_ctx.load(&cmd[..]).exec().unwrap();
    // println!("cont");
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
                table_n
                    .set("x", v.get::<String, f32>("x".to_string()).unwrap())
                    .unwrap();
                table_n
                    .set("y", v.get::<String, f32>("y".to_string()).unwrap())
                    .unwrap();
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
    let RS_deltas = lua_ctx.globals().get("RS_deltas");
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
    if RS_deltas.is_ok() {
        let RS_deltas: Table = RS_deltas.unwrap();
        for elem in RS_deltas.pairs::<usize, Table>() {
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

fn create_entity<'a>(lua_ctx: Context<'a>, entities: &[Entity]) {
    let mut idx = 2u32.pow(30); // if you hit this you have bigger problems
    for (key, entity) in entities.iter().enumerate() {
        if entity.deleted == true {
            let mut cmd = r"
			RS_created = RS_created or {}
			for k,v in ipairs(RS_created) do
				print(k)
				print(v)
				if k == "
                .to_owned();
            cmd.push_str(&key.to_string()[..]);
            cmd.push_str(
                r"
				then
				return -1",
            );
            cmd.push_str(
                r"
			end
			end
			return 1",
            );
			println!("{cmd}");
            if lua_ctx.load(&cmd[..]).eval::<u32>().unwrap() == 1 {
                idx = key as u32;
                break;
            }
        }
    }
    let target = if idx >= 2u32.pow(30)
    // greater than lol
    {
        -1i32
    } else {
        idx as i32
    };
    let mut cmd = "RS_created = RS_created or {};table.insert(RS_created,".to_owned();
    cmd.push_str(&target.to_string()[..]);
    cmd.push(')');
    lua_ctx.load(&cmd[..]).exec().unwrap();
}

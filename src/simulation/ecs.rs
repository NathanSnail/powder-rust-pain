use std::default;

use rlua::Value::Nil;
use rlua::{Context, Function, Table};
use vulkano::padded::Padded;

use crate::lua_funcs;
use crate::simulation::sand::sand_shader::Hitbox;
use crate::window::init::fragment_shader::Sprite;
use vulkano::buffer::Subbuffer;

#[derive(Clone, Debug)]
pub struct Entity {
    pub sprite: Sprite,
    pub hitbox: Hitbox,
    pub data: String,
    pub deleted: bool,
}

impl Default for Entity {
    fn default() -> Self {
        Entity {
            sprite: Sprite {
                ..Default::default()
            },
            hitbox: Hitbox {
                ..Default::default()
            },
            data: "".to_owned(),
            deleted: false,
        }
    }
}

fn regen_from_gpu(entities: &mut Vec<Entity>, buffer: &Subbuffer<[Padded<Hitbox, 4>]>) {
    for (key, value) in buffer.read().unwrap().into_iter().enumerate() {
        entities[key].hitbox = **value;
    }
}

fn regen_from_cpu(
    entities: &Vec<Entity>,
    sprite_buffer: &mut Subbuffer<[Padded<Sprite, 4>]>,
    hitbox_buffer: &mut Subbuffer<[Padded<Hitbox, 4>]>,
) {
    let mut buffer_writer_sprite = sprite_buffer.write().unwrap(); // locks
    let mut buffer_writer_hitbox = hitbox_buffer.write().unwrap();

    // let mut c = 0;
    for (c, entity) in entities.iter().enumerate() {
        buffer_writer_sprite[c] = Padded::from(entity.sprite);
        buffer_writer_hitbox[c] = Padded::from(entity.hitbox);
        // c += 1;
        // let hb = entity.hitbox;
        // println!("{hb:?}");
    }
}

pub fn regenerate(
    entities: &mut Vec<Entity>,
    sprite_buffer: &mut Subbuffer<[Padded<Sprite, 4>]>,
    hitbox_buffer: &mut Subbuffer<[Padded<Hitbox, 4>]>,
    ctx: Context,
    frame: usize,
    time: u128,
) {
    regen_from_gpu(entities, hitbox_buffer); // gpu can only write to hitboxes
    lua_funcs::create(ctx, entities.clone(), frame, time); // rust safety requires this massive performance hit and general difficulty causer
    ctx.globals()
        .set("RS_deltas", ctx.create_table().unwrap())
        .unwrap(); // don't leak memory
    ctx.globals()
        .set("RS_created", ctx.create_table().unwrap())
        .unwrap(); // don't leak memory
    ctx.load("RS_tick_handle()").exec().unwrap();
    // println!("tick worked");
    // we have to apply the changes here because the rust lua crate I chose kind of sucks.
    let RS_deltas = ctx.globals().get("RS_deltas");
    if RS_deltas.is_ok() {
        let RS_deltas: Table = RS_deltas.unwrap();
        for elem in RS_deltas.pairs::<usize, Table>() {
            let (_, value) = elem.unwrap();
            let eid: usize = value.get(1).unwrap();
            let cid: String = value.get(2).unwrap();
            match &cid[..] {
                "sprite.pos" => {
                    // ugly match pattern
                    let data1 = value.get(3).unwrap();
                    let data2 = value.get(4).unwrap();
                    entities[eid].sprite.pos = [data1, data2];
                }
                "sprite.size" => {
                    let data1 = value.get(3).unwrap();
                    let data2 = value.get(4).unwrap();
                    entities[eid].sprite.size = [data1, data2];
                }
                "sprite.offset" => {
                    let data1 = value.get(3).unwrap();
                    let data2 = value.get(4).unwrap();
                    entities[eid].sprite.offset = [data1, data2];
                }
                "sprite.scale" => {
                    let data1 = value.get(3).unwrap();
                    let data2 = value.get(4).unwrap();
                    entities[eid].sprite.scale = [data1, data2];
                }
                "hitbox.pos" => {
                    let data1 = value.get(3).unwrap();
                    let data2 = value.get(4).unwrap();
                    entities[eid].hitbox.pos = [data1, data2];
                }
                "hitbox.size" => {
                    let data1 = value.get(3).unwrap();
                    let data2 = value.get(4).unwrap();
                    entities[eid].hitbox.size = [data1, data2];
                }
                "hitbox.mass" => {
                    let data1 = value.get(3).unwrap();
                    entities[eid].hitbox.mass = data1;
                }
                "hitbox.simulate" => {
                    let data1 = value.get(3).unwrap();
                    entities[eid].hitbox.simulate = if data1 { 1 } else { 0 };
                }
                "data" => {
                    entities[eid].data = value.get(3).unwrap();
                }
                "deleted" => {
                    entities[eid].deleted = value.get(3).unwrap();
                    // can't pop because then eid changes
                    entities[eid].hitbox.deleted = if value.get(3).unwrap() { 1 } else { 0 };
                    entities[eid].sprite.deleted = if value.get(3).unwrap() { 1 } else { 0 };
                }
                _ => {
                    panic!("invalid path")
                }
            }
        }
    }
    let RS_created: Table = ctx.globals().get("RS_created").unwrap();
    for elem in RS_created.pairs() {
        let (_, value): (u32, i32) = elem.unwrap();
        if value == -1 {
            panic!("Ran out of entities to write into");
        } else {
            entities[value as usize] = Entity {
                data: "clean".to_owned(), // mark the new entity as safe.
                ..Default::default()
            };
        }
    }
    regen_from_cpu(entities, sprite_buffer, hitbox_buffer);
}

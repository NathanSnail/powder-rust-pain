use rlua::Value::Nil;
use rlua::{Context, Function, Table};
use vulkano::padded::Padded;

use crate::lua_funcs;
use crate::simulation::sand::sand_shader::Hitbox;
use crate::window::init::fragment_shader::Sprite;
use vulkano::buffer::Subbuffer;

#[derive(Clone)]
pub struct Entity {
    pub sprite: Sprite,
    pub hitbox: Hitbox,
    pub data: String,
    pub deleted: bool,
}

fn regen_from_gpu(entities: &mut Vec<Entity>, buffer: &Subbuffer<[Padded<Hitbox, 0>]>) {
    for (key, value) in buffer.read().unwrap().into_iter().enumerate() {
        entities[key].hitbox = **value;
    }
}

fn regen_from_cpu(
    entities: &Vec<Entity>,
    sprite_buffer: &mut Subbuffer<[Padded<Sprite, 0>]>,
    hitbox_buffer: &mut Subbuffer<[Padded<Hitbox, 0>]>,
) {
    let mut sprites_collection: Vec<Padded<Sprite, 0>> = // anonymous lambdas
        entities.into_iter().map(|e| Padded(e.sprite)).collect();
    let mut hitbox_collection: Vec<Padded<Sprite, 0>> =
        entities.into_iter().map(|e| Padded(e.sprite)).collect();

    let mut buffer_writer_sprite = sprite_buffer.write().unwrap(); // locks
    let mut buffer_writer_hitbox = hitbox_buffer.write().unwrap();

    for (key, entity) in entities.iter().enumerate() {
        buffer_writer_sprite[key] = Padded::from(entity.sprite);
        buffer_writer_hitbox[key] = Padded::from(entity.hitbox);
    }
}

pub fn regenerate(
    entities: &mut Vec<Entity>,
    sprite_buffer: &mut Subbuffer<[Padded<Sprite, 0>]>,
    hitbox_buffer: &mut Subbuffer<[Padded<Hitbox, 0>]>,
    ctx: Context,
    frame: usize,
    time: u128,
) {
    regen_from_gpu(entities, hitbox_buffer); // gpu can only write to hitboxes
    lua_funcs::create(ctx, entities.clone(), frame, time); // rust safety requires this massive performance hit and general difficulty causer
    ctx.load("RS_tick_handle()").exec().unwrap();
    // we have to apply the changes here because the rust lua crate I chose kind of sucks.
    let deltas = ctx.globals().get("deltas");
    if deltas.is_ok() {
        let deltas: Table = deltas.unwrap();
        for elem in deltas.pairs::<usize, Table>() {
            let (_, value) = elem.unwrap();
            let eid: usize = value.get(1).unwrap();
            let cid: String = value.get(2).unwrap();
            match &cid[..] {
                "sprite.pos" => { // ugly match pattern
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
                "deleted" => entities[eid].deleted = value.get(3).unwrap(),
                _ => {
                    panic!("invalid path")
                }
            }
        }
    }

    regen_from_cpu(entities, sprite_buffer, hitbox_buffer);
}

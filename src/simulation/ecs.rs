use vulkano::padded::Padded;

use crate::simulation::sand::sand_shader::Hitbox;
use crate::window::init::fragment_shader::Sprite;
use vulkano::buffer::Subbuffer;

#[derive(Clone)]
pub struct Entity {
    pub sprite: Sprite,
    pub hitbox: Hitbox,
    pub data: String,
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
) {
    regen_from_gpu(entities, &hitbox_buffer); // gpu can only write to hitboxes
    regen_from_cpu(&entities, sprite_buffer, hitbox_buffer);
}

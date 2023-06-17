use crate::window::init::fragment_shader;

#[derive(Clone)]
pub struct Script {}
#[derive(Clone)]
pub struct Entity {
    pub pos: [f32; 2],
    pub sprite: fragment_shader::Sprite,
    pub scripts: Vec<Script>,
}

pub fn tick(entity: &mut Entity) {
    entity.pos = [0f32, 0f32];
}

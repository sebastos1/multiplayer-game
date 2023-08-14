pub use bevy::{prelude::*, render::camera::ScalingMode};
pub use bevy_ggrs::*;
pub use bevy_matchbox::{matchbox_socket::PeerId, prelude::*};
pub use bytemuck::{Pod, Zeroable};

pub mod spawning;
pub use spawning::*;
pub mod gameplay;
pub use gameplay::*;

#[derive(Resource)]
pub struct InputDirection {
    pub dir: Option<Vec2>,
}

#[derive(Component)]
pub struct PlayerText;

#[derive(Component, PartialEq)]
pub struct Ball {
    pub color: BallColor,
}

#[derive(Component)]
pub struct Cue;

#[derive(Component)]
pub struct Hole;

#[derive(PartialEq)]
pub enum BallColor {
    White,
    Black,
    Filled,
    Striped,
}

#[derive(Component, Debug, Default, Reflect)]
pub struct Velocity {
    pub vel: Vec2,
}
impl Velocity {
    pub fn zero() -> Self {
        Self { vel: Vec2::ZERO, }
    }
}

#[derive(Resource)]
pub struct Playing {
    pub ready: bool,
    pub playing: usize,
}

#[derive(Resource)]
pub struct BoardSize {
    pub width: f32,
    pub height: f32,
    pub ball_diameter: f32,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Pod, Zeroable)]
pub struct CueInput {
    pub dir_x: i8,
    pub dir_y: i8,
    pub power: u8,
}

pub struct GgrsConfig;
impl ggrs::Config for GgrsConfig {
    type Input = CueInput;
    type State = u8;
    type Address = PeerId;
}
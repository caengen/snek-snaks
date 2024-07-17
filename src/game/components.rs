use bevy::{prelude::*, utils::HashMap};
use derive_more::From;

use super::prelude::SnakeDirection;

#[derive(Resource)]
pub struct Paused(pub bool);

#[derive(Component)]
pub struct ExampleGameText;

#[derive(Component)]
pub struct PausedText;
#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct Dead;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum PhysicsSet {
    Movement,
    CollisionDetection,
}

#[derive(Debug, Component, From)]
pub struct Vel(pub Vec2);

#[derive(Debug, Component, From)]
pub struct Pos(pub Vec2);

#[derive(Debug, Component, From)]
/**
 * The Bounding component is used to represent the radius of an entity in pixels.
 */
pub struct Bounding(pub f32);

#[derive(Component)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

#[derive(Component)]
pub struct SnakeHead {
    pub direction: SnakeDirection,
}

#[derive(Component)]
pub struct SnakeBodyPart;

#[derive(Event)]
pub struct MoveAppleEvent;

#[derive(Event)]
pub struct GrowSnakeEvent(pub Entity);

#[derive(Component)]
pub struct Apple;

#[derive(Component)]
pub struct Tail;

#[derive(Component, Clone, PartialEq)]
pub struct Collidible;

// pub const KeyMap1: HashMap<KeyCode, Direction> = {
//     let mut key_map = HashMap::new();
//     key_map.insert(KeyCode::KeyW, Direction::Up);
//     key_map.insert(KeyCode::KeyA, Direction::Left);
//     key_map.insert(KeyCode::KeyS, Direction::Down);
//     key_map.insert(KeyCode::KeyD, Direction::Right);
//     key_map
// };

// pub const KeyMap2: HashMap<KeyCode, Direction> = {
//     let mut key_map = HashMap::new();
//     key_map.insert(KeyCode::ArrowUp, Direction::Up);
//     key_map.insert(KeyCode::ArrowLeft, Direction::Left);
//     key_map.insert(KeyCode::ArrowDown, Direction::Down);
//     key_map.insert(KeyCode::ArrowRight, Direction::Right);
//     key_map
// };

use bevy::prelude::*;
use derive_more::From;

#[derive(Resource)]
pub struct Paused(pub bool);

#[derive(Component)]
pub struct ExampleGameText;

#[derive(Component)]
pub struct PausedText;
#[derive(Component)]
pub struct ScoreText;

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
pub struct Bounding(pub f32);

#[derive(Component)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct SnakeHead {
    pub direction: Direction,
}

#[derive(Component)]
pub struct SnakeBodyPart;

#[derive(Event)]
pub struct SpawnAppleEvent;

#[derive(Event)]
pub struct GrowSnakeEvent;

#[derive(Component)]
pub struct Apple;

#[derive(Component)]
pub struct Tail;

#[derive(Component, Clone, PartialEq)]
pub struct Collidible;

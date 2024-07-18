use bevy::{
    prelude::{Bundle, Component, Entity, KeyCode},
    utils::HashMap,
};

#[derive(Debug, Clone, PartialEq)]
pub enum SnakeDirection {
    Left,
    Right,
    Up,
    Down,
}

// stuff you want to export to other mods
#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct ControlScheme {
    pub directional_controls: HashMap<KeyCode, SnakeDirection>,
}

impl ControlScheme {
    pub fn direction_changed(&self, key: &KeyCode) -> Option<SnakeDirection> {
        self.directional_controls.get(key).cloned()
    }
}

#[derive(Component)]
pub struct SnakeHeadRef(pub Option<Entity>);

#[derive(Component)]
pub struct BodyRef(pub Vec<Entity>);

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Named(pub String);

#[derive(Component)]
pub struct Score {
    pub value: u32,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub name: Named,
    pub control_scheme: ControlScheme,
    pub snake_head_ref: SnakeHeadRef,
    pub score: Score,
}

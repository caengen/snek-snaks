use self::{
    components::{Paused, PhysicsSet},
    effects::flick_system,
    systems::{example_update, game_keys, pause_controls, setup_player},
};
use crate::{GamePhase, GameState};
use bevy::prelude::*;
use components::{SnakeMoveDelta, SnakeMoveTimer, SpawnAppleEvent};
use systems::{move_snakes, spawn_apple_handler};

mod collision;
mod components;
mod effects;
pub mod prelude;
mod systems;

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnAppleEvent>()
            .add_systems(
                OnEnter(GameState::InGame),
                (setup_player, spawn_apple_handler),
            )
            .add_systems(Update, pause_controls.run_if(in_state(GameState::InGame)))
            .add_systems(
                FixedUpdate,
                (move_snakes).run_if(in_state(GamePhase::Playing)),
            )
            .add_systems(
                Update,
                (game_keys, example_update, flick_system, spawn_apple_handler)
                    .run_if(in_state(GamePhase::Playing)),
            )
            .configure_sets(
                Update,
                PhysicsSet::Movement.before(PhysicsSet::CollisionDetection),
            )
            .insert_resource(SnakeMoveTimer(Timer::from_seconds(
                1.0,
                TimerMode::Repeating,
            )))
            .insert_resource(SnakeMoveDelta(32.0))
            .insert_resource(Paused(false))
            .insert_resource(Time::<Fixed>::from_hz(5.0));
    }
}

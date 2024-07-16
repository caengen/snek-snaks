use self::{
    components::{Paused, PhysicsSet},
    effects::flick_system,
    systems::{example_update, game_keys, pause_controls, setup_player},
};
use crate::{GamePhase, GameState, Score};
use bevy::prelude::*;
use components::{GrowSnakeEvent, SpawnAppleEvent};
use systems::{
    check_apple_collision, check_death_collision, dead_controls, dead_text, grow_snake, init_game,
    move_snakes, spawn_apple_handler, update_score_text,
};

mod collision;
mod components;
mod effects;
pub mod prelude;
mod systems;

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnAppleEvent>()
            .add_event::<GrowSnakeEvent>()
            // SETUP
            .add_systems(
                OnEnter(GameState::InGame),
                (init_game, setup_player, spawn_apple_handler),
            )
            // Playing state
            .add_systems(Update, pause_controls.run_if(in_state(GameState::InGame)))
            .add_systems(
                FixedUpdate,
                (
                    move_snakes,
                    grow_snake,
                    check_apple_collision,
                    update_score_text.run_if(resource_changed::<Score>),
                )
                    .run_if(in_state(GamePhase::Playing)),
            )
            .add_systems(
                Update,
                (
                    check_death_collision,
                    example_update,
                    game_keys,
                    flick_system,
                    spawn_apple_handler,
                )
                    .run_if(in_state(GamePhase::Playing)),
            )
            // Dead state
            .add_systems(OnEnter(GamePhase::Dead), (dead_text))
            .add_systems(Update, (dead_controls).run_if(in_state(GamePhase::Dead)))
            .configure_sets(
                Update,
                PhysicsSet::Movement.before(PhysicsSet::CollisionDetection),
            )
            .insert_resource(Paused(false))
            .insert_resource(Time::<Fixed>::from_hz(5.0));
    }
}

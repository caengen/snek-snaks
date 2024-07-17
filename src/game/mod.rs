use self::{
    components::{Paused, PhysicsSet},
    effects::flick_system,
    systems::{example_update, game_keys, pause_controls, setup_players},
};
use crate::{GamePhase, GameState, Score};
use bevy::prelude::*;
use components::{GrowSnakeEvent, MoveAppleEvent};
use systems::{
    check_all_dead, check_apple_collision, check_death_collision, dead_controls, dead_text,
    grow_snake, init_game, move_apple_handler, move_snakes, tear_down_players, update_score_text,
};

mod collision;
mod components;
mod effects;
pub mod prelude;
mod systems;

pub const INITIAL_GAME_SPEED: f64 = 8.0;
pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MoveAppleEvent>()
            .add_event::<GrowSnakeEvent>()
            // SETUP
            .add_systems(
                OnEnter(GameState::InGame),
                (init_game, setup_players, move_apple_handler),
            )
            .add_systems(OnExit(GameState::InGame), (tear_down_players))
            // Playing state
            .add_systems(
                FixedUpdate,
                (
                    move_snakes,
                    grow_snake,
                    update_score_text.run_if(resource_changed::<Score>),
                )
                    .run_if(in_state(GamePhase::Playing)),
            )
            .add_systems(
                Update,
                (
                    pause_controls.run_if(in_state(GameState::InGame)),
                    (
                        check_all_dead,
                        check_death_collision,
                        check_apple_collision,
                        example_update,
                        game_keys,
                        flick_system,
                        move_apple_handler,
                    )
                        .run_if(in_state(GamePhase::Playing)),
                    (dead_controls).run_if(in_state(GamePhase::Dead)),
                ),
            )
            // Dead state
            .add_systems(OnEnter(GamePhase::Dead), (dead_text))
            .configure_sets(
                Update,
                PhysicsSet::Movement.before(PhysicsSet::CollisionDetection),
            )
            .insert_resource(Paused(false))
            .insert_resource(Time::<Fixed>::from_hz(INITIAL_GAME_SPEED));
    }
}

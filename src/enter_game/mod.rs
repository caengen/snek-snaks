use bevy::{ecs::schedule::Stepping, prelude::*, utils::HashMap};

use crate::{
    game::prelude::{ControlScheme, Named, Player, PlayerBundle, SnakeDirection, SnakeHeadRef},
    GameState,
};

pub struct EnterGamePlugin;
impl Plugin for EnterGamePlugin {
    fn build(&self, app: &mut App) {
        let mut stepping = Stepping::new();
        stepping.add_schedule(Update);

        app.add_systems(OnEnter(GameState::EnterGame), init_char_selection)
            .add_systems(Update, on_enter_keys)
            .insert_resource(stepping);
    }
}

pub fn on_enter_keys(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::InGame);
    }
}
pub fn init_char_selection(mut commands: Commands, asset_server: Res<AssetServer>) {
    let key_map_1: HashMap<KeyCode, SnakeDirection> = {
        let mut key_map = HashMap::new();
        key_map.insert(KeyCode::ArrowUp, SnakeDirection::Up);
        key_map.insert(KeyCode::ArrowLeft, SnakeDirection::Left);
        key_map.insert(KeyCode::ArrowDown, SnakeDirection::Down);
        key_map.insert(KeyCode::ArrowRight, SnakeDirection::Right);
        key_map
    };
    let key_map_2: HashMap<KeyCode, SnakeDirection> = {
        let mut key_map = HashMap::new();
        key_map.insert(KeyCode::KeyW, SnakeDirection::Up);
        key_map.insert(KeyCode::KeyA, SnakeDirection::Left);
        key_map.insert(KeyCode::KeyS, SnakeDirection::Down);
        key_map.insert(KeyCode::KeyD, SnakeDirection::Right);
        key_map
    };

    commands.spawn(PlayerBundle {
        player: Player {},
        name: Named("Player 1".to_string()),
        control_scheme: ControlScheme {
            directional_controls: key_map_1,
        },
        snake_head_ref: SnakeHeadRef(None),
    });
    commands.spawn(PlayerBundle {
        player: Player {},
        name: Named("Player 2".to_string()),
        control_scheme: ControlScheme {
            directional_controls: key_map_2,
        },
        snake_head_ref: SnakeHeadRef(None),
    });

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            StateScoped(GameState::EnterGame),
        ))
        .with_children(|builder| {
            builder
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(40.),
                        display: Display::Flex,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                // .insert(
                //     NineSliceUiTexture::from_slice(
                //         server.load("panel_atlas.png"),
                //         Rect::new(0., 0., 32., 32.),
                //     )
                //     .with_blend_color(Color::from(LinearRgba::RED))
                //     .with_blend_mix(0.5),
                // )
                .with_children(|builder| {
                    builder.spawn((TextBundle {
                        text: Text::from_section(
                            "SPACE TO START",
                            TextStyle {
                                font_size: 50.,
                                color: Color::WHITE,
                                font: asset_server.load("fonts/visitor.ttf"),
                                ..default()
                            },
                        ),
                        ..default()
                    },));
                });
        });
}

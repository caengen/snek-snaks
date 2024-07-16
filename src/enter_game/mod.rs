pub struct EnterGamePlugin;
use bevy::{prelude::*, transform::commands};

use crate::GameState;

impl Plugin for EnterGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::EnterGame), (init_char_selection))
            .add_systems(Update, (on_enter_keys));
    }
}

pub fn on_enter_keys(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::InGame);
    }
}
pub fn init_char_selection(mut commands: Commands, asset_server: Res<AssetServer>) {
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

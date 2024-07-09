use bevy::ui::ContentSize;
use bevy::{math::vec2, prelude::*};
use bevy_turborand::DelegatedRng;
use bevy_turborand::{GlobalRng, RngComponent};
use std::borrow::Borrow;
use std::time::Duration;

use crate::{GamePhase, GameState};

use super::{
    components::{
        AnimationIndices, AnimationTimer, ExampleGameText, Paused, PausedText, Player, Pos, Vel,
    },
    effects::Flick,
};

pub fn pause_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    curr_state: Res<State<GamePhase>>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut pause_texts: Query<&mut Visibility, With<PausedText>>,
) {
    if keyboard.just_pressed(KeyCode::KeyP) {
        let mut new_state = curr_state.clone();
        match curr_state.get() {
            GamePhase::Playing => {
                new_state = GamePhase::Paused;
                next_state.set(new_state.clone());
            }
            GamePhase::Paused => {
                new_state = GamePhase::Playing;
                next_state.set(new_state.clone());
            }
            _ => {}
        }
        if next_state.is_changed() {
            for mut vis in pause_texts.iter_mut() {
                match new_state {
                    GamePhase::Playing => *vis = Visibility::Hidden,
                    GamePhase::Paused => *vis = Visibility::Inherited,
                }
            }
        }
    }
}

pub fn game_keys(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player: Query<(
        &Player,
        &mut Transform,
        &mut AnimationIndices,
        &mut TextureAtlas,
        &mut Sprite,
        &mut AnimationTimer,
    )>,
) {
    let mut direction = Vec2::ZERO;

    if keyboard.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        direction.x -= 1.0;
    }
    if keyboard.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        direction.x += 1.0;
    }
    if keyboard.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        direction.y += 1.0;
    }
    if keyboard.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        direction.y -= 1.0;
    }

    let move_speed = 1.0;
    let move_delta = (direction * move_speed).extend(0.0);

    for (_, mut transform, mut indices, mut atlas, mut sprite, mut timer) in player.iter_mut() {
        if direction == Vec2::ZERO {
            // update animation
            indices.first = 0;
            indices.last = 1;
            atlas.index = usize::clamp(atlas.index, indices.first, indices.last);
            timer.0.set_duration(Duration::from_millis(500));
            continue;
        }

        transform.translation += move_delta;

        // update animation
        indices.first = 2;
        indices.last = 3;
        atlas.index = usize::clamp(atlas.index, indices.first, indices.last);
        if move_delta.x < 0.0 {
            sprite.flip_x = true;
        } else if move_delta.x > 0.0 {
            sprite.flip_x = false;
        }
        timer.0.set_duration(Duration::from_millis(200));
    }
}

pub fn example_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut global_rng: ResMut<GlobalRng>,
) {
    let mut rng = RngComponent::from(&mut global_rng);
    // Text with multiple sections
    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([TextSection::new(
            "~In Game~",
            TextStyle {
                font: asset_server.load("fonts/visitor.ttf"),
                font_size: 40.0,
                color: Color::WHITE,
            },
        )])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(15.0),
            ..default()
        }),
        Vel(vec2(100.0 + rng.f32() * 1.5, 100.0 + rng.f32() * 1.5)),
        Pos(vec2(5.0, 15.0)),
        ExampleGameText,
        Flick {
            duration: Timer::from_seconds(60.0, TimerMode::Once),
            switch_timer: Timer::from_seconds(0.2, TimerMode::Repeating),
        },
    ));
    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([TextSection::new(
            "Paused",
            TextStyle {
                font: asset_server.load("fonts/visitor.ttf"),
                font_size: 20.0,
                color: Color::WHITE,
            },
        )])
        .with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            ..default()
        }),
        Vel(vec2(0.10, 0.10)),
        Pos(vec2(5.0, 15.0)),
        ExampleGameText,
        PausedText,
    ));
}

pub fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture_handle = asset_server.load("textures/chars/char_atlas.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 4, 1, None, None);
    let atlas_layout = texture_atlases.add(layout);
    let anim_indices = AnimationIndices { first: 0, last: 1 };
    commands.spawn((
        TextureAtlas {
            layout: atlas_layout,
            index: anim_indices.first,
            ..Default::default()
        },
        SpriteBundle {
            texture: texture_handle,
            transform: Transform::IDENTITY.with_scale(Vec3::splat(6.)),
            ..Default::default()
        },
        anim_indices,
        AnimationTimer(Timer::from_seconds(0.5, TimerMode::Repeating)),
        Player {},
    ));
}

// pub fn setup_level(mut commands: Commands, asset_server: Res<AssetServer>) {
//     // Size of the tile map in tiles.
//     let map_size = TilemapSize { x: 32, y: 32 };

//     // To create a map we use the TileStorage component.
//     // This component is a grid of tile entities and is used to help keep track of individual
//     // tiles in the world. If you have multiple layers of tiles you would have a Tilemap2dStorage
//     // component per layer.
//     let mut tile_storage = TileStorage::empty(map_size);

//     // For the purposes of this example, we consider a tilemap with rectangular tiles.
//     let map_type = TilemapType::Square;

//     let tilemap_entity = commands.spawn_empty().id();

//     // Spawn a 32 by 32 tilemap.
//     // Alternatively, you can use helpers::fill_tilemap.
//     for x in 0..map_size.x {
//         for y in 0..map_size.y {
//             let tile_pos = TilePos { x, y };
//             let tile_entity = commands
//                 .spawn(TileBundle {
//                     position: tile_pos,
//                     tilemap_id: TilemapId(tilemap_entity),
//                     ..Default::default()
//                 })
//                 .id();
//         }
//     }
// }

pub fn teardown(mut commands: Commands, texts: Query<Entity, With<ExampleGameText>>) {
    for entity in texts.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn example_update(
    window: Query<&Window>,
    mut texts: Query<(&mut Style, &mut Pos, &mut Vel), With<ExampleGameText>>,
    time: Res<Time>,
) {
    let window = window.get_single().unwrap();
    for (mut style, mut pos, mut vel) in texts.iter_mut() {
        pos.0.y += vel.0.y * time.delta_seconds();
        pos.0.x += vel.0.x * time.delta_seconds();

        if pos.0.y > window.height() {
            pos.0.y = window.height();
            vel.0.y *= -1.0;
        } else if pos.0.y < 0.0 {
            pos.0.y = 0.0;
            vel.0.y *= -1.0;
        }
        if pos.0.x > window.width() {
            pos.0.x = window.width();
            vel.0.x *= -1.0;
        } else if pos.0.x < 0.0 {
            pos.0.x = 0.0;
            vel.0.x *= -1.0;
        }

        style.top = Val::Px(pos.0.y);
        style.left = Val::Px(pos.0.x);
    }
}

pub fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut TextureAtlas)>,
) {
    for (indices, mut timer, mut atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            atlas.index = if atlas.index == indices.last {
                indices.first
            } else {
                atlas.index + 1
            };
        }
    }
}

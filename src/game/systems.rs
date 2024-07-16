use std::time::Duration;

use bevy::a11y::accesskit::Size;
use bevy::ecs::entity;
use bevy::prelude::*;
use bevy::transform::commands;
use bevy_turborand::DelegatedRng;
use bevy_turborand::{GlobalRng, RngComponent};
use bevy_tween::tween::{AnimationTarget, IntoTarget};

use crate::{GamePhase, GameState, Score, SCREEN};

use super::collision::circles_touching;
use super::components::{
    Apple, Bounding, Collidible, Direction, ExampleGameText, GrowSnakeEvent, PausedText, Player,
    Pos, ScoreText, SnakeBodyPart, SnakeHead, SpawnAppleEvent, Tail, Vel,
};
use super::effects::Flick;

const TILE_SIZE: f32 = 32.;
const WORLD_SIZE_X: u32 = 40;
const WORLD_SIZE_Y: u32 = 22;
// const TILE_SIZE: f32 = 16.;
// const WORLD_SIZE_X: u32 = 80;
// const WORLD_SIZE_Y: u32 = 45;

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
                    _ => {}
                }
            }
        }
    }
}

fn valid_direction(prev: &Direction, new: &Direction) -> bool {
    match prev {
        Direction::Left => *new != Direction::Right,
        Direction::Right => *new != Direction::Left,
        Direction::Up => *new != Direction::Down,
        Direction::Down => *new != Direction::Up,
    }
}

pub fn game_keys(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut snakes: Query<(&mut Transform, &mut SnakeHead), With<Player>>,
) {
    let mut direction = None;

    if keyboard.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        direction = Some(Direction::Left);
    }
    if keyboard.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        direction = Some(Direction::Right);
    }
    if keyboard.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        direction = Some(Direction::Up);
    }
    if keyboard.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        direction = Some(Direction::Down);
    }

    if let Some(direction) = direction {
        for (mut transform, mut head) in snakes.iter_mut() {
            if !valid_direction(&head.direction, &direction) {
                continue;
            }

            head.direction = direction.clone();
        }
    }
}

pub fn spawn_apple_handler(
    mut ev_spawn_apple: EventReader<SpawnAppleEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    for _ in ev_spawn_apple.read() {
        let apple_texture = asset_server.load("textures/chars/char_atlas.png");
        let apple_layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 4, 2, None, None);
        let apple_atlas_layout = texture_atlases.add(apple_layout);

        let mut rng = GlobalRng::new();
        let x_tiles = WORLD_SIZE_X as f32 / 2.;
        let y_tiles = WORLD_SIZE_Y as f32 / 2.;
        let x = rng.f32() * x_tiles * TILE_SIZE - TILE_SIZE / 2.;
        let x_values = [x, -x];
        let x = rng.sample(&x_values);

        let y = rng.f32() * y_tiles * TILE_SIZE - TILE_SIZE / 2.;
        let y_values = [y, -y];
        let y = rng.sample(&y_values);

        commands.spawn((
            TextureAtlas {
                layout: apple_atlas_layout.clone(),
                index: 7,
                ..Default::default()
            },
            SpriteBundle {
                texture: apple_texture.clone(),
                transform: Transform::from_translation(Vec3::new(*x.unwrap(), *y.unwrap(), 2.))
                    .with_scale(Vec3::splat(2.)),
                ..Default::default()
            },
            Apple,
            Bounding(8.),
            StateScoped(GameState::InGame),
        ));
    }
}

pub fn update_score_text(score: Res<Score>, mut query: Query<&mut Text, With<ScoreText>>) {
    for mut text in query.iter_mut() {
        text.sections[0].value = score.value.to_string();
    }
}

pub fn init_game(mut commands: Commands, mut score: ResMut<Score>, asset_server: Res<AssetServer>) {
    score.value = 0;

    // Score Text
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
            StateScoped(GameState::InGame),
        ))
        .with_children(|builder| {
            builder
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(10.),
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
                    builder.spawn((
                        TextBundle {
                            text: Text::from_section(
                                score.value.to_string(),
                                TextStyle {
                                    font_size: 50.,
                                    color: Color::WHITE,
                                    font: asset_server.load("fonts/visitor.ttf"),
                                    ..default()
                                },
                            ),
                            ..default()
                        },
                        ScoreText,
                    ));
                });
        });
}

pub fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut spawn_apple: EventWriter<SpawnAppleEvent>,
) {
    let char_texture = asset_server.load("textures/chars/char_atlas.png");
    let body_layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 6, 1, None, None);
    let body_atlas_layout = texture_atlases.add(body_layout);

    let tail_layout = TextureAtlasLayout::from_grid(UVec2::new(32, 16), 3, 3, None, None);
    let tail_atlas_layout = texture_atlases.add(tail_layout);

    // let head_layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 4, 1, None, None);
    // let head_atlas_layout = texture_atlases.add(head_layout);

    let head_sprite = AnimationTarget.into_target();

    let mut head_pos = Transform::IDENTITY;
    head_pos.translation = Vec3::new(-TILE_SIZE / 2., -TILE_SIZE / 2., 0.);

    // spawn head
    commands.spawn((
        TextureAtlas {
            layout: body_atlas_layout.clone(),
            index: 5,
            ..Default::default()
        },
        SpriteBundle {
            texture: char_texture.clone(),
            transform: Transform::IDENTITY.with_scale(Vec3::splat(2.)),
            ..Default::default()
        },
        AnimationTarget,
        Player {},
        SnakeHead {
            direction: Direction::Right,
        },
        Bounding(TILE_SIZE / 2.),
        StateScoped(GameState::InGame),
    ));
    head_pos.translation.x -= TILE_SIZE;

    // spawn body
    for _ in 0..2 {
        spawn_body_part(
            &mut commands,
            &body_atlas_layout,
            4,
            &char_texture,
            &head_pos,
        );
        head_pos.translation.x -= TILE_SIZE;
    }
    let tail = spawn_body_part(
        &mut commands,
        &body_atlas_layout,
        // 3 need to make more textures,
        4,
        &char_texture,
        &head_pos,
    );

    commands.entity(tail).insert(Tail);
    spawn_apple.send(SpawnAppleEvent);
    // .animation()
    // .repeat(Repeat::Infinitely)
    // .insert_tween_here(
    //     Duration::from_millis(500),
    //     EaseFunction::Linear,
    //     head_sprite.with(atlas_index(0, 2)),
    // );
}

pub fn grow_snake(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut grow_snake: EventReader<GrowSnakeEvent>,
    mut tail: Query<(Entity, &Transform, &mut TextureAtlas), With<Tail>>,
) {
    for _ in grow_snake.read() {
        let full_texture = asset_server.load("textures/chars/char_atlas.png");
        let body_layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 5, 1, None, None);
        let body_atlas_layout = texture_atlases.add(body_layout);

        let tail_layout = TextureAtlasLayout::from_grid(UVec2::new(32, 16), 3, 3, None, None);
        let tail_atlas_layout = texture_atlases.add(tail_layout);

        let (old_tail, transform, mut old_atlas) = tail.single_mut();
        let new_tail = spawn_body_part(
            &mut commands,
            &body_atlas_layout,
            // 3, TODO need to make more textures
            4,
            &full_texture,
            transform,
        );
        commands.entity(old_tail).remove::<Tail>();

        // set old tail to body part
        old_atlas.layout = body_atlas_layout.clone();
        old_atlas.index = 4;

        commands.entity(new_tail).insert(Tail);
    }
}

fn spawn_body_part(
    commands: &mut Commands,
    layout: &Handle<TextureAtlasLayout>,
    index: usize,
    texture: &Handle<Image>,
    pos: &Transform,
) -> Entity {
    commands
        .spawn((
            TextureAtlas {
                layout: layout.clone(),
                index,
                ..Default::default()
            },
            SpriteBundle {
                texture: texture.clone(),
                transform: pos.clone().with_scale(Vec3::splat(2.)),
                ..Default::default()
            },
            AnimationTarget,
            SnakeBodyPart,
            Collidible,
            Bounding(TILE_SIZE / 2.),
            StateScoped(GameState::InGame),
        ))
        .id()
}

pub fn move_snakes(
    mut head_query: Query<(&mut Transform, &SnakeHead), Without<SnakeBodyPart>>,
    mut snake_body_parts: Query<(&mut Transform), With<SnakeBodyPart>>,
) {
    for (mut transform, head) in head_query.iter_mut() {
        let move_speed = TILE_SIZE;
        let move_delta = match head.direction {
            Direction::Left => Vec3::new(-move_speed, 0., 0.),
            Direction::Right => Vec3::new(move_speed, 0., 0.),
            Direction::Up => Vec3::new(0., move_speed, 0.),
            Direction::Down => Vec3::new(0., -move_speed, 0.),
        };

        let mut prev_pos = transform.translation;
        transform.translation += move_delta;

        // move body
        let len = snake_body_parts.iter().len();
        for (i, mut part_transform) in snake_body_parts.iter_mut().enumerate() {
            let old = part_transform.translation;
            part_transform.translation = prev_pos;
            if i + 1 == len {
                // if prev above
                if prev_pos.y > old.y {
                    part_transform.rotation = Quat::from_rotation_z(90.0f32.to_radians());
                } else if prev_pos.y < old.y {
                    part_transform.rotation = Quat::from_rotation_z(-90.0f32.to_radians());
                } else if prev_pos.x > old.x {
                    part_transform.rotation = Quat::from_rotation_z(0.0f32.to_radians());
                } else if prev_pos.x < old.x {
                    part_transform.rotation = Quat::from_rotation_z(180.0f32.to_radians());
                }
            }
            prev_pos = old;
        }

        // rotate head
        match head.direction {
            Direction::Left => transform.rotation = Quat::from_rotation_z(-180.0f32.to_radians()),
            Direction::Right => transform.rotation = Quat::from_rotation_z(0.0f32.to_radians()),
            Direction::Up => transform.rotation = Quat::from_rotation_z(90.0f32.to_radians()),
            Direction::Down => transform.rotation = Quat::from_rotation_z(-90.0f32.to_radians()),
        }
    }
}

pub fn check_death_collision(
    mut commands: Commands,
    mut head_query: Query<(&Transform, &SnakeHead, &Bounding)>,
    mut collidibles: Query<(&Transform, &Bounding), With<Collidible>>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    let (head_transform, _, head_size) = head_query.single_mut();
    let head_pos = head_transform.translation;

    for (collidable_transform, collidable_size) in collidibles.iter() {
        if circles_touching(
            head_transform,
            head_size,
            collidable_transform,
            collidable_size,
        ) {
            next_state.set(GamePhase::Dead);
        }
    }

    if head_pos.x < -(WORLD_SIZE_X as f32 / 2. as f32 * TILE_SIZE)
        || head_pos.x > WORLD_SIZE_X as f32 / 2. as f32 * TILE_SIZE
        || head_pos.y < -(WORLD_SIZE_Y as f32 / 2. as f32 * TILE_SIZE)
        || head_pos.y > WORLD_SIZE_Y as f32 / 2. as f32 * TILE_SIZE
    {
        next_state.set(GamePhase::Dead);
    }
}

pub fn check_apple_collision(
    mut commands: Commands,
    mut head_query: Query<(&Transform, &SnakeHead, &Bounding), Without<SnakeBodyPart>>,
    apple_query: Query<(Entity, &Transform, &Bounding), With<Apple>>,
    mut spawn_apple: EventWriter<SpawnAppleEvent>,
    mut grow_snake: EventWriter<GrowSnakeEvent>,
    mut fixed_time: ResMut<Time<Fixed>>,
    mut score: ResMut<Score>,
) {
    let (head_transform, _, head_size) = head_query.single_mut();
    for (apple_entity, apple_transform, apple_size) in apple_query.iter() {
        if circles_touching(apple_transform, apple_size, head_transform, head_size) {
            // EATEN
            commands.entity(apple_entity).despawn();
            spawn_apple.send(SpawnAppleEvent);
            grow_snake.send(GrowSnakeEvent);

            score.value += 1;

            let new_timestep = fixed_time.timestep().mul_f32(0.95);
            fixed_time.set_timestep(new_timestep);
        }
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

pub fn dead_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::EnterGame);
    }
}

pub fn dead_text(mut commands: Commands, asset_server: Res<AssetServer>, score: Res<Score>) {
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
            StateScoped(GameState::InGame),
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
                            format!("You crashed! Final score {}", score.value.to_string()),
                            TextStyle {
                                font_size: 30.,
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

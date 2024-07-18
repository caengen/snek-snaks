use bevy::prelude::*;
use bevy_egui::egui::style;
use bevy_turborand::DelegatedRng;
use bevy_turborand::{GlobalRng, RngComponent};
use bevy_tween::tween::{AnimationTarget, IntoTarget};

use crate::{GamePhase, GameState, SCREEN};

use super::collision::circles_touching;
use super::components::{
    Apple, Bounding, Collidible, Dead, ExampleGameText, GameEntityRef, GrowSnakeEvent,
    MoveAppleEvent, PausedText, Pos, ScoreText, SnakeBodyPart, SnakeHead, Tail, Vel,
};
use super::prelude::{BodyRef, ControlScheme, Player, Score, SnakeDirection, SnakeHeadRef};
use super::INITIAL_GAME_SPEED;

const TILE_SIZE: f32 = 32.;
const SPLAT_SIZE: f32 = 2.;
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

fn valid_direction(prev: &SnakeDirection, new: &SnakeDirection) -> bool {
    match prev {
        SnakeDirection::Left => *new != SnakeDirection::Right,
        SnakeDirection::Right => *new != SnakeDirection::Left,
        SnakeDirection::Up => *new != SnakeDirection::Down,
        SnakeDirection::Down => *new != SnakeDirection::Up,
    }
}

pub fn game_keys(
    keyboard: Res<ButtonInput<KeyCode>>,
    players: Query<(&ControlScheme, &SnakeHeadRef), With<Player>>,
    mut snake_heads: Query<&mut SnakeHead>,
) {
    for (controls, snake_head_ref) in players.iter() {
        let mut direction = None;
        for key in keyboard.get_just_pressed() {
            direction = controls.direction_changed(key);
        }

        if let Some(direction) = direction {
            let mut snake_head = snake_heads.get_mut(snake_head_ref.0.unwrap()).unwrap();

            if !valid_direction(&snake_head.direction, &direction) {
                continue;
            }

            snake_head.direction = direction.clone();
        }
    }

    // let mut direction = None;

    // if keyboard.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
    //     direction = Some(SnakeDirection::Left);
    // }
    // if keyboard.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
    //     direction = Some(SnakeDirection::Right);
    // }
    // if keyboard.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
    //     direction = Some(SnakeDirection::Up);
    // }
    // if keyboard.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
    //     direction = Some(SnakeDirection::Down);
    // }
}

fn get_random_point() -> Vec2 {
    let mut rng = GlobalRng::new();
    let x_tiles = WORLD_SIZE_X as f32 / 2.;
    let y_tiles = WORLD_SIZE_Y as f32 / 2.;
    let x = rng.f32() * x_tiles * TILE_SIZE;
    let x_values = [x, -x];
    let x = rng.sample(&x_values);

    let y = rng.f32() * y_tiles * TILE_SIZE;
    let y_values = [y, -y];
    let y = rng.sample(&y_values);

    Vec2::new(*x.unwrap(), *y.unwrap())
}

pub fn move_apple_handler(
    mut ev_spawn_apple: EventReader<MoveAppleEvent>,
    mut apple_query: Query<(Entity, &mut Transform), With<Apple>>,
) {
    for _ in ev_spawn_apple.read() {
        let p = get_random_point();
        let mut apple = apple_query.get_single_mut().unwrap();
        apple.1.translation = p.extend(0.);
    }
}

pub fn update_score_text(
    player_query: Query<&Score, With<Player>>,
    mut text_query: Query<(&mut Text, &GameEntityRef), With<ScoreText>>,
) {
    for (mut text, ge_ref) in text_query.iter_mut() {
        let score = player_query.get(ge_ref.0).unwrap();
        text.sections[0].value = score.value.to_string();
    }
}

pub fn init_game(
    mut commands: Commands,
    score_query: Query<(Entity, &Score)>,
    asset_server: Res<AssetServer>,
    mut fixed_time: ResMut<Time<Fixed>>,
) {
    fixed_time.set_timestep_hz(INITIAL_GAME_SPEED);

    let score_len = score_query.iter().len();

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
                        display: Display::Grid,
                        grid_auto_flow: GridAutoFlow::Column,
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
                    for (entity, score) in score_query.iter() {
                        builder.spawn((
                            TextBundle {
                                style: Style {
                                    margin: UiRect {
                                        left: Val::Auto,
                                        right: Val::Auto,
                                        ..default()
                                    },
                                    ..default()
                                },
                                text: Text::from_section(
                                    score.value.to_string(),
                                    TextStyle {
                                        font_size: 50.,
                                        color: Color::WHITE,
                                        font: asset_server.load("fonts/visitor.ttf"),
                                    },
                                ),
                                ..default()
                            },
                            ScoreText,
                            GameEntityRef(entity),
                        ));
                    }
                });
        });
}

pub fn setup_players(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut snake_players: Query<(Entity, &mut SnakeHeadRef), With<Player>>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
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
    println!("spawning player");
    for (i, (_, mut snake_head_ref)) in snake_players.iter_mut().enumerate() {
        head_pos = Transform::from_translation(Vec3::new(0.0, i as f32 * TILE_SIZE * 2., 0.))
            .with_scale(Vec3::splat(SPLAT_SIZE));
        println!("spawned player");
        // spawn head
        let head_entity = commands
            .spawn((
                TextureAtlas {
                    layout: body_atlas_layout.clone(),
                    index: 5,
                    ..Default::default()
                },
                SpriteBundle {
                    texture: char_texture.clone(),
                    transform: head_pos.clone(),
                    ..Default::default()
                },
                AnimationTarget,
                SnakeHead {
                    direction: SnakeDirection::Right,
                },
                Bounding(TILE_SIZE / 2.),
                StateScoped(GameState::InGame),
            ))
            .id();
        *snake_head_ref = SnakeHeadRef(Some(head_entity));
        head_pos.translation.x -= TILE_SIZE;

        let mut body_ref = Vec::new();
        // spawn body
        for _ in 0..2 {
            let id = spawn_body_part(
                &head_entity,
                &mut commands,
                &body_atlas_layout,
                4,
                &char_texture,
                &head_pos,
            );
            body_ref.push(id);
            // head_pos.translation.x -= TILE_SIZE;
        }
        let tail_entity = spawn_body_part(
            &head_entity,
            &mut commands,
            &body_atlas_layout,
            // 3 need to make more textures,
            4,
            &char_texture,
            &head_pos,
        );
        body_ref.push(tail_entity);
        commands.entity(head_entity).insert(BodyRef(body_ref));

        commands.entity(tail_entity).insert(Tail);
    }

    // apple
    let apple_texture = asset_server.load("textures/chars/char_atlas.png");
    let apple_layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 4, 2, None, None);
    let apple_atlas_layout = texture_atlases.add(apple_layout);
    let p = get_random_point();
    commands.spawn((
        TextureAtlas {
            layout: apple_atlas_layout.clone(),
            index: 7,
            ..Default::default()
        },
        SpriteBundle {
            texture: apple_texture.clone(),
            transform: Transform::from_translation(p.extend(0.))
                .with_scale(Vec3::splat(SPLAT_SIZE)),
            ..Default::default()
        },
        Apple,
        Bounding(8.),
        StateScoped(GameState::InGame),
    ));
    // .animation()
    // .repeat(Repeat::Infinitely)
    // .insert_tween_here(
    //     Duration::from_millis(500),
    //     EaseFunction::Linear,
    //     head_sprite.with(atlas_index(0, 2)),
    // );
}

pub fn tear_down_players(player_query: Query<Entity, With<Player>>, mut commands: Commands) {
    for entity in player_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn grow_snake(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut grow_snake: EventReader<GrowSnakeEvent>,
    mut head_query: Query<(Entity, &mut BodyRef), (Without<SnakeBodyPart>, Without<Dead>)>,
    mut tail: Query<(Entity, &Transform, &mut TextureAtlas), With<Tail>>,
) {
    //todo fix unwrap bug
    for ev in grow_snake.read() {
        let head_entity = ev.0;
        let (_, mut body_ref) = head_query.get_mut(head_entity).unwrap();

        let full_texture = asset_server.load("textures/chars/char_atlas.png");
        let body_layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 5, 1, None, None);
        let body_atlas_layout = texture_atlases.add(body_layout);

        let tail_layout = TextureAtlasLayout::from_grid(UVec2::new(32, 16), 3, 3, None, None);
        let tail_atlas_layout = texture_atlases.add(tail_layout);

        // todo unwrap unwrap unwrap
        let (old_tail, transform, mut old_atlas) =
            tail.get_mut(*body_ref.0.last().unwrap()).unwrap();
        let new_tail = spawn_body_part(
            &head_entity,
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
        // add new tail to body ref of head
        body_ref.0.push(new_tail);
    }
}

fn spawn_body_part(
    snake_head_ref: &Entity,
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
                transform: pos.clone(),
                ..Default::default()
            },
            AnimationTarget,
            SnakeBodyPart,
            Collidible,
            Bounding(TILE_SIZE / 2.),
            SnakeHeadRef(Some(snake_head_ref.clone())),
            StateScoped(GameState::InGame),
        ))
        .id()
}

pub fn move_snakes(
    mut head_query: Query<
        (&mut Transform, &SnakeHead, &BodyRef),
        (Without<SnakeBodyPart>, Without<Dead>),
    >,
    mut snake_body_parts: Query<&mut Transform, With<SnakeBodyPart>>,
) {
    for (mut transform, head, body_ref) in head_query.iter_mut() {
        let move_speed = TILE_SIZE;
        let move_delta = match head.direction {
            SnakeDirection::Left => Vec3::new(-move_speed, 0., 0.),
            SnakeDirection::Right => Vec3::new(move_speed, 0., 0.),
            SnakeDirection::Up => Vec3::new(0., move_speed, 0.),
            SnakeDirection::Down => Vec3::new(0., -move_speed, 0.),
        };

        let mut prev_pos = transform.translation;
        transform.translation += move_delta;

        let len = body_ref.0.iter().len();
        for (i, body_entity) in body_ref.0.iter().enumerate() {
            let res = snake_body_parts.get_mut(*body_entity);

            match res {
                Ok(_) => {}
                Err(_) => {
                    println!("body part not found");
                    continue;
                }
            }
            let mut part_transform = res.unwrap();

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
            SnakeDirection::Left => {
                transform.rotation = Quat::from_rotation_z(-180.0f32.to_radians())
            }
            SnakeDirection::Right => {
                transform.rotation = Quat::from_rotation_z(0.0f32.to_radians())
            }
            SnakeDirection::Up => transform.rotation = Quat::from_rotation_z(90.0f32.to_radians()),
            SnakeDirection::Down => {
                transform.rotation = Quat::from_rotation_z(-90.0f32.to_radians())
            }
        }
    }
}

pub fn check_death_collision(
    mut commands: Commands,
    mut head_query: Query<(Entity, &Transform, &SnakeHead, &Bounding), Without<Dead>>,
    mut collidibles: Query<(&Transform, &Bounding), With<Collidible>>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    for (entity, head_transform, _, head_size) in head_query.iter_mut() {
        let head_pos = head_transform.translation;

        for (collidable_transform, collidable_size) in collidibles.iter() {
            if circles_touching(
                head_transform,
                head_size,
                collidable_transform,
                collidable_size,
            ) {
                commands.entity(entity).insert(Dead);
                // next_state.set(GamePhase::Dead);
            }
        }

        if head_pos.x < -(WORLD_SIZE_X as f32 / 2. as f32 * TILE_SIZE)
            || head_pos.x > WORLD_SIZE_X as f32 / 2. as f32 * TILE_SIZE
            || head_pos.y < -(WORLD_SIZE_Y as f32 / 2. as f32 * TILE_SIZE)
            || head_pos.y > WORLD_SIZE_Y as f32 / 2. as f32 * TILE_SIZE
        {
            commands.entity(entity).insert(Dead);
        }
    }
}

pub fn check_apple_collision(
    mut commands: Commands,
    mut head_query: Query<
        (Entity, &Transform, &SnakeHead, &Bounding),
        (Without<SnakeBodyPart>, Without<Dead>),
    >,
    apple_query: Query<(Entity, &Transform, &Bounding), With<Apple>>,
    mut spawn_apple: EventWriter<MoveAppleEvent>,
    mut grow_snake: EventWriter<GrowSnakeEvent>,
    mut fixed_time: ResMut<Time<Fixed>>,
    mut player_query: Query<(&mut Score, &SnakeHeadRef), With<Player>>,
) {
    for (mut score, headRef) in player_query.iter_mut() {
        //todo fixup this
        match headRef.0 {
            None => continue,
            _ => {}
        }

        let head = head_query.get(headRef.0.unwrap());
        match head {
            Ok(_) => {}
            Err(_) => continue,
        }
        let (entity, head_transform, _, head_size) = head.unwrap();
        for (apple_entity, apple_transform, apple_size) in apple_query.iter() {
            if circles_touching(apple_transform, apple_size, head_transform, head_size) {
                // EATEN
                spawn_apple.send(MoveAppleEvent);
                grow_snake.send(GrowSnakeEvent(entity));

                score.value += 1;

                let new_timestep = fixed_time.timestep().mul_f32(0.95);
                fixed_time.set_timestep(new_timestep);
            }
        }
    }
    // for (entity, head_transform, _, head_size) in head_query.iter_mut() {
    // }
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

pub fn check_all_dead(
    head_query: Query<(Entity, Has<Dead>), With<SnakeHead>>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    let all_dead = head_query.iter().all(|(_, dead)| dead);

    if all_dead {
        next_state.set(GamePhase::Dead);
    }
}

pub fn dead_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        println!("Setting next state to EnterGame");
        next_state.set(GameState::EnterGame);
    }
}

pub fn dead_text(mut commands: Commands, asset_server: Res<AssetServer>) {
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
                            format!("You crashed!"),
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

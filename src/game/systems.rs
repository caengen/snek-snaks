use bevy::a11y::accesskit::Size;
use bevy::prelude::*;
use bevy_turborand::DelegatedRng;
use bevy_turborand::{GlobalRng, RngComponent};
use bevy_tween::tween::{AnimationTarget, IntoTarget};

use crate::{GamePhase, SCREEN};

use super::collision::circles_touching;
use super::components::{
    Apple, Bounding, Collidible, Direction, ExampleGameText, GrowSnakeEvent, PausedText, Player,
    Pos, SnakeBodyPart, SnakeHead, SpawnAppleEvent, Tail, Vel,
};
use super::effects::Flick;

const TILE_SIZE: f32 = 16.;
const WORLD_SIZE_X: u32 = 80;
const WORLD_SIZE_Y: u32 = 45;

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
    mut snakes: Query<&mut SnakeHead, With<Player>>,
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
        for snakes in snakes.iter_mut() {
            let mut head = snakes;
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
        let apple_layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 3, 2, None, None);
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
                index: 3,
                ..Default::default()
            },
            SpriteBundle {
                texture: apple_texture.clone(),
                transform: Transform::from_translation(Vec3::new(*x.unwrap(), *y.unwrap(), 0.)),
                ..Default::default()
            },
            Apple,
            Bounding(TILE_SIZE / 2.),
        ));
    }
}

pub fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut spawn_apple: EventWriter<SpawnAppleEvent>,
) {
    let char_texture = asset_server.load("textures/chars/char_atlas.png");
    let char_layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 3, 1, None, None);
    let char_atlas_layout = texture_atlases.add(char_layout);
    let head_sprite = AnimationTarget.into_target();

    let mut head_pos = Transform::IDENTITY;
    head_pos.translation = Vec3::new(-TILE_SIZE / 2., -TILE_SIZE / 2., 0.);

    // spawn head
    commands.spawn((
        TextureAtlas {
            layout: char_atlas_layout.clone(),
            index: 0,
            ..Default::default()
        },
        SpriteBundle {
            texture: char_texture.clone(),
            transform: Transform::IDENTITY,
            ..Default::default()
        },
        AnimationTarget,
        Player {},
        SnakeHead {
            direction: Direction::Right,
        },
        Bounding(TILE_SIZE / 2.),
    ));
    head_pos.translation.x -= TILE_SIZE;

    // spawn body
    let mut last = Entity::PLACEHOLDER;
    for _ in 0..8 {
        last = spawn_body_part(&mut commands, &char_atlas_layout, &char_texture, &head_pos);
        head_pos.translation.x -= TILE_SIZE;
    }
    commands.entity(last).insert(Tail);
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
    tail: Query<(Entity, &Transform), With<Tail>>,
) {
    for _ in grow_snake.read() {
        let char_texture = asset_server.load("textures/chars/char_atlas.png");
        let char_layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 3, 1, None, None);
        let char_atlas_layout = texture_atlases.add(char_layout);

        let (old_tail, transform) = tail.single();
        let new_tail = spawn_body_part(&mut commands, &char_atlas_layout, &char_texture, transform);
        commands.entity(old_tail).remove::<Tail>();
        commands.entity(new_tail).insert(Tail);
    }
}

fn spawn_body_part(
    commands: &mut Commands,
    layout: &Handle<TextureAtlasLayout>,
    texture: &Handle<Image>,
    pos: &Transform,
) -> Entity {
    commands
        .spawn((
            TextureAtlas {
                layout: layout.clone(),
                index: 1,
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
        for mut part_transform in snake_body_parts.iter_mut() {
            let old = part_transform.translation;
            part_transform.translation = prev_pos;
            prev_pos = old;
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
            next_state.set(GamePhase::Paused);
        }
    }
}

pub fn check_apple_collision(
    mut commands: Commands,
    mut head_query: Query<(&Transform, &SnakeHead, &Bounding), Without<SnakeBodyPart>>,
    apple_query: Query<(Entity, &Transform, &Bounding), With<Apple>>,
    mut spawn_apple: EventWriter<SpawnAppleEvent>,
    mut grow_snake: EventWriter<GrowSnakeEvent>,
) {
    let (head_transform, _, head_size) = head_query.single_mut();
    let head_pos = head_transform.translation;

    for (apple_entity, apple_transform, apple_size) in apple_query.iter() {
        let apple_pos = apple_transform.translation;
        if head_pos.distance(apple_pos) < head_size.0 + apple_size.0 {
            commands.entity(apple_entity).despawn();
            spawn_apple.send(SpawnAppleEvent);
            grow_snake.send(GrowSnakeEvent);
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

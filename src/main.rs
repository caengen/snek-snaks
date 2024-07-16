use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    log::{Level, LogPlugin},
    prelude::*,
    window::PresentMode,
    DefaultPlugins,
};
use bevy_asset_loader::{
    loading_state::config::ConfigureLoadingState,
    prelude::{AssetCollection, LoadingState, LoadingStateAppExt},
};
use bevy_egui::EguiSettings;
use bevy_egui::{
    egui::{FontData, FontDefinitions, FontFamily},
    EguiContexts, EguiPlugin,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_turborand::prelude::RngPlugin;
use bevy_tween::DefaultTweenPlugins;
use config::Debug;
use game::{prelude::MainCamera, GamePlugin};
use interpolator::custom_interpolators_plugin;
use main_menu::*;
use std::{env, process, time::Duration};

mod config;
mod game;
mod interpolator;
mod main_menu;

pub const SCREEN: Vec2 = Vec2::from_array([1280.0, 720.0]);
pub const DARK: Color = Color::rgb(0.059, 0.219, 0.059);
pub const LIGHT: Color = Color::rgb(0.852, 0.844, 0.816);

// Example: Easy loading of assets
#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(texture_atlas(tile_size_x = 16, tile_size_y = 16, columns = 8, rows = 1))]
    #[asset(path = "textures/chars/char_atlas.png")]
    pub images: Handle<TextureAtlasLayout>,
}

#[derive(States, Hash, Clone, PartialEq, Eq, Debug, Default)]
pub enum GameState {
    #[default]
    AssetLoading,
    MainMenu,
    EnterGame,
    InGame,
    LeaveGame,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct InGame;
impl ComputedStates for InGame {
    // Computed states can be calculated from one or many source states.
    type SourceStates = GameState;

    // Now, we define the rule that determines the value of our computed state.
    fn compute(sources: GameState) -> Option<InGame> {
        match sources {
            // We can use pattern matching to express the
            //"I don't care whether or not the game is paused" logic!
            GameState::InGame { .. } => Some(InGame),
            _ => None,
        }
    }
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
// This macro means that `GamePhase` will only exist when we're in the `InGame` computed state.
// The intermediate computed state is helpful for clarity here, but isn't required:
// you can manually `impl SubStates` for more control, multiple parent states and non-default initial value!
#[source(InGame = InGame)]
enum GamePhase {
    #[default]
    Playing,
    Paused,
    Dead,
}

#[derive(Resource)]
pub struct Score {
    value: u32,
}

/**
 * The configuration for the game loop. For cleanliness
 */
fn main() {
    // Possibility for program args
    let args: Vec<String> = env::args().skip(1).collect();
    let cfg = config::ProgramConfig::build(&args).unwrap_or_else(|err| {
        println!("A problem occured when parsing args: {err}");
        process::exit(1);
    });

    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "SNAKE".into(),
                    resolution: (SCREEN.x, SCREEN.y).into(),
                    present_mode: PresentMode::AutoNoVsync,
                    // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                    prevent_default_event_handling: false,

                    ..default()
                }),
                ..default()
            })
            .set(LogPlugin {
                level: Level::DEBUG,
                filter: "wgpu=error,bevy_render=info,bevy_ecs=trace".to_string(),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
        DefaultTweenPlugins,
        custom_interpolators_plugin,
    ))
    .init_state::<GameState>()
    .enable_state_scoped_entities::<GameState>()
    .add_computed_state::<InGame>()
    .add_sub_state::<GamePhase>()
    .insert_resource(Debug(cfg.debug))
    // Example: Easy loading of assets
    .add_loading_state(
        LoadingState::new(GameState::AssetLoading)
            .continue_to_state(GameState::InGame)
            .load_collection::<ImageAssets>(),
    )
    .insert_resource(Debug(cfg.debug))
    // .add_plugins(
    //     WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
    // )
    .add_plugins((
        FrameTimeDiagnosticsPlugin::default(),
        RngPlugin::new().with_rng_seed(220718),
        EguiPlugin,
        MainMenuPlugin,
        GamePlugin,
    ))
    .add_systems(Startup, (setup_camera, setup_fonts))
    .add_systems(Update, window_resized)
    .insert_resource(Score { value: 0 });

    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(DARK),
                ..default()
            },
            ..default()
        },
        MainCamera,
    ));

    // for mut window in windows.iter_mut() {
    //     window.cursor.visible = false;
    // }
}

fn setup_fonts(mut contexts: EguiContexts) {
    let mut fonts = FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters):
    fonts.font_data.insert(
        "visitor".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/visitor.ttf")),
    ); // .ttf and .otf supported

    // Put my font first (highest priority):
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "visitor".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .push("visitor".to_owned());

    contexts.ctx_mut().set_fonts(fonts);
}

pub fn window_resized(
    windows: Query<&Window>,
    mut q: Query<&mut OrthographicProjection, With<MainCamera>>,
    mut egui_settings: ResMut<EguiSettings>,
) {
    let window = windows.single();
    let scale = SCREEN.x / window.width();
    for mut projection in q.iter_mut() {
        projection.scale = scale;
        egui_settings.scale_factor = (window.width() / SCREEN.x).into();
    }
}

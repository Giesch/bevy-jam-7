use std::time::Duration;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use bevy_asset_loader::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_seedling::prelude::*;
use inline_tweak::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<Screen>()
        .add_loading_state(
            LoadingState::new(Screen::Loading)
                .continue_to_state(Screen::InGame)
                .load_collection::<AllAssets>(),
        )
        .add_plugins((
            SeedlingPlugin::default(),
            JsonAssetPlugin::<Beats>::new(&["beats.json"]),
        ))
        .add_systems(
            OnEnter(Screen::InGame),
            (play_scherzo, spawn_camera, spawn_quill),
        )
        .init_resource::<Intent>()
        .add_systems(
            FixedUpdate,
            (read_input, move_quill, drop_ink_circles_at_quill)
                .chain()
                .run_if(in_state(Screen::InGame)),
        )
        .insert_resource(TrackTimer::new())
        .add_systems(Update, tick_track_timer.run_if(in_state(Screen::InGame)))
        .run()
}

#[derive(AssetCollection, Resource)]
struct AllAssets {
    #[asset(path = "images/Eroica_Beethoven_title.jpg")]
    eroica_score: Handle<Image>,

    #[asset(path = "audio/03_Scherzo_Allegro_vivace.flac")]
    scherzo: Handle<AudioSample>,
    #[asset(path = "audio/scherzo.beats.json")]
    scherzo_beats: Handle<Beats>,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum Screen {
    #[default]
    Loading,
    InGame,
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(Component)]
struct Quill;

fn play_scherzo(mut commands: Commands, server: Res<AssetServer>) {
    const SCHERZO: &str = "audio/03_Scherzo_Allegro_vivace.flac";
    commands.spawn(SamplePlayer::new(server.load(SCHERZO)));
}

fn spawn_quill(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Annulus::new(25.0, 50.0));
    let color = Color::hsl(360.0, 0.95, 0.7);

    commands.spawn((
        Quill,
        Mesh2d(mesh),
        MeshMaterial2d(materials.add(color)),
        Transform::default(),
    ));
}

#[derive(Resource, Default)]
struct Intent {
    /// the mouse position in world space
    mouse_pos: Option<Vec2>,
    /// whether the player is currently scribbling
    quill_down: bool,
}

fn read_input(
    mut intent: ResMut<Intent>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    let (camera, camera_transform) = *camera_query;
    intent.quill_down = mouse.pressed(MouseButton::Left);
    intent.mouse_pos = read_mouse_pos_in_world_space(&window, camera, camera_transform);
}

fn read_mouse_pos_in_world_space(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    let viewport_position = window.cursor_position()?;
    camera
        .viewport_to_world_2d(camera_transform, viewport_position)
        .ok()
}

#[tweak_fn]
fn move_quill(intent: Res<Intent>, mut quills: Query<&mut Transform, With<Quill>>) {
    let Some(mouse_pos) = intent.mouse_pos else {
        return;
    };

    let quill_speed = 0.2;
    for mut quill_transform in &mut quills {
        let quill_pos = quill_transform.translation.xy();
        let moved = quill_pos.lerp(mouse_pos, quill_speed);
        quill_transform.translation.x = moved.x;
        quill_transform.translation.y = moved.y;
    }
}

#[derive(Component)]
struct Ink;

#[tweak_fn]
fn drop_ink_circles_at_quill(
    intent: Res<Intent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    quills: Query<&Transform, With<Quill>>,
) {
    if !intent.quill_down {
        return;
    }

    let mesh = meshes.add(Circle::new(10.0));
    let color = Color::hsl(360.0, 0.85, 0.7);

    for quill_transform in &quills {
        commands.spawn((
            Ink,
            Mesh2d(mesh.clone()),
            MeshMaterial2d(materials.add(color)),
            Transform::from_translation(quill_transform.translation),
        ));
    }
}

/// Elapsed time of the current music track
#[derive(Resource, Default)]
struct TrackTimer(Timer);

impl TrackTimer {
    fn new() -> Self {
        let timer = Timer::new(Duration::from_mins(100), TimerMode::Once);

        Self(timer)
    }
}

fn tick_track_timer(mut track_timer: ResMut<TrackTimer>, time: Res<Time>) {
    track_timer.0.tick(time.delta());
}

#[derive(serde::Deserialize, bevy::asset::Asset, bevy::reflect::TypePath)]
struct Beats {
    bpm: f32,
    beats: Vec<f32>,
}

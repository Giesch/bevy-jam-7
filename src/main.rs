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
                .load_collection::<AllAssetHandles>(),
        )
        .add_plugins((
            SeedlingPlugin::default(),
            JsonAssetPlugin::<Beats>::new(&["beats.json"]),
        ))
        .add_systems(
            OnEnter(Screen::InGame),
            (
                play_scherzo,
                init_beat_timer,
                spawn_camera,
                spawn_quill_reticle,
            ),
        )
        .init_resource::<Intent>()
        .init_resource::<BeatIndex>()
        .add_systems(
            FixedUpdate,
            (read_input, move_quill, drop_ink_circles_at_quill)
                .chain()
                .run_if(in_state(Screen::InGame)),
        )
        .insert_resource(TrackTimer::new())
        .init_resource::<BeatTimer>()
        .init_resource::<BeatFlash>()
        .add_systems(
            Update,
            (
                tick_track_timer,
                tick_beat_timer,
                quill_reticle_size_beat,
                set_beat_flash_background,
            )
                .chain()
                .run_if(in_state(Screen::InGame)),
        )
        .run()
}

#[derive(AssetCollection, Resource)]
struct AllAssetHandles {
    #[expect(unused)]
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

fn play_scherzo(mut commands: Commands, assets: Res<AllAssetHandles>) {
    commands.spawn(SamplePlayer::new(assets.scherzo.clone()));
}

#[derive(Component)]
struct QuillReticle;

const RETICLE_BIG_INNER_RADIUS: f32 = 25.0;
const RETICLE_BIG_OUTER_RADIUS: f32 = 50.0;

fn spawn_quill_reticle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(make_reticle(1.0));
    let color = make_reticle_color(1.0);

    commands.spawn((
        QuillReticle,
        Mesh2d(mesh),
        MeshMaterial2d(materials.add(color)),
        Transform::default(),
    ));
}

fn make_reticle(ratio: f32) -> Annulus {
    Annulus::new(
        ratio * RETICLE_BIG_INNER_RADIUS,
        ratio * RETICLE_BIG_OUTER_RADIUS,
    )
}

fn make_reticle_color(ratio: f32) -> Color {
    Color::hsl(360.0, ratio * 0.95, 0.7)
}

#[tweak_fn]
fn quill_reticle_size_beat(
    mut reticles: Query<(&mut Mesh2d, &mut MeshMaterial2d<ColorMaterial>), With<QuillReticle>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    beat_timer: Res<BeatTimer>,
) {
    let ratio = beat_timer.angle_wave();

    let shape = make_reticle(ratio);
    let color = make_reticle_color(ratio);
    for (mut mesh, mut material) in &mut reticles {
        // TODO do we need to cache these? or premake a bunch of them
        mesh.0 = meshes.add(shape);
        material.0 = materials.add(color);
    }
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
fn move_quill(intent: Res<Intent>, mut quills: Query<&mut Transform, With<QuillReticle>>) {
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
    quills: Query<&Transform, With<QuillReticle>>,
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

fn tick_track_timer(time: Res<Time>, mut track_timer: ResMut<TrackTimer>) {
    track_timer.0.tick(time.delta());
}

#[derive(serde::Deserialize, bevy::asset::Asset, bevy::reflect::TypePath)]
struct Beats {
    beats: Vec<f32>,
    #[expect(unused)]
    beats_intervals: Vec<f32>,
}

#[derive(Resource, Default)]
struct BeatIndex(usize);

#[derive(Resource, Default)]
struct BeatTimer(Timer);

impl BeatTimer {
    fn from_index(beat_index: usize, track_time: f32, beats: &Beats) -> Self {
        let next_beat_index = beat_index + 1;
        if next_beat_index > beats.beats.len() {
            log::warn!("invalid beat index");
            return Default::default();
        }

        let next_beat = beats.beats[next_beat_index];
        let to_next_beat = next_beat - track_time;
        let duration = Duration::from_secs_f32(to_next_beat);
        let timer = Timer::new(duration, TimerMode::Once);

        Self(timer)
    }

    // how far through the beat we are, 0.0-1.0
    fn elapsed_ratio(&self) -> f32 {
        self.0.elapsed_secs() / self.0.duration().as_secs_f32()
    }

    // 0.0-1.0
    // 0.0 == on downbeat, 1.0 = up between beats
    // TODO replace this with an actual curved wave of some kind
    //   faster on the way down?
    fn angle_wave(&self) -> f32 {
        let one_on_beat = (0.5 - self.elapsed_ratio()).abs() * 0.5 + 0.5;
        let zero_on_beat = 1.0 - one_on_beat;
        zero_on_beat
    }
}

fn init_beat_timer(
    assets: Res<AllAssetHandles>,
    beats_assets: Res<Assets<Beats>>,
    mut beat_timer: ResMut<BeatTimer>,
) {
    let beats = beats_assets.get(&assets.scherzo_beats).unwrap();
    *beat_timer = BeatTimer::from_index(0, 0.0, beats);
}

#[derive(Resource, Default)]
struct BeatFlash(bool);

fn tick_beat_timer(
    time: Res<Time>,
    track_timer: Res<TrackTimer>,
    assets: Res<AllAssetHandles>,
    beats_assets: Res<Assets<Beats>>,
    mut beat_index: ResMut<BeatIndex>,
    mut beat_timer: ResMut<BeatTimer>,
    mut beat_flash: ResMut<BeatFlash>,
) {
    beat_timer.0.tick(time.delta());
    beat_flash.0 = beat_timer.0.just_finished();
    if beat_flash.0 {
        beat_index.0 += 1;
        let beats = beats_assets.get(&assets.scherzo_beats).unwrap();
        let track_time = track_timer.0.elapsed_secs();
        *beat_timer = BeatTimer::from_index(beat_index.0, track_time, beats);
    }
}

#[tweak_fn]
fn set_beat_flash_background(beat_flash: Res<BeatFlash>, mut clear_color: ResMut<ClearColor>) {
    *clear_color = if beat_flash.0 {
        ClearColor(Color::hsl(360.0, 0.75, 0.4))
    } else {
        ClearColor::default()
    };
}

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use bevy_seedling::prelude::*;
use inline_tweak::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SeedlingPlugin::default())
        .add_systems(Startup, (play_scherzo, spawn_camera, spawn_quill))
        .init_resource::<Intent>()
        .add_systems(
            FixedUpdate,
            (read_input, lerp_quill_to_mouse, drop_ink_circles_at_quill).chain(),
        )
        .run()
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
    quill_down: bool,
}

fn read_input(mut intent: ResMut<Intent>, mouse: Res<ButtonInput<MouseButton>>) {
    intent.quill_down = mouse.pressed(MouseButton::Left);
}

#[tweak_fn]
fn lerp_quill_to_mouse(
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mut quills: Query<&mut Transform, With<Quill>>,
) {
    let Some(mouse_pos_screen_space) = window.cursor_position() else {
        return;
    };

    let (camera, camera_transform) = *camera_query;
    let Ok(mouse_pos) = camera.viewport_to_world_2d(camera_transform, mouse_pos_screen_space)
    else {
        return;
    };

    let quill_speed = 0.1;
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

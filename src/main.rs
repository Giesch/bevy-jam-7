use bevy::{prelude::*, window::PrimaryWindow};
use bevy_seedling::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SeedlingPlugin::default())
        .add_systems(Startup, (play_scherzo, spawn_camera, spawn_quill))
        .add_systems(FixedUpdate, (lerp_quill_to_mouse,))
        .run()
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(Component)]
struct Quill;

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

fn lerp_quill_to_mouse(
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mut quills: Query<&mut Transform, With<Quill>>,
) {
    const QUILL_SPEED: f32 = 0.1;

    let Some(mouse_pos_screen_space) = window.cursor_position() else {
        return;
    };

    let (camera, camera_transform) = *camera_query;
    let Ok(mouse_pos) = camera.viewport_to_world_2d(camera_transform, mouse_pos_screen_space)
    else {
        return;
    };

    for mut quill_transform in &mut quills {
        let quill_pos = quill_transform.translation.xy();
        let moved = quill_pos.lerp(mouse_pos, QUILL_SPEED);
        quill_transform.translation.x = moved.x;
        quill_transform.translation.y = moved.y;
    }
}

fn play_scherzo(mut commands: Commands, server: Res<AssetServer>) {
    const SCHERZO: &str = "audio/03_Scherzo_Allegro_vivace.flac";
    commands.spawn(SamplePlayer::new(server.load(SCHERZO)));
}

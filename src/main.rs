use bevy::prelude::*;
use bevy_seedling::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SeedlingPlugin::default())
        .add_systems(Startup, play_music)
        .run()
}

fn play_music(mut commands: Commands, server: Res<AssetServer>) {
    const BEETHOVEN: &str = "audio/03_Scherzo_Allegro_vivace.flac";

    commands.spawn(SamplePlayer::new(server.load(BEETHOVEN)));
}

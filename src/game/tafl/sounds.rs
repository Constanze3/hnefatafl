use crate::game::tafl::*;
use bevy::audio::*;

pub struct SoundsPlugin;

impl Plugin for SoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (move_and_capture_sound, game_end_sound))
            .insert_resource(Sounds::default());
    }
}

#[derive(Resource)]
pub struct Sounds {
    move_sound: String,
    capture_sound: String,
    game_end_sound: String,
}

impl Default for Sounds {
    fn default() -> Self {
        Self {
            move_sound: "sounds/move.ogg".to_string(),
            capture_sound: "sounds/capture.ogg".to_string(),
            game_end_sound: "sounds/game_end.ogg".to_string(),
        }
    }
}

fn move_and_capture_sound(
    mut event: EventReader<EndMoveEvent>,
    sounds: Res<Sounds>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for ev in event.read() {
        if !ev.capture_happened {
            commands.spawn(AudioBundle {
                source: asset_server.load(sounds.move_sound.clone()),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    ..default()
                },
            });
        } else {
            commands.spawn(AudioBundle {
                source: asset_server.load(sounds.capture_sound.clone()),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    ..default()
                },
            });
        }
    }
}

fn game_end_sound(
    mut event: EventReader<EndGameEvent>,
    sounds: Res<Sounds>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for _ in event.read() {
        commands.spawn(AudioBundle {
            source: asset_server.load(sounds.game_end_sound.clone()),
            settings: PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..default()
            },
        });
    }
}

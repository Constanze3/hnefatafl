use crate::game::tafl::*;
use bevy::audio::*;

pub struct SoundsPlugin;

impl Plugin for SoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), spawn_sound_manager)
            .add_systems(OnExit(GameState::InGame), despawn_sound_manager)
            .add_systems(
                Update,
                (move_and_capture_sound, game_end_sound).run_if(in_state(GameState::InGame)),
            )
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

#[derive(Component)]
struct SoundManager;

fn spawn_sound_manager(mut commands: Commands) {
    commands.spawn((Name::new("Sound Manager"), SoundManager));
}

fn despawn_sound_manager(
    q_sound_manager: Query<Entity, With<SoundManager>>,
    mut commands: Commands,
) {
    let sound_manager = q_sound_manager.single();
    commands.entity(sound_manager).despawn_recursive();
}

fn move_and_capture_sound(
    mut event: EventReader<EndMoveEvent>,
    sounds: Res<Sounds>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_sound_manager: Query<Entity, With<SoundManager>>,
) {
    let sound_manager = q_sound_manager.single();

    for ev in event.read() {
        let sound_entity = if !ev.capture_happened {
            commands
                .spawn(AudioBundle {
                    source: asset_server.load(sounds.move_sound.clone()),
                    settings: PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        ..default()
                    },
                })
                .id()
        } else {
            commands
                .spawn(AudioBundle {
                    source: asset_server.load(sounds.capture_sound.clone()),
                    settings: PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        ..default()
                    },
                })
                .id()
        };

        commands.entity(sound_manager).add_child(sound_entity);
    }
}

fn game_end_sound(
    mut event: EventReader<EndGameEvent>,
    sounds: Res<Sounds>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_sound_manager: Query<Entity, With<SoundManager>>,
) {
    let sound_manager = q_sound_manager.single();

    for _ in event.read() {
        let sound_entity = commands
            .spawn(AudioBundle {
                source: asset_server.load(sounds.game_end_sound.clone()),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    ..default()
                },
            })
            .id();

        commands.entity(sound_manager).add_child(sound_entity);
    }
}

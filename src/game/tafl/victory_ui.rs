use bevy::utils::HashMap;

use crate::game::{tafl::*, GameState};

pub struct VictoryUiPlugin;

impl Plugin for VictoryUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnVictoryUiEvent>()
            .add_systems(
                Update,
                (spawn_victory_ui, update_victory_ui_lifetime).run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnExit(GameState::InGame), despawn_victory_ui)
            .insert_resource(VictoryText::default());
    }
}

#[derive(Resource)]
struct VictoryText {
    pub side_text_map: HashMap<Side, String>,
}

impl Default for VictoryText {
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert(Side::Attacker, "Attacker wins!!!".to_string());
        map.insert(Side::Defender, "Defender wins!!!".to_string());

        Self { side_text_map: map }
    }
}

#[derive(Component)]
struct VictoryUi {
    pub lifetime: Timer,
}

#[derive(Event)]
pub struct SpawnVictoryUiEvent {
    pub winner: Side,
}

fn spawn_victory_ui(
    mut event: EventReader<SpawnVictoryUiEvent>,
    victory_text: Res<VictoryText>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for ev in event.read() {
        commands
            .spawn((
                VictoryUi {
                    lifetime: Timer::from_seconds(4., TimerMode::Once),
                },
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,

                        ..default()
                    },
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            width: Val::Px(400.),
                            height: Val::Px(100.),
                            ..default()
                        },
                        background_color: Color::BLACK.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        let text = victory_text.side_text_map.get(&ev.winner).unwrap();

                        parent.spawn(TextBundle::from_section(
                            text,
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        ));
                    });
            });
    }
}

fn despawn_victory_ui(q_victory_ui: Query<Entity, With<VictoryUi>>, mut commands: Commands) {
    for victory_ui_entity in &q_victory_ui {
        commands.entity(victory_ui_entity).despawn_recursive();
    }
}

fn update_victory_ui_lifetime(
    mut q_victory_ui: Query<(Entity, &mut VictoryUi)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (victory_ui_entity, mut victory_ui) in &mut q_victory_ui {
        victory_ui.lifetime.tick(time.delta());

        if victory_ui.lifetime.just_finished() {
            commands.entity(victory_ui_entity).despawn_recursive();
        }
    }
}

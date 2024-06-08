use bevy::utils::HashMap;

use crate::game::tafl::*;

pub struct VictoryUiPlugin;

impl Plugin for VictoryUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnVictoryUiEvent>()
            .add_systems(Update, spawn_victory_ui)
            .insert_resource(VictoryText::default());
    }
}

#[derive(Resource)]
pub struct VictoryText {
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

#[derive(Event)]
pub struct SpawnVictoryUiEvent {
    pub winner: Side,
}

#[derive(Component)]
pub struct VictoryUi;

pub fn spawn_victory_ui(
    mut event: EventReader<SpawnVictoryUiEvent>,
    victory_text: Res<VictoryText>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for ev in event.read() {
        commands
            .spawn((
                VictoryUi,
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

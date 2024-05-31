use crate::game::tafl::*;
use bevy::sprite::MaterialMesh2dBundle;

#[derive(Component, Clone)]
pub struct BoardHighlights {
    pub mesh: Handle<Mesh>,
    pub color: Handle<ColorMaterial>,
    pub z: f32,
    pub entity: Option<Entity>,
}

#[derive(Event)]
pub struct SpawnHighlightsEvent {
    pub board_entity: Entity,
    pub positions: Vec<Position>,
}

#[derive(Event)]
pub struct DespawnHighlightsEvent {
    pub board_entity: Entity,
}

/// Spawns highlights for the specified positions of the board
///
/// Unchecked pre: The positions in the SpawnHighightEvent are on the board
pub fn spawn_highlights(
    mut q_board_highlights: Query<(&Board, &mut BoardHighlights)>,
    mut event: EventReader<SpawnHighlightsEvent>,
    mut commands: Commands,
) {
    for ev in event.read() {
        let Ok((board, mut highlights)) = q_board_highlights.get_mut(ev.board_entity) else {
            return;
        };

        if highlights.entity != None {
            return;
        }

        let parent = commands
            .spawn((Name::new("Highlights"), SpatialBundle::default()))
            .id();
        commands.entity(ev.board_entity).push_children(&[parent]);

        let mesh = &highlights.mesh;
        let material = &highlights.color;

        for position in &ev.positions {
            let highlight = commands
                .spawn((
                    Name::new("highlight"),
                    MaterialMesh2dBundle {
                        mesh: mesh.clone().into(),
                        material: material.clone(),
                        transform: Transform::from_translation(
                            board.board_to_world(*position).extend(highlights.z),
                        ),
                        ..default()
                    },
                ))
                .id();

            commands.entity(parent).push_children(&[highlight]);
        }

        highlights.entity = Some(parent);
    }
}

pub fn despawn_highlights(
    mut q_highlights: Query<&mut BoardHighlights, With<Board>>,
    mut event: EventReader<DespawnHighlightsEvent>,
    mut commands: Commands,
) {
    for ev in event.read() {
        let Ok(mut highlights) = q_highlights.get_mut(ev.board_entity) else {
            return;
        };

        let Some(e) = highlights.entity else {
            return;
        };

        commands.entity(e).despawn_recursive();
        highlights.entity = None;
    }
}

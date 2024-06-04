use bevy::prelude::*;

use self::camera::*;
use self::tafl::*;

mod camera;
mod tafl;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraPlugin)
            .add_plugins(TaflPlugin)
            .add_systems(Startup, spawn_data::spawn_hnefatafl);

        // app.add_plugins(CameraPlugin)
        //     .add_systems(Startup, spawn_test);
    }
}

// fn spawn_test(
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
//     mut commands: Commands,
// ) {
//     // hover indicator
//     {
//         let mut mesh = Mesh::new(
//             bevy::render::mesh::PrimitiveTopology::TriangleList,
//             RenderAssetUsages::RENDER_WORLD,
//         );
//         mesh.insert_attribute(
//             Mesh::ATTRIBUTE_POSITION,
//             vec![[1., 0., 0.], [0., 1., 0.], [1., 1., 0.]],
//         );
//         mesh.insert_indices(Indices::U32(vec![2, 1, 9]));
//
//         commands.spawn(MaterialMesh2dBundle {
//             mesh: meshes.add(mesh).into(),
//             material: materials.add(Color::rgb(0., 1., 0.)),
//             transform: Transform::default().with_scale(Vec3::splat(128.)),
//             ..default()
//         });
//     }
// }

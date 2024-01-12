//! Balls
//!
//! A ball is a sphere with a radius and a color.

use crate::{
    graph::{Graph, Idx, UndirectedCSRGraph},
    NodeValue,
};
use bevy::{prelude::*, render::mesh::shape};

fn setup<NI: Idx, NV, EV>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    graph: UndirectedCSRGraph<NI, NV, EV>,
) {
    // Create a ball for each node
    for node in 0..graph.node_count().index() {
        let nv = graph.node_value(NI::new(node));

        let bundle = PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Circle {
                radius: 0.5,
                vertices: 32,
            })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_translation(Vec3::new(30.0, 0.0, 0.0)),
            ..Default::default()
        };
        commands.spawn(bundle);
    }
}

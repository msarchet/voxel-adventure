use bevy::prelude::*;
use bevy::render::mesh;
use generation::chunks;
use meshing::cubemeshes::CubeMeshData;
use crate::common::types::*;
use crate::common::voxels::*;
use crate::meshing::chunk::*;
use crate::generation::chunks::*;

use noise::*;

pub mod common;
pub mod meshing;
pub mod generation;


#[derive(Component)]
struct Moveable;

#[derive(Component)]
struct Generate;


#[derive(Component)]
pub struct Chunk {
    data: ChunkData,
    coords: Vector3Int,
    is_generated: bool,
}

struct State {
    chunks_load: Vec<Vector3Int>,
}

fn main() {
    App::new()
    	.add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(movement)
        .add_system(queue_new_chunks)
        .add_system(generator)
        .init_resource::<CubeMeshData>()
        .insert_resource(State {
            chunks_load: vec![]
        })
        .run();
}


fn setup(
    mut commands: Commands,
    mut state: ResMut<State>,
) {

    let center = Vector3Int { x: 0, y: 0, z: 0};
    for x in -10..10 {
        for z in -10..10 {
            state.chunks_load.push(Vector3Int { x: x, y: 0, z: z} + center);
        }
    }
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 1500.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 200.0, 4.0),
        ..default()
    });

    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-160., 180.0, -160.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    }).insert(Moveable);
}

fn movement(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Moveable>>,
) {
    for mut transform in &mut query {
        let mut direction = Vec3::ZERO;
        if input.pressed(KeyCode::Up) {
            direction.z += 1.0;
        }

        if input.pressed(KeyCode::Down)
        {
            direction.z -= 1.0;
        }

        if input.pressed(KeyCode::Left)
        {
            direction.x += 1.0;
        }

        if input.pressed(KeyCode::Right)
        {
            direction.x -= 1.0;
        }

        if input.pressed(KeyCode::Space)
        {
            direction.y += 1.0;
        }

        if input.pressed(KeyCode::LControl){
            direction.y -= 1.0;
        }

        let mut veclocity = 2.0;

        if input.pressed(KeyCode::LShift) { veclocity *= 5.0; }
        transform.translation += time.delta_seconds() * veclocity * direction;
    }
}

fn queue_new_chunks(
    mut state: ResMut<State>,
    mut commands: Commands,
) {
    let mut i = 0;
    while i < 5 {
        let next_coord = state.chunks_load.pop();
        match next_coord {
            Some(v) => {
                spawn_new_chunk(&mut commands, v);
            },
            None => i = 5,
        }
        i += 1;
    }
}

fn generator(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cube_meshes: Res<CubeMeshData>,
    mut query: Query<(Entity, &mut Chunk)>,
) {
    //noise_gen = noise_gen.set_frequency(0.00825);
    //noise_gen = noise_gen.set_octaves(8);

    for (entity, mut chunk) in query.iter_mut() {
        if chunk.is_generated { continue; }
        chunk.is_generated = true;
        chunk.data.voxels = chunks::get_height_map(Vector3{x: chunk.coords.x as f64, y: chunk.coords.y as f64, z: chunk.coords.z as f64});

        run_first_pass_meshing(&mut chunk.data.voxels);
        let mesh_data = get_mesh_data(&chunk.data, cube_meshes.as_ref());
        let indices = mesh::Indices::U32(mesh_data.indicies);

        let mut chunk_mesh = mesh::Mesh::new(mesh::PrimitiveTopology::TriangleList);

        chunk_mesh.set_indices(Some(indices));
        chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.verticies);
        chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals);
        chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_data.uvs);

        let material = StandardMaterial {
            base_color: Color::rgb(0.0, 1.0, 0.0),
            metallic: 0.0,
            ..default()
        };
        let mesh_id = commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(chunk_mesh),
            material: materials.add(material),
            ..default()
        }).id();

        let sb = SpatialBundle {
            transform: Transform::from_xyz(chunk.coords.x as f32 * 16.0, 0.0, chunk.coords.z as f32 * 16.0),
            ..default()
        };

        commands.entity(entity).insert_bundle(sb);
        commands.entity(entity).push_children(&[mesh_id]);
    }
}

fn spawn_new_chunk(commands: &mut Commands, coords: Vector3Int) {
    let chunk: Chunk = Chunk {
        is_generated: false,
        coords: coords,
        data: ChunkData {
            voxels: vec![],
        }
    };

    commands.spawn_bundle((
        chunk, 
        Transform::from_xyz((coords.x * 16) as f32, (coords.y * 128) as f32, (coords.z * 16) as f32),
    ));
}


//// CAMERA STUFF MOVE SOON from cookbox


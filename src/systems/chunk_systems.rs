use bevy::{prelude::*, utils::HashMap, render::mesh};

use crate::{common::types::*, meshing::{chunk::{run_first_pass_meshing, VoxelFaceEdges, update_edge_meshes, get_mesh_data}, cubemeshes::CubeMeshData}, generation::chunks, MaterialCache};

#[derive(Component)]
pub struct GenerationCenter;

#[derive(Component)]
pub struct Render;

#[derive(Component)]
pub struct Generate;

#[derive(Component)]
pub struct GenerateFaces;

#[derive(Component)]
pub struct Chunk {
    pub data: ChunkData,
    pub coords: Vector3Int,
}

#[derive(Default)]
pub struct ChunkState {
    pub chunks_load: Vec<Vector3Int>,
    pub chunks: HashMap<Vector3Int, Entity>,
    pub center: Vector3Int,
}


#[derive(Copy, Clone)]
pub struct ConfigurationState {
	pub height_seed: i32, 
	pub depth_adjust_seed: i32,
	pub biome_seed: i32,
	pub height_noise_freq: f64,
	pub height_noise_smooth_freq: f64,
	pub depth_adjust_noise_freq: f64,
	pub biome_noise_freq: f64,
	pub height_range: f64,
	pub min_height: i32,
    pub loading_distance: u8,
}

impl Default for ConfigurationState  {
    fn default() -> Self {
        Self { 
            height_seed: 90853,
            depth_adjust_seed: 4958,
            biome_seed: 08320,
            height_noise_freq: 0.00825,
            height_noise_smooth_freq: 0.00825,
            depth_adjust_noise_freq: 0.02125,
            biome_noise_freq: 0.00085,
            height_range: 100.0,
            min_height: 20,
            loading_distance: 16,
        }
    }
}


pub fn reload_chunk(
    mut commands: Commands,
    mut state: ResMut<ChunkState>,
    generation_state: Res<ConfigurationState>,
    input: Res<Input<KeyCode>>,
    query: Query<Entity, With<Chunk>>) {
    
    if !input.pressed(KeyCode::Home) { return }

    query.into_iter().for_each(|e| {
        commands.entity(e).despawn_recursive();
    });

    state.chunks.clear();
    state.chunks_load.clear();

    let copy_center = state.center.clone();
    let min = 0 - generation_state.loading_distance as i64;
    let max = generation_state.loading_distance as i64;

    for x in min..max {
        for z in min..max {
            state.chunks_load.push(Vector3Int { x: x, y: 0, z: z} + copy_center);
        }
    }

}


pub fn queue_new_chunks(
    mut state: ResMut<ChunkState>,
    mut commands: Commands,
) {
    let mut i = 0;
    let chunks_to_load = 10;
    while i < chunks_to_load {
        let next_coord = state.chunks_load.pop();
        match next_coord {
            Some(v) => {
                spawn_new_chunk(&mut commands, &mut state, v);
            },
            None => break,
        }
        i += 1;
    }
}

pub fn manage_loaded_chunk(
    mut state: ResMut<ChunkState>,
    config: Res<ConfigurationState>,
    mut commands: Commands,
    camera_query: Query<(Entity, &Transform), With<GenerationCenter>>,
    query: Query<(Entity, &Chunk)>
) {
    let mut camera_coords: Option::<Vector3Int> = None;

    for (e, transform) in camera_query.iter() {
        camera_coords = Some(Vector3Int {x: transform.translation.x as i64 / 16, y: transform.translation.y as i64, z: transform.translation.z as i64 / 16 });
        //println!("{},{},{}",camera_coords.x, camera_coords.y, camera_coords.z);
    }

    // do some stuff to despawn old chunks (or at least de_render)
    // queue any new chunks for spawning
    // update any chunks with appropriate flags based on location
    match camera_coords {
        Some(camera_coords) => {
            let offset_vector = Vector3Int {x: config.loading_distance as i64, y: 0, z: config.loading_distance as i64};
            let max = camera_coords + offset_vector;
            let min = camera_coords - offset_vector;
            state.center = camera_coords;
            // set some chunks to be loaded
            for x in min.x..max.x {
                for z in min.z..max.z {
                    let coords = Vector3Int {x:x, y: 0, z: z};
                    if !state.chunks.contains_key(&coords)
                        && !state.chunks_load.contains(&coords) {
                            state.chunks_load.push(coords);
                    }
                }
            }

            let unload_distance = Vector3Int {x: config.loading_distance as i64 + 4, y: 0, z: config.loading_distance as i64 + 4};
            let unload_min = camera_coords - unload_distance;
            let unload_max = camera_coords + unload_distance;
			// TODO: Unloading doesn't seem to work :(
            for (e, chunk) in &query {
                if chunk.coords.x < unload_min.x
                    || chunk.coords.z < unload_min.z
                    || chunk.coords.x > unload_max.x
                    || chunk.coords.z > unload_max.z {


                        commands.entity(e).despawn_recursive();
                    }
            }
        },
        None => return
    }

}

pub fn generator(
    mut state: ResMut<ChunkState>,
    config: Res<ConfigurationState>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Chunk), With<Generate>>,
) {
    for (entity, mut chunk) in query.iter_mut() {
        chunk.data.voxels = chunks::get_height_map(Vector3{x: chunk.coords.x as f64, y: chunk.coords.y as f64, z: chunk.coords.z as f64}, config.clone());

        run_first_pass_meshing(&mut chunk.data.voxels);
        state.chunks.insert_unique_unchecked(chunk.coords, entity.clone());
        commands.entity(entity).remove::<Generate>();
    }
}

pub fn generate_full_edge_meshes (
    mut commands: Commands,
    mut set: ParamSet<(Query<(Entity, &Chunk), With<GenerateFaces>>,
                        Query<(Entity, &Chunk), Without<Generate>>,
                        Query<(Entity, &mut Chunk), With<GenerateFaces>>)>,
    face_edges: Res<VoxelFaceEdges>,
    state: Res<ChunkState>
) {
    let mut chunk_neighbors = Vec::<(Entity, Entity, Entity, Entity, Entity)>::new();
    let mut completed_chunks = HashMap::<Entity, Vec::<Voxel>>::new();
    for (e, chunk) in &set.p0() {
        let left = chunk.coords + Vector3Int{ x:0, y: 0, z:1 };
        let right = chunk.coords + Vector3Int{ x: 0, y: 0, z: -1};
        let forward = chunk.coords + Vector3Int { x: 1, y: 0, z: 0};
        let backward = chunk.coords + Vector3Int { x: -1, y: 0, z: 0};

        let left_chunk_id = state.chunks.get(&left);
        let right_chunk_id = state.chunks.get(&right);
        let forward_chunk_id = state.chunks.get(&forward);
        let backward_chunk_id = state.chunks.get(&backward);

        if left_chunk_id.is_none() 
            || right_chunk_id.is_none()
            || forward_chunk_id.is_none()
            || backward_chunk_id.is_none() {
            continue
        }

        if left_chunk_id.is_some() 
            && right_chunk_id.is_some()
            && forward_chunk_id.is_some()
            && backward_chunk_id.is_some() {
                chunk_neighbors.push((e,
                    commands.entity(left_chunk_id.unwrap().clone()).id(),
                    commands.entity(right_chunk_id.unwrap().clone()).id(),
                    commands.entity(forward_chunk_id.unwrap().clone()).id(),
                    commands.entity(backward_chunk_id.unwrap().clone()).id(),
                ));
        }
    }
    
    for (chunk_id, left_chunk_id, right_chunk_id, forward_chunk_id, backward_chunk_id) in chunk_neighbors {
        let q = set.p1();
        let neighbors_query = q.get_many([
            chunk_id,
            left_chunk_id,
            right_chunk_id,
            forward_chunk_id,
            backward_chunk_id]);

        let [
            (_, chunk),
            (_, left_chunk_data),
            (_, right_chunk_data),
            (_, forward_chunk_data),
            (_, backward_chunk_data)] = match neighbors_query {
                Ok (q) => q,
                Err(_) => continue,
        };

        let mut copied_voxels = chunk.data.voxels.clone();

        update_edge_meshes(&mut copied_voxels,
            &left_chunk_data.data.voxels,
            &face_edges.edges[0],
            LEFT_FACE,
            NOT_LEFT_FACE);

        update_edge_meshes(&mut copied_voxels,
            &right_chunk_data.data.voxels,
            &face_edges.edges[1],
            RIGHT_FACE,
            NOT_RIGHT_FACE);

        update_edge_meshes(&mut copied_voxels,
            &forward_chunk_data.data.voxels,
            &face_edges.edges[2],
            FORWARD_FACE,
            NOT_FORWARD_FACE);

        update_edge_meshes(&mut copied_voxels,
            &backward_chunk_data.data.voxels,
            &face_edges.edges[3],
            BACKWARD_FACE,
            NOT_BACKWARD_FACE);

        completed_chunks.insert(chunk_id.clone(), copied_voxels);
    }

    for (e, mut update_chunk) in set.p2().iter_mut() {
        match completed_chunks.get(&e) {
            Some (voxels) => {
                update_chunk.data.voxels = voxels.to_vec();
                //println!("chunk {},{},{}", update_chunk.coords.x, update_chunk.coords.y, update_chunk.coords.z);
                commands.entity(e).remove::<GenerateFaces>();
                commands.entity(e).insert(Render);
            },
            None => continue
        }
    }
}

pub fn render_chunk(
    material_cache: Res<MaterialCache>,
    cube_meshes: Res<CubeMeshData>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(Entity, &Chunk), (With<Render>, Without<Generate>, Without<GenerateFaces>)>,
) {
    for (entity, chunk) in query.iter() {

        let mesh_data = get_mesh_data(&chunk.data, &cube_meshes);
        let indices = mesh::Indices::U32(mesh_data.indicies);

        let mut chunk_mesh = mesh::Mesh::new(mesh::PrimitiveTopology::TriangleList);

        chunk_mesh.set_indices(Some(indices));
        chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.verticies);
        chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals);
        chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_data.uvs);

        let chunk_material;

        match &material_cache.chunk_material {
            Some(material) => chunk_material = material.clone(),
            None => panic!("no chunk mesh material set")
        }

        let mesh_id = commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(chunk_mesh),
            material: chunk_material,
            ..default()
        }).id();

        let sb = SpatialBundle {
            transform: Transform::from_xyz(chunk.coords.x as f32 * 16.0, 0.0, chunk.coords.z as f32 * 16.0),
            ..default()
        };

        commands.entity(entity).remove::<Render>();
        commands.entity(entity).insert_bundle(sb);
        commands.entity(entity).push_children(&[mesh_id]);
    }

}

#[allow(dead_code)]
fn copy_chunk_side(voxels: &Vec<Voxel>, out_voxels: &mut [Voxel;16*128], indicies: &[usize;16*128]) {
    let mut out_index  = 0;
    	for i in 0..indicies.len() {
        out_voxels[out_index] = voxels[i];
        out_index += 1;
    };
}

pub fn spawn_new_chunk(commands: &mut Commands, state: &mut ChunkState, coords: Vector3Int) {
    let chunk: Chunk = Chunk {
        coords: coords,
        data: ChunkData {
            voxels: vec![],
        }
    };

    let id = commands.spawn_bundle((
        chunk, 
        Transform::from_xyz((coords.x * 16) as f32, (coords.y * 128) as f32, (coords.z * 16) as f32),
        Generate,
        GenerateFaces,
    )).id();

    state.chunks.insert(coords, id);
}


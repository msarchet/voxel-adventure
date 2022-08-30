use bevy::{prelude::*, utils::{HashMap, HashSet}, render::mesh};
use bevy_inspector_egui::{Inspectable, egui};

use crate::{
    common::{
        types::*,
        voxels::voxel_helpers
    }, 
    meshing::{
        chunk::{
            run_first_pass_meshing,
            VoxelFaceEdges,
            update_edge_meshes,
            get_mesh_data
        },
        cubemeshes::CubeMeshData
    },
    generation::chunks, MaterialCache
};

#[derive(Component)]
pub struct GenerationCenter;

#[derive(Component)]
pub struct NeedsRender;

#[derive(Component)]
pub struct Generate;

#[derive(Component)]
pub struct GenerateFaces;

#[derive(Component)]
pub struct Chunk {
    pub coords: Vector3Int,
    pub render: bool,
}

#[derive(Default)]
pub struct ChunkState {
    pub chunks_load: Vec<Vector3Int>,
    pub chunks: HashMap<Vector3Int, ChunkData>,
    pub center: Vector3Int,
}

trait ChunkLookup {
    fn get_voxel(&self, chunk_coords: Vector3Int, voxel_coords: VoxelCoords) -> Option<Voxel>;
    fn get_voxel_by_index(&self, chunk_coords: Vector3Int, voxel_index: usize) -> Option<Voxel>;
    fn set_voxel(&mut self, chunk_coords: Vector3Int, voxel_coords: VoxelCoords, data: Voxel) -> Option<Voxel>;
    fn set_voxel_by_index(&mut self, chunk_coords: Vector3Int, voxel_index: usize, data: Voxel) -> Option<Voxel>;
}

impl ChunkLookup for ChunkState {
    fn get_voxel(&self, chunk_coords: Vector3Int, voxel_coords: VoxelCoords) -> Option<Voxel> {
        let index = voxel_helpers::get_index_from_coords(voxel_coords);
        self.get_voxel_by_index(chunk_coords, index)
    }

    fn get_voxel_by_index(&self, chunk_coords: Vector3Int, voxel_index: usize) -> Option<Voxel> {
        if let Some(chunk) = self.chunks.get(&chunk_coords) {
            if chunk.voxels.len() <= voxel_index {
                return Some(chunk.voxels[voxel_index]);
            }
        }

        None
    }

    fn set_voxel(&mut self, chunk_coords: Vector3Int, voxel_coords: VoxelCoords, data: Voxel) -> Option<Voxel> {
        let index = voxel_helpers::get_index_from_coords(voxel_coords);
        self.set_voxel_by_index(chunk_coords, index, data)
    }

    fn set_voxel_by_index(&mut self, chunk_coords: Vector3Int, voxel_index: usize, data: Voxel) -> Option<Voxel> {
        if let Some(chunk) = self.chunks.get_mut(&chunk_coords) {
            if chunk.voxels.len() <= voxel_index {
                chunk.voxels[voxel_index] = data;
                return Some(data)
            }
        }

        None
    }
}


#[derive(Copy, Clone, Inspectable)]
pub struct BiomeConfiguration {

    #[inspectable(min = 0.0, max = 50.0)]
    pub min_height: f64,

    pub height_range: (f64, f64),
    pub range: (f64, f64),

    pub noise_config: Option::<NoiseConfiguration>,
}

#[derive(Copy, Clone, Inspectable)]
pub struct NoiseConfiguration {
    pub seed: i32,

    #[inspectable(min = 1, max = 8)]
    pub octaves: u8,

    #[inspectable(min = 0.00004, max = 1.0, speed=0.0001, wrapper=bigger_width)]
    pub freq: f64,
}

#[derive(Copy, Clone, Inspectable)]
pub struct ConfigurationState {
    pub height_noise_configuration: NoiseConfiguration,
    pub height_noise_smooth_configuration: NoiseConfiguration,

    pub depth_adjust_noise_configuration: NoiseConfiguration,
    pub biome_noise_configuration: NoiseConfiguration,
    pub ocean_biome_config: BiomeConfiguration,
    pub plains_biome_config: BiomeConfiguration,
    pub mountains_biome_config: BiomeConfiguration,

    #[inspectable(min = 5, max = 200)]
    pub loading_distance: u8,
    pub generate_ocean_water: bool,
    pub biome_range: (f64, f64),
    pub biome_smoothing: f64,
}

impl Default for ConfigurationState  {
    fn default() -> Self {
        Self { 
            height_noise_configuration : NoiseConfiguration {
                seed: 13459,
                freq: 0.00250,
                octaves: 8
            },
            biome_noise_configuration: NoiseConfiguration {
                seed: 5983,
                freq: 0.00025,
                octaves: 4,
            },
            depth_adjust_noise_configuration : NoiseConfiguration {
                seed: 4958,
                freq: 0.02125,
                octaves: 6,
            },
            height_noise_smooth_configuration : NoiseConfiguration {
                seed: 13459,
                freq: 0.00825,
                octaves: 3,
            },
            ocean_biome_config: BiomeConfiguration {
                min_height: 5.0,
                height_range: (0.0, 40.0),
                range: (0.0, 0.3),
                noise_config: Some(NoiseConfiguration {
                    seed: 13459,
                    freq: 0.00825,
                    octaves: 3,
                }),
            },
            plains_biome_config: BiomeConfiguration {
                min_height: 40.0,
                height_range: (0.0, 15.0),
                range: (0.3, 0.7),
                noise_config: Some(NoiseConfiguration {
                    seed: 13459,
                    freq: 0.00825,
                    octaves: 4,
                }),
            },
            mountains_biome_config: BiomeConfiguration {
                min_height: 40.0,
                height_range: (0.0, 100.0),
                range: (0.7, 1.0),
                noise_config: Some(NoiseConfiguration {
                seed: 13459,
                freq: 0.00250,
                octaves: 8
                }),
            },
            loading_distance: 16,
            generate_ocean_water: false,
            biome_range: (0.0, 1.0),
            biome_smoothing: 0.025,
        }
    }
}

fn bigger_width(ui: &mut egui::Ui, mut content: impl FnMut(&mut egui::Ui)) {
    ui.scope(|ui| {
        ui.set_min_width(400.0);
        content(ui);
    });
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
            state.chunks_load.push(Vector3Int { x: x, y: 0, z: z } + copy_center);
        }
    }

}

pub struct SetBlockTypeEvent(usize, Vector3Int, BlockType);

pub fn spawn_random_blocks(
    input: Res<Input<KeyCode>>,
    state: Res<ChunkState>,
    mut writer: EventWriter<SetBlockTypeEvent>,
) {
    if !input.just_pressed(KeyCode::I) { return }
    for (&coords, _) in state.chunks.iter() {
        writer.send(SetBlockTypeEvent((rand::random::<u32>() & 0x7FFF) as usize, coords, (rand::random::<u64>() % 8).try_into().unwrap()));
    }
}

pub fn handle_set_block_type_events(
    mut reader: EventReader<SetBlockTypeEvent>,
    mut state: ResMut<ChunkState>,
    mut commands: Commands,
) {
    let mut changes = HashSet::<Vector3Int>::new();

    for event in reader.iter() {
        changes.insert_unique_unchecked(event.1);

        if let Some(voxel) = state.get_voxel_by_index(event.1, event.0) {
            
            let updated = voxel_helpers::set_block_type(voxel, event.2);
            state.set_voxel_by_index(event.1, event.0, updated);
        }
    }

    for coords in changes {
        if let Some(chunk_data) = state.chunks.get_mut(&coords) {
            if let Some(entity) = chunk_data.entity {
                commands.entity(entity).insert(NeedsRender);
            }
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
                spawn_new_chunk(&mut commands, v);
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

    for (_, transform) in camera_query.iter() {
        camera_coords = Some(Vector3Int {x: transform.translation.x as i64 / 16, y: transform.translation.y as i64, z: transform.translation.z as i64 / 16 });
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


                        state.chunks.remove(&chunk.coords);
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
    mut query: Query<(Entity, &Chunk), With<Generate>>,
) {
    for (entity, chunk) in query.iter_mut() {
        let mut new_chunk_data = ChunkData { 
            voxels: chunks::get_height_map(Vector3{x: chunk.coords.x as f64, y: chunk.coords.y as f64, z: chunk.coords.z as f64}, config.clone()),
            entity: Some(entity.clone()),
            ..default()
        };

        run_first_pass_meshing(&mut new_chunk_data.voxels);
        state.chunks.insert_unique_unchecked(chunk.coords, new_chunk_data);
        commands.entity(entity).remove::<Generate>();
    }
}

pub fn generate_full_edge_meshes (
    mut commands: Commands,
    mut query : Query<(Entity, &Chunk), (With<GenerateFaces>, Without<Generate>)>,
    face_edges: Res<VoxelFaceEdges>,
    mut state: ResMut<ChunkState>
) {
    for (e, chunk) in query.iter_mut() {
        let left        = chunk.coords + Vector3Int { x:  0, y: 0, z:  1 };
        let right       = chunk.coords + Vector3Int { x:  0, y: 0, z: -1 };
        let forward     = chunk.coords + Vector3Int { x:  1, y: 0, z:  0 };
        let backward    = chunk.coords + Vector3Int { x: -1, y: 0, z:  0 };

        let mut_state = &mut state;
        if let Some([
            left_chunk_data,
            right_chunk_data,
            forward_chunk_data,
            backward_chunk_data,
            chunk_data
        ]) = mut_state.chunks.get_many_mut([&left, &right, &forward, &backward, &chunk.coords]) {
            update_edge_meshes(&mut chunk_data.voxels,
                &left_chunk_data.voxels,
                &face_edges.edges[0],
                LEFT_FACE,
                NOT_LEFT_FACE);

            update_edge_meshes(&mut chunk_data.voxels,
                &right_chunk_data.voxels,
                &face_edges.edges[1],
                RIGHT_FACE,
                NOT_RIGHT_FACE);

            update_edge_meshes(&mut chunk_data.voxels,
                &forward_chunk_data.voxels,
                &face_edges.edges[2],
                FORWARD_FACE,
                NOT_FORWARD_FACE);

            update_edge_meshes(&mut chunk_data.voxels,
                &backward_chunk_data.voxels,
                &face_edges.edges[3],
                BACKWARD_FACE,
                NOT_BACKWARD_FACE);

            commands.entity(e).remove::<GenerateFaces>();
            commands.entity(e).insert(NeedsRender);

        } 
    }
}

pub fn render_chunk(
    material_cache: Res<MaterialCache>,
    cube_meshes: Res<CubeMeshData>,
    state: Res<ChunkState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(Entity, &Chunk), (With<NeedsRender>, Without<Generate>, Without<GenerateFaces>)>,
) {
    for (entity, chunk) in query.iter() {

        let chunk_data = match state.chunks.get(&chunk.coords) {
            Some (data) => data,
            None => continue,
        };

        let mesh_data = get_mesh_data(&chunk_data, &cube_meshes);
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

        commands.entity(entity).remove::<NeedsRender>();
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

pub fn spawn_new_chunk(commands: &mut Commands, coords: Vector3Int) {
    commands.spawn_bundle((
        Chunk {
            coords: coords,
            render: false,
        },
        Transform::from_xyz((coords.x * 16) as f32, (coords.y * 128) as f32, (coords.z * 16) as f32),
        Generate,
        GenerateFaces,
    ));
}

#[derive(Default)]
pub struct FluidUpdateResult {
    pub updates: Vec<(Vector3Int, VoxelCoords, u8)>,
}

pub fn update_initial_fluids(voxels: &Vec<Voxel>) -> HashMap<u64, u8> {
    let mut fluid_map = HashMap::<u64, u8>::new();
    for i in 0..voxels.len() {
        let voxel = voxels[i];
        if voxel_helpers::get_block_type(voxel) == BlockType::Water as u64 {
            fluid_map.insert_unique_unchecked(i as u64, 8);
        }
    }

    fluid_map
}

pub fn fluid_update_system(
    mut fluid_event: EventWriter<FluidUpdateEvent>,
    query: Query<(Entity, &Chunk)>,
    chunk_state: Res<ChunkState>
) {
    for (e, chunk) in query.iter() {
        update_fluids(chunk.coords, &chunk_state, &mut fluid_event)
    }
}

pub fn fluid_update_event_processor(
    mut fluid_events: EventReader<FluidUpdateEvent>,
    mut chunk_state: ResMut<ChunkState>,
    mut commands: Commands,
) {
    let mut update_map = HashSet::<(Entity, Vector3Int)>::new();

    for event in fluid_events.iter() {
        if let Some(chunk_data) = chunk_state.chunks.get_mut(&event.0) {
            let index = voxel_helpers::get_index_from_coords(event.1);
            let mut voxel = chunk_data.voxels[index];

            // TODO: Safety check to see if it isn't filled since the fluid event?

            voxel = voxel_helpers::set_block_type(voxel, BlockType::Water);
            chunk_data.voxels[index] = voxel;
            chunk_data.flowing_fluids.insert_unique_unchecked(index, event.2);

            if let Some(entity) = chunk_data.entity {
                update_map.insert_unique_unchecked((entity, event.0));
            }
        }
    }

    for (entity, coords) in update_map {

        if let Some(chunk_data) = chunk_state.chunks.get_mut(&coords) {
            chunk_data.flowing_fluids.clear();
        }

        commands.entity(entity).insert(NeedsRender);
    }
}

// pub struct NeedsRenderEvent
pub struct FluidUpdateEvent(Vector3Int, VoxelCoords, u8);

pub fn update_fluids(chunk_coords: Vector3Int, chunk_state: &ChunkState, writer: &mut EventWriter<FluidUpdateEvent>) {

    match chunk_state.chunks.get(&chunk_coords) {
        Some(state) => {
            let mut target_coords;
            let mut target_chunk = chunk_coords;
            let mut check_vec = vec!();

            for (index, rate) in &state.flowing_fluids {
                if rate > &0 {
                    // check neighbors for voids and the push the update to results
                    let coords = voxel_helpers::get_coords_as_voxel_coords(index.clone() as u64);
                    target_coords = coords;
                    if coords.y > 0 {
                        target_coords.y -= 1;

                        if let Some(voxel) = chunk_state.get_voxel(target_chunk, target_coords) {
                            if !voxel_helpers::is_filled(voxel) {
                                writer.send(FluidUpdateEvent(target_chunk, target_coords, rate - 1));
                                continue
                            }
                        }
                    }

                    target_chunk = chunk_coords;
                    target_coords = coords;

                    // backward
                    if coords.x == 0 {
                        target_coords.x = 15;
                        target_chunk = chunk_coords + VECTOR3_INT_BACKWARD;
                        check_vec.push((target_chunk, target_coords));
                    } else {
                        target_coords.x -= 1;
                        check_vec.push((target_chunk, target_coords));
                    }

                    target_chunk = chunk_coords;
                    target_coords = coords;
                    //forward
                    if coords.x == 15 {
                        target_coords.x = 0;
                        target_chunk = chunk_coords + VECTOR3_INT_FORWARD;
                        check_vec.push((target_chunk, target_coords));
                    } else {
                        target_coords.x += 1;
                        check_vec.push((target_chunk, target_coords));
                    }

                    target_chunk = chunk_coords;
                    target_coords = coords;
                    // left
                    if coords.z == 0 {
                        target_coords.z = 15;
                        target_chunk = chunk_coords + VECTOR3_INT_LEFT;
                        check_vec.push((target_chunk, target_coords));
                    } else {
                        target_coords.z -= 1;
                        check_vec.push((target_chunk, target_coords));
                    }

                    target_chunk = chunk_coords;
                    target_coords = coords;

                    //right
                    if coords.z == 15 {
                        target_coords.z = 0;
                        target_chunk = chunk_coords + VECTOR3_INT_RIGHT;
                        check_vec.push((target_chunk, target_coords));
                    } else {
                        target_coords.z += 1;
                        check_vec.push((target_chunk, target_coords));
                    }
                    

                    //iterate and check
                    for i in 0..check_vec.len() {
                        let (target_chunk, target_coords) = check_vec[i] ;

                        if let Some(voxel) = chunk_state.get_voxel(target_chunk, target_coords) {
                            if !voxel_helpers::is_filled(voxel) {
                                writer.send(FluidUpdateEvent(target_chunk, target_coords, rate - 1));
                            }

                        }

                    }

                }

                check_vec.clear();

            }
        },
        None => ()
    }
}


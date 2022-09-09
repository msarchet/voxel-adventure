use bevy::{prelude::*, utils::{HashMap, HashSet}, render::mesh};
use bevy_inspector_egui::{Inspectable, egui};

use crate::{
    common::{
        types::*,
        voxels::voxel_helpers
    }, 
    meshing::{
        chunk::*,
        cubemeshes::CubeMeshData,
    },
    generation::chunks, MaterialCache
};

use bevy_inspector_egui::InspectorPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
enum SystemStages {
    Cleanup,
}

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

#[derive(Component)]
pub struct MeshReference {
    handle: Option<Handle<Mesh>>,
}

impl Default for MeshReference {
    fn default() -> Self {
        Self { handle: None }
    }
}

#[derive(Default)]
pub struct ChunkState {
    pub chunks_load: Vec<Vector3Int>,
    pub chunks: HashMap<Vector3Int, ChunkData>,
    pub center: Vector3Int,
}

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetBlockTypeEvent>()
            .add_event::<FluidUpdateEvent>()
            .add_startup_system(setup)
            .add_system(queue_new_chunks)
            .add_system(generator.after(queue_new_chunks))
            .add_system(generate_full_edge_meshes.after(generator))
            .add_system(spawn_random_blocks.after(generate_full_edge_meshes))
            .add_system(fluid_update_system.after(generate_full_edge_meshes))
            .add_system(fluid_update_event_processor.after(fluid_update_system))
            .add_system(handle_set_block_type_events.after(fluid_update_event_processor))
            .add_system(render_chunk.after(generate_full_edge_meshes))
            .add_system(reload_chunk.after(render_chunk))
            .add_stage_after(CoreStage::Last, SystemStages::Cleanup, SystemStage::parallel())
            .add_system_to_stage(SystemStages::Cleanup, manage_loaded_chunk)
            .init_resource::<CubeMeshData>()
            .init_resource::<ConfigurationState>()
            .init_resource::<VoxelFaceEdges>()
            .init_resource::<ChunkState>()
            .init_resource::<MaterialCache>()
            .add_plugin(InspectorPlugin::<ConfigurationState>::new());

    }
}

fn setup(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<ChunkState>,
    mut material_cache: ResMut<MaterialCache>,
    config: Res<ConfigurationState>,
    asset_server: Res<AssetServer>,
) {

    let texture_handle = asset_server.load("textures/simple_textures.png");

    let chunk_material = materials.add(StandardMaterial {
        metallic: 0.0,
        reflectance: 0.0,
        base_color_texture : Option::Some(texture_handle),
        ..default()
    });

    material_cache.chunk_material = Some(chunk_material);

    let center = state.center;

    let loading_distance = config.loading_distance as i64;
    for x in 0-loading_distance..loading_distance {
        for z in 0-loading_distance..loading_distance {
            state.chunks_load.push(Vector3Int { x: x, y: 0, z: z } + center);
        }
    }
}


pub trait ChunkLookup {
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
            if chunk.voxels.len() >= voxel_index {
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
            if chunk.voxels.len() >= voxel_index {
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

pub struct SetBlockTypeEvent {
    index: usize,
    chunk_coords: Vector3Int,
    block_type: BlockType,
    flow_rate: u8,
    replace: bool,
}

pub fn spawn_random_blocks(
    input: Res<Input<KeyCode>>,
    state: Res<ChunkState>,
    mut writer: EventWriter<FluidUpdateEvent>,
) {
    if !input.just_pressed(KeyCode::I) { return }
    for (&coords, _) in state.chunks.iter() {
        if coords.x % 2 == 0 && coords.z % 2 == 0 { continue }
        writer.send(FluidUpdateEvent(coords, VoxelCoords {x: 6, y : 100, z: 6}, 8));
    }
}

pub fn handle_set_block_type_events(
    mut reader: EventReader<SetBlockTypeEvent>,
    mut state: ResMut<ChunkState>,
    mut commands: Commands,
) {
    let mut changes = HashSet::<Vector3Int>::new();

    for event in reader.iter() {

        if let Some(voxel) = state.get_voxel_by_index(event.chunk_coords, event.index) {
            if event.replace == false && voxel_helpers::is_filled(voxel) {
                continue 
            }

            let mut updated = voxel_helpers::set_block_type(voxel, event.block_type);
            if event.block_type == BlockType::Water {
                if let Some(chunk_data) = state.chunks.get_mut(&event.chunk_coords) {
                    chunk_data.flowing_fluids.insert_unique_unchecked(event.index, event.flow_rate);
                }
            }
            updated = voxel_helpers::set_filled(updated, true);
            state.set_voxel_by_index(event.chunk_coords, event.index, updated);
            changes.insert_unique_unchecked(event.chunk_coords);
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
        let fluid_map = update_initial_fluids(&new_chunk_data.voxels);
        new_chunk_data.flowing_fluids = fluid_map;
        state.chunks.insert_unique_unchecked(chunk.coords, new_chunk_data);
        commands.entity(entity).remove::<Generate>();
    }
}

pub fn generate_full_edge_meshes (
    mut commands: Commands,
    mut query : Query<(Entity, &Chunk), (With<GenerateFaces>, Without<Generate>)>,
    mut state: ResMut<ChunkState>
) {
    for (e, chunk) in query.iter_mut() {
        let left        = chunk.coords + Vector3Int { x:  0, y: 0, z:  1 };
        let right       = chunk.coords + Vector3Int { x:  0, y: 0, z: -1 };
        let forward     = chunk.coords + Vector3Int { x:  1, y: 0, z:  0 };
        let backward    = chunk.coords + Vector3Int { x: -1, y: 0, z:  0 };

        let mut_state = &mut state;
        if let Some(_) = mut_state.chunks.get_many_mut([&left, &right, &forward, &backward, &chunk.coords]) {

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
    query: Query<(Entity, &Chunk, &MeshReference), With<NeedsRender>>,
) {
    for (entity, chunk, mesh_reference) in query.iter() {

        let face_data = generate_mesh_raw_data(chunk.coords, &state);
        let mesh_data = get_mesh_data(&face_data, &cube_meshes);
        let indices = mesh::Indices::U32(mesh_data.indicies);

        let chunk_mesh_handle = match mesh_reference.handle.clone() {
            Some(handle) => handle.clone(),
            None => { 
                let handle = meshes.add(mesh::Mesh::new(mesh::PrimitiveTopology::TriangleList));

                let chunk_material;

                match &material_cache.chunk_material {
                    Some(material) => chunk_material = material.clone(),
                    None => panic!("no chunk mesh material set")
                }

                let mesh_id = commands.spawn_bundle(PbrBundle {
                    mesh: handle.clone(),
                    material: chunk_material,
                    ..default()
                }).id();

                let sb = SpatialBundle {
                    transform: Transform::from_xyz(chunk.coords.x as f32 * 16.0, 0.0, chunk.coords.z as f32 * 16.0),
                    ..default()
                };

                commands.entity(entity).insert(MeshReference{handle: Some(handle.clone())});
                commands.entity(entity).insert_bundle(sb);
                commands.entity(entity).push_children(&[mesh_id]);
            
                handle.clone()
            }
        };

        let chunk_mesh = meshes.get_mut(&chunk_mesh_handle).unwrap();

        chunk_mesh.set_indices(Some(indices));
        chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.verticies);
        chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals);
        chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_data.uvs);


        commands.entity(entity).remove::<NeedsRender>();
    }

}

#[allow(dead_code)]
fn copy_chunk_side(voxels: &VoxelCollection, out_voxels: &mut [Voxel;16*128], indicies: &[usize;16*128]) {
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
        MeshReference {handle : None }
    ));
}

#[derive(Default)]
pub struct FluidUpdateResult {
    pub updates: Vec<(Vector3Int, VoxelCoords, u8)>,
}

pub fn update_initial_fluids(voxels: &VoxelCollection) -> HashMap<usize, u8> {
    let mut fluid_map = HashMap::<usize, u8>::new();
    for i in 0..voxels.len() {
        let voxel = voxels[i];
        if voxel_helpers::get_block_type(voxel) == BlockType::Water as u64 {
            fluid_map.insert_unique_unchecked(i, 0);
        }
    }

    fluid_map
}

pub fn fluid_update_system(
    mut fluid_event: EventWriter<FluidUpdateEvent>,
    query: Query<(Entity, &Chunk)>,
    mut chunk_state: ResMut<ChunkState>,
) {
    for (_, chunk) in query.iter() {
        update_fluids(chunk.coords, &mut chunk_state, &mut fluid_event)
    }
}

pub fn fluid_update_event_processor(
    mut fluid_events: EventReader<FluidUpdateEvent>,
    mut set_block_writer: EventWriter<SetBlockTypeEvent>,
) {
    for event in fluid_events.iter() {
        let index = voxel_helpers::get_index_from_coords(event.1);
        set_block_writer.send(SetBlockTypeEvent{
            index, 
            chunk_coords: event.0,
            block_type: BlockType::Water,
            flow_rate: event.2, 
            replace: false
        });
    }
}

// pub struct NeedsRenderEvent
pub struct FluidUpdateEvent(Vector3Int, VoxelCoords, u8);

pub fn update_fluids(chunk_coords: Vector3Int, chunk_state: &mut ChunkState, writer: &mut EventWriter<FluidUpdateEvent>) {

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
                            if !voxel_helpers::is_filled(voxel.clone()) {
                                writer.send(FluidUpdateEvent(target_chunk, target_coords, 8));
                                continue
                            }

                            if voxel_helpers::get_block_type(voxel.clone()) == BlockType::Water as u64 {
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
                    } else {
                        target_coords.x -= 1;
                    }
                    check_vec.push((target_chunk, target_coords));

                    target_chunk = chunk_coords;
                    target_coords = coords;
                    //forward
                    if coords.x == 15 {
                        target_coords.x = 0;
                        target_chunk = chunk_coords + VECTOR3_INT_FORWARD;
                    } else {
                        target_coords.x += 1;
                    }

                    check_vec.push((target_chunk, target_coords));
                    target_chunk = chunk_coords;
                    target_coords = coords;
                    // left
                    if coords.z == 0 {
                        target_coords.z = 15;
                        target_chunk = chunk_coords + VECTOR3_INT_RIGHT;
                    } else {
                        target_coords.z -= 1;
                    }

                    check_vec.push((target_chunk, target_coords));
                    target_chunk = chunk_coords;
                    target_coords = coords;

                    //right
                    if coords.z == 15 {
                        target_coords.z = 0;
                        target_chunk = chunk_coords + VECTOR3_INT_LEFT;
                    } else {
                        target_coords.z += 1;
                    }

                    check_vec.push((target_chunk, target_coords));
                    

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

            if let Some(state) = chunk_state.chunks.get_mut(&chunk_coords) {
                state.flowing_fluids.clear();
            }

        },
        None => ()
    }

}


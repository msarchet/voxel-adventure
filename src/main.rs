use bevy::input::mouse::MouseMotion;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::camera::Projection;
use bevy::render::mesh;
use bevy::utils::HashMap;
use bevy_egui::{egui, EguiContext, EguiPlugin};

use generation::chunks;
use meshing::cubemeshes::CubeMeshData;
use crate::common::types::*;
use crate::meshing::chunk::*;

pub mod common;
pub mod meshing;
pub mod generation;
pub mod systems;


#[derive(Component)]
struct Moveable;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Render;

#[derive(Component)]
struct Generate;

#[derive(Component)]
struct GenerateFaces;


#[derive(Component)]
pub struct Chunk {
    data: ChunkData,
    coords: Vector3Int,
}

struct MaterialCache {
    chunk_material: Option<Handle<StandardMaterial>>,
}

struct State {
    chunks_load: Vec<Vector3Int>,
    chunks: HashMap<Vector3Int, Entity>,
}


#[derive(Copy, Clone)]
pub struct GenerationState {
	height_seed: i32, 
	depth_adjust_seed: i32,
	biome_seed: i32,
	height_noise_freq: f64,
	height_noise_smooth_freq: f64,
	depth_adjust_noise_freq: f64,
	biome_noise_freq: f64,
	height_range: f64,
	min_height: i32,
}

impl Default for GenerationState  {
    fn default() -> Self {
        Self { 
            height_seed: 90853,
            depth_adjust_seed: 4958,
            biome_seed: 08320,
            height_noise_freq: 0.00825,
            height_noise_smooth_freq: 0.000825,
            depth_adjust_noise_freq: 0.002125,
            biome_noise_freq: 0.000085,
            height_range: 100.0,
            min_height: 20
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
enum CustomStages{
    Cleanup,
}

fn main() {
    App::new()
    	.add_plugins(DefaultPlugins)
    	.add_plugin(EguiPlugin)
        .add_startup_system(setup)
        .add_system(queue_new_chunks)
        .add_system(generator.after(queue_new_chunks))
        .add_system(generate_full_edge_meshes.after(generator))
        .add_system(render_chunk.after(generate_full_edge_meshes))
        .add_system(movement.after(generator))
        .add_system(pan_orbit_camera.after(movement))
        .add_system(reload_chunk.after(pan_orbit_camera))
        .add_system(ui_main)
        .add_stage_after(CoreStage::Last, CustomStages::Cleanup, SystemStage::parallel())
        .add_system_to_stage(CustomStages::Cleanup, manage_loaded_chunk)
        .init_resource::<CubeMeshData>()
        .init_resource::<GenerationState>()
        .init_resource::<VoxelFaceEdges>()
        .insert_resource(MaterialCache { chunk_material: Option::None })
        .insert_resource(State {
            chunks_load: vec![],
            chunks: HashMap::<Vector3Int, Entity>::new()
        })
        .run();
}


fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<State>,
    mut material_cache: ResMut<MaterialCache>,
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

    let center = Vector3Int { x: 0, y: 0, z: 0};
    for x in -5..5 {
        for z in -5..5 {
            state.chunks_load.push(Vector3Int { x: x, y: 0, z: z} + center);
        }
    }
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform { 
            translation: Vec3::new(0.0, 1000.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
        ..default()
    });

    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-5.0, 120.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    }).insert_bundle((PanOrbitCamera { ..default() }, Moveable, MainCamera));
}

fn movement(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Moveable>>,
) {
    for mut transform in &mut query {
        let mut direction = Vec3::ZERO;
        if input.pressed(KeyCode::W) {
            direction.z -= 1.0;
        }

        if input.pressed(KeyCode::S)
        {
            direction.z += 1.0;
        }

        if input.pressed(KeyCode::A)
        {
            direction.x += 1.0;
        }

        if input.pressed(KeyCode::D)
        {
            direction.x -= 1.0;
        }

        let mut veclocity = 10.0;

        if input.pressed(KeyCode::LShift) { veclocity *= 5.0; }
        let forward = transform.rotation * Vec3::X * -direction.x;
        let left = transform.rotation * Vec3::Z * direction.z;
        // make panning proportional to distance away from focus point
        direction = forward + left;

        if input.pressed(KeyCode::Space)
        {
            direction.y += 1.0;
        }

        if input.pressed(KeyCode::LControl){
            direction.y -= 1.0;
        }


        transform.translation += time.delta_seconds() * veclocity * direction;
    }
}

fn reload_chunk(
    mut commands: Commands,
    mut state: ResMut<State>,
    input: Res<Input<KeyCode>>,
    query: Query<Entity, With<Chunk>>) {
    
    if !input.pressed(KeyCode::Home) { return }

    query.into_iter().for_each(|e| {
        commands.entity(e).despawn_recursive();
    });

    state.chunks.clear();
    state.chunks_load.clear();

    let center = Vector3Int { x: 0, y: 0, z: 0};
    for x in -20..20 {
        for z in -20..20 {
            state.chunks_load.push(Vector3Int { x: x, y: 0, z: z} + center);
        }
    }

}


fn queue_new_chunks(
    mut state: ResMut<State>,
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

fn manage_loaded_chunk(
    mut state: ResMut<State>,
    mut commands: Commands,
    camera_query: Query<(Entity, &Transform), With<MainCamera>>,
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
            let max = camera_coords + Vector3Int {x: 20, y: 0, z: 20};
            let min = camera_coords - Vector3Int {x: 20, y: 0, z: 20};
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

            for (e, chunk) in &query {
                if chunk.coords.x < min.x
                    || chunk.coords.z < min.z
                    || chunk.coords.x > max.x
                    || chunk.coords.z > max.z {

                        match state.chunks.remove(&chunk.coords) {
                            Some (coords) => println!("Removed"),
                            None => panic!("didnt remove,{},{}", chunk.coords.x, chunk.coords.z)
                        }

                        commands.entity(e).despawn_recursive();
                    }
            }
        },
        None => return
    }

}

fn generator(
    mut state: ResMut<State>,
    config: Res<GenerationState>,
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

fn generate_full_edge_meshes (
    mut commands: Commands,
    mut set: ParamSet<(Query<(Entity, &Chunk), With<GenerateFaces>>,
                        Query<(Entity, &Chunk), Without<Generate>>,
                        Query<(Entity, &mut Chunk), With<GenerateFaces>>)>,
    face_edges: Res<VoxelFaceEdges>,
    state: Res<State>
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

fn render_chunk(
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

fn spawn_new_chunk(commands: &mut Commands, state: &mut State, coords: Vector3Int) {
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

fn ui_main(
    mut egui_context: ResMut<EguiContext>,
    mut config: ResMut<GenerationState>,
    mut biome_noise_freq: Local<String>,
) {
    egui::panel::SidePanel::left("config_panel").show(egui_context.ctx_mut(), |ui| {
        if biome_noise_freq.len() == 0 {
            *biome_noise_freq = config.biome_noise_freq.to_string();
        }
        let response = ui.add(egui::TextEdit::singleline(&mut *biome_noise_freq));
        if response.changed() {
           // *biome_noise_freq
        }
        if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
            config.biome_noise_freq = match biome_noise_freq.parse::<f64>() {
                Ok(val) => val,
                Err(_) => config.biome_noise_freq,
            }
        }
    });
        // …
}


//// CAMERA STUFF MOVE SOON from cookbox
/// 
/// 
/// Tags an entity as capable of panning and orbiting.
#[derive(Component)]
struct PanOrbitCamera {
    /// The "focus point" to orbit around. It is automatically updated when panning the camera
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        PanOrbitCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
        }
    }
}

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
fn pan_orbit_camera(
    windows: Res<Windows>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<Input<MouseButton>>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
) {
    // change input mapping for orbit and panning here
    let orbit_button = MouseButton::Right;
    let pan_button = MouseButton::Middle;

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;

    if input_mouse.pressed(orbit_button) {
        for ev in ev_motion.iter() {
            rotation_move += ev.delta;
        }
    } else if input_mouse.pressed(pan_button) {
        // Pan only if we're not rotating at the moment
        for ev in ev_motion.iter() {
            pan += ev.delta;
        }
    }
    for ev in ev_scroll.iter() {
        scroll += ev.y;
    }
    if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
        orbit_button_changed = true;
    }

    for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
        if orbit_button_changed {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = transform.rotation * Vec3::Y;
            pan_orbit.upside_down = up.y <= 0.0;
        }

        if rotation_move.length_squared() > 0.0 {
            let window = get_primary_window_size(&windows);
            let delta_x = {
                let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
                if pan_orbit.upside_down { -delta } else { delta }
            };
            let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation = transform.rotation * pitch; // rotate around local x axis
        } else if pan.length_squared() > 0.0 {
            // make panning distance independent of resolution and FOV,
            let window = get_primary_window_size(&windows);
            if let Projection::Perspective(projection) = projection {
                pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
            }
            // translate by local axes
            let right = transform.rotation * Vec3::X * -pan.x;
            let up = transform.rotation * Vec3::Y * pan.y;
            // make panning proportional to distance away from focus point
            let translation = (right + up) * pan_orbit.radius;
            pan_orbit.focus += translation;
        } else if scroll.abs() > 0.0 {
            pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
            // dont allow zoom to reach zero or you get stuck
            pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
        }

        //if any {
        //    // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
        //    // parent = x and y rotation
        //    // child = z-offset
        //    let rot_matrix = Mat3::from_quat(transform.rotation);
        //transform.translation = pan_orbit.focus;
        //}
    }
}

fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
    let window = windows.get_primary().unwrap();
    let window = Vec2::new(window.width() as f32, window.height() as f32);
    window
}
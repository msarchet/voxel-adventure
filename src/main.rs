use bevy::input::mouse::MouseMotion;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::camera::Projection;
use bevy::utils::HashMap;
use bevy_egui::{egui, EguiContext, EguiPlugin};

use meshing::cubemeshes::CubeMeshData;
use crate::common::types::*;
use crate::meshing::chunk::*;
use crate::systems::chunk_systems::*;

pub mod common;
pub mod meshing;
pub mod generation;
pub mod systems;


#[derive(Component)]
struct Moveable;

pub struct MaterialCache {
    chunk_material: Option<Handle<StandardMaterial>>,
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
        .insert_resource(ChunkState {
            chunks_load: vec![],
            chunks: HashMap::<Vector3Int, Entity>::new()
        })
        .run();
}


fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<ChunkState>,
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
    }).insert_bundle((PanOrbitCamera { ..default() }, Moveable, GenerationCenter));
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
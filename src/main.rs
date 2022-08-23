
use bevy::input::mouse::MouseMotion;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::camera::Projection;
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
        .init_resource::<ConfigurationState>()
        .init_resource::<VoxelFaceEdges>()
        .init_resource::<ChunkState>()
        .insert_resource(MaterialCache { chunk_material: Option::None })
        .run();
}


fn setup(
    mut commands: Commands,
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


#[derive(Default)]
struct GenerateStateEdit {
	pub height_seed: String, 
    pub height_noise_ocatves: String,
	pub height_noise_freq: String,
	pub height_noise_smooth_freq: String,
    pub height_noise_smooth_ocatves: String,
	pub depth_adjust_seed: String,
	pub depth_adjust_noise_freq: String,
    pub depth_adjust_noise_octaves: String,
	pub biome_seed: String,
	pub biome_noise_freq: String,
    pub biome_noise_octaves: String,
	pub height_range: String,
	pub min_height: String,
    pub loading_distance: String,
}

fn ui_main(
    input: Res<Input<KeyCode>>,
    state: Res<ChunkState>,
    mut egui_context: ResMut<EguiContext>,
    mut config: ResMut<ConfigurationState>,
    mut edit_config: Local<GenerateStateEdit>,
    mut ran_once: Local<bool>,
    mut is_shown: Local<bool>,
    mut is_init: Local<bool>,
) {
    if *ran_once != true {
        *ran_once = true;
        *is_shown = true;
    }


    if input.just_pressed(KeyCode::End) {
        *is_shown = !(*is_shown);
    }

    if !(*is_shown) { return }

    egui::panel::SidePanel::left("config_panel").show(egui_context.ctx_mut(), |ui| {
        if  !*is_init {
            *is_init = true;
            let existing = *config;
            edit_config.height_seed = existing.height_seed.to_string();
            edit_config.height_noise_ocatves= existing.height_noise_octaves.to_string();
            edit_config.height_noise_freq = existing.height_noise_freq.to_string();
            edit_config.height_noise_smooth_freq = existing.height_noise_smooth_freq.to_string();
            edit_config.height_noise_smooth_ocatves= existing.height_noise_smooth_octaves.to_string();
            
            edit_config.depth_adjust_seed = existing.depth_adjust_seed.to_string();
            edit_config.depth_adjust_noise_freq = existing.depth_adjust_noise_freq.to_string();
            edit_config.depth_adjust_noise_octaves= existing.depth_adjust_noise_octaves.to_string();

            edit_config.biome_seed= existing.biome_seed.to_string();
            edit_config.biome_noise_freq = existing.biome_noise_freq.to_string();
            edit_config.biome_noise_octaves= existing.biome_noise_octaves.to_string();

            edit_config.height_range = existing.height_range.to_string();
            edit_config.min_height = existing.min_height.to_string();

            edit_config.loading_distance = existing.loading_distance.to_string();
        }

        ui.set_max_width(300.0);
        ui.add(egui::Label::new("Height Seed"));
        ui.add(egui::TextEdit::singleline(&mut edit_config.height_seed));
        ui.add(egui::Label::new("Height Noise Freq"));
        ui.add(egui::TextEdit::singleline(&mut edit_config.height_noise_freq));
        ui.add(egui::Label::new("Height Noise Octaves"));
        ui.add(egui::TextEdit::singleline(&mut edit_config.height_noise_ocatves));
        ui.add(egui::Label::new("Height Noise Smooth Freq"));
        ui.add(egui::TextEdit::singleline(&mut edit_config.height_noise_smooth_freq));
        ui.add(egui::Label::new("Height Noise Smooth Octaves"));
        ui.add(egui::TextEdit::singleline(&mut edit_config.height_noise_smooth_ocatves));
        ui.add(egui::Label::new("Depth Adjust Seed"));
        ui.add(egui::TextEdit::singleline(&mut edit_config.depth_adjust_seed));
        ui.add(egui::Label::new("Depth Adjust Noise Freq"));
        ui.add(egui::TextEdit::singleline(&mut edit_config.depth_adjust_noise_freq));
        ui.add(egui::Label::new("Depth Adjust Octaves"));
        ui.add(egui::TextEdit::singleline(&mut edit_config.depth_adjust_noise_octaves));
        ui.add(egui::Label::new("Biome Seed"));
        ui.add(egui::TextEdit::singleline(&mut edit_config.biome_seed));
        ui.add(egui::Label::new("Biome Noise Freq"));
        ui.add(egui::TextEdit::singleline(&mut edit_config.biome_noise_freq));
        ui.add(egui::Label::new("Biome Noise Octaves"));
        ui.add(egui::TextEdit::singleline(&mut edit_config.biome_noise_octaves));
        ui.add(egui::Label::new("Height Range"));
        ui.add(egui::TextEdit::singleline(&mut edit_config.height_range));
        ui.add(egui::Label::new("Min Height"));
        ui.add(egui::TextEdit::singleline(&mut edit_config.min_height));
        ui.add(egui::Label::new("Loading Distance"));
        ui.add(egui::TextEdit::singleline(&mut edit_config.loading_distance));
        
        let update = ui.add(egui::Button::new("Update Config"));

        ui.horizontal_wrapped(|ui| {
            ui.add(egui::Label::new(state.center.x.to_string()));
            ui.add(egui::Label::new(state.center.z.to_string()));
            ui.add(egui::Label::new("Press Home to clear chunks."));
            ui.add(egui::Label::new("Press End to toogle this UI."));
        });
        if update.clicked() {
            config.height_seed = match edit_config.height_seed.parse::<i32>() {
                Ok(val) => val,
                Err(_) => config.height_seed,
            };
            config.height_noise_freq = match edit_config.height_noise_freq.parse::<f64>() {
                Ok(val) => val,
                Err(_) => config.height_noise_freq,
            };
            config.height_noise_smooth_freq = match edit_config.height_noise_smooth_freq.parse::<f64>() {
                Ok(val) => val,
                Err(_) => config.height_noise_smooth_freq,
            };
            config.depth_adjust_seed = match edit_config.depth_adjust_seed.parse::<i32>() {
                Ok(val) => val,
                Err(_) => config.depth_adjust_seed,
            };
            config.depth_adjust_noise_freq = match edit_config.depth_adjust_noise_freq.parse::<f64>() {
                Ok(val) => val,
                Err(_) => config.depth_adjust_noise_freq,
            };
            config.biome_seed = match edit_config.biome_seed.parse::<i32>() {
                Ok(val) => val,
                Err(_) => config.biome_seed,
            };
            config.biome_noise_freq = match edit_config.biome_noise_freq.parse::<f64>() {
                Ok(val) => val,
                Err(_) => config.biome_noise_freq,
            };
            config.height_range = match edit_config.height_range.parse::<f64>() {
                Ok(val) => val,
                Err(_) => config.height_range,
            };
            config.min_height = match edit_config.min_height.parse::<i32>() {
                Ok(val) => val,
                Err(_) => config.min_height,
            };
            config.height_noise_octaves = match edit_config.height_noise_ocatves.parse::<u8>() {
                Ok (val) => val,
                Err(_) => config.height_noise_octaves
            };
            config.biome_noise_octaves = match edit_config.biome_noise_octaves.parse::<u8>() {
                Ok (val) => val,
                Err(_) => config.biome_noise_octaves
            };
            config.height_noise_smooth_octaves = match edit_config.height_noise_smooth_ocatves.parse::<u8>() {
                Ok (val) => val,
                Err(_) => config.height_noise_smooth_octaves
            };
            config.depth_adjust_noise_octaves = match edit_config.depth_adjust_noise_octaves.parse::<u8>() {
                Ok (val) => val,
                Err(_) => config.depth_adjust_noise_octaves
            };

            config.loading_distance = match edit_config.loading_distance.parse::<u8>() {
                Ok (val) => val,
                Err(_) => config.loading_distance
            };
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
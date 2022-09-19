use noise::{OpenSimplex, Seedable};
use rand::SeedableRng;

use crate::common::types::*;

use super::noise::{noise_with_octaves, noise_with_octaves_01};

#[derive(Eq, PartialEq, Hash)]
pub enum CaveType {
	Squiggly,
	MoreVertical,
	Noodle
}


pub fn GenerateCave(
	head: Vector3, 
	offset: Vector3Int,
	segement_count: u8,
	twistiness: f64,
	seed: i32,
	thickness: f64,
	segement_length: f64,
	cave_type: CaveType,
	mut structure_data: &mut Vec<StructureData>) {
	let mut cave_noise_gen = OpenSimplex::new();

	let mut current_segement_position = head + offset;
	let noise_head_start = head + offset;

	let mut current_noise_position: Vector3 = VECTOR3ZERO;

	let mut offset_position: Vector3;
	let mut offset_normals: Vector3;
	let mut offset_floored = VECTOR3_INT_ZERO;

	let mut noise_x_value = 0.0;
	let mut noise_y_value = 0.0;
	let mut noise_z_value = 0.0;

	let mut noise_points = [0.0, 0.0, 0.0];
	for current_segment in 0..segement_count {
		current_noise_position.x = 0.00053 + noise_head_start.x * 0.5 + (current_segment as f64 * twistiness);
		current_noise_position.y = 0.00053 + noise_head_start.y * 0.5 + (current_segment as f64 * twistiness);
		current_noise_position.z = 0.00053 + noise_head_start.z * 0.5 + (current_segment as f64 * twistiness);
		let mut current_noise_scaled = VECTOR3ZERO;

		let x_seed = seed;
		let y_seed = seed + 3098;
		let z_seed = seed + 1559;

		match cave_type {
			CaveType::Squiggly => {

				current_noise_scaled = current_noise_position * 0.13;
				noise_points[0] = current_noise_scaled.x;
				noise_points[1] = current_noise_scaled.y;
				noise_points[2] = current_noise_scaled.z;

				noise_x_value = noise_with_octaves_01(&cave_noise_gen, noise_points, 1, x_seed, 0.5);
				noise_z_value = noise_with_octaves_01(&cave_noise_gen, noise_points, 1, z_seed, 0.5);

				current_noise_scaled = current_noise_position * 0.52;

				noise_points[0] = current_noise_scaled.x;
				noise_points[1] = current_noise_scaled.y;
				noise_points[2] = current_noise_scaled.z;

				noise_y_value = noise_with_octaves_01(&cave_noise_gen, noise_points, 1, y_seed, 0.5);
			},
			CaveType::MoreVertical => {
				current_noise_scaled = current_noise_position * 0.13;
				noise_points[0] = current_noise_scaled.x;
				noise_points[1] = current_noise_scaled.y;
				noise_points[2] = current_noise_scaled.z;
				noise_y_value = noise_with_octaves_01(&cave_noise_gen, noise_points, 1, y_seed, 0.5);


				current_noise_scaled = current_noise_position * 0.052;

				noise_points[0] = current_noise_scaled.x;
				noise_points[1] = current_noise_scaled.y;
				noise_points[2] = current_noise_scaled.z;

				noise_x_value = noise_with_octaves_01(&cave_noise_gen, noise_points, 1, x_seed, 0.5);
				noise_z_value = noise_with_octaves_01(&cave_noise_gen, noise_points, 1, z_seed, 0.5);
			
			},
			CaveType::Noodle => {
				current_noise_scaled = current_noise_position * 0.3;
				noise_points[0] = current_noise_scaled.x;
				noise_points[1] = current_noise_scaled.y;
				noise_points[2] = current_noise_scaled.z;
				noise_y_value = noise_with_octaves_01(&cave_noise_gen, noise_points, 1, y_seed, 0.5);


				current_noise_scaled = current_noise_position * 0.03;

				noise_points[0] = current_noise_scaled.x;
				noise_points[1] = current_noise_scaled.y;
				noise_points[2] = current_noise_scaled.z;

				noise_x_value = noise_with_octaves_01(&cave_noise_gen, noise_points, 1, x_seed, 0.5);
				noise_z_value = noise_with_octaves_01(&cave_noise_gen, noise_points, 1, z_seed, 0.5);
			
			},
		}


		offset_position = Vector3 {
			x: f64::cos(noise_x_value * std::f64::consts::PI),	
			y: f64::cos(noise_y_value * std::f64::consts::PI),	
			z: f64::cos(noise_z_value * std::f64::consts::PI),	
		};

		offset_normals = Vector3 {
			x: 2.0 + f64::abs(f64::max(offset_position.x * thickness, segement_length * offset_position.x)),
			y: 2.0 + f64::abs(f64::max(offset_position.y * thickness, segement_length * offset_position.x)),
			z: 2.0 + f64::abs(f64::max(offset_position.z * thickness, segement_length * offset_position.x)),
		};

		let half_segment_count = segement_count as f64 * 0.5;

		if cave_type == CaveType::Noodle && f64::abs(current_segment as f64 - half_segment_count) >= 0.3 {
			offset_normals = offset_normals * 2.0;
		}

		offset_floored = Vector3Int {
			x: round_to_int(offset_position.x * segement_length),
			y: round_to_int(offset_position.y * segement_length),
			z: round_to_int(offset_position.z * segement_length),
		};

		current_segement_position = current_segement_position + offset_floored;

	}

}

fn round_to_int(val: f64) -> i64 {
	if val < 0.0 {
		return val.floor() as i64;
	}

	val.ceil() as i64
}
use crate::ConfigurationState;
use crate::common::types::*;
use crate::common::voxels::voxel_helpers;
use crate::generation::noise::*;
use noise::*;


#[allow(dead_code)]
fn interpolate (lower: f64, upper: f64, value: f64) -> f64 {
	let range = upper - lower;
	lower + value * range
}

pub fn get_height_map(coords: Vector3, config: ConfigurationState) -> Vec<Voxel> {
	let mut voxels: Vec<Voxel> = vec![0;16*16*128];
    let height_map_gen = OpenSimplex::new();
	let mut y0: f64;
	let mut _blob_noise: f64;
	let mut block_variant_noise: f64;
	let mut height_noise: f64;
	let mut height ;
	let mut height_noise_smoother;
	let mut depth_adjust_noise;
	let mut depth_adjust;
	let mut biome_noise;
	let mut ocean_noise;
	let mut plains_noise;
	let mut mountain_noise;
	let mut ocean_weight;
	let mut plains_weight;
	let mut mountain_weight;
	let mut interpolated;

	let height_seed = config.height_seed;
	let depth_adjust_seed= config.depth_adjust_seed;
	let biome_seed = config.biome_seed;

	let height_noise_freq = config.height_noise_freq;
	let height_noise_smooth_freq = config.height_noise_smooth_freq;
	let depth_adjust_noise_freq = config.depth_adjust_noise_freq;
	let biome_noise_freq = config.biome_noise_freq;
	let height_range = config.height_range;
	let min_height = config.min_height;
	
	let height_noise_octaves = config.height_noise_octaves;
	let height_noise_smooth_octaves = config.height_noise_smooth_octaves;
	let biome_noise_octaves = config.biome_noise_octaves;
	let depth_adjust_noise_octaves = config.depth_adjust_noise_octaves;

	let offset_x = coords.x * 16.0;
	let offset_z = coords.z * 16.0; 

	let mut height_noise_points = [0.0, 0.0];
	let mut height_noise_smoother_points = [0.0, 0.0];
	let mut depth_adjust_points = [0.0, 0.0];
	let mut biome_noise_points = [0.0, 0.0];

	for x in 0..16u16 {
		let x0 = x as f64 + offset_x;
		height_noise_points[0] = x0 * height_noise_freq;
		height_noise_smoother_points[0] = x0 * height_noise_smooth_freq;
		depth_adjust_points[0] = x0 * depth_adjust_noise_freq;
		biome_noise_points[0] = x0 * biome_noise_freq;
		for z in 0..16u16 {
			let z0 = z as f64 + offset_z;
			height_noise_points[1] = z0 * height_noise_freq;
			height_noise_smoother_points[1] = z0 * height_noise_smooth_freq;
			depth_adjust_points[1] = z0 * depth_adjust_noise_freq;
			biome_noise_points[1] = z0 * biome_noise_freq;

			height_noise = noise_with_octaves_vec2_01(height_map_gen,height_noise_points, height_noise_octaves, height_seed, 1.0);
			height_noise = f64::powf(height_noise, 1.2);

			height_noise_smoother = noise_with_octaves_vec2_01(height_map_gen, height_noise_smoother_points, height_noise_smooth_octaves, height_seed, 1.0);
			depth_adjust_noise = noise_with_octaves_vec2_01(height_map_gen, depth_adjust_points, depth_adjust_noise_octaves , depth_adjust_seed, 1.0);
			depth_adjust = depth_adjust_noise as i16 * 10 - 5;

			biome_noise = noise_with_octaves_vec2_01(height_map_gen, biome_noise_points, biome_noise_octaves, biome_seed, 1.0);
			//biome_noise = interpolate(0.0, 0.5, biome_noise);
			// TODO: Weight biomes on interpolation by distance from edge, so that things like Mountains and oceans
			// aren't effecting each others results very much
			
			ocean_noise =f64::powf(height_noise_smoother * 0.35, 2.0);
			plains_noise = f64::powf(height_noise_smoother * 0.3 + 0.2, 2.0) + 0.3;
			mountain_noise = f64::powf(height_noise * 0.6 + 0.5, 0.8);

			ocean_weight = if biome_noise <= 0.3 { 2.5 - f64::powf(4.0, biome_noise) } else {f64::powf(1.2 - biome_noise, 3.0) };

			plains_weight = f64::powf(if biome_noise < 0.5 { biome_noise } else { 1.2 - biome_noise } * 2.0, 2.0) * 0.8;
			mountain_weight = if biome_noise >= 0.7 { 1.2 * f64::powf(biome_noise, 2.0) } else { f64::powf(biome_noise, 2.4) };

			interpolated = (ocean_noise * ocean_weight + plains_noise * plains_weight + mountain_noise * mountain_weight) / (ocean_weight + plains_weight + mountain_weight);
			// max height based on biome?
			height = (interpolated * height_range) as i32 + min_height;

			if depth_adjust <= 2 { height += depth_adjust as i32; }

			for y in 0..128u16 {
				y0 = y as f64;

				let index = voxel_helpers::get_index(x, y, z);

				let mut voxel = index as u64;

				if y <= height as u16 { 
					voxel = voxel_helpers::set_filled(voxel);
				} else if y < 40 {
					voxel = voxel_helpers::set_filled(voxel);
					voxel = voxel_helpers::set_block_type(voxel , 4);
					voxels[index] = voxel;
					continue;
				} else {
					voxels[index] = voxel;
					continue;
				}
				
				//if y >= 90 && y <= 110 {
				//	blob_noise = noise_with_octaves_01(height_map_gen,[x0 * 0.0125, z0 * 0.0125, y0 * 0.0125], 8, 1049, 1.0);
				//	blob_noise = f64::powf(blob_noise, 1.2) * (1.0 - f64::abs((100 - y as i16) as f64 / 20.0));
				//	if blob_noise >= 0.8 { voxel = voxel_helpers::set_filled(voxel); }
				//}

				block_variant_noise = noise_with_octaves_01(height_map_gen,[x0  * 0.0125, z0 * 0.0125, y0 * 0.0125], 3, 12984, 0.7);
				block_variant_noise = f64::powf(block_variant_noise, 2.0);
				let block_type: u64 = if y < 10 {
					1 // stone
				} else if y < 50 {
					if block_variant_noise <= 0.3 {
						3
					} else {
						1
					}
				} else if y < 70 {
					if block_variant_noise <= 0.3 {
						2
					} else if block_variant_noise <= 0.7 {
						1
					} else {
						3
					}
				} else if y < 100 {
					1
				} else {
					2
				};
				voxel = voxel_helpers::set_block_type(voxel, block_type);
				voxels[index] = voxel;

			}
		}

	}

	voxels
}

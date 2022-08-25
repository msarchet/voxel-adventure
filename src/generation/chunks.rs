use crate::ConfigurationState;
use crate::common::types::*;
use crate::common::voxels::voxel_helpers;
use crate::generation::noise::*;
use noise::*;


#[allow(dead_code)]
fn interpolate (range : (f64,f64), value: f64) -> f64 {
	let (lower, upper) = range;
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
	let mut ocean_height;
	let mut plains_height;
	let mut mountain_height;
	let mut ocean_weight;
	let mut plains_weight;
	let mut mountain_weight;
	let mut interpolated;

	let height_seed = config.height_noise_configuration.seed;
	let depth_adjust_seed= config.depth_adjust_noise_configuration.seed;
	let biome_seed = config.biome_noise_configuration.seed;

	let height_noise_freq = config.height_noise_configuration.freq;
	let height_noise_smooth_freq = config.height_noise_smooth_configuration.freq;
	let depth_adjust_noise_freq = config.depth_adjust_noise_configuration.freq;
	let biome_noise_freq = config.biome_noise_configuration.freq;
	
	let height_noise_octaves = config.height_noise_configuration.octaves;
	let height_noise_smooth_octaves = config.height_noise_smooth_configuration.octaves;
	let biome_noise_octaves = config.biome_noise_configuration.octaves;
	let depth_adjust_noise_octaves = config.depth_adjust_noise_configuration.octaves;

	let biome_smoothing_delta = config.biome_smoothing;
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

			biome_noise = f64::powf(noise_with_octaves_vec2_01(height_map_gen, biome_noise_points, biome_noise_octaves, biome_seed, 1.0), 1.0);
			//biome_noise = interpolate(0.0, 0.5, biome_noise);
			// TODO: Weight biomes on interpolation by distance from edge, so that things like Mountains and oceans
			// aren't effecting each others results very much
			let (ocean_min_noise, ocean_max_noise) = config.ocean_biome_config.range;
			let (plains_min_noise, plains_max_noise) = config.plains_biome_config.range;
			let (mountain_min_noise, mountain_max_noise) = config.mountains_biome_config.range;

			let (ocean_min_height, ocean_max_height) = config.ocean_biome_config.height_range;
			let (plains_min_height, plains_max_height) = config.plains_biome_config.height_range;
			let (mountain_min_height, mountain_max_height) = config.mountains_biome_config.height_range;

			
			ocean_height = interpolate(config.ocean_biome_config.height_range, f64::powf(height_noise_smoother, 2.0)) + config.ocean_biome_config.min_height;
			plains_height = interpolate(config.plains_biome_config.height_range, f64::powf(height_noise_smoother+ 0.2, 2.0)) + config.plains_biome_config.min_height;
			mountain_height = interpolate(config.mountains_biome_config.height_range, f64::powf(0.5 + height_noise * 0.8, 1.2)) + config.mountains_biome_config.min_height;

			ocean_weight = if biome_noise <= config.ocean_biome_config.range.1 { 
				1.2 - f64::powf(4.0, biome_noise) 
			} else {
				f64::powf(1.0 - biome_noise, 3.0) 
			};

			if biome_noise > ocean_max_noise - 0.1 && biome_noise < plains_min_noise + 0.1 {
				ocean_height = interpolate((ocean_height, ocean_max_height), f64::abs(ocean_max_noise - biome_noise) * 10.0);
				plains_height = interpolate((plains_height, plains_min_height), f64::abs(biome_noise - plains_min_noise) * 10.0);
			}

			if biome_noise > config.plains_biome_config.range.1 - 0.1 && biome_noise < config.mountains_biome_config.range.0 + 0.1 {
				plains_height = interpolate((plains_height, plains_max_height), (plains_max_noise - biome_noise) * 10.0);
				mountain_height = interpolate((mountain_height, mountain_min_height), (biome_noise - mountain_min_noise) * 10.0);
			}

			plains_weight = f64::powf(if biome_noise < config.plains_biome_config.range.0 { biome_noise } else { 1.1 - biome_noise } * 2.0, 2.0) * 0.8;
			mountain_weight = if biome_noise >= config.mountains_biome_config.range.0 { 1.2 * f64::powf(biome_noise, 2.0) } else { f64::powf(biome_noise, 2.4) };

			ocean_height = ocean_height * if biome_noise >= ocean_max_noise { interpolate((1.0, 0.0), (biome_noise - ocean_max_noise) / ocean_max_noise) } else { 1.0 };
			plains_height = plains_height * if biome_noise >= plains_max_noise { interpolate((1.0, 0.0), (biome_noise - plains_max_noise) / plains_max_noise) } else { 1.0 };
			interpolated = (ocean_height + plains_height + mountain_height) / 3.0;
			// max height based on biome?
			height = interpolated as u8;

			if i16::abs(depth_adjust) <= 2 { 
				let mod_height = height as i16 + depth_adjust;
				height = (mod_height & 0xFF) as u8; 
			}

			for y in 0..128u16 {
				y0 = y as f64;

				let index = voxel_helpers::get_index(x, y, z);

				let mut voxel = index as u64;

				if biome_noise <= config.biome_range.0 || biome_noise >= config.biome_range.1 {
					voxel = voxel_helpers::set_filled(voxel, false);
					voxels[index] = voxel;
					continue;
				}

				if y <= height as u16 { 
					voxel = voxel_helpers::set_filled(voxel, true);
				} else if y < 40 {
					if config.generate_ocean_water {
						voxel = voxel_helpers::set_filled(voxel, true);
						voxel = voxel_helpers::set_block_type(voxel , 4);
					}
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
				} 
				else if config.generate_ocean_water == false && y < 40 {
					if y == 39 {
						5
					} else {
						2
					}
				}
				else if y < 50 {
					if block_variant_noise <= 0.3 {
						3
					} else {
						1
					}
				} 
				else if y < 70 {
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
					5
				};
				voxel = voxel_helpers::set_block_type(voxel, block_type);
				voxels[index] = voxel;

			}
		}

	}

	voxels
}

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

fn cubic_interpolate(points: (i16, i16, i16, i16), mu: f64) -> u16 {
	let (p0,
		p1,
		p2, 
		p3) = points;
	
	//return (p1 as f64 * (1.0 - mu) + p2 as f64 * mu) as u16;

	let a0 = (p3 - p2 - p0 + p1) as f64;
	let a1 = (p0 - p1) as f64 - a0;
	let a2 = (p2 - p0) as f64;
	let a3 = p1 as f64;

	let mu2 = mu * mu;
	(a0 * mu * mu2 + a1 * mu2 + a2 * mu + a3) as u16
}

struct InterpolationSample {
	height: i32,
	height_noise: f64,
	height_noise_smoother: f64,
	depth_adjust_noise: f64,
	biome_noise: f64,
	ocean_noise: f64,
	plains_noise: f64,
	ocean_weight: f64,
	plains_weight: f64,
	mountain_weight: f64,
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

	let step_size = 2;
	let sample_distance_offsets = [-4.0, -2.0, 0.0, 2.0, 4.0, 6.0, 8.0, 10.0,12.0, 14.0, 16.0, 18.0, 20.0];

	let mut samples = vec![];

	for x_offset in sample_distance_offsets {
		for z_offset in sample_distance_offsets {
			let x0 = offset_x + x_offset;
			let z0 = offset_z + z_offset;
			height_noise_points[0] = x0 * height_noise_freq;
			height_noise_smoother_points[0] = x0 * height_noise_smooth_freq;
			depth_adjust_points[0] = x0 * depth_adjust_noise_freq;
			biome_noise_points[0] = x0 * biome_noise_freq;
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

			let sample = InterpolationSample {
				height: height,
				height_noise: height_noise,
				height_noise_smoother: height_noise_smoother,
				depth_adjust_noise: depth_adjust_noise,
				biome_noise: biome_noise,
				ocean_noise: ocean_noise,
				plains_noise: plains_noise,
				ocean_weight: ocean_weight,
				plains_weight: plains_weight,
				mountain_weight: mountain_weight
			};
			samples.push(sample);
		}
	}

	// start a 0, 0 relative to our chunk coordinates
	// check each point keeping track of where we are in the walk
	// then to get our current point we do some form of interpolation to find the value
	// at the actual lookup points we instead use the actual value
	// Note that this interpolation is relative to teh 	

	// TODO: Interpolation types, options, doing things based on biomes, derivatives of local height maps
	let sample_x_index = 2;
	let sample_z_index = 2;

	let x = 0;
	let x_index_increase = sample_distance_offsets.len();
	
	for sample_x_increment in 0..16/step_size {
		// take two points left, and two points right
		let sample_x_offset = (sample_x_increment + sample_x_index) * x_index_increase;

		// x0, z0
		let x_up_points = (
			samples[(sample_x_offset - x_index_increase)].height as i16,
			samples[(sample_x_offset)].height as i16,
			samples[(sample_x_offset + x_index_increase)].height as i16,
			samples[(sample_x_offset + 2 * x_index_increase)].height as i16,
		);

		// x1, z0
		let x_down_points = (
			samples[(sample_x_offset)].height as i16,
			samples[(sample_x_offset + x_index_increase)].height as i16,
			samples[(sample_x_offset + 2 * x_index_increase)].height as i16,
			samples[(sample_x_offset + 3 * x_index_increase)].height as i16,
		);

		for x0 in 0..step_size  {
			let x_up_height = cubic_interpolate(x_up_points, x0 as f64 / step_size as f64);
			let x_down_height = cubic_interpolate(x_down_points, 1.0 - x0 as f64 / step_size as f64);

			for sample_z_increment in 0..16/step_size {
				//x0
				let sample_z_offset = sample_x_offset + sample_z_increment + sample_z_index;

				// x0, z0
				let z_up_points = (
				samples[sample_z_offset - 1].height as i16,
				samples[sample_z_offset].height as i16,
				samples[sample_z_offset + 1].height as i16,
				samples[sample_z_offset + 2].height as i16,
				);

				//x1, z1
				let z_down_points = (
				samples[sample_z_offset + 0 + x_index_increase].height as i16,
				samples[sample_z_offset + 1 + x_index_increase].height as i16,
				samples[sample_z_offset + 2 + x_index_increase].height as i16,
				samples[sample_z_offset + 3 + x_index_increase].height as i16,
				);

				for z0 in 0..step_size {
					let z_up_height = cubic_interpolate(z_up_points, z0 as f64 / step_size as f64);
					let z_down_height = cubic_interpolate(z_down_points, 1.0 - z0 as f64 / step_size as f64);
					let height = (x_up_height + x_down_height + z_up_height + z_down_height) / 4;
					for y in 0..128u16 {
						y0 = y as f64;

						let index = voxel_helpers::get_index((x0 + step_size * sample_x_increment) as u16, y, (z0 + step_size * sample_z_increment) as u16);

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

						block_variant_noise = noise_with_octaves_01(height_map_gen,[x0 as f64  * 0.0125, z0 as f64 * 0.0125, y0 as f64 * 0.0125], 3, 12984, 0.7);
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
		}
	}

	voxels
}

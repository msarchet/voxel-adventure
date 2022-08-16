use crate::common::types::*;
use crate::common::voxels::voxel_helpers;
use crate::generation::noise::*;
use noise::*;
use bevy::utils::HashMap;


fn interpolate (lower: f64, upper: f64, value: f64) -> f64 {
	let range = upper - lower;
	lower + value * range
}

pub fn get_height_map(coords: Vector3) -> Vec<Voxel>{
	let mut voxels: Vec<Voxel> = vec![0;16*16*128];
    let height_map_gen = OpenSimplex::new();
	let mut y0: f64;
	let mut blobNoise: f64;
	let mut blockVariantNoise: f64;
	let mut heightNoise: f64;
	let mut height ;
	let mut heightNoiseSmoother;
	let mut depthAdjustNoise;
	let mut depthAdjust;
	let mut biomeNoise;
	let mut oceanNoise;
	let mut plainsNoise;
	let mut mountainNoise;
	let mut oceanWeight;
	let mut plainsWeight;
	let mut mountainWeight;
	let mut interpolated;

	for x in 0..16u16 {
		let x0 = x as f64 + (coords.x * 16.0) as f64;
		for z in 0..16u16 {
			let z0 = z as f64 + (coords.z as f64 * 16.0);

			heightNoise = noise_with_octaves_vec2_01(height_map_gen,[x0 * 0.00825, z0 * 0.00825], 8, 3498, 1.0);
			heightNoise = f64::powf(heightNoise, 1.2);

			heightNoiseSmoother = noise_with_octaves_vec2_01(height_map_gen,[x0 * 0.00085, z0 * 0.00085],3, 123498, 1.0);
			depthAdjustNoise = noise_with_octaves_vec2_01(height_map_gen,[x0 * 0.002125, z0 * 0.002125],  6, 239048, 1.0);
			depthAdjust = depthAdjustNoise as i16 * 10 - 5;

			biomeNoise = noise_with_octaves_vec2_01(height_map_gen, [x0 * 0.0005, z0 * 0.0005], 6, 23598, 1.0);

			// TODO: Weight biomes on interpolation by distance from edge, so that things like Mountains and oceans
			// aren't effecting each others results very much
			
			oceanNoise =f64::powf(heightNoiseSmoother * 0.4, 2.0);
			plainsNoise = f64::powf(heightNoiseSmoother * 0.3 + 0.2, 2.0) + 0.3;
			mountainNoise = f64::powf(heightNoise * 0.6 + 0.5, 0.8);

			oceanWeight = if biomeNoise <= 0.3 { 2.0 - f64::powf(2.0, biomeNoise) } else {f64::powf(1.0 - biomeNoise, 3.0) };

			plainsWeight = f64::powf(if biomeNoise < 0.5 { biomeNoise } else { 1.0 - biomeNoise } * 2.0, 2.0) * 0.8;
			mountainWeight = if biomeNoise >= 0.7 { 1.2 * f64::powf(biomeNoise, 2.0) } else { f64::powf(biomeNoise, 2.4) };

			interpolated = (oceanNoise * oceanWeight + plainsNoise * plainsWeight + mountainNoise * mountainWeight) / (oceanWeight + plainsWeight + mountainWeight);
			// max height based on biome?
			height = (interpolated * 80.0) as i16 + 20;

			if depthAdjust <= 2 { height += depthAdjust; }

			for y in 0..128u16 {
				y0 = y as f64;

				let index = voxel_helpers::get_index(x, y as u16, z);

				let mut voxel = index as u64;

				if y <= height as u16 { 
					voxel = voxel_helpers::set_filled(voxel);
				}
				
				if y >= 90 && y <= 110 {
					blobNoise = noise_with_octaves_01(height_map_gen,[x0 * 0.0125, z0 * 0.0125, y0 * 0.0125], 8, 1049, 1.0);
					blobNoise = f64::powf(blobNoise, 1.2) + (1.0 - f64::abs((110 - y) as f64 / 20.0));
					if blobNoise >= 0.6 { voxel = voxel_helpers::set_filled(voxel); }
				}

				blockVariantNoise = noise_with_octaves_01(height_map_gen,[x0  * 0.0125, z0 * 0.0125, y0 * 0.0125], 12, 12984, 1.0);
				blockVariantNoise = f64::powf(blockVariantNoise, 2.0);
				voxels[index] = voxel;
			}
		}

	}

	voxels
}

use noise::*;

pub fn noise_with_octaves(gen: impl NoiseFn<[f64;3]>, point: [f64;3], octaves: u8, seed: u32, persist: f64) -> f64 {
	let mut noise = 0.0;
	let mut freq_sum = 0.0;
	let mut freq;
	let mut weight;
	let mut persistence = 1.0;

	for i in 0..octaves
	{
		freq = (1 << i) as f64;
		weight = 1.0 / freq;

		let temp_noise = gen.get([freq * point[0] + seed as f64, freq * point[1] + seed as f64, freq * point[2] + seed as f64]);
		noise += persistence * weight * (temp_noise + 1.0) * 0.5;
		persistence *= persist;
		freq_sum += weight;
	}

	return noise / freq_sum;	
}

pub fn noise_with_octaves_01(gen: impl NoiseFn<[f64;3]>, point: [f64;3], octaves: u8, seed: u32, persist: f64) -> f64 {
	noise_with_octaves(gen, point, octaves, seed, persist) + 1.0 * 0.5
}


pub fn noise_with_octaves_vec2_01(gen: impl NoiseFn<[f64;2]>, point: [f64;2], octaves: u8, seed: u32, persist: f64) -> f64 {
	noise_with_octaves_vec2(gen, point, octaves, seed, persist) + 1.0 * 0.5
}

pub fn noise_with_octaves_vec2(gen: impl NoiseFn<[f64;2]>, point: [f64;2], octaves: u8, seed: u32, persist: f64) -> f64 {
	let mut noise = 0.0;
	let mut freq_sum = 0.0;
	let mut freq;
	let mut weight;
	let mut persistence = 1.0;

	for i in 0..octaves
	{
		freq = (1 << i) as f64;
		weight = 1.0 / freq;

		let temp_noise = gen.get([freq * point[0] + seed as f64, freq * point[1] + seed as f64]);
		noise += persistence * weight * (temp_noise + 1.0) * 0.5;
		persistence *= persist;
		freq_sum += weight;
	}

	return noise / freq_sum;	
}

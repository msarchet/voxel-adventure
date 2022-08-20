use noise::*;

fn normalize(value: f64) -> f64 { value + 0.5 }

pub fn noise_with_octaves(gen: impl NoiseFn<[f64;3]>, point: [f64;3], octaves: u8, seed: i32, persist: f64) -> f64 {
	let mut noise = 0.0;
	let mut freq_sum = 0.0;
	let mut freq;
	let mut weight;
	let mut persistence = 1.0;
	let mut points = [0.0, 0.0, 0.0];
	for i in 0..octaves
	{
		freq = (1 << i) as f64;
		weight = 1.0 / freq;

		points[0] = freq * point[0] + seed as f64;
		points[1] = freq * point[1] + seed as f64;
		points[2] = freq * point[2] + seed as f64;

		let temp_noise = gen.get(points);
		noise += persistence * weight * temp_noise;
		persistence *= persist;
		freq_sum += weight;
	}

	return noise / freq_sum;	
}

pub fn noise_with_octaves_vec2(gen: impl NoiseFn<[f64;2]>, point: [f64;2], octaves: u8, seed: i32, persist: f64) -> f64 {
	let mut noise = 0.0;
	let mut freq_sum = 0.0;
	let mut freq;
	let mut weight;
	let mut persistence = 1.0;

	let mut points : [f64;2] = [0.0,0.0];
	for i in 0..octaves
	{
		freq = (1 << i) as f64;
		weight = 1.0 / freq;

		points[0] = freq * point[0] + seed as f64;
		points[1] = freq * point[1] + seed as f64;
		let temp_noise = gen.get(points);
		noise += persistence * weight * temp_noise;
		persistence *= persist;
		freq_sum += weight;
	}

	return noise / freq_sum;	
}

pub fn noise_with_octaves_01(gen: impl NoiseFn<[f64;3]>, point: [f64;3], octaves: u8, seed: i32, persist: f64) -> f64 {
	let mut noise = 0.0;
	let mut freq_sum = 0.0;
	let mut freq;
	let mut weight;
	let mut persistence = 1.0;
	let mut points = [0.0, 0.0, 0.0];
	for i in 0..octaves
	{
		freq = (1 << i) as f64;
		weight = 1.0 / freq;

		points[0] = freq * point[0] + seed as f64;
		points[1] = freq * point[1] + seed as f64;
		points[2] = freq * point[2] + seed as f64;

		let temp_noise = gen.get(points);
		noise += persistence * weight * normalize(temp_noise);
		persistence *= persist;
		freq_sum += weight;
	}

	return noise / freq_sum;	
}


pub fn noise_with_octaves_vec2_01(gen: impl NoiseFn<[f64;2]>, point: [f64;2], octaves: u8, seed: i32, persist: f64) -> f64 {
	let mut noise = 0.0;
	let mut freq_sum = 0.0;
	let mut freq;
	let mut weight;
	let mut persistence = 1.0;

	let mut points : [f64;2] = [0.0,0.0];
	for i in 0..octaves
	{
		freq = (1 << i) as f64;
		weight = 1.0 / freq;

		points[0] = freq * point[0] + seed as f64;
		points[1] = freq * point[1] + seed as f64;
		let temp_noise = gen.get(points);
		noise += persistence * weight * normalize(temp_noise);
		persistence *= persist;
		freq_sum += weight;
	}

	return noise / freq_sum;	
}

use crate::common::types::*;
use crate::common::voxels::voxel_helpers;
use crate::meshing::cubemeshes::*;
use bevy::utils::HashMap;

pub fn get_mesh_data(chunk_data: &ChunkData, cube_data: &CubeMeshData) -> MeshData {
	let vertex_count = chunk_data.voxels.len() * 12; // voxels.len() / 2 * 24 (maximum visible triangles)
	let mut mesh_data = get_mesh_data_container(vertex_count);

	let mut voxel;
	let mut mesh_key;
	let mut faces_res;
	let mut coords;
	let mut vertex_index;
	let mut adjusted: [f32;3] = [0.0, 0.0, 0.0];
	for i in 0..chunk_data.voxels.len() {
		voxel = chunk_data.voxels[i];
		if !voxel_helpers::is_filled(voxel) { continue; }
		mesh_key = voxel_helpers::get_mesh_data(voxel);
		faces_res = cube_data.cubes.get(&mesh_key);
		match faces_res {
			Some(faces) => {
				// TODO: Improve perf here
				coords = voxel_helpers::get_coords_vec3(voxel);
				vertex_index = mesh_data.verticies.len() as u32;

				for i in 0..faces.vertex_count as usize {
					// adjust verticies
					adjusted[0] = faces.verticies[i][0] + coords.x as f32;
					adjusted[1] = faces.verticies[i][1] + coords.y as f32;
					adjusted[2] = faces.verticies[i][2] + coords.z as f32;
					mesh_data.verticies.push(adjusted);
				}

				for i in 0..faces.uvs.len() { mesh_data.uvs.push(faces.uvs[i]); }
				for i in 0..faces.normals.len() { mesh_data.normals.push(faces.normals[i]); }
				for i in 0..faces.indicies.len() { mesh_data.indicies.push(faces.indicies[i] + vertex_index); }
			},
			None => panic!("invalid mesh face {}", mesh_key),
		}
	}

	return mesh_data;
}

pub fn run_first_pass_meshing(voxels: &mut Vec<Voxel>) {
	for index in 0..voxels.len() {
			let voxel = voxels[index];

            let coords = voxel_helpers::get_coords(voxel);
			let x = coords.x;
			let y = coords.y;
			let z = coords.z;

            if !voxel_helpers::is_filled(voxel)
            {
				continue;
            }


            // since we are filled we only create the faces that we need
            let mut key = 0u64;
            let mut target_index;
			let mut target_voxel;

            // don't mesh the top of the world
            if y < 127
            {
                target_index = voxel_helpers::get_index(x, y + 1, z);
				target_voxel = voxels[target_index];
				if voxel_helpers::should_create_face(voxel, target_voxel) { key |= 0b1; }
            }
			else {
				//key |= 0b1;
			}

            // don't mesh the bottom face of the world
            if y > 1
            {
                target_index = voxel_helpers::get_index(x, y - 1, z);
				target_voxel = voxels[target_index];
				if voxel_helpers::should_create_face(voxel, target_voxel) { key |= 0b10; }
            }
			else {
				//key |= 0b10;
			}
            if z < 15
            {
				target_index = voxel_helpers::get_index(x, y, z + 1);
				target_voxel = voxels[target_index];
				if voxel_helpers::should_create_face(voxel, target_voxel) { key |= 0b100; }
 
            } else {
				//key |= 0b100;
			}
            if z > 0
            {
                target_index = voxel_helpers::get_index(x, y, z - 1);
				target_voxel = voxels[target_index];
				if voxel_helpers::should_create_face(voxel, target_voxel) { key |= 0b1000; }
            }
			else {
				//key |= 0b1000; 
			}
            if x < 15
            {
                target_index = voxel_helpers::get_index(x + 1, y, z);
				target_voxel = voxels[target_index];
				if voxel_helpers::should_create_face(voxel, target_voxel) { key |= 0b10000; }
            }
			else {
				//key |= 0b10000;
			}

            if x > 0
            {
                target_index = voxel_helpers::get_index(x - 1, y, z);
				target_voxel = voxels[target_index];
				if voxel_helpers::should_create_face(voxel, target_voxel) { key |= 0b100000; }
            }
			else {
				//key |= 0b100000;
			}

            voxels[index] = voxel_helpers::set_mesh_data(voxel, key & 0xFF);
	}
}
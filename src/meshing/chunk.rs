use crate::common::types::*;
use crate::common::voxels::voxel_helpers;
use crate::meshing::cubemeshes::*;

use bevy::prelude::{Component, FromWorld };

pub type UVArray = [[f32;2];4];

pub enum BlockType {
	Water,
	Stone,
	Grass,
	Dirt,
	Snow,
	Sand,
	Ice,
	DarkStone,
}

impl TryFrom<u64> for BlockType {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
		  match value {
			v if v == BlockType::Water as u64 => Ok(BlockType::Water),
			v if v == BlockType::Stone as u64 => Ok(BlockType::Stone),
			v if v == BlockType::Grass as u64 => Ok(BlockType::Grass),
			v if v == BlockType::Dirt as u64 => Ok(BlockType::Dirt),
			v if v == BlockType::Snow as u64 => Ok(BlockType::Snow),
			v if v == BlockType::Sand as u64 => Ok(BlockType::Sand),
			v if v == BlockType::Ice as u64 => Ok(BlockType::Ice),
			v if v == BlockType::DarkStone as u64 => Ok(BlockType::DarkStone),
			_ => Err(())
		  }
    }
}
pub fn get_uvs_for_block(uvs: & mut UVArray, block_type: BlockType) {
	let grid_size = [4.0, 4.0];
	match block_type {
		BlockType::Grass		=> get_uvs(uvs, [0.0, 0.0], grid_size), 
		BlockType::Snow			=> get_uvs(uvs, [0.0, 1.0], grid_size), 
		BlockType::Sand			=> get_uvs(uvs, [0.0, 2.0], grid_size),
		BlockType::Water		=> get_uvs(uvs, [0.0, 3.0], grid_size),
		BlockType::Dirt			=> get_uvs(uvs, [1.0, 0.0], grid_size), 
		BlockType::Stone		=> get_uvs(uvs, [1.0, 1.0], grid_size),
		BlockType::Ice			=> get_uvs(uvs, [1.0, 2.0], grid_size), 
		BlockType::DarkStone	=> get_uvs(uvs, [1.0, 3.0], grid_size),
	}
}

fn get_uvs(uvs: & mut UVArray, coords: [f32;2], grid_size: [f32;2]) {
	let tiny = 0.05;
	let x_scale =  1.0 / grid_size[0];
	let y_scale = 1.0 / grid_size[1];

	uvs[0] = [coords[0] * x_scale + tiny, coords[1] * y_scale + tiny];
	uvs[1] = [(coords[0] + 1.0) * x_scale - tiny, coords[1] * y_scale + tiny];
	uvs[2] = [(coords[0]) * x_scale + tiny, (coords[1] + 1.0) * y_scale - tiny];
	uvs[3] = [(coords[0] + 1.0) * x_scale - tiny, (coords[1] + 1.0) * y_scale - tiny];
}


#[derive(Component)]
pub struct VoxelFaceEdges {
	pub edges: [Vec<(usize, usize)>;4],
}

#[allow(unused_variables)]
impl FromWorld for VoxelFaceEdges {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
		  VoxelFaceEdges {
		  	edges: get_voxel_edges()
		  }
    }
}

fn get_voxel_edges() -> [Vec<(usize, usize)>;4] {
	let mut voxels = [vec![], vec![], vec![], vec![]];
	let max_x = 15;
	let max_z = 15;

	// get indicies for both sies of the Z seam
	for y in 0..128 {
		for x in 0..16 {
			// positions are relative to the seam
			let left_index = voxel_helpers::get_index(x, y, max_z);
			let right_index = voxel_helpers::get_index(x, y, 0);
			voxels[0].push((left_index, right_index)); // left +z 
			voxels[1].push((right_index, left_index)); // right -z
		}
	}

	// get indicies for boths sides of the X seam
	for y in 0..128 {
		for z in 0..16 {
			let forward_index = voxel_helpers::get_index(0, y, z);
			// ---- seam +x ^
			let backward_index = voxel_helpers::get_index(max_x, y, z);
			voxels[2].push((backward_index, forward_index)); // forward +x
			voxels[3].push((forward_index, backward_index)); // backward -x
		}
	}

	voxels
}

pub fn get_mesh_data(chunk_data: &ChunkData, cube_data: &CubeMeshData) -> MeshData {
	let mut mesh_data = get_mesh_data_container();

	let mut voxel;
	let mut mesh_key;
	let mut faces_res;
	let mut coords;
	let mut vertex_index;
	let mut adjusted: [f32;3] = [0.0, 0.0, 0.0];
	let mut uvs :UVArray = [[0.0, 0.0];4];
	let mut block_type;
	for i in 0..chunk_data.voxels.len() {
		voxel = chunk_data.voxels[i];
		if !voxel_helpers::is_filled(voxel) { continue; }
		mesh_key = voxel_helpers::get_mesh_data(voxel);
		block_type = voxel_helpers::get_block_type(voxel);
		get_uvs_for_block(&mut uvs, block_type.try_into().unwrap());

		faces_res = cube_data.cubes.get(&mesh_key);
		match faces_res {
			Some(faces) => {
				// TODO: Improve perf here
				coords = voxel_helpers::get_coords_as_vec3(voxel);
				vertex_index = mesh_data.verticies.len() as u32;

				for i in 0..faces.vertex_count as usize {
					// adjust verticies
					adjusted[0] = faces.verticies[i][0] + coords.x as f32;
					adjusted[1] = faces.verticies[i][1] + coords.y as f32;
					adjusted[2] = faces.verticies[i][2] + coords.z as f32;
					mesh_data.uvs.push(uvs[i % uvs.len()]); 
					mesh_data.verticies.push(adjusted);
				}

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

            let coords = voxel_helpers::get_coords_as_voxel_coords(voxel);
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

            // don't mesh the bottom face of the world
            if y > 1
            {
                target_index = voxel_helpers::get_index(x, y - 1, z);
				target_voxel = voxels[target_index];
				if voxel_helpers::should_create_face(voxel, target_voxel) { key |= 0b10; }
            }
            if z < 15
            {
				target_index = voxel_helpers::get_index(x, y, z + 1);
				target_voxel = voxels[target_index];
				if voxel_helpers::should_create_face(voxel, target_voxel) { key |= 0b100; }
 
            }            

			if z > 0
            {
                target_index = voxel_helpers::get_index(x, y, z - 1);
				target_voxel = voxels[target_index];
				if voxel_helpers::should_create_face(voxel, target_voxel) { key |= 0b1000; }
            }

            if x < 15
            {
                target_index = voxel_helpers::get_index(x + 1, y, z);
				target_voxel = voxels[target_index];
				if voxel_helpers::should_create_face(voxel, target_voxel) { key |= 0b10000; }
            }

            if x > 0
            {
                target_index = voxel_helpers::get_index(x - 1, y, z);
				target_voxel = voxels[target_index];
				if voxel_helpers::should_create_face(voxel, target_voxel) { key |= 0b100000; }
            }

            voxels[index] = voxel_helpers::set_mesh_data(voxel, key & 0xFF);
	}
}



pub fn update_edge_meshes(our_voxels: &mut Vec<Voxel>,
	their_voxels: &Vec<Voxel>,
	edge_lookup_pairs: &Vec<(usize, usize)>,
	our_face: u64,
	not_our_face: u64,
) {
	for (ours_index, their_index) in edge_lookup_pairs {
		let ours = our_voxels[ours_index.clone()];
		let theirs = their_voxels[their_index.clone()];

		if !voxel_helpers::is_filled(ours) { continue }

		let our_mesh_data = voxel_helpers::get_mesh_data(ours);

		if voxel_helpers::should_create_face(ours, theirs) {
			our_voxels[ours_index.clone()] = voxel_helpers::set_mesh_data(ours, our_mesh_data | our_face);
		} else {
			our_voxels[ours_index.clone()] = voxel_helpers::set_mesh_data(ours, our_mesh_data & not_our_face);
		}
	}	
}
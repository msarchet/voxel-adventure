 use crate::common::types::*;
use bevy::{prelude::*, utils::HashMap};

pub const MAX_VERTICIES: usize = (16 * 16 * 128) / 2 * 24;

pub struct MeshData {
	pub verticies: Vec<[f32;3]>,
	pub normals: Vec<[f32;3]>,
	pub uvs: Vec<[f32;2]>,
	pub indicies: Vec<u32>,
	pub vertex_count: u32,
}

pub fn get_mesh_data_container() -> MeshData {
	return MeshData{
		verticies: vec![],
		normals: vec![],
		uvs: vec![],
		indicies: vec![],
		vertex_count: 0,
	};
}

pub struct CubeMeshData {
	pub cubes: HashMap<u8, MeshData>,
}

const UP_INDEX: u8 = 0;
const DOWN_INDEX: u8 = 1;
const LEFT_INDEX: u8 = 2;
const RIGHT_INDEX: u8 = 3;
const FORWARD_INDEX: u8 = 4;
const BACKWARD_INDEX: u8 = 5;

static A: [f32;3] = [ 0.0,  0.0, 0.0 ];
static B: [f32;3] = [ 0.0,  0.0, 1.0 ];
static C: [f32;3] = [ 1.0,  0.0, 1.0 ];
static D: [f32;3] = [ 1.0,  0.0, 0.0 ];
static E: [f32;3] = [ 0.0,  1.0, 0.0 ];
static F: [f32;3] = [ 0.0,  1.0, 1.0 ];
static G: [f32;3] = [ 1.0,  1.0, 1.0 ];
static H: [f32;3] = [ 1.0,  1.0, 0.0 ];

//static Down : [u32;4]= [0, 3, 1, 2 ];
//static Up : [u32;4] =[4, 5, 7, 6 ];
//static Left : [u32;4] = [1, 2, 5, 6 ];
//static Right : [u32;4] = [0, 4, 3, 7 ];
//static Forward : [u32;4] = [3, 7, 2, 6 ];
//static Backward : [u32;4] = [0, 1, 4, 5 ];

fn get_mesh_for_face(face_key: u8) -> MeshData {
	let mut vertex_count  = 0;
	let mut verticies = Vec::<[f32;3]>::new();
	let mut normals = Vec::<[f32;3]>::new();
	let mut indicies = Vec::<u32>::new();

	if ((face_key >> UP_INDEX) & 0b1) == 1 {
		verticies.push(E);
		verticies.push(F);
		verticies.push(H);
		verticies.push(G);

		normals.push([VECTOR3UP.x as f32, VECTOR3UP.y as f32, VECTOR3UP.z as f32]);

		indicies.push(0 + vertex_count);
		indicies.push(1 + vertex_count);
		indicies.push(2 + vertex_count);
		indicies.push(2 + vertex_count);
		indicies.push(1 + vertex_count);
		indicies.push(3 + vertex_count);

		vertex_count += 4;
	}
	if ((face_key >> DOWN_INDEX) & 0b1) == 1 {
		verticies.push(A);
		verticies.push(D);
		verticies.push(B);
		verticies.push(C);
		normals.push([VECTOR3DOWN.x as f32, VECTOR3DOWN.y as f32, VECTOR3DOWN.z as f32]);

		indicies.push(0 + vertex_count);
		indicies.push(1 + vertex_count);
		indicies.push(2 + vertex_count);
		indicies.push(2 + vertex_count);
		indicies.push(1 + vertex_count);
		indicies.push(3 + vertex_count);

		vertex_count += 4;
	}
	if ((face_key >> LEFT_INDEX) & 0b1) == 1 {
		verticies.push(B);
		verticies.push(C);
		verticies.push(F);
		verticies.push(G);
		normals.push([VECTOR3LEFT.x as f32, VECTOR3LEFT.y as f32, VECTOR3LEFT.z as f32]);

		indicies.push(0 + vertex_count);
		indicies.push(1 + vertex_count);
		indicies.push(2 + vertex_count);
		indicies.push(2 + vertex_count);
		indicies.push(1 + vertex_count);
		indicies.push(3 + vertex_count);

		vertex_count += 4;
	}
	if ((face_key >> RIGHT_INDEX) & 0b1) == 1 {
		verticies.push(A);
		verticies.push(E);
		verticies.push(D);
		verticies.push(H);
		normals.push([VECTOR3RIGHT.x as f32, VECTOR3RIGHT.y as f32, VECTOR3RIGHT.z as f32]);

		indicies.push(0 + vertex_count);
		indicies.push(1 + vertex_count);
		indicies.push(2 + vertex_count);
		indicies.push(2 + vertex_count);
		indicies.push(1 + vertex_count);
		indicies.push(3 + vertex_count);

		vertex_count += 4;
	}

	if ((face_key >> FORWARD_INDEX) & 0b1) == 1 {
		verticies.push(D);
		verticies.push(H);
		verticies.push(C);
		verticies.push(G);
		
		normals.push([VECTOR3FORWARD.x as f32, VECTOR3FORWARD.y as f32, VECTOR3FORWARD.z as f32]);

		indicies.push(0 + vertex_count);
		indicies.push(1 + vertex_count);
		indicies.push(2 + vertex_count);
		indicies.push(2 + vertex_count);
		indicies.push(1 + vertex_count);
		indicies.push(3 + vertex_count);

		vertex_count += 4;
	}

	if ((face_key >> BACKWARD_INDEX) & 0b1) == 1 {
		verticies.push(A);
		verticies.push(B);
		verticies.push(E);
		verticies.push(F);

		normals.push([VECTOR3BACKWARD.x as f32, VECTOR3BACKWARD.y as f32, VECTOR3BACKWARD.z as f32]);

		indicies.push(0 + vertex_count);
		indicies.push(1 + vertex_count);
		indicies.push(2 + vertex_count);
		indicies.push(2 + vertex_count);
		indicies.push(1 + vertex_count);
		indicies.push(3 + vertex_count);
		vertex_count += 4;
	}


	let mut mesh_data = get_mesh_data_container();

	for i in 0..verticies.len() as usize {
		mesh_data.verticies.push(verticies[i]);
	}

	for i in 0..normals.len() {
		mesh_data.normals.push(normals[i]);
		mesh_data.normals.push(normals[i]);
		mesh_data.normals.push(normals[i]);
		mesh_data.normals.push(normals[i]);
	}

	for i in 0..indicies.len() {
		mesh_data.indicies.push(indicies[i]);
	}

	mesh_data.vertex_count = vertex_count;

	mesh_data
}

impl FromWorld for CubeMeshData {
	fn from_world(_world: &mut World) -> Self {
		let mut cubes = HashMap::<u8, MeshData>::new();
		for up in 0..2 {
			for down in 0..2 {
				for left in 0..2 {
					for right in 0..2 {
						for forward in 0..2 {
							for backward in 0..2 {
								let mut key = 0u8;
								key |= up << UP_INDEX; 
								key |= down << DOWN_INDEX; 
								key |= left << LEFT_INDEX; 
								key |= right << RIGHT_INDEX; 
								key |= forward << FORWARD_INDEX; 
								key |= backward << BACKWARD_INDEX; 

								let mesh = get_mesh_for_face(key);
								cubes.insert(key, mesh);
							}
						}
					}
				}
			}
		}
		
		CubeMeshData {
			cubes: cubes
		}
	}

}
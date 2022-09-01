pub mod voxel_helpers {
	const COORDS_OFFSET: u64 = 0u64;
	const COORDS_LENGTH: u64 = 16u64;
	const COORDS_MASK: u64 = !0xFFFFu64;
	const IS_FILLED_OFFSET: u64 = COORDS_OFFSET + COORDS_LENGTH;
	const IS_FILLED_LENGTH: u64  = 1u64;
	const IS_FILLED_MASK:u64 = !(1u64 << IS_FILLED_OFFSET);
	const MESH_DATA_OFFSET: u64 = IS_FILLED_OFFSET + IS_FILLED_LENGTH;
	const MESH_DATA_LENGTH: u64 = 6u64;
	const MESH_DATA_MASK: u64 = !(0b111111 << MESH_DATA_OFFSET);
    const BLOCK_TYPE_OFFSET: u64 = MESH_DATA_OFFSET + MESH_DATA_LENGTH;
    const BLOCK_TYPE_MASK: u64 = 0xFFFF;
    const BLOCK_TYPE_CLEAR_MASK: u64 = !(BLOCK_TYPE_MASK << BLOCK_TYPE_OFFSET);

    #[allow(dead_code)]
    const BLOCK_TYPE_LENGTH: u64 = 16u64;

    use crate::{common::types::*};

    // TODO: Convert these into proper into/from methods
    pub fn get_coords_as_voxel_coords(v: Voxel) -> VoxelCoords {
        VoxelCoords {
            x: ((v >> COORDINATE_SHIFTS.0) as u16 & COORDINATE_MASKS.0) as u16,
            y: ((v >> COORDINATE_SHIFTS.1) as u16 & COORDINATE_MASKS.1) as u16,
            z: ((v >> COORDINATE_SHIFTS.2) as u16 & COORDINATE_MASKS.2) as u16,
        }
    }

    pub fn get_coords_as_vec3(v: Voxel) -> Vector3 {
        Vector3 {
            x: ((v >> COORDINATE_SHIFTS.0) as u16 & COORDINATE_MASKS.0) as f64,
            y: ((v >> COORDINATE_SHIFTS.1) as u16 & COORDINATE_MASKS.1) as f64,
            z: ((v >> COORDINATE_SHIFTS.2) as u16 & COORDINATE_MASKS.2) as f64,
        }
    }

    pub fn get_index_from_coords(c: VoxelCoords) -> usize {
        get_index(c.x, c.y, c.z)
    }

    pub fn get_index(x: u16, y: u16, z: u16) -> usize {
        usize::from((x & 0xf) | ((z & 0xF) << 4) | ((y & 0xFF) << 8))
    }

    pub fn set_coords(v: Voxel, coords: u64) -> Voxel { (v & COORDS_MASK) | coords << COORDS_OFFSET }
    pub fn get_coords(v: Voxel) -> u64 { (v >> COORDS_OFFSET) & 0xFFFFFF }

    pub fn is_filled (v: Voxel) -> bool { ((v >> IS_FILLED_OFFSET) & 0b1) == 1 }
    pub fn set_filled (v: Voxel, filled: bool) -> Voxel { (v & IS_FILLED_MASK) | ((filled as u64) << IS_FILLED_OFFSET) }

	pub fn set_mesh_data (v: Voxel, mesh_data: u64) -> Voxel { (v & MESH_DATA_MASK) | (mesh_data << MESH_DATA_OFFSET)}
	pub fn get_mesh_data (v: Voxel) -> u64 { (v >> MESH_DATA_OFFSET) & 0b111111 	}

	pub fn should_create_face (a: Voxel, b: Voxel) -> bool { is_filled(a) != is_filled(b) }

    pub fn get_block_type(v: Voxel) -> u64 { (v >> BLOCK_TYPE_OFFSET) & BLOCK_TYPE_MASK } 
    pub fn set_block_type(v: Voxel, block_type : BlockType) -> Voxel { (v & BLOCK_TYPE_CLEAR_MASK) | ((block_type as u64) << BLOCK_TYPE_OFFSET)}
}

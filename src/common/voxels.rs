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

    use crate::common::types::*;

    pub fn get_coords(v: Voxel) -> VoxelCoords {
        return VoxelCoords {
            x: (v & 0b1111) as u16,
            y: ((v >> 8) & 0xFF) as u16,
            z: ((v >> 4) & 0b1111) as u16,
        };
    }
    pub fn get_coords_vec3(v: Voxel) -> Vector3 {
        return Vector3 {
            x: (v & 0b1111) as f64,
            y: ((v >> 8) & 0xFF) as f64,
            z: ((v >> 4) & 0b1111) as f64,
        };
    }

    pub fn get_index_from_coords(c: VoxelCoords) -> usize {
        return get_index(c.x, c.y, c.z);
    }

    pub fn get_index(x: u16, y: u16, z: u16) -> usize {
        return usize::from((x & 0xf) | ((z & 0xF) << 4) | ((y & 0xFF) << 8));
    }

    pub fn is_filled (v: Voxel) -> bool { return (v >> IS_FILLED_OFFSET & 0b1) == 1; }
    pub fn set_filled (v: Voxel) -> Voxel { return (v & IS_FILLED_MASK) | (1 << IS_FILLED_OFFSET); }

	pub fn set_mesh_data (v: Voxel, mesh_data: u64) -> Voxel { return (v & MESH_DATA_MASK) | (mesh_data << MESH_DATA_OFFSET); }
	pub fn get_mesh_data (v: Voxel) -> u8 { return ((v >> MESH_DATA_OFFSET) & 0b111111) as u8; 	}

	pub fn should_create_face (a: Voxel, b: Voxel) -> bool { return is_filled(a) != is_filled(b); }
}

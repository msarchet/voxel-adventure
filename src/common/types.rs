use std::ops::{Add, Sub};

use bevy::{prelude::Entity, utils::HashMap};

use super::voxels::voxel_helpers;

pub type Voxel = u64;
pub type VoxelCollection = Vec<Voxel>;

pub const CHUNK_DIMENSIONS : Vector3Int = Vector3Int {x: 16, y: 128, z:16};

pub const COORDINATE_SHIFTS: (u16, u16, u16) = (0, 8, 4);

pub const COORDINATE_MASKS: (u16, u16, u16) = (0xF, 0xFF, 0xF);

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default, Debug)]
pub struct Vector3Int {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}


#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct VoxelCoords {
	pub x: u16,
	pub y: u16,
	pub z: u16
}

impl TryFrom<VoxelCoords> for usize {
    type Error = ();

    fn try_from(value: VoxelCoords) -> Result<Self, Self::Error> {
        Ok(voxel_helpers::get_index_from_coords(value))
    }
}

#[derive(Copy, Clone, Default)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64
}

impl Add for Vector3Int {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z
        }
    }
}

impl Sub for Vector3Int {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z
        }
    }
}

impl Add for Vector3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z
        }
    }
}

impl Sub for Vector3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z
        }
    }
}





pub static VECTOR3ZERO: Vector3 = Vector3 {x: 0.0, y: 0.0, z:0.0};
pub static VECTOR3UP: Vector3 = Vector3 {x: 0.0, y: 1.0, z:0.0};
pub static VECTOR3DOWN: Vector3 = Vector3 {x: 0.0, y: -1.0, z:0.0};
pub static VECTOR3LEFT: Vector3 = Vector3 {x: 0.0, y: 0.0, z:1.0};
pub static VECTOR3RIGHT: Vector3 = Vector3 {x: 0.0, y: 0.0, z:-1.0};
pub static VECTOR3FORWARD: Vector3 = Vector3 {x: 1.0, y: 0.0, z:0.0};
pub static VECTOR3BACKWARD: Vector3 = Vector3 {x: -1.0, y: 0.0, z:0.0};

pub static VECTOR3_INT_ZERO: Vector3Int = Vector3Int {x: 0, y: 0, z:0};
pub static VECTOR3_INT_UP: Vector3Int = Vector3Int {x: 0, y: 1, z:0};
pub static VECTOR3_INT_DOWN: Vector3Int = Vector3Int {x: 0, y: -1, z:0};
pub static VECTOR3_INT_LEFT: Vector3Int = Vector3Int {x: 0, y: 0, z:1};
pub static VECTOR3_INT_RIGHT: Vector3Int = Vector3Int {x: 0, y: 0, z:-1};
pub static VECTOR3_INT_FORWARD: Vector3Int = Vector3Int {x: 1, y: 0, z:0};
pub static VECTOR3_INT_BACKWARD: Vector3Int = Vector3Int {x: -1, y: 0, z:0};


pub const UP_FACE:              u64 = 0b1;
pub const DOWN_FACE:            u64 = 0b10;
pub const LEFT_FACE:            u64 = 0b100;
pub const RIGHT_FACE:           u64 = 0b1000;
pub const FORWARD_FACE:         u64 = 0b10000;
pub const BACKWARD_FACE:        u64 = 0b100000;

pub const NOT_UP_FACE:          u64 =  !UP_FACE;
pub const NOT_DOWN_FACE:        u64 =  !DOWN_FACE;
pub const NOT_LEFT_FACE:        u64 =  !LEFT_FACE;
pub const NOT_RIGHT_FACE:       u64 =  !RIGHT_FACE;
pub const NOT_FORWARD_FACE:     u64 =  !FORWARD_FACE;
pub const NOT_BACKWARD_FACE:    u64 =  !BACKWARD_FACE;

#[derive(Copy, Clone, Eq, PartialEq)]
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

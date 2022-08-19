use std::ops::{Add, Sub};

pub type Voxel = u64;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Vector3Int {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct VoxelCoords {
	pub x: u16,
	pub y: u16,
	pub z: u16
}

#[derive(Copy, Clone)]
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



pub struct ChunkData {
    pub voxels: Vec<Voxel>,
}


pub static VECTOR3ZERO: Vector3 = Vector3 {x: 0.0, y: 0.0, z:0.0};
pub static VECTOR3UP: Vector3 = Vector3 {x: 0.0, y: 1.0, z:0.0};
pub static VECTOR3DOWN: Vector3 = Vector3 {x: 0.0, y: -1.0, z:0.0};
pub static VECTOR3LEFT: Vector3 = Vector3 {x: 0.0, y: 0.0, z:1.0};
pub static VECTOR3RIGHT: Vector3 = Vector3 {x: 0.0, y: 0.0, z:-1.0};
pub static VECTOR3FORWARD: Vector3 = Vector3 {x: 1.0, y: 0.0, z:0.0};
pub static VECTOR3BACKWARD: Vector3 = Vector3 {x: -1.0, y: 0.0, z:0.0};

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


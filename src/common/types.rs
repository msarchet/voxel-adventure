use std::ops::{Add, Sub};

pub type Voxel = u64;

#[derive(Copy, Clone)]
pub struct Vector3Int {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

#[derive(Copy, Clone)]
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

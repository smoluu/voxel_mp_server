// src/chunk.rs

use serde::{Deserialize, Serialize};

// Represents a single voxel in the chunk
#[derive(Serialize, Deserialize, Clone)]
pub struct Voxel {
    pub index: u32, // Single index representing the voxel's position in 3D space
    pub id: u8,     // ID representing the type of voxel (e.g., dirt, air)
}

impl Voxel {
    // Constructor for creating a new Voxel
    pub fn new(index: u32, id: u8) -> Self {
        Voxel { index, id }
    }
}

// Represents a chunk of voxels
#[derive(Serialize, Deserialize, Clone)]
pub struct Chunk {
    pub coords: (i32, i32),
    pub voxels: Vec<Voxel>, // List of voxels in the chunk
}

pub static CHUNK_SIZE: usize = 64;
pub static CHUNK_HEIGHT: usize = 256;

impl Chunk {
    // Generates a new chunk of voxels
    pub fn generate_chunk(x: i32, z: i32) -> Self {
        let mut voxel_index: u32 = 0;
        let mut solid_voxel_count: u32 = 0;

        let mut voxels = Vec::with_capacity(CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE);
        // Loop through each voxel position in the chunk
        for voxel_y in 0..CHUNK_HEIGHT {
            for _voxel_x in 0..CHUNK_SIZE {
                for _voxel_z in 0..CHUNK_SIZE {
                    // Determine the voxel ID (1 for dirt, 0 for air)
                    let id = if voxel_y > 100 { 0 } else { 1 };

                    if id > 0 {
                        solid_voxel_count += 1;
                    }

                    // Create and store the voxel
                    voxels.push(Voxel::new(voxel_index, id));
                    voxel_index += 1;
                }
            }
        }
        println!("Generated chunk ({},{}) ↓", x, z);
        println!(
            "Voxels: ({}) Solid_voxel_count ({})",
            voxels.len(),
            solid_voxel_count
        );
        Chunk {
            coords: (x, z),
            voxels: voxels,
        }
    }
}

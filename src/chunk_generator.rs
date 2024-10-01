// src/chunk.rs

use serde::{Deserialize, Serialize};



// Represents a single voxel in the chunk
#[derive(Serialize, Deserialize, Clone)]
pub struct Voxel {
    pub index: usize, // Single index representing the voxel's position in 3D space
    pub id: u32,      // ID representing the type of voxel (e.g., dirt, air)
}

impl Voxel {
    // Constructor for creating a new Voxel
    pub fn new(index: usize, id: u32) -> Self {
        Voxel { index, id }
    }
}

// Represents a chunk of voxels
#[derive(Serialize, Deserialize, Clone)]
pub struct Chunk {
    pub coords: (i32, i32),
    pub voxels: Vec<Voxel>, // List of voxels in the chunk
}

impl Chunk {
    // Generates a new chunk of voxels
    pub fn generate_chunk(x: i32, z: i32) -> Self {
        static CHUNK_SIZE: usize = 64;
        static CHUNK_HEIGHT: usize = 256;
        let mut voxels = Vec::with_capacity(CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE);
        // Loop through each voxel position in the chunk
        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    // Determine the voxel ID (1 for dirt, 0 for air)
                    let id = if y > 100 { 0 } else { 1 };

                    // Calculate the voxel's linear index
                    let voxel_index = (y * CHUNK_SIZE * CHUNK_SIZE) + (z * CHUNK_SIZE) + x;

                    // Create and store the voxel
                    voxels.push(Voxel::new(voxel_index, id));
                }
            }
        }

        Self {
            coords: (x, z), // Set the coords using the provided parameters
            voxels,
        }
    }
}

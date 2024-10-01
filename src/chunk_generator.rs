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

impl Chunk {
    // Generates a new chunk of voxels
    pub fn generate_chunk(x: i32, z: i32) -> Self {
        let mut voxel_index: u32 = 0;
        static CHUNK_SIZE: usize = 64;
        static CHUNK_HEIGHT: usize = 256;
        let mut voxels = Vec::with_capacity(CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE);
        // Loop through each voxel position in the chunk
        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    // Determine the voxel ID (1 for dirt, 0 for air)
                    let id = if y > 100 { 0 } else { 1 };

                    // Create and store the voxel
                    voxels.push(Voxel::new(voxel_index, id));
                    voxel_index += 1;
                }
            }
        }
        println!("Generated chunk: ({},{})", x, z);
        println!("Voxels: ({})", voxels.len());
        Self {
            coords: (x, z), // Set the coords using the provided parameters
            voxels,
        }
    }

    
    
    
    // byte[0] identifier, byte[1],[2] x,z,
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        // Add identifier for chunk data
        buffer.insert(0,1 as u8);

        // Serialize coordinates (each as 4 bytes)
        buffer.extend(&self.coords.0.to_le_bytes()); // X coordinate
        buffer.extend(&self.coords.1.to_le_bytes()); // Z coordinate

        // Serialize voxel data
        for voxel in &self.voxels {
            buffer.extend(&voxel.index.to_le_bytes()); // Voxel index (4 bytes)
            buffer.push(voxel.id); // Voxel ID (1 byte)
        }

        buffer
    }
}

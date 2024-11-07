// src/chunk.rs

use noise::{NoiseFn, Simplex};
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
        let simplex = Simplex::new(123456789);

        // smoothness factors
        let frequency = 0.007; // Lower frequency for smoother transitions
        let amplitude = 0.1; // Controls height variation
        let octaves = 2; // More octaves = smoother terrain
        let persistence = 0.5; // Determines the weight of each successive octave

        // allocate heightmap
        let mut height_map = vec![0u32; CHUNK_SIZE * CHUNK_SIZE];

        // generate heightmap
        for voxel_x in 0..CHUNK_SIZE {
            for voxel_z in 0..CHUNK_SIZE {
                let world_x = x * CHUNK_SIZE as i32 + voxel_z as i32;
                let world_z = z * CHUNK_SIZE as i32 + voxel_x as i32;

                let mut height = 0.0;
                let mut freq = frequency;
                let mut amp = amplitude;

                for _octave in 0..octaves {
                    // Calculate noise for this octave at the current (x, z) position
                    height += simplex.get([world_x as f64 * freq, world_z as f64 * freq]) * amp;
                    freq *= 2.0;
                    amp *= persistence;
                }

                // normalize height to range 99 to CHUNK_HEIGHT
                let min_height = 99;
                let height_range = (CHUNK_HEIGHT - min_height) as f64;
                let normalized_height = ((height + 1.0) * 0.5 * height_range + min_height as f64) as u32;

                height_map[voxel_x * CHUNK_SIZE + voxel_z] = normalized_height;
            }
        }

        // Loop through each voxel position in the chunk
        for voxel_y in 0..CHUNK_HEIGHT {
            for voxel_x in 0..CHUNK_SIZE {
                for voxel_z in 0..CHUNK_SIZE {
                    // get height from heightmap
                    let height = height_map[voxel_x * CHUNK_SIZE + voxel_z];

                    // Determine the voxel ID (1 for dirt, 0 for air) based on height
                    let id = if voxel_y as u32 <= height { 1 } else { 0 };

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

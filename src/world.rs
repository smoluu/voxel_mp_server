use crate::{
    chunk::{Chunk, Voxel},
    data::DataIdentifier,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Represents a player in the world
#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: u32, // Unique ID for the player
    pub position: (f32, f32, f32),
    pub state: u32,
    // Player's position in the world
}

impl Player {
    // Constructor for creating a new Player
    pub fn new(id: u32, position: (f32, f32, f32), state: u32) -> Self {
        Player {
            id,
            position,
            state,
        }
    }
}

// Represents the world containing chunks, players, and client connections
#[derive(Serialize, Deserialize)]
pub struct World {
    pub chunks: HashMap<(i32, i32), Chunk>, // 2D map of chunks identified by their coordinates (x, z)
    pub players: HashMap<u32, Player>,      // Map of players by their unique ID
}

impl World {
    // Creates a new World instance
    pub fn new() -> Self {
        World {
            players: HashMap::new(),
            chunks: HashMap::new(),
        }
    }

    // Adds a chunk to the world
    pub fn insert_chunk(&mut self, x: i32, z: i32) {
        self.chunks.insert((x, z), Chunk::generate_chunk(x, z));
    }

    // Adds a player to the world
    pub fn add_player(&mut self, player: Player) {
        self.players.insert(player.id, player);
    }

    // Example: Gets a chunk by its coordinates
    pub fn get_chunk(&self, x: i32, z: i32) -> Option<&Chunk> {
        self.chunks.get(&(x, z))
    }

    // Example: Gets a player by their ID
    pub fn get_player(&self, id: u32) -> Option<&Player> {
        self.players.get(&id)
    }

    // reuturns vec of bytes holding data for x,z chunk
    // | Bytes 1-4      | Byte 5  | Bytes 6-9   | Bytes 10-13  | Bytes 14+ |
    // | Length (TBD)   | 02      | X Coord     | Z Coord      | RLE pairs... |
    pub fn chunk_to_bytes_rle(&self, x: i32, z: i32) -> Vec<u8> {
        let mut data = Vec::new();

        // pre allocate length header bytes (byte index 0-3)
        data.resize(4, 1);

        // Add identifier for chunk data (byte index 4)
        let data_identifier = DataIdentifier::ChunkData;
        data.push(data_identifier as u8);

        // Serialize coordinates (byte index 5-8 9-12)
        let chunk = self.chunks.get(&(x, z)).unwrap().clone();
        data.extend(chunk.coords.0.to_le_bytes()); // X coordinate
        data.extend(chunk.coords.1.to_le_bytes()); // Z coordinate

        // Serialize voxel data using RLE (Run Length Encoding) (remaining bytes)
        let mut prev_voxel: Option<Voxel> = None;
        let mut run_length: u8 = 0;

        for voxel in chunk.voxels {
            if let Some(prev_voxel) = &prev_voxel{
                // increase run length if voxel is same as previous
                if voxel.id == prev_voxel.id && run_length < 255{
                    run_length += 1;
                    continue;
                }
                // write the RLE pair
                data.push(run_length); // run length (1 byte)
                data.push(prev_voxel.id); // Voxel ID (1 byte)
            }
            prev_voxel = Some(voxel);
            run_length = 1; // Reset run length for the new voxel type
        }

        // serialize length of the data to first 4 bytes
        let length = data.len() as u32;
        let length_bytes = length.to_le_bytes();
        // add length to first 4 bytes of data
        data[..4].copy_from_slice(&length_bytes);

        data
    }
}

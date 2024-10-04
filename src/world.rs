use crate::{chunk::Chunk, data::DataIdentifier};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap};

// Represents a player in the world
#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: u32,                   // Unique ID for the player
    pub position: (f32, f32, f32), // Player's position in the world
}

impl Player {
    // Constructor for creating a new Player
    pub fn new(id: u32, position: (f32, f32, f32)) -> Self {
        Player { id, position }
    }
}

// Represents the world containing chunks, players, and client connections
#[derive(Serialize, Deserialize)]
pub struct World {
    pub chunks: HashMap<(i32, i32), Chunk>, // 2D map of chunks identified by their coordinates (x, z)
    pub players: HashMap<u32, Player>,    // Map of players by their unique ID
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
        self.chunks.insert((x, z), Chunk::generate_chunk(0, 0));
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

    // byte[0] identifier, byte[1],[2] x,z,
    pub fn chunk_to_bytes(&self, x: i32,z: i32) -> Vec<u8> {

        let mut buffer = Vec::new();


        let chunk = self.chunks.get(&(x, z)).unwrap().clone();

        // Add identifier for chunk data
        let data_identifier = DataIdentifier::ChunkData;
        buffer.insert(0,data_identifier as u8);

        // Serialize coordinates (each as 4 bytes)
        buffer.extend(chunk.coords.0.to_le_bytes()); // X coordinate
        buffer.extend(chunk.coords.1.to_le_bytes()); // Z coordinate

        // Serialize voxel data
        for voxel in chunk.voxels {
            buffer.extend(&voxel.index.to_le_bytes()); // Voxel index (4 bytes)
            buffer.push(voxel.id); // Voxel ID (1 byte)
        }

        buffer
    }

}

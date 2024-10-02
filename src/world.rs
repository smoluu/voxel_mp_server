// src/world.rs

use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use crate::chunk::Chunk;

// Represents a player in the world
#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: u32,                // Unique ID for the player
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
    pub players: HashMap<u32, Player>,       // Map of players by their unique ID
    pub client_connections: HashMap<u32, HashSet<u32>>, // Map of clients to their associated player IDs
}

impl World {
    // Creates a new World instance
    pub fn new() -> Self {
        World {
            chunks: HashMap::new(),
            players: HashMap::new(),
            client_connections: HashMap::new(),
        }
    }

    // Adds a chunk to the world
    pub fn add_chunk(&mut self, x: i32, z: i32, chunk: Chunk) {
        self.chunks.insert((x, z), chunk);
    }

    // Adds a player to the world
    pub fn add_player(&mut self, player: Player) {
        self.players.insert(player.id, player);
    }

    // Adds a client connection and associates it with a player ID
    pub fn add_client_connection(&mut self, client_id: u32, player_id: u32) {
        self.client_connections
            .entry(client_id)
            .or_insert_with(HashSet::new)
            .insert(player_id);
    }

    // Example: Gets a chunk by its coordinates
    pub fn get_chunk(&self, x: i32, z: i32) -> Option<&Chunk> {
        self.chunks.get(&(x, z))
    }

    // Example: Gets a player by their ID
    pub fn get_player(&self, id: u32) -> Option<&Player> {
        self.players.get(&id)
    }

    // Gets all players connected to a specific client
    pub fn get_players_for_client(&self, client_id: u32) -> Option<&HashSet<u32>> {
        self.client_connections.get(&client_id)
    }

    // Removes a client connection
    pub fn remove_client_connection(&mut self, client_id: u32) {
        self.client_connections.remove(&client_id);
    }
}

use crate::{
    chunk::{Chunk, Voxel, CHUNK_HEIGHT, CHUNK_SIZE},
    client::ClientManager,
    data::DataIdentifier,
    CHUNK_GENERATED_COUNTER, CHUNK_GENERATION_TIME,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use std::{
    collections::{HashMap, HashSet},
    thread::spawn,
};
use tokio::sync::RwLock;

#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: u32, // Unique ID for the player
    pub position: (f32, f32, f32),
    pub state: u32,
}

impl Player {
    pub fn new(id: u32, position: (f32, f32, f32), state: u32) -> Self {
        Player {
            id,
            position,
            state,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct World {
    pub chunks: HashMap<(i32, i32), Chunk>, // 2D map of chunks identified by their coordinates (x, z)
    pub players: HashMap<u32, Player>,      // Map of players by their unique ID
    pub spawn: (i32, i32, i32),
}

impl World {
    pub fn new() -> Self {
        let mut world = World {
            players: HashMap::new(),
            chunks: HashMap::new(),
            spawn: (0, 0, 0),
        };

        // generate starting chunks 3x3
        for x in 0..2 {
            for z in 0..2 {
                let generated_chunk = Chunk::new(x, z);
                world.chunks.insert((x, z), generated_chunk);
            }
        }

        // calculate spawn point by checking the 0,0 chunk middle voxels on y axis until 2 empty space is found
        if let Some(spawn_chunk) = world.get_chunk(0, 0) {
            // middle index
            let mut index = CHUNK_SIZE * CHUNK_SIZE / 2 - (CHUNK_SIZE / 2 + 1);
            for y in 0..CHUNK_HEIGHT {
                if let Some(voxel) = spawn_chunk.get_voxel(index) {
                    if voxel.id == 0 {
                        //check above voxel for air
                        if let Some(voxel) = spawn_chunk.get_voxel(CHUNK_SIZE * CHUNK_SIZE + index)
                        {
                            if voxel.id == 0 {
                                world.spawn = spawn_chunk.index_to_coords(CHUNK_SIZE * CHUNK_SIZE + index);
                                break;
                            }
                        }
                        index += CHUNK_SIZE * CHUNK_SIZE * 2;
                    }
                }
                index += CHUNK_SIZE * CHUNK_SIZE;
            }
            println!("{:?}", world.spawn)
        }

        world
    }

    pub fn add_player(&mut self, player: Player) {
        self.players.insert(player.id, player);
    }

    pub fn get_chunk(&self, x: i32, z: i32) -> Option<&Chunk> {
        self.chunks.get(&(x, z))
    }

    pub fn get_player(&self, id: u32) -> Option<&Player> {
        self.players.get(&id)
    }

    pub fn chunk_to_bytes_rle(&self, x: i32, z: i32) -> Vec<u8> {
        let mut data = Vec::new();
        data.resize(4, 1); // Pre-allocate length header bytes (byte index 0-3)

        let data_identifier = DataIdentifier::ChunkData;
        data.push(data_identifier as u8);

        let chunk = self.chunks.get(&(x, z)).unwrap().clone();
        data.extend(chunk.coords.0.to_le_bytes());
        data.extend(chunk.coords.1.to_le_bytes());

        let mut prev_voxel: Option<Voxel> = None;
        let mut run_length: u8 = 0;

        for voxel in chunk.voxels {
            if let Some(prev_voxel) = &prev_voxel {
                if voxel.id == prev_voxel.id && run_length < 255 {
                    run_length += 1;
                    continue;
                }

                data.push(run_length);
                data.push(prev_voxel.id);
            }
            prev_voxel = Some(voxel);
            run_length = 1;
        }

        let length = data.len() as u32;
        let length_bytes = length.to_le_bytes();
        data[..4].copy_from_slice(&length_bytes);

        data
    }

    // Function to handle world generation based on demanded chunks
    pub async fn world_generation(
        world: Arc<RwLock<World>>,
        client_manager: Arc<RwLock<ClientManager>>,
    ) {
        let mut generated_chunks: HashSet<(i32, i32)> = HashSet::new();

        loop {
            {
                // Calculate demand for chunks
                let mut client_manager = client_manager.write().await;
                client_manager.calculate_demanded_chunks().await;
            }

            // Check for connected clients
            let client_count = {
                let client_manager = client_manager.read().await;
                client_manager.clients.len()
            };

            if client_count == 0 {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                continue; // No clients, continue the loop
            }

            // Get all demanded chunks
            let demanded_chunks: Vec<(i32, i32, i32)> = {
                let client_manager = client_manager.read().await;
                client_manager.demanded_chunks.clone()
            };

            // Generate the demanded chunks
            for chunk in demanded_chunks {
                let x = chunk.0;
                let z = chunk.1;
                if !generated_chunks.contains(&(x, z)) {
                    let timer = Instant::now();
                    let generated_chunk = Chunk::new(x, z);
                    {
                        let mut world = world.write().await;
                        world.chunks.insert((x, z), generated_chunk);
                        generated_chunks.insert((x, z));
                    }
                    // Metrics (Assuming CHUNK_GENERATION_TIME and CHUNK_GENERATED_COUNTER are defined elsewhere)
                    CHUNK_GENERATION_TIME.observe(timer.elapsed().as_millis() as f64);
                    CHUNK_GENERATED_COUNTER.inc();
                }
            }

            // Sleep for a while before checking again
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
}

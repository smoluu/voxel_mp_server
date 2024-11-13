use crate::data::DataIdentifier;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::vec;
use tokio::sync::RwLock;

#[derive()]
pub struct Client {
    pub id: u32,                   // Unique ID for the client
    pub position: (f32, f32, f32), // client's position
    pub rotation: (f32, f32, f32), // client's rotation
    pub state: u32,
    pub chunk_demand: Vec<(i32, i32, i32)>,
    pub packet_count_rx: u64,
}

impl Client {
    pub fn client_to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        // pre allocate length header bytes
        data.resize(4, 1);
        // push dataIdentifier (byte index 4)
        let data_identifier = DataIdentifier::InitializeData;
        data.push(data_identifier as u8);

        // Serialize client ID (byte index 5 to 8)
        data.extend(self.id.to_le_bytes());

        // Serialize position coordinates (byte index 9 to 20)
        data.extend(self.position.0.to_le_bytes());
        data.extend(self.position.1.to_le_bytes());
        data.extend(self.position.2.to_le_bytes());

        // serialize client state (byte index 21)
        data.extend(self.state.to_le_bytes());

        // serialize length of the data & insert to bytes 0 to 3
        let length = data.len() as u32;
        let length_bytes = length.to_le_bytes();
        // add length to first 4 bytes of data
        data[..4].copy_from_slice(&length_bytes);

        data
    }
}

pub struct ClientManager {
    pub clients: HashMap<u32, Arc<RwLock<Client>>>,
    pub demanded_chunks: Vec<(i32, i32, i32)>
}

impl ClientManager {
    pub fn new() -> Self {
        ClientManager {
            clients: HashMap::new(),
            demanded_chunks: Vec::new(),
        }
    }
    pub async fn add_client(&mut self, client: Arc<RwLock<Client>>) {
        let client_id = client.read().await.id;
        self.clients.insert(client_id, client);
    }

    pub fn remove_client(&mut self, client_id: u32) {
        self.clients.remove(&client_id);
    }

    pub fn get_client(&self, client_id: u32) -> Option<Arc<RwLock<Client>>> {
        self.clients.get(&client_id).cloned()
    }
    // returns all clients id, position, rotation, state
    pub async fn get_all_client_data(&self) -> Vec<(u32, (f32, f32, f32), (f32, f32, f32), u32)> {
        let mut client_data = Vec::new();

        // Iterate through all clients in the HashMap
        for client_arc in self.clients.values() {
            let client = client_arc.read().await; // Acquire read lock on the client
            client_data.push((client.id,client.position, client.rotation, client.state)); // Collect client position
        }

        client_data
    }
    //returns a vec of chunk x,z,distance values based on clients demands & sorted by acending distance
    pub async fn calculate_demanded_chunks(&mut self) -> Vec<(i32, i32, i32)> {
        let mut filtered_demanded_chunks: HashMap<(i32, i32), i32> = HashMap::new();
        let mut demanded_chunks = Vec::new();
        // Iterate through all clients in the HashMap
        for client_arc in self.clients.values() {
            let client = client_arc.read().await;
            demanded_chunks.extend(client.chunk_demand.clone());
        }
        // remove duplicates and keep smallest distance
        for (x,z,distance) in demanded_chunks {
            match filtered_demanded_chunks.get(&(x,z)) {
                Some(&existing_distance) if existing_distance <= distance => {
                    //ignore
                }

                _ => {
                    filtered_demanded_chunks.insert((x,z),distance);
                }
            }
        }
        //convert to vec for sorting
        let mut sorted_demanded_chunks: Vec<(i32,i32,i32)> = filtered_demanded_chunks
        .into_iter()
        .map(|((x, z), distance)| (x, z, distance))
        .collect();
        // sort entries
        sorted_demanded_chunks.sort_by_key(|&(_, _, distance)| distance);
        
        self.demanded_chunks = sorted_demanded_chunks;
        self.demanded_chunks.clone()
    }
}

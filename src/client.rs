use crate::data::DataIdentifier;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive()]
pub struct Client {
    pub id: u32,                   // Unique ID for the client
    pub position: (f32, f32, f32), // client's position in the world
    pub state: u32,
    pub chunk_demand: Vec<(i32, i32)>,
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
    pub demanded_chunks: HashSet<(i32, i32)>
}

impl ClientManager {
    pub fn new() -> Self {
        ClientManager {
            clients: HashMap::new(),
            demanded_chunks: HashSet::new(),
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

    pub async fn get_all_client_positions(&self) -> Vec<(f32, f32, f32)> {
        let mut positions = Vec::new();

        // Iterate through all clients in the HashMap
        for client_arc in self.clients.values() {
            let client = client_arc.read().await; // Acquire read lock on the client
            positions.push(client.position); // Collect client position
        }

        positions
    }
    //TODO sort hashset so closest chunks to clients are first in the set
    //returns a hashset of chunk x,z values based on clients demands
    pub async fn calculate_demanded_chunks(&mut self) -> HashSet<(i32, i32)> {
        let mut all_demanded_chunks = Vec::new();
        // Iterate through all clients in the HashMap
        for client_arc in self.clients.values() {
            let client = client_arc.read().await;
            all_demanded_chunks.extend(client.chunk_demand.clone());
        }
        //remove duplicates
        for chunk in all_demanded_chunks {
            self.demanded_chunks.insert(chunk);
        }
        self.demanded_chunks.clone()
    }
}

use crate::data::{DataIdentifier};
use tokio::net::TcpStream; // Import TcpStream for client connections
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;


#[derive(Clone)]
pub struct Client {
    pub id: u32,                 // Unique ID for the client
    pub socket: Arc<Mutex<TcpStream>>,       // Socket connection for the client
    pub position: (f32, f32, f32), // client's position in the world
}

impl Client {
    pub fn new(id: u32, socket: Arc<Mutex<TcpStream>>) -> Self {
        Client {
            id,
            socket,
            position: (0.0, 0.0, 0.0), // Default position
        }
    }

    // Serialize the Client's data into a byte vector
    pub fn initialize_client(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        // Add an identifier for client data (e.g., 1)
        let data_identifier = DataIdentifier::InitializeData;
        buffer.insert(0, data_identifier as u8); // Identifier for client data

        // Serialize client ID (4 bytes)
        buffer.extend(self.id.to_le_bytes());

        // Serialize position coordinates (each as 4 bytes)
        buffer.extend(self.position.0.to_le_bytes()); // X coordinate
        buffer.extend(self.position.1.to_le_bytes()); // Y coordinate
        buffer.extend(self.position.2.to_le_bytes()); // Z coordinate

        buffer
    }
    
}

pub struct ClientManager {
    pub clients: HashMap<u32, Arc<RwLock<Client>>>, // Player ID mapped to Player}
}

impl ClientManager {
    pub fn new() -> Self {
        ClientManager {
            clients: HashMap::new(),
        }
    }
    pub fn add_client(&mut self, client: Arc<RwLock<Client>>) {
        let client_id = client.read().unwrap().id;
        self.clients.insert(client_id, client);
    }

    pub fn remove_client(&mut self, client_id: u32) {
        self.clients.remove(&client_id);
    }
    
    pub fn get_client(&self, client_id: u32) -> Option<Arc<RwLock<Client>>> {
        self.clients.get(&client_id).cloned()
    }
}


// src/main.rs

mod chunk; // declare modules
mod client;
mod data;
mod world;
use std::sync::{Arc, RwLock};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

use chunk::Chunk;
use client::{Client, ClientManager};
use world::World;

pub struct Server {
    world: Arc<RwLock<World>>,                  // The world data
    client_manager: Arc<RwLock<ClientManager>>, // The client manager
}

#[tokio::main]
async fn main() {
    let world = Arc::new(RwLock::new(World::new()));
    let client_manager = Arc::new(RwLock::new(ClientManager::new()));

    // Set up TCP listener for client connections
    let addr = "127.0.0.1:6969"; // Set the address and port
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Server running on {}", addr);

    let (mut socket, _) = listener.accept().await.unwrap();
    println!("Client connected!");

    tokio::spawn(accept_connections(listener, client_manager.clone()));
    // Start broadcasting updates
    //tokio::spawn(handle_tx(client_manager.clone()));
    // start generating chunks
    tokio::spawn(world_generation(world.clone()));

    // Keep the server running indefinitely
    tokio::signal::ctrl_c().await.unwrap();
    println!("Server shutting down");
}

async fn accept_connections(listener: TcpListener, client_manager: Arc<RwLock<ClientManager>>) {
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        handle_new_connection(stream, client_manager.clone()).await;
    }
}

async fn handle_new_connection(stream: TcpStream, client_manager: Arc<RwLock<ClientManager>>) {
    println!("debug.");
    // Read lock to get the client ID and release the lock immediately
    let client_id = {
        let manager = client_manager.read().unwrap();
        manager.clients.len() as u32 + 1
    };

    // Create the new client with a write lock
    let client = Arc::new(RwLock::new(Client {
        id: client_id,
        position: (0.0, 0.0, 2.0),            // Initial position
        socket: Arc::new(Mutex::new(stream)), // Use Arc for the socket
    }));
    client_manager.write().unwrap().add_client(client.clone());

    // Serialize the client data
    let client_data = client.read().unwrap().initialize_client();

    // Lock the socket with tokio::Mutex to avoid concurrent writes
    let client_socket = client.read().unwrap().socket.clone();

    // Lock the socket for sending data
    let mut socket = client_socket.lock().await;

    // Calculate the length of the data (including the identifier)
    let length = client_data.len() as u32;
    // Convert length to a 4-byte array
    let length_bytes = length.to_le_bytes();

    // Send the length first
    if let Err(e) = socket.write_all(&length_bytes).await {
        println!("Failed to send length: {:?}", e);
        return; // Exit if there's an error
    } else {
        println!("Sent length data.");
    }
    const PACKET_SIZE: usize = 1024;

    // Send the actual data in chunks
    let data_len = client_data.len();
    let mut offset = 0;

    while offset < data_len {
        // Calculate the number of bytes to send in this iteration
        let bytes_to_send = std::cmp::min(PACKET_SIZE, data_len - offset);

        if let Err(e) = socket
            .write_all(&client_data[offset..offset + bytes_to_send])
            .await
        {
            println!("Failed to send client data: {:?}", e);
            break;
        }

        offset += bytes_to_send; // Update offset
    }
    println!("Sent client_data ({} bytes) to client.", length);

    // Spawn a task to handle incoming data from this client
    tokio::spawn(handle_rx(client.clone()));
}

async fn handle_rx(client: Arc<RwLock<Client>>) {

    // Get the client's socket with a lock
    let mut socket = {
        client.read().unwrap().socket.clone()
    };
    let mut buffer = [0u8; 1024]; // Buffer for reading data

 loop {
        // Read data into the buffer
        match socket.lock().await.read(&mut buffer).await {
            Ok(0) => {
                println!("Client disconnected.");
                break;
            }
            Ok(bytes_read) => {
                println!("Received {} bytes from client.", bytes_read);
                // Process the received data
                handle_incoming_data(&buffer[..bytes_read], client.clone()).await;
            }
            Err(e) => {
                println!("Failed to read from socket: {:?}", e);
                break; // Handle read error and exit loop
            }
        }
    }
}

async fn handle_incoming_data(data: &[u8], client: Arc<RwLock<Client>>) {
    // Example of processing client data (e.g., updating the client's position or state)
    
    if data.len() < 12 {
        // If the data length is insufficient for a position update, ignore
        println!("Received data too short to process.");
        return;
    }

    // new position from the incoming data
    let new_x = f32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let new_y = f32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let new_z = f32::from_le_bytes([data[8], data[9], data[10], data[11]]);

    {
        // update the client's position
        let mut client = client.write().unwrap();
        client.position = (new_x, new_y, new_z);
    }

    println!(
        "Updated client position to: x = {}, y = {}, z = {}",
        new_x, new_y, new_z
    );
}

async fn handle_tx(client_manager: Arc<RwLock<ClientManager>>) {
    loop {
        let clients = {
            let manager = client_manager.read().unwrap();
            manager.clients.clone()
        };

        for (_, client) in clients.iter() {}
    }
}

async fn world_generation(world: Arc<RwLock<World>>) {
    //check demand for chunks
    // generate chunks if they dont exists
}

// src/main.rs

mod chunk; // declare modules
mod client;
mod data;
mod world;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use data::DataIdentifier;
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

    // start generating chunks
    tokio::spawn(world_generation(world.clone()));

    // spawn task to accept connections
    tokio::spawn(accept_connections(
        listener,
        client_manager.clone(),
        world.clone(),
    ));
    

    // Keep the server running indefinitely
    tokio::signal::ctrl_c().await.unwrap();
    println!("Server shutting down");
}

async fn accept_connections(
    listener: TcpListener,
    client_manager: Arc<RwLock<ClientManager>>,
    world: Arc<RwLock<World>>,
) {
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        println!("Client connected!");

        handle_new_connection(stream, client_manager.clone(), world.clone()).await;
    }
}

async fn handle_new_connection(
    stream: TcpStream,
    client_manager: Arc<RwLock<ClientManager>>,
    world: Arc<RwLock<World>>,
) {
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
        state: 0,
    }));
    // add client to client_manager
    client_manager.write().unwrap().add_client(client.clone());

    // Serialize the client data and send it to client
    let client_data = client.read().unwrap().client_to_bytes();
    send_serialized_data(client.clone(),client_data).await;

    // Spawn a task to handle incoming/outgoing data for this client
    tokio::spawn(handle_rx(client.clone()));
    tokio::spawn(handle_tx(client.clone(), world.clone()));

}

async fn handle_rx(client: Arc<RwLock<Client>>) {
    // Get the client's socket with a lock
    let socket = { client.read().unwrap().socket.clone() };
    let mut buffer = [0u8; 1024]; // Buffer for reading data

    loop {
        // Read data into the buffer
        match socket.lock().await.read(&mut buffer).await {
            Ok(0) => {
                println!("Client disconnected.");
                break;
            }
            Ok(bytes_read) => {
                println!("Received {} bytes) from client.", bytes_read);
                // Process the received data
                handle_incoming_client_data(&buffer[..bytes_read], client.clone()).await;
            }
            Err(e) => {
                println!("Failed to read from socket: {:?}", e);
                break; // Handle read error and exit loop
            }
        }
    }
}

async fn handle_incoming_client_data(bytes: &[u8], client: Arc<RwLock<Client>>) {
    // Example of processing client bytes (e.g., updating the client's position or state)

    if bytes.len() < 12 {
        // If the bytes length is insufficient for a position update, ignore
        println!("Received bytes too short to process.");
        return;
    }

    // deserialize client_id
    let client_id = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);

    // Deserialize position (next 12 bytes: 4 bytes per float)
    let pos_x = f32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]);
    let pos_y = f32::from_le_bytes([bytes[9], bytes[10], bytes[11], bytes[12]]);
    let pos_z = f32::from_le_bytes([bytes[13], bytes[14], bytes[15], bytes[16]]);

    // deserialize client state (next 4 bytes)
    let state = u32::from_le_bytes([bytes[17], bytes[18], bytes[19], bytes[20]]);

    {
        // update the client's position
        let mut client = client.write().unwrap();
        client.position = (pos_x, pos_y, pos_z);
    }

    //println!("Updated client position to: x = {}, y = {}, z = {}",pos_x, pos_y, pos_z);
}

async fn handle_tx(client: Arc<RwLock<Client>>, world: Arc<RwLock<World>>) {

    let data = world.read().unwrap().chunk_to_bytes(0, 0);
    send_serialized_data(client, data).await;

}

async fn send_serialized_data(client: Arc<RwLock<Client>>, data: Vec<u8>)
{
    // Lock the socket for sending data
    let socket = client.read().unwrap().socket.clone();
    let mut socket_lock = socket.lock().await;

    let buffer_size: usize = 1024;

    let identifier = data[4];


    // Send the data in chunks
    let data_len = data.len();
    let mut offset = 0;

    while offset < data_len {
        // Calculate the number of bytes to send in this iteration
        let bytes_to_send = std::cmp::min(buffer_size, data_len - offset);

        if let Err(e) = socket_lock
            .write_all(&data[offset..offset + bytes_to_send])
            .await
        {
            println!("Failed to send client data: {:?}", e);
            break;
        }

        offset += bytes_to_send; // Update offset
    }
    println!("Sent {} data ({} bytes) to client.", identifier, data_len);
    println!("Data contents: {:?}", &data[..data.len().min(16)]);
}

async fn world_generation(world: Arc<RwLock<World>>) {
    let mut world: std::sync::RwLockWriteGuard<'_, World> = world.write().unwrap();
    world.insert_chunk(0, 0);

    //check demand for chunks
    // generate chunks if they dont exists
}

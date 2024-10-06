// src/main.rs
mod chunk;
mod client;
mod data;
mod world;

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, RwLock};

use client::{Client, ClientManager};
use world::World;

#[tokio::main]
async fn main() {
    let world = Arc::new(RwLock::new(World::new()));
    let client_manager = Arc::new(RwLock::new(ClientManager::new()));

    // Set up TCP listener for client connections
    let addr = "127.0.0.1:6969";
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Server running on {}", addr);

    // Start generating chunks in a separate task
    tokio::spawn(world_generation(world.clone()));

    // Spawn task to accept connections
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

        let (read_half, write_half) = stream.into_split();

        // Wrap write_half in an Arc<Mutex<>> to allow safe access from multiple tasks
        let write_half = Arc::new(Mutex::new(write_half));

        handle_new_connection(
            read_half,
            write_half.clone(),
            client_manager.clone(),
            world.clone(),
        )
        .await;
    }
}

async fn handle_new_connection(
    read_half: OwnedReadHalf,
    write_half: Arc<Mutex<OwnedWriteHalf>>,
    client_manager: Arc<RwLock<ClientManager>>,
    world: Arc<RwLock<World>>,
) {
    // Assign a new client ID by locking client_manager
    let client_id = {
        let manager = client_manager.read().await;
        manager.clients.len() as u32 + 1
    };

    // Create the new client object
    let client = Arc::new(RwLock::new(Client {
        id: client_id,
        position: (0.0, 102.0, 0.0), // Initial position
        state: 0,
    }));

    // Add the client to the manager
    {
        let mut manager = client_manager.write().await;
        manager.add_client(client.clone()).await;
    }

    // Spawn a task to handle incoming data (read_half) and outgoing data (write_half)
    tokio::spawn(handle_rx(read_half, client.clone()));
    tokio::spawn(handle_tx(write_half.clone(), client.clone(), world.clone()));
}

const BUFFER_SIZE: usize = 1024; // Adjust buffer size as needed
const LENGTH_BUFFER_SIZE: usize = 4; // Adjust buffer size as needed
const IDENTIFIER_SIZE: usize = 1; // Size of the identifier

async fn handle_rx(mut read_half: OwnedReadHalf, client: Arc<RwLock<Client>>) {
    let mut length_buffer = [0u8; LENGTH_BUFFER_SIZE];
    loop {
        // read length header
        if let Err(e) = read_half.read(&mut length_buffer).await {
            eprintln!("Error reading length header: {}", e);
            println!("Length buffer: {:?}", &length_buffer[..length_buffer.len().min(16)]);

            return;
        }

        // length_buffer to int
        let total_message_length = u32::from_le_bytes(length_buffer) as usize;
        println!(
            "Total message length (including length header): {}",
            total_message_length
        );

        let mut received_data = vec![0u8; total_message_length]; // data - 4-byte length
        let mut total_bytes_read = 0;

        // read for the amount length_buffer says in 1024 byte chunks
        while total_bytes_read < total_message_length - LENGTH_BUFFER_SIZE {
            let bytes_to_read = std::cmp::min(1024, total_message_length - 4 - total_bytes_read);
            match read_half
                .read(&mut received_data[total_bytes_read..total_bytes_read + bytes_to_read])
                .await
            {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        eprintln!("Connection closed or read error.");
                        return; // or break; based on your logic
                    }
                    total_bytes_read += bytes_read; // Update the total bytes read
                }
                Err(e) => {
                    // Handle the error accordingly
                    eprintln!("Error reading data: {}", e);
                    return;
                }
            }
        }

        // process data
        if total_bytes_read == total_message_length - LENGTH_BUFFER_SIZE {
            // get identifier
            let identifier = received_data[0] as u8;
            println!("Full data received: Identifier:{} ({} bytes) ↓ ",identifier, total_bytes_read);
            println!("Bytes{:?}", &received_data[..received_data.len().min(16)]);

            // Process the received data
        } else {
            println!("Failed to read the full message.");
        }
    }
}

async fn handle_tx(
    write_half: Arc<Mutex<OwnedWriteHalf>>,
    client: Arc<RwLock<Client>>,
    world: Arc<RwLock<World>>,
) {
    let world_read = world.read().await;
    //lock write_half

    // Serialize the client data and send it to the client
    let client_data = client.read().await.client_to_bytes();
    send_data(write_half.clone(), client_data).await;

    let chunk_data_1 = world_read.chunk_to_bytes(0, 0);
    send_data(write_half.clone(), chunk_data_1).await;

    let chunk_data_2 = world_read.chunk_to_bytes(0, 1);
    send_data(write_half.clone(), chunk_data_2).await;
}

async fn send_data(write_half: Arc<Mutex<OwnedWriteHalf>>, data: Vec<u8>) {
    let mut socket = write_half.lock().await; // Lock the mutex to get access to the write_half
    let buffer_size: usize = 1024;

    let identifier = data[4];

    // Send the data in chunks
    let data_len = data.len();
    let mut offset = 0;

    while offset < data_len {
        // Calculate the number of bytes to send in this iteration
        let bytes_to_send = std::cmp::min(buffer_size, data_len - offset);
        if let Err(e) = socket
            .write_all(&data[offset..offset + bytes_to_send])
            .await
        {
            println!("Failed to send client data: {:?}", e);
            break;
        }

        offset += bytes_to_send; // Update offset
    }

    println!("Sent data: Identifier:{} ({} bytes) ↑", identifier, data_len);
    println!("Bytes{:?}", &data[..data.len().min(16)]);
}

async fn world_generation(world: Arc<RwLock<World>>) {
    let mut world = world.write().await;
    world.insert_chunk(0, 0);
    world.insert_chunk(0, 1);

    //check demand for chunks
    // generate chunks if they dont exists
}

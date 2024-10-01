// src/main.rs

mod chunk_generator; // Declare the chunk_generator module
mod world; // Declare the world module

use chunk_generator::Chunk; // Import the structs from the chunk module
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener; // Import TcpListener for handling connections
use world::{Player, World}; // Import the World and Player structs // Import async read/write traits

#[tokio::main]
async fn main() {
    // Create a new world
    let mut world = World::new();

    // Set up TCP listener for client connections
    let addr = "127.0.0.1:6969"; // Set the address and port
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Server running on {}", addr);
    // Example: Generate and add a chunk to the world
    let chunk = Chunk::generate_chunk(0, 0);
    world.add_chunk(0, 0, chunk.clone());

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        println!("Client connected!");

        // Simulate adding a player for the connected client
        let player_id = world.players.len() as u32 + 1; // Generate new player ID
        let player = Player::new(player_id, (0.0, 0.0, 0.0));
        world.add_player(player.clone()); // Add player to the world
        let client_id = world.client_connections.len() as u32 + 1; // Generate a client ID
        world.add_client_connection(client_id, player_id); // Link client to player

        // Convert chunk to binary & add identifier to front of data
        let data = chunk.to_bytes();

        // Calculate the length of the data (including the identifier)
        let length = data.len() as u32;

        // Convert length to a 4-byte array
        let length_bytes = length.to_le_bytes();

        // Send the length first
        if let Err(e) = socket.write_all(&length_bytes).await {
            println!("Failed to send length: {:?}", e);
        } else {
            println!("Sent length data.");
        }

        // Then send the actual data
        if let Err(e) = socket.write_all(&data).await {
            println!("Failed to send chunk data: {:?}", e);
        } else {
            println!("Sent chunk data to client.");
        }

        // Handle communication with the client
        tokio::spawn(async move {
            let mut buf = vec![0; 1024]; // Buffer for reading data
            let n = socket.read(&mut buf).await.unwrap();
            println!("Received: {:?}", &buf[..n]);
        });
    }
}
#[repr(u8)]
pub enum DataIdentifier {
    ChunkData = 1,
    // Add other data types here as needed
}

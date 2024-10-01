// src/main.rs

mod chunk_generator; // Declare the chunk_generator module
mod world; // Declare the world module

use chunk_generator::{Chunk}; // Import the structs from the chunk module
use world::{World, Player}; // Import the World and Player structs
use serde_json; // Import serde_json for serialization
use tokio::net::TcpListener; // Import TcpListener for handling connections
use tokio::io::{AsyncReadExt, AsyncWriteExt}; // Import async read/write traits

#[tokio::main]
async fn main() {
    // Create a new world
    let mut world = World::new();

    // Set up TCP listener for client connections
    let addr = "127.0.0.1:6969"; // Set the address and port
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Server running on {}", addr);

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        println!("Client connected!");

        // Example: Generate and add a chunk to the world
        let chunk = Chunk::generate_chunk(0,0);
        world.add_chunk(0, 0, chunk.clone());

        // Simulate adding a player for the connected client
        let player_id = world.players.len() as u32 + 1; // Generate new player ID
        let player = Player::new(player_id, (0.0, 0.0, 0.0));
        world.add_player(player.clone()); // Add player to the world
        let client_id = world.client_connections.len() as u32 + 1; // Generate a client ID
        world.add_client_connection(client_id, player_id); // Link client to player

        // Serialize the chunk and send it to the client
        let data = serde_json::to_string(&chunk).unwrap(); // Convert chunk to JSON
        let data_type = "chunk_data";

        // Create a response string with an identifier
        let response = format!(
            "{{\"type\":\"{}\", \"data\":{}}}",
            data_type,
            data
        );

        // Handle communication with the client
        tokio::spawn(async move {
            let mut buf = vec![0; 1024]; // Buffer for reading data
            let n = socket.read(&mut buf).await.unwrap();
            println!("Received: {:?}", &buf[..n]);

            // Sending the chunk data to the client
            socket.write_all(response.as_bytes()).await.unwrap();
            println!("Sent chunk data to client.");
        });
    }

}


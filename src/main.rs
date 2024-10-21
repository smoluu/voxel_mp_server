// src/main.rs
mod chunk;
mod client;
mod data;
mod metrics;
mod world;

use client::{Client, ClientManager};
use data::{process_client_data, DataIdentifier};
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use metrics::*;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;
use std::vec;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpListener;
use tokio::sync::{Mutex, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use world::{Player, World};

#[tokio::main]
async fn main() {
    let world = Arc::new(RwLock::new(World::new()));
    let client_manager = Arc::new(RwLock::new(ClientManager::new()));

    // Set up TCP & Websocket listener for client connections
    let addr = "127.0.0.1:6969";
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Server running on {}", addr);

    let ws_addr = "127.0.0.1:6970"; // WebSocket port
    let ws_listener = TcpListener::bind(ws_addr).await.unwrap();
    println!("WebSocket server running on {}", ws_addr);

    // start metrics endpoint
    tokio::spawn(metrics::start());
    // Start generating chunks in a separate task
    tokio::spawn(world_generation(world.clone(), client_manager.clone()));

    // Spawn task to accept connections
    tokio::spawn(accept_connections(
        listener,
        ws_listener,
        client_manager.clone(),
        world.clone(),
    ));

    // Keep the server running indefinitely
    tokio::signal::ctrl_c().await.unwrap();
    println!("Server shutting down");
}

async fn accept_connections(
    listener: TcpListener,
    ws_listener: TcpListener,
    client_manager: Arc<RwLock<ClientManager>>,
    world: Arc<RwLock<World>>,
) {
    // Spawn a task to handle TCP connections
    let tcp_listener_task = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    println!("TCP Client connected!");

                    let (read_half, write_half) = stream.into_split();
                    let write_half = Arc::new(Mutex::new(write_half));

                    // Handle the new TCP connection
                    handle_new_connection(
                        read_half,
                        write_half.clone(),
                        client_manager.clone(),
                        world.clone(),
                    ).await;
                }
                Err(e) => {
                    eprintln!("Failed to accept TCP connection: {:?}", e);
                }
            }
        }
    });
       // Spawn a task to handle WebSocket connections
    let ws_listener_task = tokio::spawn(async move {
        loop {
            match ws_listener.accept().await {
                Ok((stream, _)) => {
                    println!("WebSocket Client connected!");
                    // Handle the WebSocket connection
                    let ws_stream = tokio_tungstenite::accept_async(stream)
                        .await
                        .expect("Error during WebSocket handshake");

                    // Spawn a task to handle the WebSocket communication
                    tokio::spawn(handle_websocket(ws_stream));
                }
                Err(e) => {
                    eprintln!("Failed to accept WebSocket connection: {:?}", e);
                }
            }
        }
    });
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
        (manager.clients.len() as u32) + 1
    };

    // Create the new client object
    let client = Arc::new(RwLock::new(Client {
        id: client_id,
        position: (0.0, 102.0, 0.0), // Initial position
        state: 0,
        chunk_demand: vec![],
        packet_count_rx: 0,
    }));
    println!("New client created");
    //metrics
    CLIENT_COUNT.inc();
    // Add the client to the manager
    {
        let mut manager = client_manager.write().await;
        manager.add_client(client.clone()).await;
    }
    // add player to world
    {
        let mut world = world.write().await;
        world.add_player(Player::new(client_id, (0.0, 102.0, 0.0), 0));
    }

    // Spawn a task to handle incoming data (read_half) and outgoing data (write_half)
    tokio::spawn(handle_rx(
        read_half,
        client.clone(),
        world.clone(),
        client_manager,
    ));
    tokio::spawn(handle_tx(write_half, client.clone(), world.clone()));
}

const BUFFER_SIZE: usize = 1024; // Adjust buffer size as needed
const LENGTH_BUFFER_SIZE: usize = 4; // Adjust buffer size as needed

async fn handle_rx(
    mut read_half: OwnedReadHalf,
    client: Arc<RwLock<Client>>,
    world: Arc<RwLock<World>>,
    client_manager: Arc<RwLock<ClientManager>>,
) {
    let mut length_buffer = [0u8; LENGTH_BUFFER_SIZE];
    loop {
        // read length header
        if let Err(e) = read_half.read(&mut length_buffer).await {
            println!(
                "Error reading length header: {} {:?}",
                e,
                &length_buffer[..length_buffer.len().min(16)]
            );
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
            let bytes_to_read =
                std::cmp::min(BUFFER_SIZE, total_message_length - 4 - total_bytes_read);
            match read_half
                .read(&mut received_data[total_bytes_read..total_bytes_read + bytes_to_read])
                .await
            {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        let client_id = client.read().await.id;
                        //remove client from client_manager
                        let mut manager = client_manager.write().await;
                        manager.remove_client(client_id);
                        eprintln!("Connection closed or read error on client_id:{}", client_id);
                        CLIENT_COUNT.dec();

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

        // check if all data is received and process data based on identifier
        if total_bytes_read == total_message_length - LENGTH_BUFFER_SIZE {
            // get data identifier (1st byte)
            let identifier = received_data[0] as u8;
            println!(
                "Full data received: Identifier:{} ({} bytes) ↓ ",
                identifier,
                total_bytes_read + LENGTH_BUFFER_SIZE
            );
            println!("Bytes{:?}", &received_data[..received_data.len().min(16)]);
            // spawn tasks for processing data
            match identifier {
                1 => tokio::spawn(process_client_data(received_data, client.clone())),
                2 => tokio::spawn(async {}),
                3 => tokio::spawn(async { /*process keepalive*/ }),
                _ => {
                    println!("Invalid dentifier ({}) cannot process!", identifier);
                    tokio::spawn(async {})
                }
            };
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
    {
        // this is for initializing the client
        let client_data = client.read().await.client_to_bytes();
        send_data(write_half.clone(), client_data).await;
    }

    loop {
        let mut remaining_chunks = Vec::new();
        let chunk_demand = {
            let client = client.read().await;
            client.chunk_demand.clone()
        };

        // send chunks on demand
        for chunk in chunk_demand {
            //check if chunk is generated
            let world = world.read().await;
            //println!("debug {:?}", chunk);
            if world.chunks.contains_key(&chunk) {
                let chunk_data = world.chunk_to_bytes_rle(chunk.0, chunk.1);
                send_data(write_half.clone(), chunk_data).await;
            } else {
                // chunk that were not yet generated
                remaining_chunks.push(chunk);
            }
        }
        {
            let mut client = client.write().await;
            client.chunk_demand = remaining_chunks;
        };
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    //lock write_half
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
    //metrics
    NETWORK_BYTES_EGRESS_TOTAL.inc_by(data_len as u64);

    println!(
        "Sent data: Identifier:{} ({} bytes) ↑",
        identifier, data_len
    );
    println!("Bytes{:?}", &data[..data.len().min(16)]);
}

async fn world_generation(world: Arc<RwLock<World>>, client_manager: Arc<RwLock<ClientManager>>) {
    let mut generated_chunks: HashSet<(i32, i32)> = HashSet::new(); // HashSet to track generated chunks
    loop {
        {
            //calculate demand
            let mut client_manager = client_manager.write().await;
            client_manager.calculate_demanded_chunks().await;
        }
        //TODO add notify so loop doesnt run if no clients are connected
        let client_count = {
            let client_manager = client_manager.read().await;
            client_manager.clients.len()
        };
        if client_count == 0 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            continue; // No clients, continue the loop
        }

        // get all demanded chunks
        let demanded_chunks: HashSet<(i32, i32)> = {
            let client_manager = client_manager.read().await;
            client_manager.demanded_chunks.clone()
        };

        //generate the demanded chunks
        for chunk in demanded_chunks {
            let mut world = world.write().await;
            if !generated_chunks.contains(&chunk) {
                let timer = Instant::now();
                world.insert_chunk(chunk.0, chunk.1);
                generated_chunks.insert(chunk);
                //metrics
                CHUNK_GENERATION_TIME.observe(timer.elapsed().as_millis() as f64);
                CHUNK_GENERATED_COUNTER.inc();
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

async fn handle_websocket(ws_stream: WebSocketStream<tokio::net::TcpStream>) {
    let (mut ws_tx, mut ws_rx) = ws_stream.split();

    // Connect to the TCP server
    let tcp_stream = match tokio::net::TcpStream::connect("127.0.0.1:6969").await {
        Ok(stream) => {
            println!("Connected to TCP server!");
            stream
        }
        Err(e) => {
            eprintln!("Failed to connect to TCP server: {:?}", e);
            return; // Exit if unable to connect
        }
    };
    let (tcp_read_half, mut tcp_write_half) = tcp_stream.into_split();

    let tcp_to_ws = tokio::spawn(async move {
        
        let mut tcp_reader = tokio::io::BufReader::new(tcp_read_half);
        let mut buffer = vec![0; 1024]; // Buffer for reading TCP data
        loop {
            match tcp_reader.read(&mut buffer).await {
                Ok(0) => break, // Connection closed
                Ok(n) => {
                    // Send TCP data to WebSocket client
                    if let Err(e) = ws_tx.send(Message::Binary(buffer[..n].to_vec())).await {
                        eprintln!("Error sending to WebSocket: {:?}", e);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from TCP: {:?}", e);
                    break;
                }
            }
        }
    });

    while let Some(msg) = ws_rx.next().await {
        match msg {
            Ok(Message::Binary(bin)) => {
                if let Err(e) = tcp_write_half.write_all(&bin).await {
                    eprintln!("Error sending binary data to TCP: {:?}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {:?}", e);
                break;
            }
            _ => {}
        }
    }

    println!("WebSocket connection closed.");
}

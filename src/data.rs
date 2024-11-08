use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{client::Client};
use crate::metrics::*;

#[repr(u8)]
pub enum DataIdentifier {
    InitializeData = 0,
    ClientData = 1,
    ChunkData = 2,
    Keepalive = 3,
}


// data procesing functions

pub async fn process_client_data(data: Vec<u8>, client: Arc<RwLock<Client>>) {
    // ensure data is correct length in bytes
    let data_length = data.len();

    // Read identifier (1 byte)
    let identifier = data[0];

    // Read client_id (next 4 bytes, little-endian)
    let client_id = u32::from_le_bytes(data[1..5].try_into().unwrap());

    // Read position (3 x 4 bytes as f32, little-endian)
    let x = f32::from_le_bytes(data[5..9].try_into().unwrap());
    let y = f32::from_le_bytes(data[9..13].try_into().unwrap());
    let z = f32::from_le_bytes(data[13..17].try_into().unwrap());

    // Read state (next 4 bytes, little-endian)
    let state = u32::from_le_bytes(data[17..21].try_into().unwrap());

    let mut chunk_demand: Vec<(i32, i32, i32)> = Vec::new(); // temp vector to store chunk positions
    // deserialize received chunks
    for i in (21..data_length).step_by(12) {
        if i + 8 > data_length {
            continue;
        }
        let x = i32::from_le_bytes(data[i..i + 4].try_into().unwrap());
        let z = i32::from_le_bytes(data[i + 4..i + 8].try_into().unwrap());
        let distance = i32::from_le_bytes(data[i + 8..i + 12].try_into().unwrap());
        chunk_demand.push((x, z, distance));
    }
    {
        let mut client = client.write().await;
        client.position.0 = x;
        client.position.1 = y;
        client.position.2 = z;
        client.state = state;
        client.chunk_demand = chunk_demand;
        if client.packet_count_rx == 0 {}
        client.packet_count_rx += 1;
        println!(
            "Client x{} y{} z{} chunk_demanLEN{}",
            client.position.0,
            client.position.1,
            client.position.2,
            client.chunk_demand.len()
        );
        //metrics
        NETWORK_BYTES_INGRESS_TOTAL.inc_by(data_length as u64);
    }
}
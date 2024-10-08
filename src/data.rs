
#[repr(u8)]
pub enum DataIdentifier {
    InitializeData = 0,
    ClientData = 1,
    ChunkData = 2,
    Keepalive = 3,
}


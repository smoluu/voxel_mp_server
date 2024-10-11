use prometheus::{
    gather, register_counter, register_gauge, register_histogram, register_int_counter, Counter,
    Encoder, Gauge, Histogram, IntCounter, TextEncoder,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time;

lazy_static::lazy_static! {
    pub static ref SERVER_UPTIME: IntCounter = register_int_counter!("server_uptime"," ").unwrap();
    pub static ref CLIENT_COUNT:Gauge = register_gauge!("client_count"," ").unwrap();
    pub static ref CHUNK_GENERATED_COUNTER: IntCounter = register_int_counter!("chunk_generated_after_restart"," ").unwrap();
    pub static ref CHUNK_GENERATION_TIME: Histogram = register_histogram!("chunk_generation_time"," ").unwrap();
    pub static ref NETWORK_BYTES_EGRESS_TOTAL:IntCounter = register_int_counter!("network_bytes_egress_total"," ").unwrap();
    pub static ref NETWORK_BYTES_INGRESS_TOTAL:IntCounter = register_int_counter!("network_bytes_ingress_total"," ").unwrap();
    pub static ref NETWORK_BYTES_EGRESS_S:Gauge = register_gauge!("network_bytes_egress_s"," ").unwrap();
    pub static ref NETWORK_BYTES_INGRESS_S:Gauge = register_gauge!("network_bytes_ingress_s"," ").unwrap();
}

pub async fn start() {
    // Reset metrics
    CHUNK_GENERATED_COUNTER.reset();
    SERVER_UPTIME.reset();
    CLIENT_COUNT.set(0.0);

    // Create a counter metric
    let request_counter =
        register_counter!("requests_total", "Total number of HTTP requests").unwrap();

    // Start tracking bytes sent per second
    tokio::spawn(track_bytes_per_second());

    // Bind the TCP listener asynchronously
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind to address");

    println!("Metrics server listening on port 8080");

    loop {
        // Accept incoming connections asynchronously
        match listener.accept().await {
            Ok((stream, _addr)) => {
                // Spawn a new task to handle each connection
                tokio::spawn(handle_connection(stream, request_counter.clone()));
            }
            Err(e) => {
                println!("Failed to accept connection: {}", e);
            }
        }
    }
}

// Update handle_connection to accept a TcpStream and request_counter
async fn handle_connection(mut stream: TcpStream, request_counter: Counter) {
    let mut buffer = [0; 1024];
    // Use the async read method
    let n = match stream.read(&mut buffer).await {
        Ok(n) if n == 0 => return, // Connection closed
        Ok(n) => n,
        Err(e) => {
            eprintln!("Failed to read from stream: {}", e);
            return;
        }
    };

    // Convert the buffer to a string and check the request path
    let request = String::from_utf8_lossy(&buffer[..n]);
    let request_line = request.lines().next().unwrap_or("");

    if request_line.starts_with("GET /metrics") {
        // Increment counter
        request_counter.inc();

        // Gather metrics and encode to a buffer
        let encoder = TextEncoder::new();
        let metric_families = gather();
        let mut metrics_buffer = Vec::new();
        encoder
            .encode(&metric_families, &mut metrics_buffer)
            .unwrap();

        // Send HTTP response headers and the metrics data
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n\r\n",
            metrics_buffer.len()
        );

        if let Err(e) = stream.write_all(response.as_bytes()).await {
            eprintln!("Failed to write response: {}", e);
            return;
        }

        if let Err(e) = stream.write_all(&metrics_buffer).await {
            eprintln!("Failed to write metrics: {}", e);
        }
    } else {
        // If the path is not /metrics, respond with 404 Not Found
        let response = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
        if let Err(e) = stream.write_all(response.as_bytes()).await {
            eprintln!("Failed to write response: {}", e);
        }
    }

    // Flushing the stream is usually not necessary, as write_all will flush it.
}

async fn track_bytes_per_second() {
    let mut egress_last_total = NETWORK_BYTES_EGRESS_TOTAL.get(); // Initialize with the current total
    let mut ingress_last_total = NETWORK_BYTES_INGRESS_TOTAL.get(); // Initialize with the current total

    loop {
        time::sleep(time::Duration::from_secs(1)).await;
        SERVER_UPTIME.inc();
        
        let current_egress_total = NETWORK_BYTES_EGRESS_TOTAL.get();
        let current_ingress_total = NETWORK_BYTES_INGRESS_TOTAL.get();

        let bytes_sent_last_second = current_egress_total - egress_last_total;
        let bytes_received_last_second = current_ingress_total - ingress_last_total;

        egress_last_total = current_egress_total;
        ingress_last_total = current_ingress_total;

        // Observe the bytes sent and received
        NETWORK_BYTES_EGRESS_S.set(bytes_sent_last_second as f64);
        NETWORK_BYTES_INGRESS_S.set(bytes_received_last_second as f64);
    }
}

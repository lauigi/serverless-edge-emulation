use serde::{Deserialize, Serialize};
use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
struct Task {
    id: String,
    size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    id: String,
    status: String,
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <speed>", args[0]);
        std::process::exit(1);
    }

    let speed: u64 = args[1].parse().expect("Speed must be a positive integer");

    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    println!("E-computer simulator listening on port {}", port);

    // Set non-blocking mode for the listener
    listener.set_nonblocking(true)?;

    // Process tasks sequentially
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream, speed).unwrap_or_else(|error| eprintln!("Error: {}", error));
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No incoming connection, sleep a bit to prevent busy-waiting
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream, speed: u64) -> std::io::Result<()> {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;

    let task: Task = serde_json::from_slice(&buffer[..bytes_read])?;
    println!("Received task: {:?}", task);

    let processing_time = Duration::from_secs(task.size / speed);
    thread::sleep(processing_time);

    let response = Response {
        id: task.id,
        status: "success".to_string(),
    };

    let response_json = serde_json::to_string(&response)?;
    stream.write_all(response_json.as_bytes())?;

    Ok(())
}

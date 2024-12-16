use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

#[derive(Debug, Clone)]
struct Computer {
    weight: f32,
    deficit: f32,
    last_updated: f64,
    removed: bool,
    probing: bool,
    stale_period: f64,
}
#[derive(Serialize, Deserialize, Debug)]
struct Task {
    id: String,
    size: u64,
}

#[derive(Debug)]
struct QueueElement {
    deficit: f32,
    destination: u16,
}

impl Ord for QueueElement {
    fn cmp(&self, other: &Self) -> Ordering {
        self.deficit
            .partial_cmp(&other.deficit)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for QueueElement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for QueueElement {
    fn eq(&self, other: &Self) -> bool {
        self.deficit == other.deficit
    }
}

impl Eq for QueueElement {}

struct RouterRoundRobin {
    cache: HashMap<u16, Computer>,
    queue: BinaryHeap<QueueElement>,
}

impl RouterRoundRobin {
    fn new() -> Self {
        RouterRoundRobin {
            cache: HashMap::new(),
            queue: BinaryHeap::new(),
        }
    }

    fn add_destination(&mut self, destination: u16, weight: f32) {
        self.cache.insert(
            destination.clone(),
            Computer {
                weight,
                deficit: 2.0,
                last_updated: -1.0,
                removed: false,
                probing: false,
                stale_period: 1.0,
            },
        );
        self.update_active_set();
        println!("cache after adding destination: {:?}", self.cache);
        println!("queue after adding destination: {:?}", self.queue);
    }

    fn update_weight(&mut self, destination: &u16, weight: f32) {
        if let Some(cache_elem) = self.cache.get_mut(destination) {
            cache_elem.weight = weight;
            cache_elem.last_updated = Utc::now().timestamp_millis() as f64;
            self.update_active_set();
        }
    }

    fn min_deficit(&self) -> f32 {
        self.cache
            .values()
            .filter(|elem| !elem.removed)
            .map(|elem| elem.deficit)
            .fold(f32::MAX, |acc, x| acc.min(x))
    }

    fn update_active_set(&mut self) {
        let now = Utc::now().timestamp_millis() as f64;
        let count = self.cache.values().filter(|elem| !elem.removed).count();
        let min_weight = self
            .cache
            .values()
            .filter(|elem| !elem.removed)
            .map(|elem| elem.weight)
            .fold(f32::MAX, |acc, x| acc.min(x));
        let min_deficit = self.min_deficit();

        self.queue.clear();

        for (dest, cache_elem) in &mut self.cache {
            let mut my_active = false;

            if count == 1 || cache_elem.weight <= (min_weight * 2.0) {
                // Reset stale period if it was under probing
                if cache_elem.probing {
                    cache_elem.probing = false;
                    cache_elem.last_updated = now; // Reset last updated time
                    cache_elem.stale_period = 1.0; // Reset stale period
                }
                my_active = true;
            } else if cache_elem.last_updated < 0.0 {
                // New or expired stale timer
                my_active = true;
            } else {
                if cache_elem.probing {
                    cache_elem.probing = false;
                    cache_elem.last_updated = now; // Reset last updated time
                    cache_elem.stale_period = (2.0 * cache_elem.stale_period).min(30.0);
                    // Double stale period
                }
                if Utc::now().timestamp_millis() as f64 - cache_elem.last_updated
                    >= cache_elem.stale_period
                {
                    // Stale timer expired
                    cache_elem.probing = true;
                    cache_elem.last_updated = -1.0; // Mark as probing
                    cache_elem.deficit = min_deficit; // Reset deficit
                    my_active = true;
                }
            }

            if my_active {
                // Add the element to the active set
                self.queue.push(QueueElement {
                    destination: dest.clone(),
                    deficit: cache_elem.deficit,
                });
            }
        }
        println!("cache after updating active set: {:?}", self.cache);
    }

    fn select_destination(&mut self) -> Option<u16> {
        println!("\n\nqueue: {:?}", self.queue);
        if let Some(elem) = self.queue.pop() {
            let destination = elem.destination;
            if let Some(dest) = self.cache.get_mut(&destination) {
                dest.deficit += dest.weight;
                // Update the queue
                self.queue.push(QueueElement {
                    deficit: dest.deficit,
                    destination,
                });
                return Some(destination);
            }
        }
        None
    }
}

fn handle_client(stream: &mut TcpStream, router: &mut RouterRoundRobin) -> std::io::Result<()> {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;

    let task: Task = serde_json::from_slice(&buffer[..bytes_read])?;
    println!("\n\nReceived task: {:?}", task);

    let destination = router.select_destination();
    println!("Selected destination: {:?}", destination);
    if let Some(port) = destination {
        // port is now bound and can be used directly

        let start_time = Instant::now();

        let mut e_computer_stream = TcpStream::connect(format!("127.0.0.1:{:?}", port))?;
        e_computer_stream.write_all(&buffer[..bytes_read])?;

        let mut response_buffer = [0; 1024];
        let response_bytes = e_computer_stream.read(&mut response_buffer)?;

        stream.write_all(&response_buffer[..response_bytes])?;
        let latency = start_time.elapsed().as_secs_f32();

        router.update_weight(&port, latency);

        println!("Task completed. Latency: {:?}", latency);
    } else {
        println!("No destination available");
        stream.write_all(b"No destination available")?;
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <port1> [<port2> ...]", args[0]);
        std::process::exit(1);
    }

    let router = Arc::new(Mutex::new(RouterRoundRobin::new()));

    for arg in &args[1..] {
        let parts: Vec<&str> = arg.split(':').collect();
        let port = parts[0].parse().unwrap();
        router.lock().unwrap().add_destination(port, 0.0);
    }

    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    println!("E-router simulator listening on port {}", port);

    let max_threads = 5;
    let mut thread_handles: Vec<std::thread::JoinHandle<()>> = vec![];

    for stream in listener.incoming() {
        let mut stream = stream?;

        if thread_handles.len() >= max_threads {
            if let Some(handle) = thread_handles.pop() {
                handle.join().unwrap();
            }
        }
        let router_clone = Arc::clone(&router);
        let handle = thread::spawn(move || {
            handle_client(&mut stream, &mut router_clone.lock().unwrap())
                .unwrap_or_else(|error| eprintln!("Error: {}", error));
        });

        thread_handles.push(handle);
    }

    for handle in thread_handles {
        handle.join().unwrap();
    }
    Ok(())
}

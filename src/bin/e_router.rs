use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

#[derive(Clone, Debug)]
struct Computer {
    port: u16,
    hops: u32,
    expiry: SystemTime,
    last_update: f64,
    backoff: f64,
}

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

#[derive(Debug, Clone)]
enum Algorithm {
    LI,
    RP,
    RR,
    AlwaysClosest,
}

#[derive(Clone)]
struct Router {
    computers: Vec<Computer>,
    algorithm: Algorithm,
    weight_table: HashMap<u16, f64>,
    delta_table: HashMap<u16, f64>,
    active_set: HashSet<u16>,
    probed_set: HashSet<u16>,
    b_min: f64,
}

impl Router {
    fn new(computers: Vec<Computer>, algorithm: Algorithm) -> Self {
        let weight_table = computers.iter().map(|c| (c.port, 0.0)).collect();
        let delta_table = computers.iter().map(|c| (c.port, 0.0)).collect();
        // let active_set = computers.iter().map(|c| c.port).collect();
        let b_min = 1.0; // Set appropriate minimum backoff value
        Router {
            computers,
            algorithm,
            weight_table,
            delta_table,
            active_set: HashSet::new(),
            probed_set: HashSet::new(),
            b_min,
        }
    }

    fn select_destination(&mut self) -> &Computer {
        match self.algorithm {
            Algorithm::LI => self
                .computers
                .iter()
                .min_by(|a, b| {
                    self.weight_table[&a.port]
                        .partial_cmp(&self.weight_table[&b.port])
                        .unwrap()
                })
                .unwrap(),

            Algorithm::RP => {
                let total_inverse_weight: f64 = self
                    .computers
                    .iter()
                    .map(|c| 1.0 / self.weight_table[&c.port])
                    .sum();
                let mut rng = rand::thread_rng();
                let random_value = rng.gen::<f64>() * total_inverse_weight;
                let mut cumulative = 0.0;
                self.computers
                    .iter()
                    .find(|c| {
                        cumulative += 1.0 / self.weight_table[&c.port];
                        cumulative >= random_value
                    })
                    .unwrap()
            }

            Algorithm::RR => {
                let now = SystemTime::now();
                // Select random destination from ready and non-probed computers
                let available_dests: Vec<&Computer> = self
                    .computers
                    .iter()
                    .filter(|c| (c.expiry > now) && !self.probed_set.contains(&c.port))
                    .collect();
                println!("Active set: {:?}", self.active_set);
                println!("Probed set: {:?}", self.probed_set);
                println!("Available destinations: {:?}", available_dests);
                println!("Delta table: {:?}", self.delta_table);

                if let Some(dest) = available_dests.choose(&mut rand::thread_rng()) {
                    self.probed_set.insert(dest.port);
                    println!(
                        "========================= Selected destination from branch 1: {:?}",
                        dest
                    );
                    dest
                } else {
                    // Find destination with minimum delta from active set
                    let min_delta_dest = self
                        .computers
                        .iter()
                        .filter(|c| self.active_set.contains(&c.port))
                        .min_by(|a, b| {
                            self.delta_table[&a.port]
                                .partial_cmp(&self.delta_table[&b.port])
                                .unwrap()
                        })
                        .unwrap();
                    println!("Min delta destination: {:?}", min_delta_dest);
                    println!("weight: {:?}", self.weight_table);
                    // Update delta
                    self.delta_table
                        .entry(min_delta_dest.port)
                        .and_modify(|delta| *delta += self.weight_table[&min_delta_dest.port]);
                    println!(
                        "========================= Selected destination from branch 2: {:?}",
                        min_delta_dest
                    );
                    min_delta_dest
                }
            }

            Algorithm::AlwaysClosest => self.computers.iter().min_by_key(|c| c.hops).unwrap(),
        }
    }

    fn update_weight(&mut self, port: u16, latency: Duration) {
        match self.algorithm {
            Algorithm::LI | Algorithm::RP => {
                let alpha = 0.95;
                if self.weight_table[&port] == 0.0 {
                    *self.weight_table.get_mut(&port).unwrap() = latency.as_secs_f64();
                } else {
                    *self.weight_table.get_mut(&port).unwrap() =
                        alpha * self.weight_table[&port] + (1.0 - alpha) * latency.as_secs_f64();
                }
            }

            Algorithm::RR => {
                println!(
                    "====================Updating weight for port {} with latency {:?}",
                    port, latency
                );
                println!("probed set: {:?}", self.probed_set);
                println!("active set: {:?}", self.active_set);
                if self.probed_set.contains(&port) {
                    self.probed_set.remove(&port);

                    let min_active_weight = self
                        .active_set
                        .iter()
                        .map(|p| self.weight_table[p])
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap_or(0.0);
                    println!("Min active weight: {}", min_active_weight);

                    if latency.as_secs_f64() <= 2.0 * min_active_weight || self.active_set.len() < 2
                    {
                        // Update all deltas in active set
                        let min_delta = self
                            .active_set
                            .iter()
                            .map(|p| self.delta_table[p])
                            .min_by(|a, b| a.partial_cmp(b).unwrap())
                            .unwrap_or(0.0);

                        for p in &self.active_set {
                            self.delta_table.entry(*p).and_modify(|d| *d -= min_delta);
                        }

                        // Add to active set and update parameters
                        self.active_set.insert(port);
                        self.weight_table.insert(port, latency.as_secs_f64());
                        self.delta_table.insert(port, latency.as_secs_f64());

                        if let Some(computer) = self.computers.iter_mut().find(|c| c.port == port) {
                            computer.backoff = self.b_min;
                        }
                    } else {
                        // Double backoff and update expiry
                        if let Some(computer) = self.computers.iter_mut().find(|c| c.port == port) {
                            if computer.last_update < 0.0 {
                                println!("First update for port {}", port);
                                self.active_set.insert(port);
                            } else {
                                println!("Doubling backoff for port {}", port);
                                // max backoff is 30 seconds from the paper source code
                                computer.backoff = (2.0 * computer.backoff).min(30.0);
                                computer.expiry =
                                    SystemTime::now() + Duration::from_secs_f64(computer.backoff);
                                let now_f64 = SystemTime::now()
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs_f64();
                                if (now_f64 - computer.last_update) >= computer.backoff {
                                    println!("Reprobing port {}", port);
                                    computer.last_update = -1.0;
                                    let min_delta = self
                                        .active_set
                                        .iter()
                                        .map(|p| self.delta_table[p])
                                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                                        .unwrap_or(0.0);
                                    self.delta_table.insert(port, min_delta);
                                    self.probed_set.insert(port);
                                    self.active_set.insert(port);
                                }
                            }
                        }
                    }
                } else {
                    let alpha = 0.95;
                    if self.weight_table[&port] == 0.0 {
                        *self.weight_table.get_mut(&port).unwrap() = latency.as_secs_f64();
                    } else {
                        *self.weight_table.get_mut(&port).unwrap() = alpha
                            * self.weight_table[&port]
                            + (1.0 - alpha) * latency.as_secs_f64();
                    }
                    println!("check min active weight start here====");
                    println!("Active set: {:?}", self.active_set);
                    println!("Weight table: {:?}", self.weight_table);
                    println!("check min active weight end here======");
                    let min_active_weight = self
                        .active_set
                        .iter()
                        .map(|p| self.weight_table[p])
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap_or(0.0);
                    println!("Min active weight: {}", min_active_weight);
                    if self.weight_table[&port] > 2.0 * min_active_weight {
                        self.active_set.remove(&port);
                    }
                }
                if let Some(computer) = self.computers.iter_mut().find(|c| c.port == port) {
                    computer.last_update = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64();
                }
            }

            Algorithm::AlwaysClosest => {}
        }
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!(
            "Usage: {} <algorithm> <port1:hops1> <port2:hops2> ...",
            args[0]
        );
        std::process::exit(1);
    }

    let algorithm = match args[1].as_str() {
        "LI" => Algorithm::LI,
        "RP" => Algorithm::RP,
        "RR" => Algorithm::RR,
        "AC" => Algorithm::AlwaysClosest,
        _ => {
            eprintln!("Invalid algorithm. Choose LI, RP, RR, or AC.");
            std::process::exit(1);
        }
    };

    let computers: Vec<Computer> = args[2..]
        .iter()
        .map(|arg| {
            let parts: Vec<&str> = arg.split(':').collect();
            Computer {
                port: parts[0].parse().unwrap(),
                hops: parts[1].parse().unwrap(),
                expiry: SystemTime::now(),
                backoff: 2.0,
                last_update: -1.0,
            }
        })
        .collect();
    let router = Arc::new(Mutex::new(Router::new(computers, algorithm)));

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

fn handle_client(stream: &mut TcpStream, router: &mut Router) -> std::io::Result<()> {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;

    let task: Task = serde_json::from_slice(&buffer[..bytes_read])?;
    println!("\n\nReceived task: {:?}", task);

    let destination = router.select_destination();
    println!("Selected destination: {:?}", destination);

    let port = destination.port; // Clone the port value

    let start_time = Instant::now();

    let mut e_computer_stream = TcpStream::connect(format!("127.0.0.1:{}", port))?;
    e_computer_stream.write_all(&buffer[..bytes_read])?;

    let mut response_buffer = [0; 1024];
    let response_bytes = e_computer_stream.read(&mut response_buffer)?;

    let latency = start_time.elapsed();

    stream.write_all(&response_buffer[..response_bytes])?;

    router.update_weight(port, latency);

    println!("Task completed. Latency: {:?}", latency);

    Ok(())
}

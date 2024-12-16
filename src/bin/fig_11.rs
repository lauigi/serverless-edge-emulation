use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// Define CPU speeds for e-computers (in MIPS)
const CPU_SPEEDS: [u32; 4] = [1000, 2000, 3000, 4000];

// Struct to represent an emulated e-computer
struct EComputer {
    cpu_speed: u32,
    queue: Arc<Mutex<Vec<Task>>>,
}

// Struct to represent a task
struct Task {
    size: u32, // in millions of instructions
    arrival_time: Instant,
}

impl EComputer {
    fn new(cpu_speed: u32) -> Self {
        EComputer {
            cpu_speed,
            queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn process_task(&self, task: Task) -> Duration {
        let processing_time = Duration::from_secs_f64((task.size as f64) / (self.cpu_speed as f64));
        let queue_length = self.queue.lock().unwrap().len() as u32;

        // Simulate queuing delay
        let queuing_delay = Duration::from_secs_f64(
            (queue_length as f64) * (task.size as f64) / (self.cpu_speed as f64),
        );

        processing_time + queuing_delay
    }
}

// Enum for different selection algorithms
#[derive(Debug, Clone, Copy)]
enum SelectionAlgorithm {
    LI, // Least Impedance
    RP, // Random Proportional
    RR, // Round Robin
}

// Function to simulate client requests
fn simulate_client(
    client_id: usize,
    computers: Arc<Vec<EComputer>>,
    algorithm: SelectionAlgorithm,
    task_count: u32,
) -> Vec<Duration> {
    let mut delays = Vec::new();
    let mut rng = rand::thread_rng();

    for _ in 0..task_count {
        let task = Task {
            size: 1000, // 1000 MI per task
            arrival_time: Instant::now(),
        };

        // Select e-computer based on algorithm
        let selected_computer = match algorithm {
            SelectionAlgorithm::LI => {
                // Select computer with highest CPU speed
                computers
                    .iter()
                    .enumerate()
                    .max_by_key(|(_, c)| c.cpu_speed)
                    .map(|(i, _)| i)
                    .unwrap()
            }
            SelectionAlgorithm::RP => {
                // Random selection weighted by CPU speed
                let total_speed: u32 = computers.iter().map(|c| c.cpu_speed).sum();
                let mut selected = rng.gen_range(0..total_speed);
                let mut index = 0;
                for (i, computer) in computers.iter().enumerate() {
                    if selected < computer.cpu_speed {
                        index = i;
                        break;
                    }
                    selected -= computer.cpu_speed;
                }
                index
            }
            SelectionAlgorithm::RR => {
                // Round Robin selection
                client_id % computers.len()
            }
        };

        // Process task and measure delay
        let computer = &computers[selected_computer];
        let delay = computer.process_task(task);
        delays.push(delay);

        // Add some think time between requests
        thread::sleep(Duration::from_millis(100));
    }

    delays
}

fn main() {
    // Create e-computers
    let computers: Vec<EComputer> = CPU_SPEEDS
        .iter()
        .map(|&speed| EComputer::new(speed))
        .collect();
    let computers = Arc::new(computers);

    // Test different numbers of clients
    let client_counts = vec![1, 2, 4, 6, 8, 10, 12];
    let algorithms = vec![
        SelectionAlgorithm::LI,
        SelectionAlgorithm::RP,
        SelectionAlgorithm::RR,
    ];

    for &num_clients in &client_counts {
        for algorithm in &algorithms {
            let mut handles = vec![];
            let computers_clone = Arc::clone(&computers);

            // Spawn client threads
            for client_id in 0..num_clients {
                let computers = Arc::clone(&computers_clone);
                let algorithm = algorithm.clone();

                let handle =
                    thread::spawn(move || simulate_client(client_id, computers, algorithm, 100));
                handles.push(handle);
            }

            // Collect all delays
            let mut all_delays = vec![];
            for handle in handles {
                all_delays.extend(handle.join().unwrap());
            }
            println!("All delays: {:?}", all_delays);
            // Calculate 95th percentile
            all_delays.sort_unstable();
            let percentile_95 = all_delays[all_delays.len() * 95 / 100];

            println!(
                "Clients: {}, Algorithm: {:?}, 95th percentile delay: {:?}",
                num_clients, algorithm, percentile_95
            );
        }
    }
}

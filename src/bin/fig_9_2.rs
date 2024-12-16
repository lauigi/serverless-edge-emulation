use rand::Rng;
use statrs::distribution::Normal;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// Define a struct for tasks
#[derive(Debug)]
struct Task {
    id: usize,
    size: usize, // Size of the task in millions of instructions
}

// Define a struct for edge computers
struct EComputer {
    id: usize,
    cpu_speed: f64, // CPU speed in MIPS
}

impl EComputer {
    fn new(id: usize, cpu_speed: f64) -> Self {
        EComputer { id, cpu_speed }
    }

    // Process a task and return the processing delay
    fn process_task(&self, task: &Task) -> Duration {
        let processing_time = (task.size as f64) / self.cpu_speed; // Calculate processing time
        let delay = Duration::from_secs_f64(processing_time);

        // Simulate processing delay with a slight random variation
        let mut rng = rand::thread_rng();
        let variation: f64 = rng.gen_range(-0.1..0.1); // Random variation of ±10%
        let total_delay = delay + Duration::from_secs_f64(variation.abs() * processing_time);

        thread::sleep(total_delay); // Simulate processing delay
        total_delay
    }
}

// Define a struct for managing multiple edge computers
struct EComputerManager {
    computers: Vec<EComputer>,
}

impl EComputerManager {
    fn new() -> Self {
        EComputerManager {
            computers: Vec::new(),
        }
    }

    // Add an edge computer to the manager
    fn add_computer(&mut self, computer: EComputer) {
        self.computers.push(computer);
    }

    // Select an edge computer based on Round Robin strategy
    fn select_computer(&self) -> Option<&EComputer> {
        self.computers.first() // Simple selection for demonstration; can be improved
    }

    // Receive and process a task and return the processing delay
    fn receive_task(&self, task: Task) -> Duration {
        if let Some(computer) = self.select_computer() {
            let delay = computer.process_task(&task);
            return delay;
        } else {
            return Duration::new(0, 0); // Return 0 delay if not processed
        }
    }
}

// Function to run the Edge/Dynamic experiment
fn run_experiment(client_count: usize, task_count: usize) -> Vec<Duration> {
    let mut manager = EComputerManager::new();

    // Create multiple edge computers and add them to the manager
    for i in 0..4 {
        let cpu_speed = (i + 1) as f64 * 1000.0; // Assume CPU speeds of 1000, 2000, 3000, 4000 MIPS
        manager.add_computer(EComputer::new(i, cpu_speed));
    }

    // Simulate clients sending tasks with a fixed large size for CPU-intensive tasks
    let task_size = 10000; // Set a large fixed size for CPU-intensive tasks
    let tasks: Vec<Task> = (1..=task_count)
        .map(|id| Task {
            id,
            size: task_size,
        }) // All tasks have the same size
        .collect();

    // Store delays for CDF calculation
    let mut delays = Vec::new();

    // Concurrently process tasks
    let manager_arc = Arc::new(Mutex::new(manager));

    let handles: Vec<_> = tasks
        .into_iter()
        .map(|task| {
            let manager_clone = Arc::clone(&manager_arc);
            thread::spawn(move || {
                let manager = manager_clone.lock().unwrap();
                manager.receive_task(task)
            })
        })
        .collect();

    // Wait for all threads to finish and collect delays
    for handle in handles {
        delays.push(handle.join().unwrap());
    }

    delays
}

// Function to calculate CDF from delays
fn calculate_cdf(delays: &[Duration]) -> Vec<(f64, f64)> {
    let mut sorted_delays: Vec<f64> = delays.iter().map(|d| d.as_secs_f64()).collect();

    sorted_delays.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let n = sorted_delays.len();

    sorted_delays
        .into_iter()
        .enumerate()
        .map(|(i, value)| (value, (i + 1) as f64 / n as f64))
        .collect()
}

fn main() {
    const CLIENT_COUNT: usize = 24; // Fixed number of clients
    const TASK_COUNT: usize = 10; // Each client sends this many tasks

    println!("Running experiment with {} clients...", CLIENT_COUNT);

    let delays = run_experiment(CLIENT_COUNT, TASK_COUNT * CLIENT_COUNT); // Total tasks sent by all clients

    // Calculate CDF from collected delays
    let cdf_data = calculate_cdf(&delays);

    // Output CDF data for analysis (you can modify this part to save or plot the data)
    println!("CDF Data:");
    for (delay, cdf) in cdf_data {
        println!("Delay: {:.2} seconds, CDF: {:.2}", delay, cdf);
    }
}

use rand::Rng;
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
        thread::sleep(delay); // Simulate processing delay
        delay
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

    // Receive and process a task
    fn receive_task(&self, task: Task) -> Duration {
        if let Some(computer) = self.select_computer() {
            let delay = computer.process_task(&task);
            println!(
                "Task {} processed by Computer {} with delay {:?}",
                task.id, computer.id, delay
            );
            return delay;
        } else {
            println!("No available e-computers to process Task {}", task.id);
            return Duration::new(0, 0); // Return 0 delay if not processed
        }
    }
}

// Function to run the Edge/Dynamic experiment
fn run_edge_dynamic(client_count: usize, task_count: usize) {
    let mut manager = EComputerManager::new();

    // Create multiple edge computers and add them to the manager
    for i in 0..4 {
        let cpu_speed = (i + 1) as f64 * 1000.0; // Assume CPU speeds of 1000, 2000, 3000, 4000 MIPS
        manager.add_computer(EComputer::new(i, cpu_speed));
    }

    // Simulate clients sending tasks
    let tasks: Vec<Task> = (1..=task_count)
        .map(|id| Task {
            id,
            size: rand::thread_rng().gen_range(100..=1000),
        }) // Random task sizes
        .collect();

    // Concurrently process tasks
    let manager_arc = Arc::new(Mutex::new(manager));

    let handles: Vec<_> = tasks
        .into_iter()
        .map(|task| {
            let manager_clone = Arc::clone(&manager_arc);
            thread::spawn(move || {
                let manager = manager_clone.lock().unwrap();
                manager.receive_task(task);
            })
        })
        .collect();

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }
}

// Function to run the Edge/Static experiment
fn run_edge_static(client_count: usize, task_count: usize) {
    let mut manager = EComputerManager::new();

    // Create multiple edge computers and add them to the manager
    for i in 0..4 {
        let cpu_speed = (i + 1) as f64 * 1000.0; // Assume CPU speeds of 1000, 2000, 3000, 4000 MIPS
        manager.add_computer(EComputer::new(i, cpu_speed));
    }

    // Simulate clients sending tasks directly to the closest executor without e-routers
    let tasks: Vec<Task> = (1..=task_count)
        .map(|id| Task {
            id,
            size: rand::thread_rng().gen_range(100..=1000),
        }) // Random task sizes
        .collect();

    // Process tasks directly without using e-routers
    for task in tasks {
        manager.receive_task(task);
    }
}

// Function to run the Distributed experiment with a powerful executor
fn run_distributed(client_count: usize, task_count: usize) {
    let powerful_computer = EComputer::new(0, 10000.0); // Single powerful executor with 10,000 MIPS

    let tasks: Vec<Task> = (1..=task_count)
        .map(|id| Task {
            id,
            size: rand::thread_rng().gen_range(100..=1000),
        }) // Random task sizes
        .collect();

    for task in tasks {
        let delay = powerful_computer.process_task(&task);
        println!(
            "Task {} processed by Distributed Computer {} with delay {:?}",
            task.id, powerful_computer.id, delay
        );
    }
}

fn main() {
    let client_counts = vec![1, 2, 4, 6]; // Different numbers of clients

    for &client_count in &client_counts {
        println!(
            "Running Edge/Dynamic experiment with {} clients...",
            client_count
        );
        run_edge_dynamic(client_count, 10); // Each client sends 10 tasks

        println!(
            "Running Edge/Static experiment with {} clients...",
            client_count
        );
        run_edge_static(client_count, 10); // Each client sends 10 tasks

        println!(
            "Running Distributed experiment with {} clients...",
            client_count
        );
        run_distributed(client_count, 10); // Each client sends 10 tasks

        println!();
    }
}

use rand::Rng; // For random number generation
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Define the ERouter struct
struct ERouter {
    e_table: Arc<Mutex<HashMap<String, Vec<(String, f64)>>>>, // e-table storing lambda functions and their corresponding e-computers with weights
    forward_count: Arc<Mutex<u64>>,                           // Count of forwards
}

impl ERouter {
    // Create a new ERouter
    fn new() -> Self {
        ERouter {
            e_table: Arc::new(Mutex::new(HashMap::new())),
            forward_count: Arc::new(Mutex::new(0)), // Initialize forward count
        }
    }

    // Add a lambda function to the e-table with multiple entries
    fn add_entry(&self, function_name: &str, e_computer: &str, weight: f64) {
        let mut table = self.e_table.lock().unwrap();
        table
            .entry(function_name.to_string())
            .or_insert(Vec::new())
            .push((e_computer.to_string(), weight));
    }

    // Select a destination e-computer
    fn select_destination(&self, function_name: &str) -> Option<(String, f64)> {
        let mut table = self.e_table.lock().unwrap();
        if let Some(computers) = table.get_mut(function_name) {
            if !computers.is_empty() {
                // Rotate the first computer to the end of the list for Round Robin
                let selected = computers.remove(0);
                computers.push(selected.clone());
                return Some(selected);
            }
        }
        None
    }

    // Handle a request from a client
    fn handle_request(&self, _: usize, function_name: &str) {
        if let Some(_) = self.select_destination(function_name) {
            let mut count = self.forward_count.lock().unwrap();
            *count += 1;
        }
    }

    // Function to report processing rate every second
    fn report_processing_rate(&self) {
        let forward_count_clone = Arc::clone(&self.forward_count);

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(1));
                let mut count = forward_count_clone.lock().unwrap();
                println!("Processing rate: {} forwards per second", *count);
                *count = 0; // Reset count after reporting
            }
        });
    }
}

// Simulate client behavior sending requests
fn client_simulation(client_id: usize, e_router: Arc<ERouter>, function_names: Vec<String>) {
    loop {
        let function_name =
            function_names[rand::thread_rng().gen_range(0..function_names.len())].clone();
        e_router.handle_request(client_id, &function_name);
    }
}

fn main() {
    let e_router = Arc::new(ERouter::new());

    // Generate 1000 diverse entries in the e-table
    for i in 0..6000000 {
        let lambda_function = format!("lambda_{}", i % 60000); // Create 100 unique lambda functions
        let e_computer = format!("e-computer-{}", i % 100); // Create 10 unique e-computers
        let weight = rand::thread_rng().gen_range(1.0..10.0); // Assign random weight between 1.0 and 10.0
        e_router.add_entry(&lambda_function, &e_computer, weight);
    }

    // Start reporting processing rate
    e_router.report_processing_rate();

    // Start client threads with random lambda requests
    let mut clients = vec![];
    let function_names: Vec<String> = (0..60000).map(|i| format!("lambda_{}", i)).collect(); // Collect all unique lambda functions

    for i in 1..=10 {
        // Start 10 clients
        let router_clone = Arc::clone(&e_router);
        let function_names_clone = function_names.clone();
        let client_thread =
            thread::spawn(move || client_simulation(i, router_clone, function_names_clone));
        clients.push(client_thread);
    }

    // Wait for all client threads to finish (in practice they run indefinitely)
    for client in clients {
        let _ = client.join();
    }
}

import time
import argparse
from client import EClient

class SingleThreadManager:
    def __init__(self, count):
        self.count = count
        self.process_times = []

    def run_client(self, e_router_port, workload, hop):
        for _ in range(self.count):
            client = EClient(e_router_port, workload, hop)
            process_time = client.send_task()
            
            # Store the process time
            self.process_times.append(process_time)
            print(f"Processed task with time: {process_time:.4f} seconds")
            
            # Wait 1 second before next request
            time.sleep(1)

    def start_client(self):
        # Single client configuration (port, workload, hop)
        client_config = (63943, 5123, 2)
        self.run_client(*client_config)

    def log_results(self):
        with open('process_times_single.log', 'w') as log_file:
            for time in self.process_times:
                log_file.write(f"{time:.4f}\n")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='Single Thread Client Manager for E-Router')
    parser.add_argument('--count', type=int, required=True,
                        help='Number of tasks to process')
    
    args = parser.parse_args()
    
    manager = SingleThreadManager(args.count)
    manager.start_client()
    manager.log_results()
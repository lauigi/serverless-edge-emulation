import threading
from client import EClient

class ClientManager:
    def __init__(self, count):
        self.count = count
        self.process_times = []
        self.clients_config = [
            (49769, 5000, 2),
        ]

    def run_client(self, e_router_port, workload, hop):
        for _ in range(self.count):
            client = EClient(e_router_port, workload, hop)
            process_time = client.send_task()
            
            # Store the process time
            self.process_times.append(process_time)
            print(f"Processed task with time: {process_time:.4f} seconds")

    def start_clients(self):

        threads = []
        
        for i in range(len(self.clients_config)):
            config = self.clients_config[i]
            thread = threading.Thread(target=self.run_client, args=config)
            threads.append(thread)
            thread.start()

        for thread in threads:
            thread.join()

    def log_results(self):
        with open(f'./per95/process-times-de-{len(self.clients_config)}.log', 'w') as log_file:
            for time in self.process_times:
                log_file.write(f"{time:.4f},\n")
        
if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description='Client Manager for E-Router')
    parser.add_argument('--count', type=int, required=True,
                        help='Number of tasks to process per client')
    
    args = parser.parse_args()
    
    manager = ClientManager(count=args.count)
    manager.start_clients()
    manager.log_results()

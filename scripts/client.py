import socket
import json
import time
import random

class EClient:
    def __init__(self, e_router_port, workload, hop):
        self.e_router_port = e_router_port
        self.workload = workload
        self.hop = hop

    def simulate_network_delay(self):
        # Simulate a random network delay between 0.1 and 0.5 seconds
        delay = random.uniform(0.1, 0.5)
        time.sleep(delay)

    def send_task(self):
        # Create a task
        task = {
            "id": f"task-{random.randint(1, 1000)}",
            "size": self.workload
        }

        # Record start time
        start_time = time.time()
        print(start_time)

        # Simulate network delay
        self.simulate_network_delay()

        # Connect to e-router
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.connect(('localhost', self.e_router_port))
            s.sendall(json.dumps(task).encode('utf-8'))

            # Wait for response
            response = s.recv(1024)
            end_time = time.time()

        # Calculate process time
        process_time = end_time - start_time
        return process_time

# Example usage of EClient (for testing purposes)
# client = EClient(e_router_port=12345, workload=100, hop=1)
# process_time = client.send_task()
# print(f"Process time: {process_time:.4f} seconds")

import random
import time
from threading import Thread, Lock

class ERouter:
    def __init__(self):
        self.e_table = {}  # save e-computer information
        self.lock = Lock()  # for thread safety

    def add_entry(self, function_name):
        """add lambda function to e-table"""
        with self.lock:
            if function_name not in self.e_table:
                self.e_table[function_name] = []

    def register_e_computer(self, function_name, e_computer):
        """register a new e-computer"""
        with self.lock:
            if function_name in self.e_table:
                self.e_table[function_name].append(e_computer)

    def select_destination(self, function_name):
        """select a destination e-computer for the function"""
        with self.lock:
            if function_name in self.e_table and self.e_table[function_name]:
                selected = self.e_table[function_name].pop(0)
                self.e_table[function_name].append(selected)
                return selected
            return None

    def handle_request(self, client_id, function_name):
        print(f"Client {client_id} requesting {function_name}...")
        destination = self.select_destination(function_name)
        if destination:
            print(f"Request for {function_name} forwarded to {destination} by Client {client_id}.")
            time.sleep(0.1)
            print(f"Client {client_id} received response for {function_name}.")
        else:
            print(f"No available e-computers for {function_name}.")

def client_simulation(client_id, e_router, function_name):
    """客户端模拟，发送请求"""
    while True:
        e_router.handle_request(client_id, function_name)
        time.sleep(random.uniform(0.5, 1.5))

def main():
    e_router = ERouter()
    
    lambda_function = "my_lambda"
    e_router.add_entry(lambda_function)

    for i in range(5):
        e_router.register_e_computer(lambda_function, f"e-computer-{i+1}")

    clients = []
    for i in range(10):
        client_thread = Thread(target=client_simulation, args=(i+1, e_router, lambda_function))
        clients.append(client_thread)
        client_thread.start()

    for client in clients:
        client.join()

if __name__ == "__main__":
    main()

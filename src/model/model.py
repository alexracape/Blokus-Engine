from xmlrpc.server import SimpleXMLRPCServer, SimpleXMLRPCRequestHandler
import socket

import numpy as np
import torch

PORT = 8000

class SimpleModelServer(SimpleXMLRPCServer):

    def __init__(self, addr, request_handler=SimpleXMLRPCRequestHandler):
        super().__init__(addr, requestHandler=request_handler)
        self.quit: bool = False

    def run(self):
        self.quit = False
        while not self.quit:
            self.handle_request()

def shutdown_server(server):
    server.quit = True


class Model():
    """Model to be hosted by the server"""

    def __init__(self, model_path=None) -> None:
        self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")

        if model_path:
            self.model = torch.load(model_path, map_location=self.device)
        else:
            self.model = torch.nn.Sequential(
                torch.nn.Linear(1, 10),
                torch.nn.ReLU(),
                torch.nn.Linear(10, 1)
            ).to(self.device)

        self.optimizer = torch.optim.Adam(self.model.parameters(), lr=0.01)
        self.loss = torch.nn.MSELoss() # Might need to change to custom

    def predict(self, x: np.ndarray) -> np.ndarray:
        """Takes game state and predicts...
        1. The probability distribution over moves - the policy
        2. The expected outcome of the game - the value
        """

        self.model.eval()
        x = torch.tensor(x, dtype=torch.float32).to(self.device)
        with torch.no_grad():
            return self.model(x).cpu().numpy()

    def train_batch(self, X_train, Y_train):
        self.optimizer.zero_grad()                  # Flush memory
        pred = self.model(X_train)                  # Get predictions
        batch_loss = self.loss(pred, Y_train)       # Compute loss
        batch_loss.backward()                       # Compute gradients
        self.optimizer.step()                       # Make a GD step
        return batch_loss.detach().cpu().numpy()

    def train(self, data):
        print(f"Training {data}")
        self.model.train()


def main():

    addr = (socket.gethostbyname(socket.gethostname()), PORT)
    request_handler = SimpleXMLRPCRequestHandler
    with SimpleModelServer(addr, request_handler) as server:
        server.register_introspection_functions()
        server.register_instance(Model())
        server.register_function(shutdown_server, "shutdown")
        print(f"Server listening on {addr}")
        server.run()


if __name__ == "__main__":
    main()

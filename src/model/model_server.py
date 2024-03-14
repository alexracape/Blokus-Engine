
from concurrent import futures
import grpc
import model_pb2
import model_pb2_grpc

import numpy as np
import torch
from torch.nn import Linear, ReLU, Conv2d


class BlokusModelServicer(model_pb2_grpc.BlokusModelServicer):
    """Servicer for the Blokus model using gRPC
    
    The model is a CNN that takes input of size 20x20x4 + 21x4 + 4. 
    This is from 4 planes for each player's pieces on the board then each
    player's remaining pieces and the player who's turn it is.
    The model outputs a policy and a value. The policy is a probability
    distribution over the possible moves and the value is the expected
    outcome of the game for each player.
    """

    def __init__(self, model_path=None):

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
        self.loss = torch.nn.MSELoss().to(self.device)  # Might need to change to custom

    def Predict(self, request, context):
        
        print("Predicting...")
        state = np.array(request.data).reshape(20, 20, 28)
        state = torch.tensor(state, dtype=torch.bool).to(self.device)
        with torch.no_grad():
            policy, values = self.model(state).numpy()
        print(policy, values)
        return model_pb2.Prediction(policy=policy, value=values)
    

    def Train(self, request, context):
        data = np.array(request.data).reshape(-1, 1, 14, 14)
        data = torch.tensor(data, dtype=torch.float32)
        target = np.array(request.target).reshape(-1, 1)
        target = torch.tensor(target, dtype=torch.float32)
        loss = self.model.train(data, target)
        return model_pb2.Status(value=loss)


def serve():
    print("Starting up server...")
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    model_pb2_grpc.add_BlokusModelServicer_to_server(BlokusModelServicer(), server)
    server.add_insecure_port("[::]:50051")
    server.start()
    server.wait_for_termination()


if __name__ == "__main__":
    serve()

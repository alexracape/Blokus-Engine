
from concurrent import futures
import grpc
import model_pb2
import model_pb2_grpc

import numpy as np
import torch
from torch.nn import Linear, ReLU, Conv2d
from torchsummary import summary


class BlokusModel(torch.nn.Module):
    """ML model that will predict policy and value for game states"""

    def __init__(self):
        super(BlokusModel, self).__init__()

        self.conv1 = Conv2d(4, 64, kernel_size=5, stride=1, padding=2)
        self.conv2 = Conv2d(64, 128, kernel_size=3, stride=1, padding=1)
        self.conv3 = Conv2d(128, 1, kernel_size=3, stride=1, padding=1)

        self.fc1 = Linear(20*20, 512)
        self.fc2 = Linear(512, 256)

        self.policy_head = Linear(256, 21)
        self.value_head = Linear(256, 4)
        
        self.relu = ReLU()

    def forward(self, board, pieces, player):
        
        x = self.relu(self.conv1(board))
        x = self.relu(self.conv2(x))
        x = self.relu(self.conv3(x))

        x = x.view(-1, 20*20)
        x = self.relu(self.fc1(x))
        x = self.relu(self.fc2(x))

        policy = self.policy_head(x)
        value = self.value_head(x)

        return policy, value


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
            self.model = BlokusModel().to(self.device)

        summary(self.model, [(4, 20, 20), (1, 21, 4), (1, 1, 1)])
        self.optimizer = torch.optim.Adam(self.model.parameters(), lr=0.01)
        self.loss = torch.nn.MSELoss().to(self.device)  # Might need to change to custom

    def Predict(self, request, context):
        
        print("Predicting...")
        board = np.array(request.board).reshape(4, 20, 20)
        pieces = np.array(request.pieces).reshape(4, 21)
        player = request.player

        board = torch.tensor(board, dtype=torch.float32).to(self.device)

        with torch.no_grad():
            policy, values = self.model(board, pieces, player)
        print(policy, values)
        return model_pb2.Prediction(policy=policy[0], value=values[0])
    

    def Train(self, request, context):
        loss = 0
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

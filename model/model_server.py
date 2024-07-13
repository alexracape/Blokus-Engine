from concurrent import futures
import os
import logging

import grpc
import numpy as np
import torch
from torch.nn import Linear, ReLU, Conv2d
from torchsummary import summary
from dotenv import load_dotenv

import model_pb2
import model_pb2_grpc


# Configure logging
logging.basicConfig(level=logging.INFO)

# Load the .env file once if the script is not running in a Docker environment
if not os.environ.get("DOCKER_ENV"):
    load_dotenv()

# Function to load environment variables
def load_env_var(key, cast_type=str, default=None):
    value = os.getenv(key, default)
    
    if not value:
        logging.warning(f"Environment variable {key} not found, using default: {default}")
        return default
    
    try:
        return cast_type(value)
    except ValueError:
        logging.error(f"Error casting environment variable {key}. Using default: {default}")
        return default
        

# Load environment variables
PORT = load_env_var("PORT")
BUFFER_CAPACITY = load_env_var("BUFFER_CAPACITY", int)
LEARNING_RATE = load_env_var("LEARNING_RATE", float)
BATCH_SIZE = load_env_var("BATCH_SIZE", int)
TRAINING_STEPS = load_env_var("TRAINING_STEPS", int)
NUM_CLIENTS = load_env_var("NUM_CLIENTS", int)
GAMES_PER_CLIENT = load_env_var("GAMES_PER_CLIENT", int)
GAMES_PER_ROUND = NUM_CLIENTS * GAMES_PER_CLIENT
TRAINING_ROUNDS = load_env_var("TRAINING_ROUNDS", int)
DIM = 20

if None in [PORT, BUFFER_CAPACITY, LEARNING_RATE, BATCH_SIZE, TRAINING_STEPS, NUM_CLIENTS, GAMES_PER_CLIENT, TRAINING_ROUNDS]:
    logging.error("One or more critical environment variables are missing.")
    

class BlokusModel(torch.nn.Module):
    """ML model that will predict policy and value for game states"""

    def __init__(self):
        super(BlokusModel, self).__init__()

        self.conv1 = Conv2d(5, 64, kernel_size=5, stride=1, padding=2)
        self.conv2 = Conv2d(64, 128, kernel_size=3, stride=1, padding=1)
        self.conv3 = Conv2d(128, 1, kernel_size=3, stride=1, padding=1)

        self.fc1 = Linear(DIM * DIM, 512)
        self.fc2 = Linear(512, 256)

        self.policy_head = Linear(256, 400)
        self.value_head = Linear(256, 4)

        self.relu = ReLU()

    def forward(self, boards):
        """Get the policy and value for the given board state

        For now, the board is represented by a 20x20x5 tensor where the first 4 channels are
        binary boards for each player's pieces on the board. The 5th channel is a binary board
        with the valid moves for the current player. For now, I'm just going to use the boards.
        It is unclear why the player color is needed in the state.
        """
        # print(board.shape)
        x = self.relu(self.conv1(boards))
        x = self.relu(self.conv2(x))
        x = self.relu(self.conv3(x))

        x = x.view(-1, DIM * DIM)
        x = self.relu(self.fc1(x))
        x = self.relu(self.fc2(x))

        policy = self.policy_head(x)
        value = self.value_head(x)

        if len(boards.shape) == 3:
            mask = boards[4].flatten()
        else:
            mask = boards[:, 4, :, :].view(boards.size(0), -1)
        policy = policy * mask

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
        self.buffer = ReplayBuffer()
        self.training_round = 0
        self.num_saves = 0
        if model_path:
            self.model = torch.load(model_path, map_location=self.device)
        else:
            self.model = BlokusModel().to(self.device)

        # summary(self.model, [(5, 20, 20), (1, 1, 1)]) # for some reason dimension when summarizing is (2, 5, 20, 20)
        self.optimizer = torch.optim.Adam(self.model.parameters(), lr=LEARNING_RATE)
        self.policy_loss = torch.nn.CrossEntropyLoss().to(self.device)
        self.value_loss = torch.nn.MSELoss().to(self.device)

    def Predict(self, request, context):
        boards = np.array(request.boards).reshape(5, DIM, DIM)
        boards = torch.tensor(boards, dtype=torch.float32).to(self.device)
        player = request.player

        with torch.no_grad():
            policy, values = self.model(boards)
        print(values)
        return model_pb2.Target(policy=policy[0], value=values[0])


    def Check(self, request, context):
        """Check in with the server to see if it is on the next round of training

        This is used intermitently by the client to check if it is in sync
        with the server. If the server is on the next round of training, the
        client will start the next round of self-play / data generation.
        Returns the current training round.
        """

        return model_pb2.Status(code=self.training_round)

    def Save(self, request, context):
        """Store data in the replay buffer"""

        self.buffer.add(request.history, request.policies, request.values)
        print("Buffer size: ", len(self.buffer.buffer))
        # self.Train() # For testing
        self.num_saves += 1
        if self.num_saves == GAMES_PER_ROUND:
            self.Train()
            self.num_saves = 0
        if self.training_round == TRAINING_ROUNDS:
            torch.save(self.model, "model.pt")

        return model_pb2.Status(code=0)


    def Train(self, batch_size=BATCH_SIZE, training_steps=TRAINING_STEPS):
        """Train the model using the data in the replay buffer"""

        for _ in range(training_steps):
            print("Training step: ", _)

            # Get a batch of data from the replay buffer
            batch = self.buffer.sample(batch_size)
            inputs, policies, values = zip(*batch)
            inputs = torch.stack(inputs).to(self.device)
            policies = torch.stack(policies).to(self.device)
            values = torch.stack(values).to(self.device)

            # Train the model
            self.optimizer.zero_grad()
            policy, value = self.model(inputs)
            policy_loss = self.policy_loss(policy, policies)
            value_loss = self.value_loss(value, values)
            loss = policy_loss + value_loss
            loss.backward()
            self.optimizer.step()

        self.training_round += 1
        return model_pb2.Status(code=0)


class ReplayBuffer:
    """Buffer for storing game states for training the model"""

    def __init__(self, capacity=BUFFER_CAPACITY):
        self.capacity = capacity
        self.buffer = []
        self.total_moves = 0

    def add(self, history, policies, values):
        if len(self.buffer) >= self.capacity:
            old = self.buffer.pop(0)
            self.total_moves -= len(old[0])
        self.buffer.append((history, policies, values))
        self.total_moves += len(history)

    def sample(self, batch_size):
        weights = [len(game[0]) / self.total_moves for game in self.buffer]
        games = np.random.choice(len(self.buffer), batch_size, p=weights)
        return [self.training_data(self.buffer[i]) for i in games]

    def training_data(self, game):

        # Get random move from the game
        i = np.random.randint(len(game[0]))

        # Get key data from the game
        history, policies, values = game
        state = torch.zeros(5, DIM, DIM, dtype=torch.float32)
        policy = torch.zeros(DIM * DIM, dtype=torch.float32)
        values = torch.tensor(values, dtype=torch.float32)

        # Reconstruct state representation
        for j in range(i):
            player, tile = history[j].player, history[j].tile
            row, col = tile // DIM, tile % DIM
            state[player, row, col] = True

        # reconstruct policy representation
        for action in policies[i].probs:
            tile, prob = action.action, action.prob
            policy[tile] = prob

        # Apply random transform for data augmentation to both state and policy
        horizontal = np.random.choice([True, False])
        vertical = np.random.choice([True, False])
        if horizontal:
            state = state.flip(0)
            policy = policy.view(DIM, DIM).flip(0).flatten()
        if vertical:
            state = state.flip(1)
            policy = policy.view(DIM, DIM).flip(1).flatten()

        return state, policy, values


def serve():
    print("Starting up server...", flush=True)
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    model_pb2_grpc.add_BlokusModelServicer_to_server(BlokusModelServicer(), server)
    server.add_insecure_port(f"[::]:{PORT}")
    server.start()
    server.wait_for_termination()


if __name__ == "__main__":
    serve()

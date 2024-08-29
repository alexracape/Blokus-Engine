import argparse
import multiprocessing as mp
import logging
import os
import time
from queue import Empty

import wandb
from tqdm import trange, tqdm
import torch
from torchrl.data import ReplayBuffer, LazyTensorStorage
from tensordict import tensorclass

from blokus_self_play import play_training_game
from resnet import ResNet

DIM = 20
MODEL_PATH = "./weights"

@tensorclass
class Data:
    states: torch.Tensor
    policies: torch.Tensor
    scores: torch.Tensor


def get_batch(size, queue, device):
    ids = []
    items = []
    for _ in range(size):
        try:
            id, input = queue.get_nowait()
            ids.append(id)
            items.append(input)
        except Empty as e:
            break

    return ids, torch.tensor(items).view(-1, 5, DIM, DIM).to(device)


def empty_queue(queue, device):
    ids = []
    items = []
    while True:
        try:
            id, input = queue.get(block=False)
            ids.append(id)
            items.append(input)
        except Empty as e:
            break

    return ids, torch.tensor(items).view(-1, 5, DIM, DIM).to(device)


def handle_inference_batch(model, device, inference_queue, pipes_to_workers):
    """Process batches of inputs from the self-play games

    Tries to create a batch of size num_workers // 2 from the inference queue.
    If this runs for too long, there are likely stragglers in the queue and we
    should just empty the queue with what is left. All batches are sent to the
    GPU for processing and the outputs are sent back to the appropriate worker.
    """

    time.sleep(.001)
    ids, batch = empty_queue(inference_queue, device)
    num_requests = len(ids)
    if num_requests == 0:
        return 0

    # Query the model for the batch of inputs
    with torch.no_grad():
        policies, values = model(batch)

    # Send the outputs to the appropriate worker
    for i, id in enumerate(ids):
        response = (policies[i].cpu().tolist(), values[i].cpu().tolist())
        pipes_to_workers[id].send(response)

    return num_requests


def save(game, buffer: ReplayBuffer,):
    """Save the game data to the replay buffer"""

    # Allocate space for the data
    history, policies, values = game
    num_moves = len(history)
    logging.debug(f"Saving game with {num_moves} moves to the replay buffer")

    state_data = torch.zeros(num_moves, 5, DIM, DIM, dtype=torch.float32)
    policy_data = torch.zeros(num_moves, DIM * DIM, dtype=torch.float32)
    value_data = torch.tensor(values, dtype=torch.float32).repeat(num_moves, 1)

    # For each move from this game, update the state and policy
    new_state = torch.zeros(5, DIM, DIM, dtype=torch.float32)
    for i, (move, policy) in enumerate(zip(history, policies)):

        # Keep running track of state in new_state
        player, tile = move
        row, col = tile // DIM, tile % DIM
        new_state[player, row, col] = 1

        # Shift the state to the correct player's perspective
        state_data[i] = torch.cat((new_state[player:], new_state[:player]), dim=0)

        # Update the policy for this move
        for element in policy:
            action, prob = element
            policy_data[i, action] = prob

            # Update which squares are legal on this move
            row, col = action // DIM, action % DIM
            state_data[i, 4, row, col] = 1

        # Rotate state and policy so perspective is the same
        state_data[i] = torch.rot90(state_data[i], k=player, dims=(1, 2))
        policy_data[i] = torch.rot90(policy_data[i].reshape(DIM, DIM), k=player).reshape(-1)

    data = Data(
        states = state_data,
        policies = policy_data,
        scores = value_data,
        batch_size = [num_moves]
    )
    buffer.extend(data)


def train(step, model, buffer, optimizer, policy_loss, value_loss, device, testing):
    """Train the model on a batch of data from the replay buffer"""

    # Get a batch of data from the replay buffer
    batch = buffer.sample()
    inputs = batch.get("states").to(device)
    policies = batch.get("policies").to(device)
    values = batch.get("scores").to(device)

    # Train the model
    optimizer.zero_grad()
    policy, value = model(inputs)
    policy_loss = policy_loss(policy, policies)
    value_loss = value_loss(value, values)
    loss = policy_loss + value_loss
    loss.backward()
    optimizer.step()

    # Store training statistics
    if not testing:
        wandb.log({"policy_loss": policy_loss, "value_loss": value_loss}, step=step)


def main():
    """Train the model

    Creates the model then spawns multiple processes to generate
    training data through self-play. The training data is then used
    to train the model. This process is repeated until the model
    reaches a certain number of training steps.
    """

    # Parse args for number of CPUs and testing mode
    parser = argparse.ArgumentParser(description="Training the Blokus Deep Neural Network with Self-Play")
    parser.add_argument('--test', action='store_true', help="Run the program in testing mode")
    parser.add_argument('--cpus', type=int, default=1, help="Number of CPUs to use (default: 1)")
    args = parser.parse_args()
    logging.info(f"Using {args.cpus} CPUs")
    logging.info(f"Running in {'test' if args.test else 'full power'} mode")

    # Load environment variables
    device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
    logging.info(f"Using device: {device}")
    if args.test:
        config = TestConfig(args.cpus)
    else:
        config = Config(args.cpus)


    # Create the model, optimizer, and loss
    model = ResNet(config.nn_depth, config.nn_width, config.custom_filters).to(device)
    optimizer = torch.optim.Adam(model.parameters(), lr=config.learning_rate)
    policy_loss = torch.nn.CrossEntropyLoss().to(device)
    value_loss = torch.nn.MSELoss().to(device)

    # Configure Weights and Biases
    if not args.test:
        wandb.login()
        wandb.init(project="blokus", config=config.to_dict())
        wandb.watch(model, log_freq=100)

    # Set up replay buffer
    buffer = ReplayBuffer(
        storage=LazyTensorStorage(config.buffer_capacity),
        batch_size=config.batch_size
    )

    # Train the model
    global_step = 0
    for round in trange(config.training_rounds):

        # Create the queues and pipes
        manager = mp.Manager()
        request_queue = manager.Queue(maxsize=config.cpus * config.games_per_cpu)
        pipes_to_model = []
        pipes_to_workers = []
        for i in range(config.games_per_round()):
            a, b = mp.Pipe()
            pipes_to_model.append(a)
            pipes_to_workers.append(b)

        # Generate spawn asynchronous self-play processes
        with mp.get_context("spawn").Pool(config.cpus) as pool:
            game_data = pool.starmap_async(
                play_training_game,
                [(config.games_per_worker, id, config, request_queue, pipes_to_model[id]) for id in range(config.games_per_round())]
            )

            # Start handling inference requests
            total_requests_ish = config.requests_per_round()
            pbar = tqdm(total=total_requests_ish, desc=f"Self-Play Requests Round {round}")
            while not game_data.ready():
                num_requests = handle_inference_batch(model, device, request_queue, pipes_to_workers)
                pbar.update(num_requests)
            pbar.close()

            # Save the game data to the replay buffer
            for worker_games in game_data.get():
                for game in worker_games:
                    save(game, buffer)

        # Train the model
        for step in trange(config.training_steps, desc=f"Training round {round}", leave=False):
            train(global_step, model, buffer, optimizer, policy_loss, value_loss, device, args.test)
            global_step += 1
        torch.save(model.state_dict(), f"{MODEL_PATH}/model_{round}.pt")

    # Clean up
    logging.info("Training complete")



class Config:
    """Configuration for training the model

    AlphaZero used the following values for training:
        buffer_capacity = 1000000 games
        learning_rate = 0.01 -> 0.0001 with a scheduler
        batch_size = 2048
        training_steps = 700000
        num_workers = 5000
        sims_per_move = 800
        sample_moves = 30
        c_base = 19652
        c_init = 1.25
        dirichlet_alpha = 0.3
        exploration_fraction = 0.25

    ~2.5 hours per round right now
    TODO: Figure out how to tune this
    """

    def __init__(self, num_cpus):
        self.training_rounds = 10

        self.buffer_capacity = 500000
        self.learning_rate = 0.01
        self.batch_size = 512
        self.training_steps = 10000
        self.cpus = num_cpus
        self.games_per_cpu = 4
        self.games_per_worker = 1

        self.custom_filters = True
        self.nn_width = 256
        self.nn_depth = 10

        self.sims_per_move = 100
        self.sample_moves = 30
        self.c_base = 19652
        self.c_init = 1.25
        self.dirichlet_alpha = 0.3
        self.exploration_fraction = 0.25

    def to_dict(self):
        return self.__dict__

    def games_per_round(self):
        return self.cpus * self.games_per_cpu * self.games_per_worker

    def requests_per_round(self):
        return self.games_per_round() *  DIM**2 * (self.sims_per_move + 2)


class TestConfig(Config):
    """Configuration with testing values to speed things up"""

    def __init__(self, num_cpus):
        self.training_rounds = 2

        self.buffer_capacity = 500000
        self.learning_rate = 0.01
        self.batch_size = 64
        self.training_steps = 10
        self.cpus = num_cpus
        self.games_per_cpu = 4
        self.games_per_worker = 1

        self.custom_filters = True
        self.nn_width = 16
        self.nn_depth = 2

        self.sims_per_move = 2
        self.sample_moves = 30
        self.c_base = 19652
        self.c_init = 1.25
        self.dirichlet_alpha = 0.3
        self.exploration_fraction = 0.25


if __name__ == '__main__':
    logging.basicConfig(level=logging.INFO)
    main()

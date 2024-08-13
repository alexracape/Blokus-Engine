import multiprocessing as mp
import logging
import os
import time
from queue import Empty

import numpy as np
import pandas as pd
from tqdm import trange
import torch
from torchrl.data import ReplayBuffer, LazyTensorStorage
from tensordict import tensorclass

from blokus_self_play import generate_game_data
from resnet import ResNet

DIM = 20
STATS_PATH = "./data/training_stats.csv"
MODEL_PATH = "./models"

@tensorclass
class Data:
    states: torch.Tensor
    policies: torch.Tensor
    scores: torch.Tensor


def get_batch(queue, device):
    ids = []
    items = []
    while True:
        try:
            id, input = queue.get_nowait()
            input = torch.tensor(input).reshape(5, DIM, DIM).to(device)
            ids.append(id)
            items.append(input)
        except Empty as e:
            break

    if not ids:
        return [], torch.tensor([])

    return ids, torch.stack(items)


def handle_inference_batch(model, device, inference_queue, pipes_to_workers):
    """Process batches of inputs from the self-play games"""

    # Pause then get the next batch of inputs
    ids, batch = get_batch(inference_queue, device)
    if not ids:
        return

    # Query the model for the batch of inputs
    with torch.no_grad():
        policies, values = model(batch)

    # Send the outputs to the appropriate worker
    for i, id in enumerate(ids):
        response = (policies[i].cpu().tolist(), values[i].cpu().tolist())
        pipes_to_workers[id].send(response)


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

    data = Data(
        states = state_data,
        policies = policy_data,
        scores = value_data,
        batch_size = [num_moves]
    )
    buffer.extend(data)


def train(step, model, buffer, optimizer, policy_loss, value_loss, device, stats):
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
    row = pd.DataFrame([{
        "round": step,
        "loss": loss.item(),
        "value_loss": value_loss.item(),
        "policy_loss": policy_loss.item(),
        "buffer_size": len(buffer)
    }])
    if stats.empty:
        stats = row
    else:
        stats = pd.concat([stats, row])
    stats.to_csv(STATS_PATH)


def main(num_cpus):
    """Train the model

    Creates the model then spawns multiple processes to generate
    training data through self-play. The training data is then used
    to train the model. This process is repeated until the model
    reaches a certain number of training steps.
    """

    # Load environment variables
    config = Config(num_cpus)
    device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
    logging.info(f"Using device: {device}")

    # Create the model, optimizer, and loss
    model = ResNet(config.nn_depth, config.nn_width).to(device)
    optimizer = torch.optim.Adam(model.parameters(), lr=config.learning_rate)
    policy_loss = torch.nn.CrossEntropyLoss().to(device)
    value_loss = torch.nn.MSELoss().to(device)

    # Set up replay buffer
    buffer = ReplayBuffer(
        storage=LazyTensorStorage(config.buffer_capacity),
        batch_size=config.batch_size
    )

    # Set up stats for tracking training progress
    stats = pd.DataFrame(columns=["round", "loss", "value_loss", "policy_loss", "buffer_size"])
    training_round = 0
    num_saves = 0

    # Create the queues and pipes
    manager = mp.Manager()
    request_queue = manager.Queue()
    pipes_to_model = []
    pipes_to_workers = []
    for i in range(config.num_workers):
        a, b = mp.Pipe()
        pipes_to_model.append(a)
        pipes_to_workers.append(b)

    for round in trange(config.training_rounds):

        # Generate spawn asynchronous self-play processes
        with mp.Pool(config.num_workers) as pool:
            game_data = pool.starmap_async(
                generate_game_data,
                [(config.games_per_worker, id, config, request_queue, pipes_to_model[id]) for id in range(config.num_workers)]
            )

            # Start handling inference requests
            while not game_data.ready():
                time.sleep(config.inference_interval)
                handle_inference_batch(model, device, request_queue, pipes_to_workers)

            # Save the game data to the replay buffer
            for worker_games in game_data.get():
                for game in worker_games:
                    save(game, buffer)

        # Train the model
        pbar = trange(config.training_steps, desc=f"Training round {round}", leave=False)
        for step in trange(config.training_steps):
            train(step, model, buffer, optimizer, policy_loss, value_loss, device, stats)
        pbar.close()
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
    """

    def __init__(self, num_cpus=1):
        self.training_rounds = 2
        self.buffer_capacity = 500000
        self.learning_rate = 0.01
        self.batch_size = 64
        self.inference_interval = .001  # seconds
        self.training_steps = 10
        self.num_workers = num_cpus
        self.games_per_worker = 2
        self.rounds = 2
        self.nn_width = 32
        self.nn_depth = 2
        self.sims_per_move = 5
        self.sample_moves = 30
        self.c_base = 19652
        self.c_init = 1.25
        self.dirichlet_alpha = 0.3
        self.exploration_fraction = 0.25


if __name__ == '__main__':
    logging.basicConfig(level=logging.INFO)
    num_cpus = int(os.environ.get("SLURM_NPROCS", 2))
    logging.info(f"Number of CPUs: {num_cpus}")

    main(num_cpus)

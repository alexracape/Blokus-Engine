"""Testing a trained model against another"""

import sys
import torch.multiprocessing as mp
import logging
import os
import time
from queue import Empty

from tqdm import trange, tqdm
import torch
from torchrl.data import ReplayBuffer, LazyTensorStorage
from tensordict import tensorclass

from resnet import ResNet
from training import TestConfig, handle_inference_batch
from blokus_self_play import play_test_game

TEST_GAMES = 10
DIM = 20


def main():

    # Parse args for number of CPUs and testing mode
    first_model_path = sys.argv[1]
    second_model_path = sys.argv[2]
    logging.info(f"Testing with {TEST_GAMES} games")
    config = TestConfig(num_cpus=8)

    # Load environment variables
    device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
    logging.info(f"Using device: {device}")

    # Load models and set up loss and optimizer
    model = ResNet(2, 16)
    model.load_state_dict(torch.load(first_model_path, weights_only=True))
    model.to(device)
    model.eval()
    baseline = ResNet(1, 16, False).to(device)
    optimizer = torch.optim.Adam(model.parameters(), lr=config.learning_rate)
    policy_loss = torch.nn.CrossEntropyLoss().to(device)
    value_loss = torch.nn.MSELoss().to(device)

    # Create the queues and pipes
    manager = mp.Manager()
    model_queue = manager.Queue(maxsize=TEST_GAMES)
    baseline_queue = manager.Queue(maxsize=TEST_GAMES)
    pipes_to_model = []
    pipes_to_workers = []
    for i in range(TEST_GAMES):
        a, b = mp.Pipe()
        pipes_to_model.append(a)
        pipes_to_workers.append(b)

    # Generate spawn asynchronous self-play processes
    with mp.Pool(config.cpus) as pool:
        game_data = pool.starmap_async(
            play_test_game,
            [(id, model_queue, baseline_queue, pipes_to_model[id]) for id in range(TEST_GAMES)]
        )

        # Start handling inference requests
        total_requests_ish = TEST_GAMES * DIM * DIM
        pbar = tqdm(total=total_requests_ish, desc=f"Testing Requests Round {round}")
        while not game_data.ready():
            model_requests = handle_inference_batch(model, device, model_queue, pipes_to_workers)
            baseline_requests = handle_inference_batch(baseline, device, baseline_queue, pipes_to_workers)
            pbar.update(model_requests + baseline_requests)
        pbar.close()

    # Clean up
    logging.info("Test Run Complete")
    logging.info(f"Score = {sum(game_data.get())}/{TEST_GAMES}")


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    main()

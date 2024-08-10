# import logging


# class BlokusModelServicer(model_pb2_grpc.BlokusModelServicer):
#     """Servicer for the Blokus model using gRPC

#     The model is a CNN that takes input of size 20x20x4 + 21x4 + 4.
#     This is from 4 planes for each player's pieces on the board then each
#     player's remaining pieces and the player who's turn it is.
#     The model outputs a policy and a value. The policy is a probability
#     distribution over the possible moves and the value is the expected
#     outcome of the game for each player.
#     """

#     def __init__(self, model_path=None, batch_duration=0.01):
#         """Create the model server

#         Args:
#             model_path (str): Path to a saved model to load
#             batch_duration (float): Duration to wait before processing batches during self-play
#         """

#         # Set the device
#         self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
#         logging.info(f"Using device: {self.device}")

#         # Load or create the model
#         if model_path:
#             self.model = torch.load(model_path, map_location=self.device)
#         else:
#             self.model = ResNet(NN_BLOCKS, NN_WIDTH).to(self.device)

#         # Set up the optimizer and loss functions
#         self.optimizer = torch.optim.Adam(self.model.parameters(), lr=LEARNING_RATE)
#         self.policy_loss = torch.nn.CrossEntropyLoss().to(self.device)
#         self.value_loss = torch.nn.MSELoss().to(self.device)

#         # Set up the replay buffer and stats to be stored
#         self.buffer = ReplayBuffer(
#             storage=LazyTensorStorage(BUFFER_CAPACITY),
#             batch_size=BATCH_SIZE
#         )

#         # Important state while training
#         self.stats = pd.DataFrame(columns=["round", "loss", "value_loss", "policy_loss", "buffer_size"])
#         self.training_round = 0
#         self.num_saves = 0

#         # Set up threading for batching client requests during self-play
#         self.self_playing = True
#         self.lock = threading.Lock()
#         self.requests = []
#         self.responses = []
#         self.batch_duration = batch_duration
#         self.processing_thread = None
#         self.start_processing()


#     def start_processing(self):
#         """Start threading for processing self-play batches"""
#         self.self_playing = True
#         self.processing_thread = threading.Thread(target=self.process_batches)
#         self.processing_thread.start()

#     def stop_processing(self):
#         """Clean up threading for processing self-play batches"""
#         self.self_playing = False
#         if self.processing_thread and self.processing_thread.is_alive():
#             self.processing_thread.join()


#     def process_batches(self):
#         """Periodically process the batches of data from the clients"""

#         while self.self_playing:
#             time.sleep(self.batch_duration)

#             # Acquire the lock and get the batch of requests
#             with self.lock:
#                 if not self.requests:
#                     continue

#                 batch = torch.stack(self.requests)
#                 response_futures = self.responses
#                 self.requests = []
#                 self.responses = []

#             # Query the model
#             with torch.no_grad():
#                 policies, values = self.model(batch)

#             # Set the responses for each request
#             for i, response_future in enumerate(response_futures):
#                 response = model_pb2.Target(policy=policies[i], value=values[i])
#                 response_future.result = response
#                 response_future.set()


#     def Predict(self, request, context):
#         boards = torch.tensor(request.boards, dtype=torch.float32).reshape(5, DIM, DIM).to(self.device)
#         player = request.player  # Not used yet

#         with self.lock:
#             self.requests.append(boards)

#             # Hold the current thread until the response is ready
#             response_future = threading.Event()
#             self.responses.append(response_future)

#         # Wait for the batch processing thread to handle this request
#         response_future.wait()
#         assert hasattr(response_future, "result"), "Response future was not set"
#         return response_future.result


#     def Check(self, request, context):
#         """Check in with the server to see if it is on the next round of training

#         This is used intermitently by the client to check if it is in sync
#         with the server. If the server is on the next round of training, the
#         client will start the next round of self-play / data generation.
#         Returns the current training round.
#         """

#         return model_pb2.Status(code=self.training_round)

#     def Save(self, request, context):
#         """Store data in the replay buffer"""

#         # Allocate space for the data
#         num_moves = len(request.history)
#         states = torch.zeros(num_moves, 5, DIM, DIM, dtype=torch.float32)
#         policies = torch.zeros(num_moves, DIM * DIM, dtype=torch.float32)
#         scores = torch.tensor(request.values, dtype=torch.float32).repeat(num_moves, 1)

#         # For each move from this game, update the state and policy
#         new_state = torch.zeros(5, DIM, DIM, dtype=torch.float32)
#         for i, (move, policy) in enumerate(zip(request.history, request.policies)):

#             # Keep running track of state in new_state
#             player, tile = move.player, move.tile
#             row, col = tile // DIM, tile % DIM
#             new_state[player, row, col] = 1

#             # Shift the state to the correct player's perspective
#             states[i] = torch.cat((new_state[player:], new_state[:player]), dim=0)

#             # Update the policy for this move
#             for element in policy.probs:
#                 action, prob = element.action, element.prob
#                 policies[i, action] = prob

#                 # Update which squares are legal on this move
#                 row, col = action // DIM, action % DIM
#                 states[i, 4, row, col] = 1

#         data = Data(
#             states = states,
#             policies = policies,
#             scores = scores,
#             batch_size = [num_moves]
#         )
#         self.buffer.extend(data)
#         print("Buffer size: ", len(self.buffer))

#         # Save the model after every round of training
#         self.num_saves += 1
#         if self.num_saves == GAMES_PER_ROUND:
#             self.stop_processing()
#             self.Train()
#             self.num_saves = 0

#         return model_pb2.Status(code=0)


#     def Train(self, training_steps=TRAINING_STEPS):
#         """Train the model using the data in the replay buffer"""

#         for step in range(training_steps):
#             logging.info(f"Training step: {step}")

#             # Get a batch of data from the replay buffer
#             batch = self.buffer.sample()
#             inputs = batch.get("states").to(self.device)
#             policies = batch.get("policies").to(self.device)
#             values = batch.get("scores").to(self.device)

#             # Train the model
#             self.optimizer.zero_grad()
#             policy, value = self.model(inputs)
#             policy_loss = self.policy_loss(policy, policies)
#             value_loss = self.value_loss(value, values)
#             loss = policy_loss + value_loss
#             loss.backward()
#             self.optimizer.step()

#             # Store training statistics
#             row = pd.DataFrame([{
#                 "round": self.training_round,
#                 "loss": loss.item(),
#                 "value_loss": value_loss.item(),
#                 "policy_loss": policy_loss.item(),
#                 "buffer_size": len(self.buffer)
#             }])
#             if self.stats.empty:
#                 self.stats = row
#             else:
#                 self.stats = pd.concat([self.stats, row])
#             self.stats.to_csv("./data/training_stats.csv")

#         torch.save(self.model, f"./models/model_{self.training_round}.pt")
#         self.training_round += 1
#         self.start_processing()

#         return model_pb2.Status(code=0)


# def serve():
#     logging.info("Starting up server...")
#     logging.debug(f"ENV VARS:\n"
#                   f"PORT: {PORT}\n"
#                   f"BUFFER_CAPACITY: {BUFFER_CAPACITY}\n"
#                   f"LEARNING_RATE: {LEARNING_RATE}\n"
#                   f"BATCH_SIZE: {BATCH_SIZE}\n"
#                   f"TRAINING_STEPS: {TRAINING_STEPS}\n"
#                   f"TRAINING_ROUNDS: {TRAINING_ROUNDS}\n"
#                   f"NUM_CLIENTS: {NUM_CLIENTS}\n"
#                   f"GAMES_PER_ROUND: {GAMES_PER_ROUND}\n"
#                   f"BATCHING_FREQUENCY: {BATCHING_FREQUENCY}\n"
#                   f"WIDTH: {NN_WIDTH}\n"
#                   f"BLOCKS: {NN_BLOCKS}\n")
#     server = grpc.server(futures.ThreadPoolExecutor(max_workers=8))
#     model_pb2_grpc.add_BlokusModelServicer_to_server(BlokusModelServicer(), server)
#     server.add_insecure_port(f"[::]:{PORT}")
#     server.start()
#     server.wait_for_termination()


# if __name__ == "__main__":
#     serve()

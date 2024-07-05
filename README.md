# Blokus Engine

## Usage:

To open the GUI in the browser run:
`trunk serve --open`

To run server:
`python model/model_server.py`

To run simulation client:
`cargo run --bin self_play`

Generate server code: `python -m grpc_tools.protoc -Iproto --python_out=./model --pyi_out=./model --grpc_python_out=./model ./proto/model.proto`

## Configuration

All configuration is stored in the environment in the form of environment variables. This follows the [12 factor app](https://12factor.net/config) methodology, and an example env file is provided in the root of the project.

#### Example .env file:

```
PORT=8000
SERVER_URL=http://[::1]:8000

# Server
BUFFER_CAPACITY=1000 # Games
LEARNING_RATE=0.001  # Changes from 0.01 to 0.0001 for AlphaZero
BATCH_SIZE=256       # 2048 for AlphaZero
TRAINING_STEPS=10    # 700,000 for AlphaZero

# Client
NUM_CLIENTS=1
GAMES_PER_CLIENT=1  # 21 million total games for Go in AlphaZero
SIMS_PER_MOVE=10    # 800 for AlphaZero
SAMPLE_MOVES=30     # 30 for AlphaZero
C_BASE=19652        # 19652 for AlphaZero
C_INIT=1.25         # 1.25 for AlphaZero
DIRICHLET_ALPHA=0.3 # 0.03 for AlphaZero
EXPLORATION_FRAC=0.25   # 0.25 for AlphaZero
```

## On Tap:

Gui:

- Fix basic functionality
- Add undo
- Figure out how to connect to model
- Elegant way to handle terminal states

Self Play:

- Add loop for coordinating training rounds
- Add capability to record data from training rounds
- ResNet Backbone for neural network

Miscelaneous:

- Benchmarks for testing performance
- Is it worth refactoring the way pieces are structured

Questions:

- What does it mean for the staet to be oriented to the current player? Should I shuffle the order of the boards to match,
  or should I just keep track of the current player and rotate the board accordingly?
- Should I pass in remaining pieces for the hard coded filters or something? Shouldn't the model be able to figure that out?

## References

- https://sebastianbodenstein.com/post/alphazero/
- https://arxiv.org/pdf/1712.01815.pdf
- https://arc.net/folder/7FE3479D-1752-401F-9DC3-49AAD02B5DF3

# Blokus Engine

## Usage:

To open the GUI in the browser run:
`trunk serve --open`

To run server:
`python model/model_server.py`

To run simulation client:
`cargo run --bin self_play`

### Development

To generate server gRPC code: `python -m grpc_tools.protoc -Iproto --python_out=./model --pyi_out=./model --grpc_python_out=./model ./proto/model.proto`

### Docker

To train the model using docker: `docker compose up`

To copy model X from the running server: `docker cp <container-id>:/server/models/model_X.pt /destination/path`

To copy training data from the running server: `docker cp <container-id>:/server/data/training_stats.csv /destination/path`


## Configuration

All configuration is stored in the environment in the form of environment variables. This follows the [12 factor app](https://12factor.net/config) methodology, and an example env file is provided below.

#### Example .env file:

```
PORT=8082
SERVER_URL=http://[::1]:8082
TRAINING_ROUNDS=1

# -- Server --
# In AlphaZero...
# Learning rate changes from .01 to .0001 over time
# Batch size is 2048
# 700,000 training steps
# ResNet had 20 blocks with 256 filters for each convolution
BUFFER_CAPACITY=1000
LEARNING_RATE=0.01
BATCH_SIZE=128
TRAINING_STEPS=10
NN_WIDTH=32
NN_BLOCKS=2

# -- Client --
# In AlphaZero...
# 5000 clients
# 800 simulations per move
# 30 sample moves
# 19652 c_base
# 1.25 c_init
# 0.03 dirichlet_alpha
# 0.25 exploration_frac
NUM_CLIENTS=1
GAMES_PER_CLIENT=1
SIMS_PER_MOVE=2
SAMPLE_MOVES=30
C_BASE=19652
C_INIT=1.25
DIRICHLET_ALPHA=0.3
EXPLORATION_FRAC=0.25
CHECK_INTERVAL=2
```

## References

- https://sebastianbodenstein.com/post/alphazero/
- https://arxiv.org/pdf/1712.01815.pdf
- https://arc.net/folder/7FE3479D-1752-401F-9DC3-49AAD02B5DF3

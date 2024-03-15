# Blokus Engine

Usage:

To open the GUI in the browser run:
`trunk serve --open`

To run server:
`python src/model/model_server.py`

To run simulation client:
`cargo run --bin client`

Generate server code: `python -m grpc_tools.protoc -Iproto --python_out=./src/model --pyi_out=./src/model --grpc_python_out=./src/model ./proto/model.proto`


On Tap:
- Undo
- Benchmark board performance
- Simulation for self-play
    - MCTS
- Something is up with trunk server now, lots of compilation errors
    - This is due to parts or tokio / tonic that are incompatible with wasm
    - I could try to disable these modules for wasm build, but then I won't be able to use the 
    GUI to connect to the model. Maybe I could use another library later for that
- Explore repeated tile moves and maybe tree to represent possible moves
- Can I get rid of gloo console and use yew services console instead?
- Handle game over in gui instead of just resetting (state.rs)

Plan:
1. Fill in logic for ending the game
2. Set up MCTS
3. Explore repeated moves approach

References
- https://sebastianbodenstein.com/post/alphazero/
- https://arxiv.org/pdf/1712.01815.pdf
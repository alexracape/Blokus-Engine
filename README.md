# Blokus Engine

Usage:

To open the GUI in the browser run:
`trunk serve --open`

To run server:
`python model/model_server.py`

To run simulation client:
`cargo run --bin client`

Generate server code: `python -m grpc_tools.protoc -Iproto --python_out=./model --pyi_out=./model --grpc_python_out=./model ./proto/model.proto`


On Tap:
- Undo
- Benchmark board performance
- Something is up with trunk server now, lots of compilation errors
    - This is due to parts or tokio / tonic that are incompatible with wasm
    - I could try to disable these modules for wasm build, but then I won't be able to use the 
    GUI to connect to the model. Maybe I could use another library later for that
- Handle game over in gui instead of just resetting (state.rs)
- Piece variants seem like a mess, is it worth refactoring?
    - I made it thinking about the bit boards, but now there are also a lot of shape applications
- Update the way moves are applied for GUI
- Look into docker and kubernetes for deployment


Plan:
1. Fix GUI
2. Implement undo
3. Look into deployment

Questions:
- What does it mean for the staet to be oriented to the current player? Should I shuffle the order of the boards to match,
or should I just keep track of the current player and rotate the board accordingly?
- Should I pass in remaining pieces for the hard coded filters or something? Shouldn't the model be able to figure that out?
- Is it weird that values are the same throughout the whole game? No unexpected moves?

References
- https://sebastianbodenstein.com/post/alphazero/
- https://arxiv.org/pdf/1712.01815.pdf
- https://arc.net/folder/7FE3479D-1752-401F-9DC3-49AAD02B5DF3
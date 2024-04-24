# Self Play Module

This module allows the model to simulate games against itself using Monte
Carlo Tree Search. It uses gRPC to communicate with the model server.
This approach is modeled after the AlphaZero algorithm.

## Usage

To run the self play client:

```bash
cargo run --bin self_play
```

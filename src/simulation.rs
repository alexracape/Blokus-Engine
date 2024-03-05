// One game of self-play using MCTS and a neural network

use std::net::TcpStream;
use std::io::{Read, Write};

use crate::game_tree::{GameTree, Node};
use crate::state::State;

const NUM_SIMULATIONS: usize = 100;
const SERVER_ADDRESS: &str = "localhost:8000";

fn MCTS(root: Node, stream: TcpStream) {}

pub struct Simulation {
    game_tree: GameTree,
}

impl Simulation {
    pub fn new() -> Simulation {
        Simulation {
            game_tree: GameTree::new(),
        }
    }

    pub fn run(&mut self) {

        // Connect to neural network
        let mut nn_stream = TcpStream::connect(SERVER_ADDRESS)?;

        // Run self-play to generate data
        let data = Vec::new();
        let mut current_state = self.game_tree.root;
        while !current_state.state.is_terminal() {
            // Get MCTS policy for current state
            policy = MCTS(current_state, nn_stream);
            state_rep = current_state.get_representation();
            data.append((state_rep, policy, 0));

            // Pick action from policy and continue self-play
            current_state = current_state.select_child();
        }

        // Update the data points with payoffs for the final state

        data
    }
}

fn main() {
    let mut sim = Simulation::new();
    sim.run();
}

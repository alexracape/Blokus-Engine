// One game of self-play using MCTS and a neural network

use crate::game_tree::{GameTree, Node};
use crate::state::State;

fn MCTS(root: Node) {}

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
        let data = Vec::new();
        let mut current_state = self.game_tree.root;
        while !self.current_state.state.is_terminal() {
            // Get MCTS policy for current state
            policy = MCTS(current_state, neural_network);
            state_rep = current_state.get_representation();
            data.append((state_rep, policy, 0));

            // Pick action from policy and continue self-play
            current_state = current_state.select_child();
        }

        // Update the data points with payoffs for the final state

        data
    }
}

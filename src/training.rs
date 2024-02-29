// Set up rust client to represent one simulation
// Set up overall training process to manage multiple simulations
// Interface with the python server
// Need to figure out exporting state and marshalling

use std::thread;

use crate::game_tree::GameTree;
use crate::simulation::Simulation;

const NUM_SIMULATIONS: u32 = 800;

fn main() {
    println!("Starting training...");
    game_tree = GameTree::new();

    let handle = thread::spawn(|| {
        for _ in 0..NUM_SIMULATIONS {
            Simulation::new().run();
        }
    });
}

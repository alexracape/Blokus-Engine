// One game of self-play using MCTS and a neural network

use blokus_model::blokus_model_client::BlokusModelClient;
use blokus_model::State as StateRep;
use blokus_model::Prediction as PredRep;

use tonic::transport::Channel;

use crate::game_tree::{GameTree, Node};
use crate::state::State;


const SELF_PLAY_GAMES: usize = 100;
const SIMULATIONS: usize = 100;
const SERVER_ADDRESS: &str = "localhost:8000";


pub mod blokus_model {
    tonic::include_proto!("blokusmodel");
}


fn MCTS(root: Node, model: BlokusModelClient<Channel>) {}

pub struct Simulation {
    game_tree: GameTree,
}

impl Simulation {
    pub fn new() -> Simulation {
        Simulation {
            game_tree: GameTree::new(),
        }
    }

    
}


#[tokio::main]
async fn run_simulation(game_tree: GameTree) -> Result<(), Box<dyn std::error::Error>> {

    // Connect to neural network
    let mut model = BlokusModelClient::connect("http://[::1]:50051").await?;

    // Run self-play to generate data
    let data = Vec::new();
    let mut current_state = game_tree.root;
    while !current_state.state.is_terminal() {
        
        // Get MCTS policy for current state
        let policy = MCTS(current_state, model);
        let state_rep = current_state.get_representation();
        data.append((state_rep, policy, 0));

        // Pick action from policy and continue self-play
        current_state = current_state.select_child();
    }

    // Update the data points with payoffs for the final state

    // Train neural network on data
    let data = tonic::Request::new(ModelInput {
        boards: data,
        player: 1,
    });
    model.train(data).await?;

    Ok(())
}


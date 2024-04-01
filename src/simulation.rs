// One game of self-play using MCTS and a neural network

use blokus_model::blokus_model_client::BlokusModelClient;
use blokus_model::Prediction as PredRep;
use blokus_model::Data as DataRep;

use tonic::transport::Channel;

use crate::game_tree::{GameTree, Node};


const SELF_PLAY_GAMES: usize = 100;
const SIMULATIONS: usize = 100;
const SERVER_ADDRESS: &str = "localhost:8000";


pub mod blokus_model {
    tonic::include_proto!("blokusmodel");
}


fn mcts(root: &Node, model: BlokusModelClient<Channel>) -> Vec<f32>{
    // TODO
    vec![0.0; 400]
}

// pub struct Simulation {
//     game_tree: GameTree,
// }

// impl Simulation {
//     pub fn new() -> Simulation {
//         Simulation {
//             game_tree: GameTree::new(),
//         }
//     }

    
// }


#[tokio::main]
async fn run_simulation(game_tree: GameTree) -> Result<(), Box<dyn std::error::Error>> {

    // Connect to neural network
    let mut model = BlokusModelClient::connect("http://[::1]:50051").await?;

    // Run self-play to generate data
    let states = Vec::new();
    let ground_truth = Vec::new();
    let mut current_state = &game_tree.root;
    while !current_state.is_terminal() {
        
        // Get MCTS policy for current state
        let policy = mcts(current_state, model);
        let state_rep = current_state.get_representation();
        states.push(state_rep);
        ground_truth.push(PredRep {
            policy: policy,
            value: vec![0.0, 0.0, 0.0, 0.0],
        });

        // Pick action from policy and continue self-play
        current_state = current_state.select_child().unwrap();
    }

    // Update the data points with payoffs for the final state
    let payoff = current_state.get_payoff();
    for i in 0..states.len() {
        ground_truth[i].value = payoff;
    }

    // Train the model
    let data = tonic::Request::new(DataRep {
        states: states,
        predictions: ground_truth,
    });
    model.train(data).await?;

    Ok(())
}


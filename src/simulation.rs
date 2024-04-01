// One game of self-play using MCTS and a neural network

use crate::grpc::blokus_model_client::BlokusModelClient;
use crate::grpc::Prediction as PredRep;
use crate::grpc::Data as DataRep;

use tonic::transport::Channel;

use crate::game_tree::{GameTree, Node};


const SELF_PLAY_GAMES: usize = 100;
const SIMULATIONS: usize = 100;
const SERVER_ADDRESS: &str = "localhost:8000";


fn mcts(root: &Node, model: &BlokusModelClient<Channel>) -> Vec<f32>{
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
    let mut states = Vec::new();
    let mut policies = Vec::new();
    let mut current_state = &game_tree.root;
    while !current_state.is_terminal() {
        
        // Get MCTS policy for current state
        let policy = &mut mcts(current_state, &model);
        let state_rep = current_state.get_representation();
        states.push(state_rep);
        policies.append(policy);

        // Pick action from policy and continue self-play
        current_state = current_state.select_child().unwrap();
    }

    // Train the model
    let data = tonic::Request::new(DataRep {
        states: states,
        policies: policies,
        values: current_state.get_payoff(),
    });
    model.train(data).await?;

    Ok(())
}


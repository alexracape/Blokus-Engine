// One game of self-play using MCTS and a neural network

use blokus_model::blokus_model_client::BlokusModelClient;
use blokus_model::State as ModelInput;

use crate::game_tree::{GameTree, Node};
use crate::state::State;

const SELF_PLAY_GAMES: usize = 100;
const SIMULATIONS: usize = 100;
const SERVER_ADDRESS: &str = "localhost:8000";


pub mod blokus_model {
    tonic::include_proto!("blokusmodel");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = BlokusModelClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(ModelInput {
        board: vec![false; 400],
        pieces: vec![false; 21],
        player: 1,
    });

    let response = client.predict(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}


// fn MCTS(root: Node, stream: TcpStream) {}

// pub struct Simulation {
//     game_tree: GameTree,
// }

// impl Simulation {
//     pub fn new() -> Simulation {
//         Simulation {
//             game_tree: GameTree::new(),
//         }
//     }

//     pub fn run(&mut self) {

//         // Connect to neural network
//         let mut nn_stream = TcpStream::connect(SERVER_ADDRESS)?;



//         // Run self-play to generate data
//         let data = Vec::new();
//         let mut current_state = self.game_tree.root;
//         while !current_state.state.is_terminal() {
//             // Get MCTS policy for current state
//             policy = MCTS(current_state, nn_stream);
//             state_rep = current_state.get_representation();
//             data.append((state_rep, policy, 0));

//             // Pick action from policy and continue self-play
//             current_state = current_state.select_child();
//         }

//         // Update the data points with payoffs for the final state

//         data
//     }
// }

// fn main() {
//     let mut sim = Simulation::new();
//     sim.run();
// }

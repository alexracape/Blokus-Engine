// One game of self-play using MCTS and a neural network
use rand::Rng;
use std::fmt::Error;
use std::vec;

use crate::grpc::blokus_model_client::BlokusModelClient;
use crate::grpc::Prediction as PredRep;
use crate::grpc::Data as DataRep;

use tonic::transport::Channel;

use crate::node::Node;
use crate::game::Game;


const SELF_PLAY_GAMES: usize = 100;
const SIMULATIONS: usize = 100;
const SERVER_ADDRESS: &str = "localhost:8000";


/// Evaluate and Expand the Node
async fn evaluate(node: &mut Node, game: &Game, model: &mut BlokusModelClient<Channel>) -> Result<Vec<f32>, Box<dyn std::error::Error>> {

    // Get the policy and value from the neural network
    let representation = game.get_representation();
    let legal_moves = representation.boards[1599..2000].to_vec();
    let request = tonic::Request::new(representation);
    let prediction = model.predict(request).await?.into_inner();
    let policy = prediction.policy;
    let value = prediction.value;

    // Normalize policy for node priors, filter out illegal moves
    let exp_policy: Vec<(usize, f32)> = policy.iter().enumerate().filter_map(|(i, &logit)| {
        if legal_moves[i] {
            Some((i, logit.exp()))
        } else {
            None 
        }
    }).collect();
    let total: f32 = exp_policy.iter().map(|(_, p)| p).sum();
    
    // Expand the node with the policy
    node.set_player(game.current_player());
    for (tile, prob) in exp_policy {
        node.children.insert(tile, Node::new(prob / total));
    }

    Ok(value)
}


/// Select child node to explore
/// Uses UCB1 formula to balance exploration and exploitation
/// Returns the action and the child node
fn select_child(node: Node) -> (usize, Node) {
    let mut best_score = 0.0;
    let mut best_tile = 0;
    let mut best_child = Node::new(0.0);
    for (tile, child) in node.children {
        let score = child.value_sum / child.visits as f32 + 1.41 * (node.visits as f32).sqrt() / (1.0 + child.visits as f32).sqrt();
        if score > best_score {
            best_score = score;
            best_tile = tile;
            best_child = child;
        }
    }
    (best_tile, best_child)
}


/// Select action from policy
fn select_action(policy: Vec<f32>) -> usize {
    let mut rng = rand::thread_rng();
    let mut action = 0;
    let mut best_prob = 0.0;
    let mut total_prob = 0.0;
    for (i, prob) in policy.iter().enumerate() {
        total_prob += prob;
        let random = rng.gen_range(0.0..1.0);
        if random < total_prob {
            action = i;
            break;
        }
    }
    action
}


/// Update node when visitied during backpropagation
fn backpropagate(search_path: Vec<Node>, values: Vec<f32>) -> () {

    for mut node in search_path {
        let player = node.to_play;
        node.visits += 1;
        node.value_sum += values[player];
    }
}


/// Run MCTS simulations to get policy for root node
async fn mcts(game: &Game, model: &mut BlokusModelClient<Channel>) -> Result<Vec<f32>, Box<dyn std::error::Error>>{
    
    // Initialize root for these sims, evaluate it, and add children
    let root = Node::new(0.0);
    evaluate(&mut root, game, model);
    // TODO: Add noise to tree

    for _ in 0..SIMULATIONS {

        // Select a leaf node
        let node = root;
        let scratch_game = game.clone();
        let search_path = vec![node];
        let action = 0;
        while !node.is_leaf() {
            let (action, node) = select_child(node);
            scratch_game.apply(action); // TODO: Need to implement apply and tile based moves
            search_path.push(node);
        }

        // Expand and evaluate the leaf node
        let values = evaluate(&mut node, &scratch_game, model).await?;

        // Backpropagate the value
        backpropagate(search_path, values)
    }

    // Return the policy for the root node
    let total_visits: u32 = root.children.iter().map(|(tile, node)| node.visits).sum();
    let policy = vec![0.0; 400];
    for (tile, node) in root.children {
        policy[tile] = node.visits as f32 / total_visits as f32;
    }
    Ok(policy)
    

}


#[tokio::main]
async fn play_game() -> Result<(), Box<dyn std::error::Error>> {

    // Connect to neural network
    let mut model = BlokusModelClient::connect("http://[::1]:50051").await?;

    // Run self-play to generate data
    let mut game = Game::reset();
    let mut states = Vec::new();
    let mut policies = Vec::new();
    while !game.is_terminal() {

        // Get MCTS policy for current state
        let policy = &mut mcts(&game, &mut model).await?;
        policies.append(policy);
        states.push(game.get_representation());


        // Pick action from policy and continue self-play
        let action = select_action(policy);
        game.apply(action);

    }

    // Train the model
    let data = tonic::Request::new(DataRep {
        states: states,
        policies: policies,
        values: game.get_payoffs(),
    });
    model.train(data).await?;

    Ok(())
}


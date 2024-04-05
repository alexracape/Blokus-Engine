// One game of self-play using MCTS and a neural network
use rand::Rng;
use std::vec;

use crate::grpc::blokus_model_client::BlokusModelClient;
use crate::grpc::Data as DataRep;

use tonic::transport::Channel;

use crate::node::Node;
use crate::game::Game;


const SIMULATIONS: usize = 1;
const C_BASE: f32 = 19652.0;
const C_INIT: f32 = 1.25;


/// Evaluate and Expand the Node
async fn evaluate(node: &mut Node, game: &Game, model: &mut BlokusModelClient<Channel>) -> Result<Vec<f32>, Box<dyn std::error::Error>> {

    // Get the policy and value from the neural network
    let representation = game.get_representation();
    let legal_moves = representation.boards[1600..2000].to_vec();
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
    node.to_play = game.current_player();
    for (tile, prob) in exp_policy {
        node.children.insert(tile, Node::new(prob / total));
    }

    Ok(value)
}


/// Get UCB score for a child node
/// Exploration constant is based on the number of visits to the parent node
/// so that it will encourage exploration of nodes that have not been visited
fn ucb_score(parent: &Node, child: &Node) -> f32 {
    let exploration_constant = (parent.visits as f32 + C_BASE + 1.0 / C_BASE).ln() + C_INIT;
    let prior_score = exploration_constant * child.prior;
    let value_score = child.value();
    prior_score + value_score
}

/// Select child node to explore
/// Uses UCB formula to balance exploration and exploitation
/// Returns the action and the child node's key
fn select_child(node: &Node) -> usize {
    let mut best_score = 0.0;
    let mut best_action = 0;
    for (action, child) in &node.children {
        let score = ucb_score(node, child);
        if score > best_score {
            best_score = score;
            best_action = *action;
        }
    }
    best_action
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
fn backpropagate(search_path: Vec<usize>, root: &mut Node, values: Vec<f32>) -> () {

    let node = root;
    for tile in search_path {
        let node = node.children.get_mut(&tile).unwrap();
        node.visits += 1;
        node.value_sum += values[node.to_play];
    }
}


/// Run MCTS simulations to get policy for root node
async fn mcts(game: &Game, model: &mut BlokusModelClient<Channel>) -> Result<Vec<f32>, Box<dyn std::error::Error>>{
    
    // Initialize root for these sims, evaluate it, and add children
    let mut root = Node::new(0.0);
    match evaluate(&mut root, game, model).await {
        Ok(_) => (),
        Err(e) => {
            println!("Error evaluating root node: {:?}", e);
            return Err(e);
        
        }
    }
    // TODO: Add noise to tree

    for _ in 0..SIMULATIONS {

        // Select a leaf node
        let mut node = &mut root;
        let mut scratch_game = game.clone();
        let mut search_path = Vec::new();
        while node.is_expanded() {
            let action = select_child(node);
            node = node.children.get_mut(&action).unwrap();
            scratch_game.apply(action); 
            search_path.push(action);
        }

        // Expand and evaluate the leaf node
        let values = evaluate(&mut node, &scratch_game, model).await?;

        // Backpropagate the value
        backpropagate(search_path, &mut root, values) // Pass reference to node
    }

    // Return the policy for the root node
    let total_visits: u32 = root.children.iter().map(|(tile, child)| child.visits).sum();
    let mut policy = vec![0.0; 400];
    for (tile, node) in root.children {
        policy[tile] = node.visits as f32 / total_visits as f32;
    }
    Ok(policy)
    

}


#[tokio::main]
pub async fn play_game(server_address: String) -> Result<String, Box<dyn std::error::Error>> {

    // Connect to neural network
    let mut model = BlokusModelClient::connect(server_address).await?;

    // Run self-play to generate data
    let mut game = Game::reset();
    let mut states = Vec::new();
    let mut policies = Vec::new();
    while !game.is_terminal() {

        // Get MCTS policy for current state
        // let mut policy = mcts(&game, &mut model).await?;
        let policy = match mcts(&game, &mut model).await {
            Ok(p) => p,
            Err(e) => {
                println!("Error running MCTS: {:?}", e);
                return Err(e);
            }
        };

        policies.push(policy.clone());
        states.push(game.get_representation());


        // Pick action from policy and continue self-play
        let action = select_action(policy);
        println!("Player {} --- {}", game.current_player(), action);
        game.apply(action);
        game.board.print_board();

    }

    // Train the model
    let data = tonic::Request::new(DataRep {
        states: states,
        policies: policies.into_iter().flatten().collect(),
        values: game.get_payoff(),
    });
    model.train(data).await?;

    Ok(format!("Game finished with payoff: {:?}", game.get_payoff()))
}


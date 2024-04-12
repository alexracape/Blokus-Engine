// One game of self-play using MCTS and a neural network
use rand::Rng;
use rand_distr::{Dirichlet, Distribution};
use std::vec;

use crate::grpc::blokus_model_client::BlokusModelClient;
use crate::grpc::ActionProb;
use crate::grpc::Game as GameRep;
use crate::grpc::Policy;
use crate::grpc::Move;

use tonic::transport::Channel;

use crate::node::Node;
use crate::game::Game;


const SIMULATIONS: usize = 1; // 800 in AlphaZero
const SAMPLE_MOVES: usize = 30;

// Constants for UCB formula
const C_BASE: f32 = 19652.0;
const C_INIT: f32 = 1.25;

// Constants for exploration noise
const DIRICHLET_ALPHA: f32 = 0.3; // Scaled for number of moves iin average game - revisit?
const EXPLORATION_FRACTION: f32 = 0.25;



/// Evaluate and Expand the Node
async fn evaluate(node: &mut Node, game: &Game, model: &mut BlokusModelClient<Channel>) -> Result<Vec<f32>, Box<dyn std::error::Error>> {

    // If the game is over, return the payoff
    if game.is_terminal() {
        return Ok(game.get_payoff());
    }

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
    node.to_play = game.current_player().unwrap();
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


/// Add noise to the root node to encourage exploration
fn add_exploration_noise(root: &mut Node) -> () {
    let num_actions = root.children.len();
    if num_actions <= 1 {
        return;
    }

    let alpha_vec = vec![DIRICHLET_ALPHA; num_actions];
    let dirichlet = Dirichlet::new(&alpha_vec).unwrap();
    let noise = dirichlet.sample(&mut rand::thread_rng());
    for (i, (_tile, node)) in root.children.iter_mut().enumerate() {
        node.prior = node.prior * (1.0 - EXPLORATION_FRACTION) + noise[i] * EXPLORATION_FRACTION;
    }

}



/// Sample from a softmax distribution
/// Used to select actions during the first few moves to encourage exploration
fn softmax_sample(visit_dist: Vec<(usize, u32)>) -> usize {
    let total_visits: u32 = visit_dist.iter().fold(0, |acc, (_, visits)| acc + visits);
    let sample = rand::thread_rng().gen_range(0.0..1.0);
    let mut sum = 0.0;

    for (tile, visits) in &visit_dist {
        sum += (*visits as f32) / (total_visits as f32);
        if sum > sample {
            return *tile;
        }
    }
    visit_dist.last().unwrap().0
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
fn select_action(root: &Node, num_moves: usize) -> usize {
    let visit_dist: Vec<(usize, u32)> = root.children.iter().map(|(tile, node)| (*tile, node.visits)).collect();
    if num_moves < SAMPLE_MOVES {
        softmax_sample(visit_dist)
    } else {
        visit_dist.iter().max_by(|a, b| a.1.cmp(&b.1)).unwrap().0
    }

}


/// Update node when visitied during backpropagation
fn backpropagate(search_path: Vec<usize>, root: &mut Node, values: Vec<f32>) -> () {

    let mut node = root;
    for tile in search_path {
        node = node.children.get_mut(&tile).unwrap();
        node.visits += 1;
        node.value_sum += values[node.to_play];
    }
}


/// Run MCTS simulations to get policy for root node
async fn mcts(game: &Game, model: &mut BlokusModelClient<Channel>, policies: &mut Vec<Policy>) -> Result<usize, Box<dyn std::error::Error>>{
    
    // Initialize root for these sims, evaluate it, and add children
    let mut root = Node::new(0.0);
    match evaluate(&mut root, game, model).await {
        Ok(_) => (),
        Err(e) => {
            println!("Error evaluating root node: {:?}", e);
            return Err(e);
        }
    }
    add_exploration_noise(&mut root);

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

    // Save policy for this state
    let total_visits: u32 = root.children.iter().map(|(_tile, child)| child.visits).sum();
    let probs = root.children.iter().map(|(tile, child)| {
        let p = (child.visits as f32) / (total_visits as f32);
        ActionProb {action: *tile as i32, prob: p}
    }).collect();
    policies.push(Policy {probs: probs});

    // Pick action to take
    let action = select_action(&root, policies.len());
    Ok(action)
}


#[tokio::main]
pub async fn play_game(server_address: String) -> Result<String, Box<dyn std::error::Error>> {

    // Connect to neural network
    let mut model = BlokusModelClient::connect(server_address).await?;

    // Run self-play to generate data
    let mut game = Game::reset();
    let mut states = Vec::new();
    let mut policies: Vec<Policy> = Vec::new();
    while !game.is_terminal() {

        // Get MCTS policy for current state
        // let mut policy = mcts(&game, &mut model).await?;
        states.push(game.get_representation());
        let action = match mcts(&game, &mut model, &mut policies).await {
            Ok(a) => a,
            Err(e) => {
                println!("Error running MCTS: {:?}", e);
                return Err(e);
            }
        };
        
        // println!("Player {} --- {}", game.current_player(), action);
        game.apply(action);
    }

    // Train the model
    let game_data = tonic::Request::new(GameRep {
        history: game.history.iter().map(|(p, t)| Move{player: *p, tile: *t}).collect(),
        policies: policies,
        values: game.get_payoff(),
    });
    model.save(game_data).await?;
    
    // game.board.print_board();
    Ok(format!("Game finished with payoff: {:?}", game.get_payoff()))
}


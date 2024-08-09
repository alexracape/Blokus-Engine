// One game of self-play using MCTS and a neural network
use rand::Rng;
use rand_distr::{Dirichlet, Distribution};
use std::vec;

use indicatif::ProgressBar;
use pyo3::prelude::*;

use crate::node::Node;
use blokus::board::BOARD_SIZE as D;
use blokus::game::Game;

const BOARD_SIZE: usize = D * D;

#[derive(FromPyObject)]
pub struct Config {
    sims_per_move: usize,
    sample_moves: usize,
    c_base: f32,
    c_init: f32,
    dirichlet_alpha: f32,
    exploration_fraction: f32,
}

trait StateRepr {
    fn get_representation(&self) -> [[f32; BOARD_SIZE]; 5];
}

impl StateRepr for Game {
    /// Get a representation of the state for the neural network
    /// This representation includes the board and the legal tiles
    /// Oriented to the current player
    fn get_representation(&self) -> [[f32; BOARD_SIZE]; 5] {
        // Get rep for the pieces on the board
        let current_player = self.current_player().expect("No current player");
        let board = &self.board.board;
        let mut board_rep = [[0.0; BOARD_SIZE]; 5];
        for i in 0..BOARD_SIZE {
            let player = (board[i] & 0b1111) as usize; // check if there is a player piece
            let player_board = (4 + player - current_player) % 4; // orient to current player (0 indexed)
            if player != 0 {
                board_rep[player_board][i] = 1.0;
            }
        }

        // Get rep for the legal spaces
        let legal_moves = self.legal_tiles();
        for tile in legal_moves {
            board_rep[4][tile] = 1.0;
        }

        board_rep
    }
}

/// Evaluate and Expand the Node
fn evaluate(
    node: &mut Node,
    game: &Game,
    config: &Config,
    inference_queue: &Bound<PyAny>,
    pipe: &Bound<PyAny>,
    id: i32,
) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    // If the game is over, return the payoff
    if game.is_terminal() {
        return Ok(game.get_payoff());
    }

    // Get the policy and value from the neural network
    let representation = game.get_representation();
    let legal_moves = representation[4].to_vec();

    // Put the request in the queue
    let request = (id, representation);
    inference_queue.call_method1("put", (request,))?;

    // Wait for the result
    let inference = pipe.call_method0("recv")?;
    let policy: Vec<f32> = inference.get_item(0)?.extract()?;
    let value: Vec<f32> = inference.get_item(1)?.extract()?;
    let current_player = game.current_player().unwrap();

    // Normalize policy for node priors, filter out illegal moves
    let exp_policy: Vec<(usize, f32)> = policy
        .iter()
        .enumerate()
        .filter_map(|(i, &logit)| {
            if legal_moves[i] == 1.0 {
                Some((i, logit.exp()))
            } else {
                None
            }
        })
        .collect();
    let total: f32 = exp_policy.iter().map(|(_, p)| p).sum();

    // Expand the node with the policy
    node.to_play = current_player;
    for (tile, prob) in exp_policy {
        node.children.insert(tile, Node::new(prob / total));
    }

    // Reorient the values so they are in order: player 0, player 1, player 2, player 3
    let mut values = vec![0.0; 4];
    for i in 0..4 {
        values[(4 + i - current_player) % 4] = value[i];
    }
    Ok(value)
}

/// Get UCB score for a child node
/// Exploration constant is based on the number of visits to the parent node
/// so that it will encourage exploration of nodes that have not been visited
fn ucb_score(parent: &Node, child: &Node, config: &Config) -> f32 {
    let c_base = config.c_base;
    let c_init = config.c_init;
    let exploration_constant = (parent.visits as f32 + c_base + 1.0 / c_base).ln() + c_init;
    let prior_score = exploration_constant * child.prior;
    let value_score = child.value();
    prior_score + value_score
}

/// Add noise to the root node to encourage exploration
fn add_exploration_noise(root: &mut Node, config: &Config) -> () {
    let num_actions = root.children.len();
    if num_actions <= 1 {
        return;
    }

    let alpha_vec = vec![config.dirichlet_alpha; num_actions];
    let dirichlet = Dirichlet::new(&alpha_vec).unwrap();
    let noise = dirichlet.sample(&mut rand::thread_rng());
    for (i, (_tile, node)) in root.children.iter_mut().enumerate() {
        node.prior = node.prior * (1.0 - config.exploration_fraction)
            + noise[i] * config.exploration_fraction;
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
fn select_child(node: &Node, config: &Config) -> usize {
    let mut best_score = 0.0;
    let mut best_action = 0;
    for (action, child) in &node.children {
        let score = ucb_score(node, child, config);
        if score > best_score {
            best_score = score;
            best_action = *action;
        }
    }
    best_action
}

/// Select action from policy
fn select_action(root: &Node, num_moves: usize, config: &Config) -> usize {
    let visit_dist: Vec<(usize, u32)> = root
        .children
        .iter()
        .map(|(tile, node)| (*tile, node.visits))
        .collect();
    if num_moves < config.sample_moves {
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
fn mcts(
    game: &Game,
    policies: &mut Vec<Vec<(i32, f32)>>,
    config: &Config,
    inference_queue: &Bound<PyAny>,
    pipe: &Bound<PyAny>,
    id: i32,
) -> Result<usize, Box<dyn std::error::Error>> {
    // Initialize root for these sims, evaluate it, and add children
    let mut root = Node::new(0.0);
    match evaluate(&mut root, game, config, inference_queue, pipe, id) {
        Ok(_) => (),
        Err(e) => {
            println!("Error evaluating root node: {:?}", e);
            return Err(e);
        }
    }
    add_exploration_noise(&mut root, config);

    for _ in 0..config.sims_per_move {
        // Select a leaf node
        let mut node = &mut root;
        let mut scratch_game = game.clone();
        let mut search_path = Vec::new();
        while node.is_expanded() {
            let action = select_child(node, config);
            node = node.children.get_mut(&action).unwrap();
            let _ = scratch_game.apply(action, None);
            search_path.push(action);
        }

        // Expand and evaluate the leaf node
        let values = evaluate(&mut root, &scratch_game, config, inference_queue, pipe, id).unwrap();

        // Backpropagate the value
        backpropagate(search_path, &mut root, values)
    }

    // Save policy for this state
    let total_visits: u32 = root
        .children
        .iter()
        .map(|(_tile, child)| child.visits)
        .sum();
    let probs = root
        .children
        .iter()
        .map(|(tile, child)| {
            let p = (child.visits as f32) / (total_visits as f32);
            (*tile as i32, p)
        })
        .collect();
    policies.push(probs);

    // Pick action to take
    let action = select_action(&root, policies.len(), config);
    Ok(action)
}

pub fn play_game(
    config: Config,
    inference_queue: &Bound<PyAny>,
    pipe: &Bound<PyAny>,
    id: i32,
) -> Result<(Vec<(i32, i32)>, Vec<Vec<(i32, f32)>>, Vec<f32>), String> {
    // Storage for game data
    let mut game = Game::reset();
    let mut policies: Vec<Vec<(i32, f32)>> = Vec::new();

    // Run self-play to generate data
    let bar = ProgressBar::new(BOARD_SIZE as u64);

    // Run self-play to generate data
    while !game.is_terminal() {
        // Get MCTS policy for current state
        let action = match mcts(&game, &mut policies, &config, inference_queue, pipe, id) {
            Ok(a) => a,
            Err(e) => {
                println!("Error running MCTS: {:?}", e);
                return Err("Error running MCTS".to_string());
            }
        };

        // println!("Player {} --- {}", game.current_player(), action);
        let _ = game.apply(action, None);
        bar.inc(1);
    }

    // Train the model
    let values = game.get_payoff();
    let scores = game.get_score();
    let game_data = (game.history, policies, values.clone());
    bar.finish();
    println!(
        "Game finished with payoff: {:?} and score: {:?}",
        values, scores
    );

    // game.board.print_board();
    Ok(game_data)
}

// Game Tree to be used by each MCTS simulation
use std::collections::HashMap;

const BOARD_SPACES: usize = 400;


/// Each node in the game tree represents a state of the game
/// There is one state for every tile placement
#[derive(Clone)]
pub struct Node {
    pub children: HashMap<usize, Node>,
    pub to_play: usize,
    pub value_sum: f32,
    pub visits: u32,
    prior: f32,
}

impl Node {
    pub fn new(prior: f32) -> Node {
        Node {
            children: HashMap::new(),
            to_play: 0,
            value_sum: 0.0,
            visits: 0,
            prior: prior,
        }
    }

    /// Select a child node to explore
    pub fn select_child(&self) -> Option<&mut Node> {
        // TODO
        None
    }

    /// Set current player
    pub fn set_player(&mut self, player: usize) {
        self.to_play = player;
    }

    /// Is the node a leaf
    pub fn is_leaf(&self) -> bool {
        self.children.len() == 0
    }

}

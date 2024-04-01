// Game Tree to be used by each MCTS simulation
use blokus_model::State as StateRep;

use crate::state::State;

const BOARD_SPACES: usize = 400;


/// Each node in the game tree represents a state of the game
/// There is one state for every tile placement
pub struct Node {
    state: State,
    children: Vec<Node>,
    parent: Option<Box<Node>>,
    value_sum: u32,
    visits: u32,
    prior: f32,
    tile: usize,
}

impl Node {
    fn new(state: State, tile: usize, prior: f32) -> Node {
        Node {
            state: state,
            children: Vec::new(),
            parent: None,
            value_sum: 0,
            visits: 0,
            prior: prior,
            tile: tile,
        }
    }

    /// Expand the tree by adding children to the current node
    pub fn get_children(&self) -> () {
        self.children.push(Node::new(self.state, 0, 0.0));
    }

    /// Select a child node to explore
    pub fn select_child(&self) -> Option<&Node> {
        self.children.get(0)
    }

    // Is the node a terminal state
    pub fn is_terminal(&self) -> bool {
        self.state.is_terminal()
    }

    pub fn get_payoff(&self) -> Vec<f32> {
        // TODO
        vec![0.0, 0.0, 0.0, 0.0]
    }

    /// Get a representation of the state for the neural network
    /// This representation includes the board and the legal tiles
    pub fn get_representation(&self) -> StateRep {

        // Get rep for the pieces on the board
        let mut board_rep = [[false; BOARD_SPACES]; 5];
        board_rep[0..4].copy_from_slice(&self.state.get_representation().0);

        // Get rep for the legal spaces
        let legal_moves = self.state.get_legal_moves();
        for (piece, variant, offset) in legal_moves {
            let variant = self.state.get_current_player_pieces()[piece].variants[variant];
            let shape = variant.get_shape();
            
            // Mark legal spaces on the representation
            for i in 0..shape.len() {
                for j in 0..shape[i].len() {
                    if shape[i][j] {
                        let global_offset = offset + i * 20 + j;
                        board_rep[4][global_offset] = true;
                    }
                }
            }
        }

        StateRep {
            board: board_rep,
        }

    }
}

pub struct GameTree {
    pub root: Node,
}

impl GameTree {
    pub fn new() -> GameTree {
        GameTree {
            root: Node::new(State::reset(), 0, 1.0),
        }
    }
}

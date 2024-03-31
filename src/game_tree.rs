// Game Tree to be used by each MCTS simulation

use crate::state::State;
use blokus_model::State as StateRep;

pub struct Node {
    pub state: State,
    children: Vec<Node>,
    parent: Option<Box<Node>>,
    value_sum: u32,
    visits: u32,
    prior: f32,
}

impl Node {
    fn new(state: State, prior: f32) -> Node {
        Node {
            state: state,
            children: Vec::new(),
            parent: None,
            value_sum: 0,
            visits: 0,
            prior: prior,
        }
    }

    fn select_child(&self) -> Option<&Node> {
        self.children.get(0)
    }

    pub fn get_representation(&self) -> StateRep {
        let simple_rep = self.state.get_representation();
        
    }
}

pub struct GameTree {
    pub root: Node,
}

impl GameTree {
    pub fn new() -> GameTree {
        GameTree {
            root: Node::new(State::reset(), 1.0),
        }
    }
}

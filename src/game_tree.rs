// Game Tree to be used by each MCTS simulation

use crate::state::State;

struct Node {
    state: State,
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

    fn select_child(&Self) -> Node {
        children.get(0)
    }
}

struct GameTree {
    root: Node,
}

impl GameTree {
    fn new() -> GameTree {
        GameTree {
            root: Node::new(State::reset(), 1.0),
        }
    }
}

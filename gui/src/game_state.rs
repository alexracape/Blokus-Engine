use std::rc::Rc;
use std::collections::HashSet;

use gloo_console as console;
use yew::prelude::*;

use blokus::game::{Action, Game};
use blokus::pieces::{Piece, PieceVariant};


/// Wrapper for the game state so that we can implement Reducible
#[derive(Clone)]
pub struct GameState(Game);

impl GameState {

    pub fn reset() -> Self {
        GameState(Game::reset())
    }

    pub fn current_player(&self) -> Option<usize> {
        self.0.current_player()
    }

    pub fn eliminate_player(&mut self) {
        self.0.eliminate_player();
    }

    pub fn is_terminal(&self) -> bool {
        self.0.is_terminal()
    }

    pub fn get_board(&self) -> &[u8; 400] {
        self.0.get_board()
    }

    pub fn get_current_player_pieces(&self) -> Vec<Piece> {
        self.0.get_current_player_pieces()
    }

    pub fn get_current_anchors(&self) -> HashSet<usize> {
        self.0.get_current_anchors()
    }

    pub fn is_valid_move(&self, player: usize, piece: &PieceVariant, offset: usize) -> bool {
        self.0.board.is_valid_move(player, piece, offset)
    }

    pub fn get_piece(&self, player: usize, piece: usize, variant: usize) -> PieceVariant {
        self.0.board.get_pieces(player)[piece].variants[variant].clone()
    }
}


impl Reducible for GameState {
    type Action = Action;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Action::PlacePiece(p, v, o) => {
                let player = self.current_player().expect("No current player");
                let new_state = (*self).clone();
                let piece = self.get_piece(player, p, v);

                // Check if move is valid
                if !new_state.is_valid_move(player, &piece, o) {
                    console::log!("Invalid move");
                    return self.into();
                }

                // Remove piece from player and place piece
                // player.pieces.remove(p);
                // let used_spaces = new_state.board.place_piece(player, &piece, o);
                // new_state.current_player = self.next_player();

                // // Update anchors for all players
                // for player in &mut new_state.players {
                //     player.use_anchors(&used_spaces);
                // }

                // // Add move to stack
                // new_state.history.push(used_spaces.into_iter().collect());

                // // Return new state
                new_state.into()
            }
            Action::Pass => {
                let mut new_state = (*self).clone();
                new_state.eliminate_player();

                if new_state.is_terminal() {
                    return GameState::reset().into(); // TODO - need to handle better with message or something
                }

                new_state.into()
            }
            Action::Undo => {
                let new_state = (*self).clone();
                // TODO: Need to implement undo
                new_state.into()
            }
            Action::ResetGame => GameState::reset().into(),
        }
    }
}

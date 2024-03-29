use std::collections::HashSet;
use std::rc::Rc;

use gloo_console as console;
use yew::prelude::*;

use crate::board::Board;
use crate::pieces::Piece;
use crate::player::Player;

const BOARD_SPACES: usize = 400;

pub enum Action {
    PlacePiece(usize, usize, usize),
    Undo,
    ResetGame,
}

#[derive(Clone)]
pub struct State {
    board: Board,
    players: Vec<Player>,
    move_stack: Vec<(usize, usize, usize)>,
    current_player: usize,
    players_remaining: Vec<usize>
}

impl Reducible for State {
    type Action = Action;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Action::PlacePiece(p, v, o) => {
                let mut new_state = (*self).clone();
                let player = &mut new_state.players[self.current_player];
                console::log!(
                    "Anchors",
                    player
                        .get_anchors()
                        .iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                );
                let piece = player.pieces[p].variants[v].clone();

                // Check if move is valid
                if !new_state.board.is_valid_move(&player, &piece, o) {
                    console::log!("Invalid move");
                    return self.into();
                }

                // Remove piece from player and place piece
                player.pieces.remove(p);
                let used_spaces = new_state.board.place_piece(player, &piece, o);
                new_state.current_player = (self.current_player + 1) % self.players.len();

                // Update anchors for all players
                for player in &mut new_state.players {
                    player.use_anchors(&used_spaces);
                }

                // Add move to stack
                new_state.move_stack.push((p, v, o));

                // Return new state
                new_state.into()
            }
            Action::Undo => {
                let mut new_state = (*self).clone();
                let (p, v, o) = new_state.move_stack.pop().unwrap();
                let player = &new_state.players[self.current_player];
                let piece = player.pieces[p].variants[v].clone();
                new_state.board.remove_piece(player, &piece, o);
                new_state.current_player =
                    (self.current_player + self.players.len() - 1) % self.players.len();
                new_state.into()
            }
            Action::ResetGame => State::reset().into(),
        }
    }
}

impl State {
    pub fn reset() -> Self {
        let mut players = Vec::new();
        for i in 1..5 {
            players.push(Player::new(i));
        }
        State {
            board: Board::new(),
            players,
            move_stack: Vec::new(),
            current_player: 0,
            players_remaining: (0..5).collect() 
        }
    }

    pub fn get_board(&self) -> &[u8; BOARD_SPACES] {
        &self.board.board
    }

    pub fn get_current_player_pieces(&self) -> Vec<Piece> {
        self.players[self.current_player].pieces.clone()
    }

    pub fn get_current_player(&self) -> usize {
        self.current_player
    }

    pub fn get_current_anchors(&self) -> HashSet<usize> {
        self.players[self.current_player].get_anchors()
    }

    pub fn is_terminal(&self) -> bool {
        self.players_remaining.len() == 0
    }

    pub fn get_representation(&self) -> ([[bool; BOARD_SPACES]; 4], [[bool; 21]; 4]) {
        // Get rep of the board where players pieces are divided to seperate boards
        let board = &self.board.board;
        let mut board_rep = [[false; BOARD_SPACES]; 4];
        for i in 0..BOARD_SPACES {
            let player = board[i] & 0b1111; // check if there is a player piece
            if player != 0 {
                board_rep[player as usize - 1][i] = true;
            }
        }

        // Get rep of pieces remaining for each player
        let mut pieces_rep = [[false; 21]; 4];
        for i in 0..4 {
            for piece in &self.players[i].pieces {
                pieces_rep[i][piece.id] = true;
            }
        }

        (board_rep, pieces_rep)
    }
}

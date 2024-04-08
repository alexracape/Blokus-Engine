use std::collections::{HashMap, HashSet};
use std::iter::zip;
use std::rc::Rc;

use gloo_console as console;
use yew::prelude::*;

use crate::grpc::StateRepresentation;
use crate::board::Board;
use crate::pieces::Piece;

const BOARD_SPACES: usize = 400;

pub enum Action {
    PlacePiece(usize, usize, usize),
    Pass,
    Undo,
    ResetGame,
}


/// Get the legal moves for a piece
fn get_piece_moves(piece_i: usize, board: &Board, player: usize) -> (Vec<(usize, usize, usize)>, Vec<Vec<usize>>) {
    let mut moves = Vec::new();
    let mut tile_groups = Vec::new();
    let piece = &board.get_pieces(player)[piece_i];
    for anchor in &board.get_anchors(player) {
        for (var_i, variant) in piece.variants.iter().enumerate() {
            for offset in &variant.offsets {

                // Check underflow
                if offset > anchor {
                    continue;
                }

                let total_offset = anchor - offset; // offset to anchor, then offset to line up piece
                if board.is_valid_move(player, variant, total_offset) {
                    let mut tiles = Vec::new();
                    for (j, square) in variant.variant.iter().enumerate() {
                        if *square {
                            tiles.push(total_offset + j);
                        }
                    }
                    tile_groups.push((tiles));
                    moves.push((piece_i, var_i, total_offset))
                }
            }
        }
    }

    (moves, tile_groups)
}


/// Get the legal moves for a player, tile placements grouped by move
fn get_moves(board: &Board, player: usize) -> (Vec<(usize, usize, usize)>, Vec<Vec<usize>>){
    let mut moves = Vec::new();
    let mut tile_groups = Vec::new();
    for piece in 0..board.get_pieces(player).len() {
        let (piece_moves, piece_tiles) = get_piece_moves(piece, board, player);
        moves.extend(piece_moves);
        tile_groups.extend(piece_tiles);
    }

    (moves, tile_groups)
}


/// Get the tile bases representation for legal moves
fn get_tile_moves(board: &Board, player: usize, ) -> HashMap<usize, HashSet<(usize, usize, usize)>> {
    let mut tile_rep = HashMap::new();
    let (moves, tile_groups) = get_moves(board, player);
    
    for (id, tiles) in zip(moves, tile_groups) {
        for tile in tiles {
            if !tile_rep.contains_key(&tile) {
                tile_rep.insert(tile, HashSet::new());
            }
            tile_rep.get_mut(&tile).unwrap().insert(id);
        }
    }

    tile_rep
}


#[derive(Clone)]
pub struct Game {
    pub board: Board,
    history: Vec<Vec<usize>>, // Each row is a move consisting of its tiles
    players_remaining: Vec<usize>, // Indices of players still in the game
    player_index: usize, // Index of the current player in players_remaining
    legal_tiles: HashMap<usize, HashSet<(usize, usize, usize)>>, // Map tile to index of the overall move
    last_piece_lens: [u32; 4], // Size of the last piece placed by each player
}

impl Reducible for Game {
    type Action = Action;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Action::PlacePiece(p, v, o) => {
                let player = self.current_player();
                let mut new_state = (*self).clone();
                let piece = self.board.get_pieces(player)[p].variants[v].clone();

                // Check if move is valid
                if !new_state.board.is_valid_move(player, &piece, o) {
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
                    return Game::reset().into(); // TODO - need to handle better with message or something
                }

                new_state.into()
            }
            Action::Undo => {
                let mut new_state = (*self).clone();
                let last_move = new_state.history.pop().unwrap();
                // TODO: Need to implement undo
                new_state.into()
            }
            Action::ResetGame => Game::reset().into(),
        }
    }
}

impl Game {
    pub fn reset() -> Self {
        let board = Board::new();
        let legal_tiles = get_tile_moves(&board, 0);

        Game {
            board: board,
            history: Vec::new(),
            players_remaining: vec![0, 1, 2, 3],
            player_index: 0,
            legal_tiles: legal_tiles,
            last_piece_lens: [0; 4],
        }
    }

    pub fn apply(&mut self, tile: usize) -> Result<(), String> {

        // Place piece on board
        self.board.place_tile(tile, self.current_player());

        // Update legal tiles
        // let valid_moves = self.legal_tiles.remove(&tile).unwrap();
        let valid_moves = match self.legal_tiles.remove(&tile) {
            Some(moves) => moves,
            None => return Err("Invalid move".to_string())
        };
        for (tile, move_set) in self.legal_tiles.clone() {
            self.legal_tiles.insert(tile, move_set.intersection(&valid_moves).map(|m| *m).collect());
            if self.legal_tiles.get(&tile).unwrap().len() == 0 {
                self.legal_tiles.remove(&tile);
            }
        }

        // Advance to next player if necessary
        if self.legal_tiles.len() == 0 {

            // Removing the player's piece
            let (piece, _variant, _offset) = valid_moves.iter().next().unwrap();
            self.last_piece_lens[self.current_player()] = self.board.get_pieces(self.current_player()).remove(*piece).points;
            self.board.use_piece(self.current_player(), *piece);
            self.board.print_board();
            println!();

            // Advance to next player
            let next = self.next_player();
            self.legal_tiles = get_tile_moves(&self.board, next);
            if self.legal_tiles.len() == 0 {
                self.eliminate_player();
            }
        }

        Ok(())
    }

    pub fn get_board(&self) -> &[u8; BOARD_SPACES] {
        &self.board.board
    }

    pub fn next_player(&mut self) -> usize {
        self.player_index = (self.player_index + 1) % self.players_remaining.len();
        self.players_remaining[self.player_index]
    }

    pub fn current_player(&self) -> usize {
        self.players_remaining[self.player_index]
    }

    /// Remove the current player from the game
    pub fn eliminate_player(&mut self) {
        self.players_remaining.remove(self.player_index);
        if self.players_remaining.len() == 0 {
            return;
        }

        self.player_index = self.player_index % self.players_remaining.len();
        self.legal_tiles = get_tile_moves(&self.board, self.current_player());
    }

    pub fn get_current_player_pieces(&self) -> Vec<Piece> {
        self.board.get_pieces(self.current_player())
    }

    pub fn get_current_anchors(&self) -> HashSet<usize> {
        self.board.get_anchors(self.current_player())
    }

    pub fn legal_tiles(&self) -> Vec<usize> {
        self.legal_tiles.keys().map(|k| *k).collect()
    }

    /// Player fewest tiles remaining wins
    pub fn get_payoff(&self) -> Vec<f32> {
        let scores = self.board.get_scores(self.last_piece_lens);
        let mut payoff = vec![0.0; 4];
        let mut indices = Vec::new();
        let mut highest_score = scores[0];
        for (i, score) in scores.iter().enumerate() {
            if *score == highest_score {
                indices.push(i);
            } else if *score > highest_score {
                indices.clear();
                indices.push(i);
                highest_score = *score;
            }
        }
        
        for i in &indices {
            payoff[*i] = 1.0 / indices.len() as f32;
        }

        println!("Scores: {:?}", scores);
        payoff
    }

    pub fn is_terminal(&self) -> bool {
        self.players_remaining.len() == 1 || self.legal_tiles.len() == 0
    }

    /// Get a representation of the state for the neural network
    /// This representation includes the board and the legal tiles
    pub fn get_representation(&self) -> StateRepresentation {

        // Get rep for the pieces on the board
        let board = &self.board.board;
        let mut board_rep = [[false; BOARD_SPACES]; 5];
        for i in 0..BOARD_SPACES {
            let player = board[i] & 0b1111; // check if there is a player piece
            if player != 0 {
                board_rep[player as usize - 1][i] = true;
            }
        }

        // Get rep for the legal spaces
        let legal_moves = self.legal_tiles();
        for tile in legal_moves {
            board_rep[4][tile] = true;
        }

        StateRepresentation {
            boards: board_rep.into_iter().flat_map(|inner| inner).collect(),
            player: self.current_player() as i32,
        }

    }
}

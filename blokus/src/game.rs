use std::collections::{HashMap, HashSet};
use std::iter::zip;

use crate::board::Board;
use crate::pieces::{Piece, PieceVariant};

const BOARD_SPACES: usize = 400;

/// Get the legal moves for a piece
fn get_piece_moves(
    piece_i: usize,
    board: &Board,
    player: usize,
) -> (Vec<(usize, usize, usize)>, Vec<Vec<usize>>) {
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
                    tile_groups.push(tiles);
                    moves.push((piece_i, var_i, total_offset))
                }
            }
        }
    }

    (moves, tile_groups)
}

/// Get the legal moves for a player, tile placements grouped by move
fn get_moves(board: &Board, player: usize) -> (Vec<(usize, usize, usize)>, Vec<Vec<usize>>) {
    let mut moves = Vec::new();
    let mut tile_groups = Vec::new();
    for piece in 0..board.get_pieces(player).len() {
        let (piece_moves, piece_tiles) = get_piece_moves(piece, board, player);
        moves.extend(piece_moves);
        tile_groups.extend(piece_tiles);
    }

    (moves, tile_groups)
}

/// Get the tile based representation for legal moves
fn get_tile_moves(board: &Board, player: usize) -> HashMap<usize, HashSet<(usize, usize, usize)>> {
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
    pub history: Vec<(i32, i32)>,  // Stack of moves
    players_remaining: Vec<usize>, // Indices of players still in the game
    player_index: usize,           // Index of the current player in players_remaining
    legal_tiles: HashMap<usize, HashSet<(usize, usize, usize)>>, // Map tile to index of the overall move
    last_piece_lens: [u32; 4], // Size of the last piece placed by each player
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

    /// TODO: Implement
    pub fn pass(&self) -> Result<Game, String> {
        Ok(self.clone())
    }

    pub fn place_piece(&self, p: usize, v: usize, o: usize) -> Result<Game, String> {
        let mut new_state = self.clone();
        let player = self.current_player().expect("No current player");
        let piece = self.get_piece(player, p, v);

        // Check if move is valid
        if !new_state.board.is_valid_move(player, &piece, o) {
            return Err("Invalid move".to_string());
        }

        // Break move into tiles and apply individually
        let offsets = piece.offsets.iter().collect::<Vec<_>>();
        let last_index = offsets.len().saturating_sub(1);
        for (i, tile_offset) in offsets.iter().enumerate() {
            let tile = o + *tile_offset;
            let result = if i == last_index {
                new_state.apply(tile, Some(p))
            } else {
                new_state.apply(tile, None)
            };

            match result {
                Ok(_) => (),
                Err(e) => return Err(e),
            }
        }

        Ok(new_state)
    }

    // Plays a tile on the board
    // Not thrilled with the implementation
    // Right now it forces you to place as many tiles as is legal or you can pass a piece you
    // want to finish playing. This is really only used by the GUI rn
    pub fn apply(&mut self, tile: usize, piece_to_finish: Option<usize>) -> Result<(), String> {
        // Place piece on board
        let current_player = match self.current_player() {
            Some(p) => p,
            None => return Err("No current player".to_string()),
        };
        self.board.place_tile(tile, current_player);
        self.history.push((current_player as i32, tile as i32));

        // Update legal tiles
        let valid_moves = match self.legal_tiles.remove(&tile) {
            Some(moves) => moves,
            None => return Err("Invalid move".to_string()),
        };
        for (tile, move_set) in self.legal_tiles.clone() {
            self.legal_tiles.insert(
                tile,
                move_set.intersection(&valid_moves).map(|m| *m).collect(),
            );
            if self.legal_tiles.get(&tile).unwrap().len() == 0 {
                self.legal_tiles.remove(&tile);
            }
        }

        // Advance to next player if necessary
        if self.legal_tiles.len() == 0 || piece_to_finish.is_some() {
            // Removing the player's piece
            let piece = match piece_to_finish {
                Some(p) => p,
                None => valid_moves.iter().next().unwrap().0,
            };
            self.last_piece_lens[current_player] =
                self.board.get_pieces(current_player).remove(piece).points;
            self.board.use_piece(current_player, piece);

            // Advance to next player
            self.advance_player();
        }

        Ok(())
    }

    pub fn get_board(&self) -> &[u8; BOARD_SPACES] {
        &self.board.board
    }

    /// Gets the next player, and advances the player index
    pub fn next_player(&mut self) -> usize {
        self.player_index = (self.player_index + 1) % self.players_remaining.len();
        self.players_remaining[self.player_index]
    }

    /// Cycle to the next player
    /// Eliminates any players that have no legal moves
    /// Returns index of the current player or None if the game is terminal
    pub fn advance_player(&mut self) -> Option<usize> {
        loop {
            let next = self.next_player();
            self.legal_tiles = get_tile_moves(&self.board, next);
            if self.legal_tiles.len() == 0 {
                self.eliminate_player();
                if self.is_terminal() {
                    return None;
                }
            } else {
                return Some(next);
            }
        }
    }

    pub fn current_player(&self) -> Option<usize> {
        self.players_remaining.get(self.player_index).copied()
    }

    /// Remove the current player from the game
    pub fn eliminate_player(&mut self) {
        self.players_remaining.remove(self.player_index);
        if self.players_remaining.len() == 0 {
            return;
        }

        self.player_index = self.player_index % self.players_remaining.len();
        self.legal_tiles = get_tile_moves(
            &self.board,
            self.current_player().expect("No current player"),
        );
    }

    pub fn get_current_player_pieces(&self) -> Vec<Piece> {
        let current_player = self.current_player().expect("No current player");
        self.board.get_pieces(current_player)
    }

    pub fn get_piece(&self, player: usize, piece: usize, variant: usize) -> PieceVariant {
        self.board.get_pieces(player)[piece].variants[variant].clone()
    }

    pub fn get_current_anchors(&self) -> HashSet<usize> {
        let current_player = self.current_player().expect("No current player");
        self.board.get_anchors(current_player)
    }

    pub fn legal_tiles(&self) -> Vec<usize> {
        self.legal_tiles.keys().map(|k| *k).collect()
    }

    /// Get the scores for the end of the game
    pub fn get_score(&self) -> Vec<i32> {
        self.board.get_scores(self.last_piece_lens)
    }

    /// Player fewest tiles remaining wins, payoff is between 0 and 1
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

        payoff
    }

    pub fn is_terminal(&self) -> bool {
        self.players_remaining.len() == 0
    }
}

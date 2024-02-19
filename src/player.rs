
use std::collections::HashSet;

use crate::pieces::{Piece, PIECE_TYPES};
use crate::board::{Board, BOARD_SIZE};

#[derive(Clone)]
pub struct Player {
    pub num: u8,
    pub pieces: Vec<Piece>,
    anchors: HashSet<usize>,
}

impl Player {
    pub fn new(num: u8) -> Player {
        let start = match num {
            1 => 0,
            2 => BOARD_SIZE - 1,
            3 => BOARD_SIZE * BOARD_SIZE - 1,
            4 => BOARD_SIZE * (BOARD_SIZE - 1),
            _ => panic!("Invalid player number"),
        };

        let mut pieces = Vec::new();
        for piece_type in PIECE_TYPES {
            pieces.push(Piece::new(piece_type));
        }

        let mut anchors = HashSet::new();
        anchors.insert(start);

        Player {
            num: num,
            pieces: pieces,
            anchors: anchors,
        }
    }

    /// Removes an anchor from the player
    /// Used when a piece is placed
    pub fn use_anchors(&mut self, spaces: &HashSet<usize>) {
        self.anchors = &self.anchors - spaces;
    }

    /// Adds an anchor to the player
    pub fn update_anchors(&mut self, anchors: HashSet<usize>) {
        for anchor in anchors {
            self.anchors.insert(anchor);
        }
    }

    pub fn get_anchors(&self) -> HashSet<usize> {
        self.anchors.clone()
    }

    /// Gets all possible moves for a piece
    /// Returns a list of (variant, offset) tuples
    pub fn get_piece_moves(&self, piece: &Piece, board: &mut Board) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        for anchor in &self.anchors {
            
            // Check piece placements on anchor
            for (i, variant) in piece.variants.iter().enumerate() {
                for offset in &variant.offsets {
                    
                    // Check for underflow
                    if offset > anchor {
                        continue;
                    }

                    let global_offset = anchor - offset; // offset to anchor, then offset to line up piece
                    if board.is_valid_move(self, variant, global_offset) {
                        moves.push((i, global_offset));
                    }
                }
            }

        }
        moves
    }

    /// Gets all possible moves for a player
    /// Returns a list of indices (piece, variant, offset) tuples
    pub fn get_moves(&self, board: &mut Board) -> Vec<(usize, usize, usize)> {
        let mut moves = Vec::new();
        for (i, piece) in self.pieces.iter().enumerate() {
            for (variant, offset) in self.get_piece_moves(piece, board) {
                moves.push((i, variant, offset));
            }
        }
        moves
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.num == other.num
    }
}


// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_creation() {
        let player = Player::new(1);
        assert_eq!(player.pieces.len(), PIECE_TYPES.len());
        assert_eq!(player.anchors.len(), 1);
    }

    #[test]
    fn test_get_piece_moves() {
        let mut board = Board::new();
        let mut player = Player::new(1);
        let piece = player.pieces[0].clone();
        let moves = player.get_piece_moves(&piece, &mut board);
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0], (0, 0));

        // Place piece then check for moves
        board.place_piece(&mut player, &piece.variants[0], 0);
        let piece = player.pieces[1].clone();
        let moves = player.get_piece_moves(&piece, &mut board);
        assert_eq!(moves.len(), 2); // two orientations of the two-y
        assert_eq!(moves[0], (0, 21));
    }
}


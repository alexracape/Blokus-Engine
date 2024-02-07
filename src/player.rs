
use crate::pieces::{Piece, PIECE_TYPES};
use crate::board::{Board, BOARD_SIZE};

#[derive(Clone)]
pub struct Player {
    pub num: u8,
    pub pieces: Vec<Piece>,
    anchors: Vec<usize>,
}

impl Player {
    pub fn new(num: u8) -> Player {
        let start = match num {
            1 => 0,
            2 => BOARD_SIZE - 1,
            3 => BOARD_SIZE * (BOARD_SIZE - 1),
            4 => BOARD_SIZE * BOARD_SIZE - 1,
            _ => panic!("Invalid player number"),
        };

        let mut pieces = Vec::new();
        for piece_type in PIECE_TYPES {
            pieces.push(Piece::new(piece_type));
        }

        Player {
            num: num,
            pieces: pieces,
            anchors: vec![start],
        }
    }

    /// Gets all possible moves for a piece
    /// Returns a list of (variant, offset) tuples
    pub fn get_piece_moves(&self, piece: &Piece, board: &mut Board) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        for anchor in &self.anchors {
            
            // Check piece placements on anchor
            for (i, variant) in piece.variants.iter().enumerate() {
                for offset in &variant.offsets {
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
        assert_eq!(player.pieces.len(), 2);
        assert_eq!(player.anchors.len(), 1);
    }
}


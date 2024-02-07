
use crate::pieces::{Piece, PIECE_TYPES};
use crate::board::BOARD_SIZE;

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


/*
Defines Pieces for Blokus Game
*/


pub enum PieceType {
    One,
    Two,
}

pub const PIECE_TYPES: [PieceType; 2] = [PieceType::One, PieceType::Two];


pub struct Piece {
    pub points: u32,
    pub variants: Vec<Vec<bool>>,
}

impl Piece {

    pub fn new(piece_type: PieceType) -> Piece {
        match piece_type {
            PieceType::One => Piece {
                points: 1,
                variants: vec![vec![true]],
            },

            PieceType::Two => Piece {
                points: 2,
                variants: vec![vec![true, true]],
            },
        
        }
    }
}


// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_creation() {
        let piece = Piece::new(PieceType::One);
        assert_eq!(piece.points, 1);
        assert_eq!(piece.variants, vec![vec![true]]);
    }
}

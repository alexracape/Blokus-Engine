/*
Defines Pieces for Blokus Game
*/
use crate::board::BOARD_SIZE;

pub enum PieceType {
    One,
    Two,
}

pub const PIECE_TYPES: [PieceType; 2] = [PieceType::One, PieceType::Two];

/// A piece variant is a specific orientation of a piece
/// It is a list of bools, where true represents a filled square
/// Offsets is a list of offsets to move a filled square to an anchor
#[derive(Clone, Debug)]
pub struct PieceVariant {
    pub offsets: Vec<usize>,
    pub variant: Vec<bool>,
}

impl PieceVariant {
    pub fn new(shape: Vec<Vec<bool>>) -> PieceVariant {
        let mut offsets = Vec::new();
        let mut variant = Vec::new();
        
        // Build the variant that is fully padded to the right
        for row in shape {
            for square in &row {
                variant.push(*square);
            }
            for _ in 0..BOARD_SIZE - row.len() {
                variant.push(false);
            }
        }

        // Store offsets to allign pieces later
        for (i, square) in variant.iter().enumerate() {
            if *square {
                offsets.push(i);
            }
        }
        PieceVariant {
            offsets: offsets,
            variant: variant,
        }
    }
}

impl PartialEq for PieceVariant {
    fn eq(&self, other: &Self) -> bool {
        self.variant == other.variant
    }
}


#[derive(Clone)]
pub struct Piece {
    pub shape: Vec<Vec<bool>>,
    pub points: u32,
    pub variants: Vec<PieceVariant>,
}

impl Piece {

    /// Takes a PieceType and redirects to the correct constructor
    /// Those constructors define the shape and create variant shapes
    pub fn new(piece_type: PieceType) -> Piece {
        match piece_type {
            PieceType::One => Piece::new_one(),
            PieceType::Two => Piece::new_two(),
        }
    }

    fn new_one() -> Piece {
        let shape = vec![vec![true]];
        let variants = Piece::gen_variants(shape.clone());
        Piece {
            shape: shape,
            points: 1,
            variants: variants,
        }
    }

    fn new_two() -> Piece {
        let shape = vec![vec![true, true]];
        let variants = Piece::gen_variants(shape.clone());
        Piece {
            shape: shape,
            points: 2,
            variants: variants,
        }
    }

     // Rotate a piece 90 degrees
     fn rotate(shape: Vec<Vec<bool>>) -> Vec<Vec<bool>> {
        let mut new_shape = Vec::new();
        for i in 0..shape[0].len() {
            let mut row = Vec::new();
            for j in (0..shape.len()).rev() {
                row.push(shape[j][i]);
            }
            new_shape.push(row);
        }
        
        new_shape
    }

    // Flip a piece over
    fn flip(shape: Vec<Vec<bool>>) -> Vec<Vec<bool>> {
        let mut new_shape = Vec::new();
        for row in shape {
            let mut new_row = Vec::new();
            for square in row.iter().rev() {
                new_row.push(*square);
            }
            new_shape.push(new_row);
        }
        new_shape
    }

    fn gen_variants(shape: Vec<Vec<bool>>) -> Vec<PieceVariant> {
        let mut variants = Vec::new();
        let mut variant_shape = shape.clone();

        // Generate all 8 variants
        for _ in 0..4 {

            let new_variant = PieceVariant::new(variant_shape.clone());
            if !variants.contains(&new_variant) {
                variants.push(new_variant);
            }
            variant_shape = Piece::rotate(variant_shape);
        }
        variant_shape = Piece::flip(shape);
        for _ in 0..4 {

            let new_variant = PieceVariant::new(variant_shape.clone());
            if !variants.contains(&new_variant) {
                variants.push(new_variant);
            }
            variant_shape = Piece::rotate(variant_shape);
        }

        variants
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
        assert_eq!(piece.variants, Piece::gen_variants(vec![vec![true]]));
    }

    #[test]
    fn test_piece_rotation() {
        let shape = vec![vec![true, true]];
        let rotated = Piece::rotate(shape.clone());
        assert_eq!(rotated, vec![vec![true], vec![true]]);

        let shape = vec![vec![true, true], vec![true, false]];
        let rotated = Piece::rotate(shape.clone());
        assert_eq!(rotated, vec![vec![true, true], vec![false, true]]);
    }

    #[test]
    fn test_piece_flip() {
        let shape = vec![vec![true, true]];
        let flipped = Piece::flip(shape.clone());
        assert_eq!(flipped, vec![vec![true, true]]);

        let shape = vec![vec![true, true], vec![true, false]];
        let flipped = Piece::flip(shape.clone());
        assert_eq!(flipped, vec![vec![true, true], vec![false, true]]);
    }

    #[test]
    fn test_piece_variants() {
        let shape = vec![vec![true, true]];
        let variants = Piece::gen_variants(shape.clone());
        assert_eq!(variants.len(), 2);

        let shape = vec![vec![true, true], vec![true, false]];
        let variants = Piece::gen_variants(shape.clone());
        assert_eq!(variants.len(), 4);

        let shape = vec![vec![true, true, true], vec![true, false, false]];
        let variants = Piece::gen_variants(shape.clone());
        assert_eq!(variants.len(), 8);
    }
}

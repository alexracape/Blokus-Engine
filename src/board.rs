/*
Blokus Board
*/

use std::collections::HashSet;

use crate::{pieces::PieceVariant, player::Player};


pub const BOARD_SIZE: usize = 20;

pub struct Board {
    pub board: [u8; BOARD_SIZE * BOARD_SIZE], // 20x20 board
}

impl Board {
    pub fn new() -> Board {
        Board {
            board: [0; BOARD_SIZE * BOARD_SIZE],
        }
    }

    pub fn is_valid_move(&mut self, player: &Player, piece_variant: &PieceVariant, offset: usize) -> bool {

        // Check piece is within bounds and does not go over edge of board
        let variant = &piece_variant.variant;
        if offset + variant.len() > self.board.len() {
            return false;
        } else if offset % BOARD_SIZE + variant.len() > BOARD_SIZE {
            return false;
        }

        let board_slice = &self.board[offset..offset + variant.len()];
        let player_restricted: u8 = 1 << player.num + 3;
        board_slice.iter().zip(variant.iter()).all(|(a, b)| {
            if *b {
                if *a & player_restricted != 0 {
                    return false;
                }
            }
            true
        })
    }

    /// Places a piece onto the board, assumes that the move is valid
    pub fn place_piece(&mut self, player: &mut Player, piece: &PieceVariant, offset: usize) {
        
        // Remove anchor from player
        player.use_anchor(offset);

        // Place piece on board
        let shape = &piece.variant;
        let fully_restricted: u8 = 0b1111_0000;
        let player_restricted: u8 = 1 << player.num + 3;
        let mut new_anchors = HashSet::new();
        for i in 0..shape.len() {
            if shape[i] {
                self.board[offset + i] = fully_restricted | player.num;
                println!("{} {}", offset + i, fully_restricted | player.num);

                // Restrict adjacent squares
                if i % BOARD_SIZE != 0 { // Not on left edge
                    self.board[offset + i - 1] |= player_restricted;
                } 
                if i % BOARD_SIZE != BOARD_SIZE - 1 { // Not on right edge
                    self.board[offset + i + 1] |= player_restricted;
                } 
                if i >= BOARD_SIZE { // Not on top edge
                    self.board[offset + i - BOARD_SIZE] |= player_restricted;
                } 
                if i < BOARD_SIZE * (BOARD_SIZE - 1) { // Not on bottom edge
                    self.board[offset + i + BOARD_SIZE] |= player_restricted;
                }

                // Add new anchors
                if i % BOARD_SIZE != 0 && self.board[offset + i - 1] == 0 {
                    new_anchors.insert(offset + i - 1);
                }

            }
        }

        // Update player anchors
        player.update_anchors(new_anchors);


    }

    pub fn print_board(&self) {
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                print!("[{}]", self.board[i * BOARD_SIZE + j] & 0b0000_1111);
            }
            println!();
        }
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_creation() {
        let board = Board::new();
        assert_eq!(board.board.len(), 400);
    }

    #[test]
    fn test_is_valid_move() {
        let mut board = Board::new();
        let player = Player::new(1);
        let piece = PieceVariant::new(vec![vec![true, true]]);
        assert_eq!(board.is_valid_move(&player, &piece, 0), true);
        assert!(board.is_valid_move(&player, &piece, 19) == false);
    }

    #[test]
    fn test_place_piece() {
        let mut board = Board::new();
        let mut player = Player::new(1);
        let piece = PieceVariant::new(vec![vec![true, true]]);
        board.place_piece(&mut player, &piece, 0);
        assert_eq!(board.board[0], 0b1111_0001);
        assert_eq!(board.board[1], 0b1111_0001);
    }

    #[test]
    fn test_overlapping_piece() {
        let mut board = Board::new();
        let mut player = Player::new(1);
        let piece = PieceVariant::new(vec![vec![true, true]]);
        board.place_piece(&mut player, &piece, 0);
        assert_eq!(board.board[0], 0b1111_0001);
        assert_eq!(board.board[1], 0b1111_0001);
        assert!(board.is_valid_move(&player, &piece, 1) == false);
        assert!(board.is_valid_move(&player, &piece, 2) == false);
    }
}

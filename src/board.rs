/*
Blokus Board
*/

use std::collections::HashSet;

use crate::{pieces::PieceVariant, player::Player};


pub const BOARD_SIZE: usize = 20;

#[derive(Clone)]
pub struct Board {
    pub board: [u8; BOARD_SIZE * BOARD_SIZE], // 20x20 board
}

impl Board {
    pub fn new() -> Board {
        Board {
            board: [0; BOARD_SIZE * BOARD_SIZE],
        }
    }

    pub fn is_valid_move(&self, player: &Player, piece_variant: &PieceVariant, offset: usize) -> bool {

        // Check piece is within bounds and does not go over edge of board
        let variant = &piece_variant.variant;
        let piece_squares = &piece_variant.offsets;
        if offset + variant.len() > self.board.len() {
            return false;
        } else if offset % BOARD_SIZE + piece_variant.width > BOARD_SIZE {
            return false;
        }

        let board_slice = &self.board[offset..offset + variant.len()];
        let player_restricted: u8 = 1 << player.num + 3;
        let on_blanks = board_slice.iter().zip(variant.iter()).all(|(a, b)| {
            if *b {
                if *a & player_restricted != 0 {
                    return false;
                }
            }
            true
        });

        let on_anchor = piece_squares.iter().any(|i| player.get_anchors().contains(&(offset + i)));
        on_blanks && on_anchor
    }

    
    /// Place a tile on the board
    pub fn place_tile(&mut self, tile: usize, player: u8) {
        self.board[tile] = 0b1111_0000 | (player + 1);
        // TODO: may need more logic here to account for anchors or other things
    }

    /// Places a piece onto the board, assumes that the move is valid
    /// Returns a set of spaces that the piece occupies, so that all players can update their anchors
    pub fn place_piece(&mut self, player: &mut Player, piece: &PieceVariant, offset: usize) -> HashSet<usize>{

        // Place piece on board
        let shape = &piece.variant;
        let fully_restricted: u8 = 0b1111_0000;
        let player_restricted: u8 = 1 << player.num + 3;
        let fill_value = fully_restricted | player.num;
        let mut new_anchor_candidates = HashSet::new();
        let mut used_spaces = HashSet::new();
        for i in 0..shape.len() {
            
            // Skip if square is not filled
            if !shape[i] {
                continue;
            }
            let space = offset + i;
            used_spaces.insert(space);

            self.board[space] = fill_value;
            println!("{} {}", offset + i, fully_restricted | player.num);

            // Restrict adjacent squares
            let on_left_edge = space % BOARD_SIZE == 0;
            let on_right_edge = space % BOARD_SIZE == BOARD_SIZE - 1;
            let on_top_edge = space < BOARD_SIZE;
            let on_bottom_edge = space >= BOARD_SIZE * (BOARD_SIZE - 1);
            if !on_left_edge {
                self.board[space - 1] |= player_restricted;
            } 
            if !on_right_edge { 
                self.board[space + 1] |= player_restricted;
            } 
            if !on_top_edge {
                self.board[space - BOARD_SIZE] |= player_restricted;
            } 
            if !on_bottom_edge {
                self.board[space + BOARD_SIZE] |= player_restricted;
            }

            // Add new anchors - TODO need to refine / think of strategy here
            let corner_offsets: Vec<i32> = vec![
                1 + BOARD_SIZE as i32,  // bottom right
                -1 - BOARD_SIZE as i32, // top left
                1 - BOARD_SIZE as i32,  // top right
                -1 + BOARD_SIZE as i32  // bottom left
            ];
            for corner_offset in corner_offsets {
                let corner = offset as i32 + i as i32 + corner_offset;

                // Skip if corner is above or below board
                if corner < 0 || corner >= (BOARD_SIZE * BOARD_SIZE) as i32 {
                    continue;
                }

                // Skip if corner wraps around to other side of board
                if on_left_edge && (corner as usize) % BOARD_SIZE == BOARD_SIZE - 1 {
                    continue;
                }
                if on_right_edge && (corner as usize) % BOARD_SIZE == 0 {
                    continue;
                }   
                new_anchor_candidates.insert(corner as usize);
            }
        }

        // Update player anchors
        for anchor in new_anchor_candidates.clone() {
            if self.board[anchor] & player_restricted != 0 {
                new_anchor_candidates.remove(&anchor);
            }
        }
        player.update_anchors(new_anchor_candidates);
        used_spaces

    }

    pub fn remove_piece(&mut self, player: &Player, piece: &PieceVariant, offset: usize) {
        let shape = &piece.variant;
        let player_restricted: u8 = 1 << player.num + 3;
        // for i in 0..shape.len() {
        //     if shape[i] {
        //         self.board[offset + i] = 0;
        //         let on_left_edge = i % BOARD_SIZE == 0;
        //         let on_right_edge = i % BOARD_SIZE == BOARD_SIZE - 1;
        //         let on_top_edge = i < BOARD_SIZE;
        //         let on_bottom_edge = i >= BOARD_SIZE * (BOARD_SIZE - 1);
        //         if !on_left_edge {
        //             self.board[offset + i - 1] &= !player_restricted;
        //         } 
        //         if !on_right_edge { 
        //             self.board[offset + i + 1] &= !player_restricted;
        //         } 
        //         if !on_top_edge {
        //             self.board[offset + i - BOARD_SIZE] &= !player_restricted;
        //         } 
        //         if !on_bottom_edge {
        //             self.board[offset + i + BOARD_SIZE] &= !player_restricted;
        //         }
        //     }
        // }

        // Need to check this stuff, just placeholder for now
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
        let board = Board::new();
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
        let used_spaces = board.place_piece(&mut player, &piece, 0);
        assert_eq!(board.board[0], 0b1111_0001);
        assert_eq!(board.board[1], 0b1111_0001);
        player.use_anchors(&used_spaces);
        println!("{:?}", player.get_anchors());
        assert_eq!(player.get_anchors().len(), 1);
        assert_eq!(player.get_anchors().contains(&22), true);
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

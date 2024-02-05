/*
The goal of this module is to implement the core logic for Blokus.
Eventually this will be an API that can be used in a web app that
can also interface with a Python ML model.

Roadmap:
- Create a board
- Create a piece
- Create functions for board (check_move, place_piece, etc.)
- Create basic game loop
- Create a basic web app
*/

use std::ops::BitAnd;

enum PieceType {
    One,
    Two,
}

const BOARD_SIZE: usize = 20;
const PIECE_TYPES: [PieceType; 2] = [PieceType::One, PieceType::Two];

struct Piece {
    points: u32,
    variants: Vec<Vec<bool>>,
}

impl Piece {

    fn new(piece_type: PieceType) -> Piece {
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


struct Player {
    pieces: Vec<Piece>,
    anchors: Vec<usize>,
}

impl Player {
    fn new(num: i8) -> Player {
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
            pieces: pieces,
            anchors: vec![start],
        }
    }
}

struct Board {
    board: [bool; BOARD_SIZE * BOARD_SIZE], // 20x20 board
}

impl Board {
    fn new() -> Board {
        Board {
            board: [false; BOARD_SIZE * BOARD_SIZE],
        }
    }

    fn is_valid_move(&self, piece: &Piece, offset: usize) -> bool {
        // TODO
        // let overlap = &self.board[offset..piece.length] & piece;
        false
    }

    /// Places a piece onto the board, assumes that the move is valid
    fn place_piece(&mut self, shape: &Vec<bool>, offset: usize) {
        
        for i in 0..shape.len() {
            self.board[offset + i] = shape[i];
        }

    }

    fn print_board(&self) {
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                print!("[{}]", if self.board[i * BOARD_SIZE + j] { "X" } else { " " });
            }
            println!();
        }
    }
}

// impl BitAnd for Board {
//     type Output = Self;

//     fn bitand(self, rhs: &Piece) -> Self::Output {
//         self.board
//     }
// }


fn main() {

    // Create board and players
    let mut board = Board::new();
    let mut players = Vec::new();
    for i in 1..5 {
        players.push(Player::new(i));
    }

    let test_piece = &players[0].pieces[0].variants[0];
    board.place_piece(test_piece, 0);
    board.print_board();

    // // Game loop
    // loop {
    //     // Loop through players
    //     for player in &mut players {

    //         for anchor in &player.anchors {
    //             for piece in &player.pieces {
    //                 // Check if piece can be placed
    //                 if board.is_valid_move(piece, *anchor, 0) {
    //                     // Place piece
    //                     board.place_piece(piece, *anchor, 0);
    //                     board.print_board();
    //                 }
    //             }
    //         }
    //     }
    // }
}

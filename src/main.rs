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


const BOARD_SIZE: usize = 20;

struct One {
    shape: [[u8; 1]; 1],
}


enum Piece {
    One,
    Two,
    Three,
    Four,
    Five,
    Right,
}


struct Board {
    board: [[u8; BOARD_SIZE]; BOARD_SIZE], // 20x20 board
}

impl Board {
    fn new() -> Board {
        Board {
            board: [[0; BOARD_SIZE]; BOARD_SIZE],
        }
    }

    fn is_valid_move(&self, piece: &Piece, x: usize, y: usize) -> bool {
        // TODO
        true
    }

    fn place_piece(&mut self, piece: &Piece, x: usize, y: usize) {
        // TODO
    }

    fn print_board(&self) {
        for row in self.board.iter() {
            for cell in row.iter() {
                print!("{} ", cell);
            }
            println!();
        }
    }
}


fn main() {
    println!("Hello, world!");
}

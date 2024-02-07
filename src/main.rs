
mod gui;
mod pieces;
mod player;
mod board;

use crate::player::Player;
use crate::board::Board;
use crate::gui::app::App;


fn main() {

    // Create board and players
    let mut board = Board::new();
    let mut players = Vec::new();
    for i in 1..5 {
        players.push(Player::new(i));
    }

    let test_piece = &players[0].pieces[0].variants[0];
    board.place_piece(&players[0], test_piece, 0);
    board.print_board();

    yew::Renderer::<App>::new().render();

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




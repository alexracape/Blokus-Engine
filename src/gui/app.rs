use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, Window};
use gloo_console as console;
use yew::prelude::*;

use crate::board::Board;
use crate::pieces::{Piece, PieceVariant};
use crate::player::Player;
use crate::gui::board::BlokusBoard;
use crate::gui::pieces::PieceTray;

#[function_component]
pub fn App() -> Html {

    // Create board and players
    let mut board = use_state(|| Board::new());
    let mut players = Vec::new();
    for i in 1..5 {
        players.push(Player::new(i));
    }

    let on_board_drop = Callback::from(|(piece, variant, offset)| {
        console::log!("Piece", piece);
        console::log!("Variant", variant);
        console::log!("Offset", offset);
    });

    html! {
        <div>
            <h1>{ "BLOKUS" }</h1>
            <BlokusBoard board={board.board} on_board_drop={on_board_drop} />
            <PieceTray player={players[0].clone()} />

            <h2>{ "Testing Buttons" }</h2>
            // <button onclick={onclick}>{ "Place Piece" }</button>
        </div>
    }
}
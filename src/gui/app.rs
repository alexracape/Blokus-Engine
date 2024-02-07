use yew::prelude::*;

use crate::board::Board;
use crate::player::Player;
use crate::gui::board::BlokusBoard;

#[function_component]
pub fn App() -> Html {

    // Create board and players
    let mut board = Board::new();
    let mut players = Vec::new();
    for i in 1..5 {
        players.push(Player::new(i));
    }

    // let counter = use_state(|| 0);
    // let onclick = {
    //     let counter = counter.clone();
    //     move |_| {
    //         let value = *counter + 1;
    //         counter.set(value);
    //     }
    // };

    html! {
        <div>
            <h1>{ "BLOKUS" }</h1>
            <BlokusBoard board={board.board} />
        </div>
    }
}
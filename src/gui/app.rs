
use gloo_console as console;
use yew::prelude::*;

use crate::state::{State, Action};
use crate::gui::board::BlokusBoard;
use crate::gui::pieces::PieceTray;

#[function_component]
pub fn App() -> Html {

    let state = use_reducer(State::reset);

    let on_board_drop = {
        let state = state.clone();
        Callback::from(move |(p, v, offset)| {
        
            // Debug to console
            console::log!("Piece", p);
            console::log!("Variant", v);
            console::log!("Offset", offset);

            // Place piece on board
            state.dispatch(Action::PlacePiece(p, v, offset));
        })
    };


    let on_reset = {
        let state = state.clone();
        Callback::from(move |_| {state.dispatch(Action::ResetGame)})
    };

    html! {
        <div>
            <h1>{ "Blokus Engine" }</h1>
            
            <BlokusBoard board={state.get_board()} on_board_drop={on_board_drop} anchors={state.get_current_anchors()} />
            <PieceTray pieces={state.get_current_player_pieces()} player_num={state.get_current_player()} />

            <button onclick={on_reset}>{ "Reset Game" }</button>

            <p style={"white-space: pre-line"}>{"
                Select Piece: click\n
                Rotate Piece: r\n
                Flip Piece: f\n
                Place Piece: Drag onto board\n
            "}</p>
            // <button onclick={onclick}>{ "Place Piece" }</button>
        </div>
    }
}
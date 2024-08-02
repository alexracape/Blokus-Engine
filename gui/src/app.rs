use gloo_console as console;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::board::BlokusBoard;
use crate::game_state::GameState;
use crate::pieces::PieceTray;

use blokus::game::Action;

/// Takes state and returns tile to place
async fn get_ai_move() -> usize {}

/// Applies AI moves to state after player has gone
async fn handle_ai_moves(state: GameState) -> GameState {}

#[function_component]
pub fn App() -> Html {
    let state = use_state(|| GameState::new());

    let on_board_drop = {
        let state = state.clone();
        Callback::from(move |(p, v, offset)| {
            // Debug to console
            console::log!("Piece", p);
            console::log!("Variant", v);
            console::log!("Offset", offset);

            // Place piece on board
            state.dispatch(Action::PlacePiece(p, v, offset));

            // Handle AI moves
            spawn_local({
                let state = state.clone();
                async move {
                    state.set(handle_ai_moves(*state).await);
                }
            });
        })
    };

    let on_reset = {
        let state = state.clone();
        Callback::from(move |_| state.dispatch(Action::ResetGame))
    };

    let on_pass = {
        let state = state.clone();
        Callback::from(move |_| {
            state.dispatch(Action::Pass);
        })
    };

    html! {
        <div>
            <h1>{ "Blokus Engine" }</h1>

            <BlokusBoard board={state.state.get_board()} on_board_drop={on_board_drop} anchors={state.state.get_current_anchors()} />
            <PieceTray pieces={state.state.get_current_player_pieces()} player_num={state.state.current_player().unwrap() as u8 + 1} />

            <button onclick={on_pass}>{ "Pass" }</button>
            <button onclick={on_reset}>{ "Reset Game" }</button>

            <p style={"white-space: pre-line"}>{"
                Select Piece: click\n
                Rotate Piece: r\n
                Flip Piece: f\n
                Place Piece: Drag onto board\n
            "}</p>
        </div>
    }
}

use gloo_console as console;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::board::BlokusBoard;
use crate::pieces::PieceTray;
use blokus::game::Game;

/// Takes state and returns tile to place
async fn get_ai_move() -> usize {
    1
}

/// Applies AI moves to state after player has gone
async fn handle_ai_moves(state: Game) -> Game {
    state
}

#[function_component]
pub fn App() -> Html {
    let state = use_state(|| Game::reset());

    let on_board_drop = {
        let state = state.clone();
        Callback::from(move |(p, v, offset)| {
            // Debug to console
            console::log!("Piece", p);
            console::log!("Variant", v);
            console::log!("Offset", offset);

            // Place piece on board
            //state.dispatch(Action::PlacePiece(p, v, offset));
            let new_state = match state.place_piece(p, v, offset) {
                Ok(s) => s,
                Err(e) => {
                    console::error!("Failed to place piece: {:?}", e);
                    return;
                }
            };
            state.set(new_state);

            // Handle AI moves
            let state = state.clone();
            spawn_local({
                async move {
                    let game = (*state).clone();
                    let new_state = handle_ai_moves(game).await;
                    state.set(new_state);
                }
            });
        })
    };

    let on_reset = {
        let state = state.clone();
        Callback::from(move |_| state.set(Game::reset()))
    };

    // TODO: Would I need to call AI moves after passing here?
    // Wouldn't game calculate for you when you are done?
    let on_pass = {
        let state = state.clone();
        Callback::from(move |_| {
            let game = (*state).clone();
            let new_state = match game.pass() {
                Ok(s) => s,
                Err(e) => {
                    console::error!("Failed to pass: {:?}", e);
                    return;
                }
            };
            state.set(new_state);
        })
    };

    html! {
        <div>
            <h1>{ "Blokus Engine" }</h1>

            <BlokusBoard board={state.get_board()} on_board_drop={on_board_drop} anchors={state.get_current_anchors()} />
            <PieceTray pieces={state.get_current_player_pieces()} player_num={state.current_player().unwrap() as u8 + 1} />

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

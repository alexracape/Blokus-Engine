use gloo_console as console;
use gloo_dialogs::alert;
use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use serde_json;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::board::BlokusBoard;
use crate::pieces::PieceTray;
use blokus::game::Game;

const SERVER_ADDRESS: &str = "http://127.0.0.1:8000/process_request";
const D: usize = 20;

#[derive(Serialize, Deserialize, Debug)]
struct GameStateRequest {
    player: usize,
    data: [[[bool; D]; D]; 5],
}

#[derive(Serialize, Deserialize, Debug)]
struct GameStateResponse {
    policy: Vec<f32>,
    values: Vec<f32>,
    status: i32,
}

fn print_rep(rep: [[[bool; D]; D]; 5]) {
    let mut str_rep = String::new();
    for i in 0..5 {
        for j in 0..D {
            for k in 0..D {
                if rep[i][j][k] {
                    str_rep.push_str("[X]");
                } else {
                    str_rep.push_str("[ ]");
                }
            }
            str_rep.push_str("\n");
        }
        str_rep.push_str("\n");
    }
    console::log!(str_rep);
}

fn get_state_rep(game: &Game) -> GameStateRequest {
    GameStateRequest {
        player: game.current_player(),
        data: game.get_board_state(),
    }
}

/// Takes state and returns tile to place
async fn get_ai_move(state: &Game) -> Result<usize, String> {
    let request = get_state_rep(state);
    print_rep(request.data);
    let serialized_request = serde_json::to_string(&request).unwrap();

    // Send POST request to FastAPI server
    match Request::post(SERVER_ADDRESS)
        .header("Content-Type", "application/json")
        .body(serialized_request)
        .send()
        .await
    {
        Ok(response) => {
            let json_value = response.json().await.unwrap();
            let response: GameStateResponse = serde_json::from_value(json_value).unwrap();
            if response.status != 200 {
                console::error!("AI failed to find a move");
            }
            let tile = response
                .policy
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(i, _)| i)
                .expect("No policy found");

            // Return tile to right perspective
            let row = tile / D;
            let col = tile % D;
            let (new_row, new_col) = match state.current_player() {
                0 => (row, col),
                1 => (col, D - row - 1),
                2 => (D - row - 1, D - col - 1),
                3 => (D - col - 1, row),
                _ => panic!("Invalid player number"),
            };

            Ok(new_row * D + new_col)
        }
        Err(e) => Err(format!("Failed to get AI move: {:?}", e)),
    }
}

/// Applies AI moves to state after player has gone
async fn handle_ai_moves(state: Game) -> Game {
    let mut next_state = state.clone();
    let mut current_ai = next_state.current_player();
    while current_ai != 0 && !next_state.is_terminal() {
        // THIS IS THE CONDITION, DOESN'T WORK WHEN HUMAN IS ELIMINATED
        let tile = get_ai_move(&next_state).await.unwrap();
        if let Err(e) = next_state.apply(tile, None) {
            console::error!("Failed to apply AI move:m", e);
            break;
        }

        current_ai = next_state.current_player();
        console::log!("AI placed piece at: {:?}", tile);
        console::log!("Current player: ", current_ai);
    }

    next_state
}

fn alert_game_over(game: &Game) {
    let scores = game.get_score();
    let best_score = scores.iter().max().unwrap();
    let winners = scores
        .iter()
        .enumerate()
        .filter_map(|(i, s)| if s == best_score { Some(i) } else { None })
        .collect::<Vec<_>>();

    let mut message = if winners.len() == 1 {
        format!("Player {} wins!", winners[0] + 1)
    } else {
        format!("Players {:?} tie!", winners.iter().map(|w| w + 1).collect::<Vec<_>>())
    };

    message.push_str("\n\nScores:\n");
    for (i, score) in scores.iter().enumerate() {
        message.push_str(&format!("Player {}: {}\n", i + 1, score));
    }
    alert(&message);
}

#[function_component]
pub fn App() -> Html {
    let state = use_state(|| Game::reset());

    let on_board_drop = {
        let state = state.clone();
        Callback::from(move |(p, v, offset)| {
            // Don't do anything if game is over
            if state.is_terminal() {
                return;
            }

            // Place piece on board
            let new_state = match state.place_piece(p, v, offset) {
                Ok(s) => s,
                Err(e) => {
                    console::error!("Failed to place piece: {:?}", e);
                    return;
                }
            };
            let game = new_state.clone();
            state.set(new_state);

            // Check if game is over
            if game.is_terminal() {
                alert_game_over(&game);
                return;
            }

            // Handle AI moves
            let state = state.clone();
            spawn_local({
                async move {
                    let new_state = handle_ai_moves(game.clone()).await;
                    state.set(new_state.clone());
                    if new_state.is_terminal() {
                        alert_game_over(&game);
                    }
                }
            });
        })
    };

    let on_reset = {
        let state = state.clone();
        Callback::from(move |_| state.set(Game::reset()))
    };

    html! {
        <div>
            <h1>{ "Blokus Engine" }</h1>

            <BlokusBoard board={state.get_board()} on_board_drop={on_board_drop} anchors={state.get_current_anchors()} />
            <PieceTray pieces={state.get_current_player_pieces()} player_num={state.current_player() as u8 + 1} />

            <button onclick={on_reset}>{ "Reset Game" }</button>

            <h2>{ "Players Remaining" }</h2>
            <p>{ format!("Player 1: {}", state.is_player_active(0)) }</p>
            <p>{ format!("Player 2: {}", state.is_player_active(1)) }</p>
            <p>{ format!("Player 3: {}", state.is_player_active(2)) }</p>
            <p>{ format!("Player 4: {}", state.is_player_active(3)) }</p>

            <h2>{ "Controls" }</h2>

            <p style={"white-space: pre-line"}>{"
                Select Piece: click\n
                Rotate Piece: r\n
                Flip Piece: f\n
                Place Piece: Drag onto board\n
            "}</p>
        </div>
    }
}

use gloo_console as console;
use tonic_web_wasm_client::Client;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::board::BlokusBoard;
use crate::grpc::blokus_model_client::BlokusModelClient;
use crate::grpc::StateRepresentation;
use crate::pieces::PieceTray;
use blokus::game::Game;

const SERVER_ADDRESS: &str = "http://0.0.0.0:8092";
const BOARD_SIZE: usize = 400;

trait RpcRep {
    fn get_rpc_rep(&self) -> StateRepresentation;
}

impl RpcRep for Game {
    /// Get a representation of the state for the neural network
    /// This representation includes the board and the legal tiles
    /// Oriented to the current player
    fn get_rpc_rep(&self) -> StateRepresentation {
        // Get rep for the pieces on the board
        let current_player = self.current_player().expect("No current player");
        let board = &self.board.board;
        let mut board_rep = [[false; BOARD_SIZE]; 5];
        for i in 0..BOARD_SIZE {
            let player = (board[i] & 0b1111) as usize; // check if there is a player piece
            let player_board = (4 + player - current_player) % 4; // orient to current player (0 indexed)
            if player != 0 {
                board_rep[player_board][i] = true;
            }
        }

        // Get rep for the legal spaces
        let legal_moves = self.legal_tiles();
        console::log!(format!("Legal moves: {:?}", legal_moves));
        for tile in legal_moves {
            board_rep[4][tile] = true;
        }

        StateRepresentation {
            boards: board_rep.into_iter().flat_map(|inner| inner).collect(),
            player: current_player as i32,
        }
    }
}

/// Takes state and returns tile to place
async fn get_ai_move(
    state: &Game,
    client: &mut BlokusModelClient<Client>,
) -> Result<usize, String> {
    let representation = state.get_rpc_rep();
    let request = tonic::Request::new(representation);

    let prediction = client
        .predict(request)
        .await
        .map_err(|e| format!("Prediction error: {}", e))?
        .into_inner();
    let policy = prediction.policy;
    console::log!(format!("Policy: {:?}", policy));

    // Process policy and value as needed
    let tile = policy
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(i, _)| i)
        .expect("No policy found");

    Ok(tile)
}

/// Applies AI moves to state after player has gone
async fn handle_ai_moves(state: Game, client: &mut BlokusModelClient<Client>) -> Game {
    let mut next_state = state.clone();
    let mut current_ai = next_state.current_player().unwrap();
    while current_ai != 0 {
        match get_ai_move(&next_state, client).await {
            Ok(tile) => {
                if let Err(e) = next_state.apply(tile, None) {
                    console::error!("Failed to apply AI move: {:?}", e);
                    break;
                }
                console::log!("AI placed piece at: {:?}", tile);
            }
            Err(e) => {
                console::error!("Failed to get AI move: {:?}", e);
                break;
            }
        }
        current_ai = next_state.current_player().unwrap();
    }

    next_state
}

#[function_component]
pub fn App() -> Html {
    let state = use_state(|| Game::reset());

    let on_board_drop = {
        let state = state.clone();
        Callback::from(move |(p, v, offset)| {
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

            // Handle AI moves
            let state = state.clone();
            spawn_local({
                async move {
                    let mut client =
                        BlokusModelClient::new(Client::new(SERVER_ADDRESS.to_string()));
                    let new_state = handle_ai_moves(game, &mut client).await;
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

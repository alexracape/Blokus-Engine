use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use gloo_console as console;
use tonic_web_wasm_client::Client;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::grpc::blokus_model_client::BlokusModelClient;
use crate::grpc::StateRepresentation;
use blokus::game::{Action, Game};
use blokus::pieces::{Piece, PieceVariant};

const SERVER_ADDRESS: &str = "http://[::1]:8082";
const BOARD_SIZE: usize = 400;

/// Wrapper for the game state so that we can implement Reducible
#[derive(Clone)]
pub struct GameState {
    pub state: Rc<Game>,
    on_state_change: Callback<Rc<Game>>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            state: Rc::new(Game::reset()),
            on_state_change: Callback::noop(),
        }
    }

    pub fn dispatch(&self, action: Action) {
        match action {
            Action::PlacePiece(piece_index, variant, offset) => {
                let player = self.state.current_player().unwrap();
                let piece = self.state.get_piece(player, piece_index, variant);

                // Check if move is valid
                if !self.state.board.is_valid_move(player, &piece, offset) {
                    console::error!("Invalid move");
                    return;
                }

                // Apply move
                let mut new_state = (*self.state).clone(); // new state with move applied
                let offsets = piece.offsets.clone();
                let last_index = offsets.len().saturating_sub(1);
                for (i, tile_offset) in offsets.iter().enumerate() {
                    let tile = offset + tile_offset;
                    let piece_to_finish = if i == last_index {
                        Some(piece_index)
                    } else {
                        None
                    };
                    new_state
                        .apply(tile, piece_to_finish)
                        .expect("Failed to apply move");
                }

                // Notify components of state change
                self.state = Rc::new(new_state);
                self.on_state_change.emit(self.state.clone());

                // Handle AI moves
                spawn_local(async move {
                    let mut client =
                        BlokusModelClient::new(Client::new(SERVER_ADDRESS.to_string()));
                    let state_for_ai = (*self.state).clone();
                    let current_ai = state_for_ai.next_player();
                    while current_ai != player {
                        if let Err(e) = handle_ai_move(&mut client, &mut state_for_ai, player).await
                        {
                            console::log!("Failed to get AI move: {:?}", e);
                            break;
                        }
                        current_ai = state_for_ai.next_player();
                        self.state = Rc::new(state_for_ai.clone());
                        self.on_state_change.emit(self.state.clone());
                    }
                })
            }
            Action::Pass => {
                self.state.eliminate_player();
                let player = self.state.current_player().unwrap();
                if self.state.is_terminal() {
                    console::log!("Game over!");
                    return; // TODO: handle game over
                }
            }
            Action::Undo => (),
            Action::ResetGame => (),
        }
    }

    // pub fn apply(&mut self, tile: usize, piece_to_finish: Option<usize>) -> Result<(), String> {
    //     self.0.apply(tile, piece_to_finish)
    // }

    // pub fn current_player(&self) -> Option<usize> {
    //     self.0.current_player()
    // }

    // pub fn next_player(&self) -> usize {
    //     self.0.next_player()
    // }

    // pub fn eliminate_player(&mut self) {
    //     self.0.eliminate_player();
    // }

    // pub fn is_terminal(&self) -> bool {
    //     self.0.is_terminal()
    // }

    // pub fn get_board(&self) -> &[u8; 400] {
    //     self.0.get_board()
    // }

    // pub fn get_current_player_pieces(&self) -> Vec<Piece> {
    //     self.0.get_current_player_pieces()
    // }

    // pub fn get_current_anchors(&self) -> HashSet<usize> {
    //     self.0.get_current_anchors()
    // }

    // pub fn legal_tiles(&self) -> Vec<usize> {
    //     self.0.legal_tiles()
    // }

    // pub fn is_valid_move(&self, player: usize, piece: &PieceVariant, offset: usize) -> bool {
    //     self.0.board.is_valid_move(player, piece, offset)
    // }
}

trait RpcRep {
    fn get_representation(&self) -> StateRepresentation;
}

impl RpcRep for Game {
    /// Get a representation of the state for the neural network
    /// This representation includes the board and the legal tiles
    /// Oriented to the current player
    fn get_representation(&self) -> StateRepresentation {
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
        for tile in legal_moves {
            board_rep[4][tile] = true;
        }

        StateRepresentation {
            boards: board_rep.into_iter().flat_map(|inner| inner).collect(),
            player: current_player as i32,
        }
    }
}

async fn handle_ai_move(
    model: &mut BlokusModelClient<Client>,
    new_state: &mut Game,
    player: usize,
) -> Result<(), String> {
    let representation = new_state.get_representation();
    let request = tonic::Request::new(representation);

    let prediction = model
        .predict(request)
        .await
        .map_err(|e| format!("Prediction error: {}", e))?
        .into_inner();
    let policy = prediction.policy;
    let value = prediction.value;

    // Process policy and value as needed
    let tile = policy
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(i, _)| i)
        .expect("No policy found");
    new_state.apply(tile, None);

    Ok(())
}

// impl Reducible for GameState {
//     type Action = Action;

//     fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
//         match action {
//             Action::PlacePiece(p, v, o) => {
//                 let player = self.current_player().expect("No current player");
//                 let mut new_state = (*self).clone();
//                 let piece = self.get_piece(player, p, v);

//                 // Check if move is valid
//                 if !new_state.is_valid_move(player, &piece, o) {
//                     console::log!("Invalid move");
//                     return self.into();
//                 }

//                 // Break move into tiles and apply individually
//                 let offsets = piece.offsets.iter().collect::<Vec<_>>();
//                 let last_index = offsets.len().saturating_sub(1);
//                 for (i, tile_offset) in offsets.iter().enumerate() {
//                     let tile = o + *tile_offset;
//                     let result = if i == last_index {
//                         new_state.apply(tile, Some(p))
//                     } else {
//                         new_state.apply(tile, None)
//                     };

//                     if let Err(e) = result {
//                         console::log!("Error applying move: {}", e);
//                         return self.into();
//                     }
//                 }

//                 wasm_bindgen_futures::spawn_local(async {
//                     let mut client =
//                         BlokusModelClient::new(Client::new(SERVER_ADDRESS.to_string()));
//                     let mut current_ai = new_state.next_player();
//                     while current_ai != player {
//                         if let Err(e) = handle_ai_move(&mut client, &mut new_state, player).await {
//                             console::log!("{}", e);
//                             break;
//                         }
//                         current_ai = new_state.next_player();
//                     }
//                 });

//                 new_state.into()
//             }
//             Action::Pass => {
//                 let mut new_state = (*self).clone();
//                 new_state.eliminate_player();

//                 if new_state.is_terminal() {
//                     return GameState::reset().into(); // TODO - need to handle better with message or something
//                 }

//                 new_state.into()
//             }
//             Action::Undo => {
//                 let new_state = (*self).clone();
//                 // TODO: Need to implement undo
//                 new_state.into()
//             }
//             Action::ResetGame => GameState::reset().into(),
//         }
//     }
// }

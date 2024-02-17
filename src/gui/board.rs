use std::collections::HashSet;

use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use yew::prelude::*;
use yew::{function_component, html, Properties};
use yew::events::DragEvent;

use crate::board::BOARD_SIZE;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub board: [u8; BOARD_SIZE * BOARD_SIZE],
    pub on_board_drop: Callback<(usize, usize, usize)>,
    pub anchors: HashSet<usize>,
}

#[function_component]
pub fn BlokusBoard(props: &Props) -> Html {
    let Props { board, on_board_drop, anchors } = props.clone();

    let ondragover = {
        move |event: DragEvent| {
            event.prevent_default();
        }
    };

    html! {
        <div class="board">
        {for (0..BOARD_SIZE).map(|i| {
            
            html! {
                <div class="board-row">
                {
                    for (0..BOARD_SIZE).map(|j| {
                        let index = i * BOARD_SIZE + j;
                        let mut square_style = match board[index] & 0b1111 {
                            1 => "square red".to_string(),
                            2 => "square blue".to_string(),
                            3 => "square green".to_string(),
                            4 => "square yellow".to_string(),
                            _ => "square empty".to_string(),
                        };

                        if anchors.contains(&index) {
                            square_style = format!("{} anchor", square_style);
                        }

                        let ondrop = {
                            on_board_drop.reform(move |e: DragEvent| {
                                e.prevent_default();

                                let target: HtmlElement = e.target().unwrap().dyn_into().unwrap();
                                let data = e.data_transfer().expect("Data transfer should exist");
                                let id = data.get_data("piece_num").expect("Dragged piece should have an id");
                                let variant = data.get_data("variant").unwrap().parse().unwrap();
                                let clicked_square: usize = data.get_data("piece_offset").unwrap().parse().unwrap();

                                let piece: usize = id.parse().unwrap();
                                let offset: usize = target.id().parse().unwrap();
                                let offset = offset - clicked_square;
                                (piece, variant, offset)
                            })
                        };

                        html! {
                            <div id={index.to_string()}  class={square_style} {ondrop} {ondragover} ></div>
                        }
                    })
                }
                </div>
            }
        })}
        </div>
    }
}
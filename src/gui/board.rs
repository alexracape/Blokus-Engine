use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, Window};
use gloo_console as console;

use yew::prelude::*;
use yew::{function_component, html, Properties};
use yew::events::DragEvent;

use crate::board::BOARD_SIZE;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub board: [u8; BOARD_SIZE * BOARD_SIZE],
    pub on_board_drop: Callback<(usize, usize, usize)>,
}

#[function_component]
pub fn BlokusBoard(props: &Props) -> Html {
    let Props { board, on_board_drop } = props.clone();

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
                        let square_style = match board[index] & 0b1111 {
                            1 => "square red",
                            2 => "square blue",
                            3 => "square green",
                            4 => "square yellow",
                            _ => "square empty"
                        };

                        let ondrop = {
                            on_board_drop.reform(move |e: DragEvent| {
                                e.prevent_default();

                                let target: HtmlElement = e.target().unwrap().dyn_into().unwrap();
                                let data = e.data_transfer().expect("Data transfer should exist");
                                let id = data.get_data("id").expect("Dragged piece should have an id");
                                let variant = data.get_data("variant").unwrap().parse().unwrap();

                                let piece: usize = id.parse().unwrap();
                                let offset: usize = target.id().parse().unwrap();
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
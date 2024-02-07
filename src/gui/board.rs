use yew::prelude::*;
use yew::{function_component, html, Properties};

use crate::board::BOARD_SIZE;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub board: [u8; BOARD_SIZE * BOARD_SIZE],
}
#[function_component]
pub fn BlokusBoard(props: &Props) -> Html {
    html! {
        <div class="board">
        {for (0..BOARD_SIZE).map(|i| {
            html! {
                <div class="board-row">
                {
                    for (0..BOARD_SIZE).map(|j| {
                        let index = i * BOARD_SIZE + j;
                        let square_style = match props.board[index] & 0b1111 {
                            1 => "square red",
                            2 => "square blue",
                            3 => "square green",
                            4 => "square yellow",
                            _ => "square empty"
                        };
                        html! {

                            <div class={square_style}></div>
                        }
                    })
                }
                </div>
            }
        })}
        </div>
    }
}
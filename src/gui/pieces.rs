use yew::prelude::*;
use yew::{function_component, html, Properties};

use crate::player::Player;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub player: Player,
}

#[function_component]
pub fn PieceTray(props: &Props) -> Html {
    html! {
        <div class="piece-tray">
            <div class="piece-tray-inner">
                { for props.player.pieces.iter().map(|piece| html! {
                    <Piece piece={piece.shape.clone()} />
                })
                }
            </div>
        </div>
    }
}


#[derive(Properties, PartialEq)]
pub struct PieceProps {
    pub piece: Vec<Vec<bool>>,
}

#[function_component]
fn Piece(props: &PieceProps) -> Html {
    html! {
        <div class="piece">
            { for props.piece.iter().enumerate().map(|(row_index, row)| html! {
                <div class="grid-row" key={row_index}>
                    { for row.iter().enumerate().map(|(col_index, &cell)| html! {
                        <div class={classes!("square", if cell { "red" } else { "empty" })} key={col_index}></div>
                    })}
                </div>
            })}
        </div>
    }
}
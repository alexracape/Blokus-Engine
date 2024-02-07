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
            <h3>{ format!("Player {}", props.player.num) }</h3>
            <div class="piece-tray-inner">
                { for props.player.pieces.iter().map(|piece| html! {
                    <div class="piece">
                        <Piece piece={piece.shape.clone()} />
                    </div>
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
            { for props.piece.iter().map(|row| html! {
                    { for row.iter().map(|cell| html! {
                        <div class={if *cell { "square red" } else { "square" }}></div>
                    })}
            })}
        </div>
    }
}
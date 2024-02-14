use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use gloo_console as console;

use yew::prelude::*;
use yew::{function_component, html, Properties};
use yew::events::DragEvent;

use crate::player::Player;
use crate::pieces::Piece;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub player: Player,
}

#[function_component]
pub fn PieceTray(props: &Props) -> Html {
    html! {
        <div class="piece-tray">
            <div class="piece-tray-inner">
                { for props.player.pieces.iter().enumerate().map(|(idx, piece)| html! {
                    <GUIPiece piece={piece.clone()} idx={idx.to_string()} />
                })
                }
            </div>
        </div>
    }
}


#[derive(Properties, PartialEq)]
pub struct PieceProps {
    pub piece: Piece,
    pub idx: String,
}

#[function_component]
fn GUIPiece(props: &PieceProps) -> Html {

    // State
    let variant = use_state(|| 0);

    let ondragstart = {
        move |event: DragEvent| {
            let target = event.target().unwrap();
            let target: HtmlElement = target.dyn_into().unwrap();
            target.class_list().add_1("dragging").unwrap();

            let data = event.data_transfer().unwrap();
            let _ = data.set_data("id", target.id().as_str());
            console::log!("Drag start", event);
        } 
    };

    let ondragend = {
        move |event: DragEvent| {
            let target = event.target().unwrap();
            let target: HtmlElement = target.dyn_into().unwrap();
            target.class_list().remove_1("dragging").unwrap();
            console::log!("Drag end", event);
        }
    };

    html! {
        <div id={props.idx.clone()} class={classes!("piece")} draggable="true" {ondragstart} {ondragend}>
            { for props.piece.shape.iter().enumerate().map(|(row_index, row)| html! {
                <div class="grid-row" key={row_index}>
                    { for row.iter().enumerate().map(|(col_index, &cell)| html! {
                        <div class={classes!("square", if cell { "red" } else { "blank" })} key={col_index}></div>
                    })}
                </div>
            })}
        </div>
    }
}
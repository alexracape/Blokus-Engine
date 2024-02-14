
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use gloo_console as console;

use yew::prelude::*;
use yew::{function_component, html, Properties};
use yew::events::DragEvent;

use crate::player::Player;
use crate::pieces::Piece;
use yew::Callback;
use web_sys::KeyboardEvent;

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
    let mut shape = props.piece.shape.clone();
    let variant = use_state(|| 0);
    let rotation = use_state(|| 0);
    let flip = use_state(|| false);

    // TODO: need better logic to keep track of variant
    // Flip then rotate is different than rotate then flip
    // Maybe just keep track of variant and reconstruct shape from that?
    // Shape from variant method?

    // Rotate piece for each rotation in state before rendering
    for _ in 0..*rotation / 90 {
        shape = Piece::rotate(shape);
    }

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

    let rotate = {
        let num_variants = props.piece.variants.len();
        let variant = variant.clone();
        Callback::from(move |_| {
            variant.set((*variant + 1) % num_variants);
            rotation.set((*rotation + 90) % 360);
        })
    };

    let flip = {
        let num_variants = props.piece.variants.len();
        let variant = variant.clone();
        Callback::from(move |_| {
            let num_variants = num_variants;
            variant.set((*variant + 1) % num_variants);
        })
    };

    let onkeypress = {
        Callback::from(move |event: KeyboardEvent| {
            match event.key().as_str() {
                "r" => rotate.emit(event),
                "f" => flip.emit(event),
                _ => console::log!("Key pressed", event.key()),
            }
        })
    };

    html! {
        <div id={props.idx.clone()} class={classes!("piece")} draggable="true" {ondragstart} {ondragend} {onkeypress} tabindex="0">
            { for shape.iter().enumerate().map(|(row_index, row)| html! {
                <div class="grid-row" key={row_index}>
                    { for row.iter().enumerate().map(|(col_index, &cell)| html! {
                        <div class={classes!("square", if cell { "red" } else { "blank" })} key={col_index}></div>
                    })}
                </div>
            })}
        </div>
    }
}
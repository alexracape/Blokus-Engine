
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use gloo_console as console;

use yew::prelude::*;
use yew::{function_component, html, Properties};
use yew::events::DragEvent;

use crate::pieces::Piece;
use yew::Callback;
use web_sys::KeyboardEvent;


#[derive(Properties, PartialEq)]
pub struct Props {
    pub pieces: Vec<Piece>,
}

#[function_component]
pub fn PieceTray(props: &Props) -> Html {
    html! {
        <div class="piece-tray">
            <div class="piece-tray-inner">
                { for props.pieces.iter().enumerate().map(|(idx, piece)| html! {
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
        let variant = variant.clone();
        move |event: DragEvent| {
            let target = event.target().unwrap();
            let target: HtmlElement = target.dyn_into().unwrap();
            target.class_list().add_1("dragging").unwrap();

            let data = event.data_transfer().unwrap();
            let _ = data.set_data("id", target.id().as_str());
            let _ = data.set_data("variant", &*variant.to_string().as_str());
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
            let next = match num_variants {
                1 => 0,
                2 => {(*variant + 1) % 2},
                4 => {(*variant + 1) % 4},
                8 => {
                    if *variant > 3 {
                        (*variant + 1) % 4 + 4
                    } else {
                        (*variant + 1) % 4
                    }
                },
                _ => 0,
            };
            variant.set(next);
            console::log!("ROTATE", *variant)
        })
    };

    let flip = {
        let num_variants = props.piece.variants.len();
        let variant = variant.clone();
        Callback::from(move |_| {
            let next = match num_variants  {
                1 => 0,
                2 => *variant,
                4 => {(*variant + 2) % 4}, // Depends on symmetry of shape but this is okay for now
                8 => {(*variant + 4) % 8},
                _ => 0,
            };
            variant.set(next); // Edit to go to opposite side of cycle
            console::log!("FLIP", *variant)
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
            { for props.piece.variants.get(*variant).unwrap().get_shape().iter().enumerate().map(|(row_index, row)| html! {
                <div class="grid-row" key={row_index}>
                    { for row.iter().enumerate().map(|(col_index, &cell)| html! {
                        <div class={classes!("square", if cell { "red" } else { "blank" })} key={col_index}></div>
                    })}
                </div>
            })}
        </div>
    }
}
use gloo_console as console;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use yew::events::DragEvent;
use yew::prelude::*;
use yew::{function_component, html, Properties};

use blokus::pieces::{Piece, PieceVariant};
use web_sys::KeyboardEvent;
use yew::Callback;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub pieces: Vec<Piece>,
    pub player_num: u8,
}

#[function_component]
pub fn PieceTray(props: &Props) -> Html {
    let color = match props.player_num {
        1 => "red",
        2 => "blue",
        3 => "green",
        4 => "yellow",
        _ => "empty",
    };
    html! {
        <div class="piece-tray">
            <div class="piece-tray-inner">
                { for props.pieces.iter().enumerate().map(|(idx, piece)| html! {
                    <GUIPiece key={piece.id} piece={piece.clone()} piece_num={idx.to_string()} color={color} />
                })
                }
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct PieceProps {
    pub piece: Piece,
    pub piece_num: String,
    pub color: &'static str,
}

#[function_component]
fn GUIPiece(props: &PieceProps) -> Html {
    // State
    let variant = use_state(|| 0);
    let clicked_square = use_state(|| 0);

    let ondragstart = {
        let variant = variant.clone();
        let clicked_square = clicked_square.clone();
        let piece = props.piece.clone();
        move |event: DragEvent| {
            let target = event.target().unwrap();
            let target: HtmlElement = target.dyn_into().unwrap();
            target.class_list().add_1("dragging").unwrap();
            let piece_variant = piece.variants.get(*variant).unwrap();
            let offset = piece_variant
                .offsets
                .get(*clicked_square)
                .unwrap()
                .to_string();

            let data = event.data_transfer().unwrap();
            let _ = data.set_data(
                "piece_num",
                target.get_attribute("data-piece-num").unwrap().as_str(),
            );
            let _ = data.set_data("variant", &*variant.to_string().as_str());
            let _ = data.set_data("piece_offset", offset.as_str());
            console::log!("Drag start", event);
            console::log!("Piece offset", offset);
        }
    };

    let squareclicked = {
        let clicked_square = clicked_square.clone();
        move |event: MouseEvent| {
            let target = event.target().unwrap();
            let target: HtmlElement = target.dyn_into().unwrap();
            let square = target.get_attribute("data-square").unwrap();
            clicked_square.set(square.clone().parse().unwrap());
            console::log!("Square clicked", square);
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
                2 => (*variant + 1) % 2,
                4 => (*variant + 1) % 4,
                8 => {
                    if *variant > 3 {
                        (*variant + 1) % 4 + 4
                    } else {
                        (*variant + 1) % 4
                    }
                }
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
            let next = match num_variants {
                1 => 0,
                2 => *variant,
                4 => (*variant + 2) % 4, // Depends on symmetry of shape but this is okay for now
                8 => (*variant + 4) % 8,
                _ => 0,
            };
            variant.set(next);
            console::log!("FLIP", *variant)
        })
    };

    let onkeypress = {
        Callback::from(move |event: KeyboardEvent| match event.key().as_str() {
            "r" => rotate.emit(event),
            "f" => flip.emit(event),
            _ => console::log!("Key pressed", event.key()),
        })
    };

    let mut square_num = -1;
    let v: &PieceVariant = props
        .piece
        .variants
        .get(*variant)
        .expect(format!("Variant {:?} not found", props.piece.variants).as_str());
    html! {
        <div data-piece-num={props.piece_num.clone()} class={classes!("piece")} draggable="true" {ondragstart} {ondragend} {onkeypress} tabindex="0">
            {for v.get_shape().iter().enumerate().map(|(row_index, row)| html! {
                <div class="grid-row" key={row_index}>
                    { for row.iter().enumerate().map(|(col_index, &cell)|
                        if cell {
                            square_num += 1;
                            let onmousedown = squareclicked.clone();
                            html! {
                                <div class={classes!("square", props.color)} key={col_index} {onmousedown} data-square={square_num.to_string()}></div>
                            }
                        } else {
                            html! {
                                <div class={classes!("square", "blank")} key={col_index}></div>
                            }
                        }
                    )}
                </div>
            })}
        </div>
    }
}

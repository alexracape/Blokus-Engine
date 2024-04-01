mod board;
mod gui;
mod pieces;
mod player;
mod game;

mod simulation;
mod node;

pub mod grpc {
    tonic::include_proto!("blokusmodel");
}

use crate::gui::app::App;


fn main() {

    // Run the GUI, will be called by trunk serve
    yew::Renderer::<App>::new().render();

}

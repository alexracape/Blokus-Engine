mod board;
mod gui;
mod pieces;
mod player;
mod state;

mod simulation;
mod game_tree;

pub mod grpc {
    tonic::include_proto!("blokusmodel");
}

use crate::gui::app::App;


fn main() {

    // Run the GUI, will be called by trunk serve
    yew::Renderer::<App>::new().render();

}

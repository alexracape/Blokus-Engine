mod board;
mod game_tree;
mod gui;
mod pieces;
mod player;
mod state;

mod simulation;

use crate::gui::app::App;


fn main() {

    // Run the GUI, will be called by trunk serve
    yew::Renderer::<App>::new().render();

}

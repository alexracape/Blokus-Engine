mod app;
mod board;
mod game_state;
mod pieces;
mod grpc {
    tonic::include_proto!("blokusmodel");
}

use app::App;

fn main() {
    // Run the GUI, will be called by trunk serve
    yew::Renderer::<App>::new().render();
}

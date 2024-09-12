mod app;
mod board;
mod pieces;

use app::App;

fn main() {
    // Run the GUI, will be called by trunk serve
    yew::Renderer::<App>::new().render();
}

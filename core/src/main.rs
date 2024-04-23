use blokus_backend::gui::app::App;

fn main() {
    // Run the GUI, will be called by trunk serve
    yew::Renderer::<App>::new().render();
}

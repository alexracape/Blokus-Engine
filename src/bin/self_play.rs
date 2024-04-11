use blokus_backend::simulation::play_game;


const SELF_PLAY_GAMES: usize = 1; // 21 million for Go in AlphaZero


fn main() {

    let server_address = std::env::args().nth(1).unwrap_or("http://[::1]:50051".to_string());
    for _ in 0..SELF_PLAY_GAMES {
        let result = play_game(server_address.clone());
        match result {
            Ok(status) => println!("{}", status),
            Err(e) => {
                println!("Error playing game: {:?}", e);
                break;
            }
        }
    }
}
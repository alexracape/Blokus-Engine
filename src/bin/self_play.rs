use std::env;
use dotenv;
use blokus_backend::client::simulation::play_game;


fn main() {

    // Load environment variables from .env file
    dotenv::dotenv().ok();
    let games: usize = env::var("GAMES_PER_CLIENT").unwrap().parse::<usize>().unwrap(); 
    let server_address = env::var("SERVER_ADDRESS").unwrap();

    for _ in 0..games {
        let result = play_game(format!("http://{}", server_address));
        match result {
            Ok(status) => println!("{}", status),
            Err(e) => {
                println!("Error playing game: {:?}", e);
                break;
            }
        }
    }
}
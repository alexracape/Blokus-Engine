mod simulation;
mod node;
mod grpc {
    tonic::include_proto!("blokusmodel");
}

use dotenv;
use simulation::play_game;
use std::env;
// use std::{fmt, default, result, future, pin, marker};

fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    let games: usize = env::var("GAMES_PER_CLIENT")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let server_address = env::var("SERVER_URL").unwrap();

    for _ in 0..games {
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

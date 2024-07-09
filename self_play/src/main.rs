mod node;
mod simulation;
mod grpc {
    tonic::include_proto!("blokusmodel");
}

use crate::grpc::blokus_model_client::BlokusModelClient;
use crate::grpc::Empty;
use dotenv;
use simulation::play_game;
use std::env;

// use std::{fmt, default, result, future, pin, marker};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    let games: usize = env::var("GAMES_PER_CLIENT")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let rounds: usize = env::var("TRAINING_ROUNDS")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let server_address = env::var("SERVER_URL").unwrap();

    // Connect to neural network
    println!("Connecting to server at: {}", server_address);
    let mut model = BlokusModelClient::connect(server_address).await?;

    let mut round = 0;
    while round < rounds {
        
        // Play games to generate data
        for i in 0..games {
            let result = play_game(&mut model).await;
            match result {
                Ok(status) => println!("Game {i} finished: {}", status),
                Err(e) => {
                    println!("Error playing game: {:?}", e);
                    break;
                }
            }
        }

        // Wait for model to train
        println!("Waiting for model to train...");
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let response = model.check(tonic::Request::new(Empty{})).await?;
            let current_round = response.into_inner().code as usize;
            if current_round > round {
                break;
            }
        } 
        round += 1;

    }

    println!("Training complete!");
    Ok(())
}

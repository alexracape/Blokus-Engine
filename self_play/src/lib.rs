mod node;
mod simulation;

use pyo3::prelude::*;
use pyo3::types::{PyList, PyTuple};
use simulation::play_game;
use simulation::Config;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn generate_game_data(
    id: i32,
    config: PyObject,
    inference_queue: PyObject,
    pipe: PyObject,
) -> PyResult<(Vec<(i32, i32)>, Vec<Vec<(i32, f32)>>, Vec<f32>)> {
    let game_data = Python::with_gil(|py| {
        let config: Config = config.extract::<Config>(py).unwrap();
        let i_queue = inference_queue.bind(py);
        let r_queue = pipe.bind(py);
        play_game(config, i_queue, r_queue, id)
    });

    match game_data {
        Ok(data) => Ok(data),
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!(
            "{:?}",
            e
        ))),
    }
}

#[pymodule]
fn blokus_self_play(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(generate_game_data, m)?)
}

// use crate::grpc::blokus_model_client::BlokusModelClient;
// use crate::grpc::Empty;
// use dotenv;
// use simulation::play_game;
// use std::env;

// // use std::{fmt, default, result, future, pin, marker};

// #[tokio::main]
// pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // Load environment variables from .env file
//     dotenv::dotenv().ok();
//     let games: usize = env::var("GAMES_PER_CLIENT")
//         .unwrap()
//         .parse::<usize>()
//         .unwrap();
//     let rounds: usize = env::var("TRAINING_ROUNDS")
//         .unwrap()
//         .parse::<usize>()
//         .unwrap();
//     let check_interval: u64 = env::var("CHECK_INTERVAL").unwrap().parse::<u64>().unwrap();
//     let server_address = env::var("SERVER_URL").unwrap();

//     // Connect to neural network
//     let mut model = BlokusModelClient::connect(server_address.clone()).await?;
//     println!("Connected to server at: {}", server_address);

//     let mut round = 0;
//     while round < rounds {
//         // Play games to generate data
//         for i in 0..games {
//             let result = play_game(&mut model).await;
//             match result {
//                 Ok(status) => println!("Game {i} finished: {}", status),
//                 Err(e) => {
//                     println!("Error playing game: {:?}", e);
//                     break;
//                 }
//             }
//         }

//         // Wait for model to train
//         println!("Waiting for model to train...");
//         loop {
//             tokio::time::sleep(tokio::time::Duration::from_secs(check_interval)).await;

//             let response = model.check(tonic::Request::new(Empty {})).await?;
//             let current_round = response.into_inner().code as usize;
//             if current_round > round {
//                 break;
//             }
//         }
//         round += 1;
//     }

//     println!("Training complete!");
//     Ok(())
// }

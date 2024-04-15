mod board;
mod pieces;
mod game;
mod node;

pub mod client;
pub mod gui;

pub mod grpc {
    tonic::include_proto!("blokusmodel");
}

mod board;
mod pieces;
mod game;
mod node;

pub mod simulation;
pub mod gui;

pub mod grpc {
    tonic::include_proto!("blokusmodel");
}

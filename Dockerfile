# Use the official Rust image as the base image
FROM rust:latest

# Set the working directory inside the container
WORKDIR /client

# Copy the Cargo.toml and Cargo.lock files to the working directory
COPY Cargo.toml Cargo.lock /client/

# Copy the source code to the working directory
COPY ./self_play /client/self_play
COPY ./blokus /client/blokus
COPY ./gui /client/gui

# Get protobuf dependencies
RUN apt update && apt upgrade -y
RUN apt install -y protobuf-compiler libprotobuf-dev
COPY /proto/model.proto ./proto/model.proto

# Pre-compile dependencies to cache them
# COPY Cargo.toml Cargo.lock ./
RUN mkdir src/ && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f src/main.rs

# Build the source code
RUN cargo build --release --bin self_play

# Run the simulation client
ENTRYPOINT ["cargo", "run", "--bin", "self_play"]

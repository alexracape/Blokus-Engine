# Use the official Rust image as the base image
FROM rust:latest

# Set the working directory inside the container
WORKDIR /client

# Copy the Cargo.toml and Cargo.lock files to the working directory
COPY Cargo.toml Cargo.lock build.rs ./

# Build the dependencies separately to leverage Docker layer caching - Should add --release flag later
# RUN cargo build

# Copy the source code to the working directory
COPY ./src ./src

# Get protobuf dependencies
RUN apt update && apt upgrade -y
RUN apt install -y protobuf-compiler libprotobuf-dev
COPY proto/model.proto ./proto/model.proto

# Build the source code
RUN cargo build --bin self_play

# Run the simulation client
ENTRYPOINT ["cargo", "run", "--bin", "self_play"]
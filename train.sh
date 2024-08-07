#!/usr/bin/env bash

# Load necessary modules, if any
module load python
module load rust

# Source the .env file and export variables
set -o allexport
source .env
set +o allexport

# Array to store PIDs of background processes
PIDS=()

terminate() {
    echo "Terminating all processes..."
    pkill -f "model_server.py"
    pkill -f "self_play"
    wait
    exit 0
}

# Trap termination signals
trap terminate SIGINT SIGTERM

# Start the Python server
python model/model_server.py &
SERVER_PID=$!

# Allow some time for the server to start
sleep 3

# Start the Rust clients
CLIENT_PIDS=()
echo "Starting $NUM_CLIENTS clients..."
for i in $(seq 1 $NUM_CLIENTS)
do
    cargo run --bin self_play &
    CLIENT_PID=$!
    echo "Client $i started with PID $CLIENT_PID"
    CLIENT_PIDS+=($CLIENT_PID)
done

# Wait for all client processes to complete
echo "Waiting for all client processes to complete..."
for pid in "${CLIENT_PIDS[@]}"; do
   wait "$pid"
done

# Once all clients are done, terminate the server
echo "All client processes completed. Terminating server..."
terminate

echo "Server terminated. Exiting script."

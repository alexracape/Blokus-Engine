import logging
import sys

from flask import Flask, request, jsonify
import torch

from resnet import ResNet

PORT = 8082
DIM = 20

app = Flask(__name__)
device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
model = ResNet(10, 256)

@app.route('/process_request', methods=['POST'])
def process_tensor():
    # Get the JSON data from the request
    data = request.json
    if data is None or 'state' not in data:
        return jsonify({"error": "Invalid data"}), 400

    # Convert the list of numbers to a numpy array
    boards = torch.tensor(data['state'], dtype=torch.float32).reshape(5, DIM, DIM).to(device)

    # Query the model
    with torch.no_grad():
        batch = boards.unsqueeze(0)
        policy, values = model(batch)

    # Format response
    result = {
        "policy": policy.tolist(),
        "values": values,
        "message": "Data processed successfully"
    }

    return jsonify(result), 200


if __name__ == '__main__':
    logging.basicConfig(level=logging.INFO)

    path = sys.argv[1]
    model.load_state_dict(torch.load(path, weights_only=True, map_location=device))
    logging.info(f"Loaded model from {path}")

    app.run(host='0.0.0.0', port=PORT)

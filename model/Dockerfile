# Image for the Python server for the hosting the model
FROM python:3.10-slim

# Set the working directory
WORKDIR /server

# Copy server code to the container
COPY . .

# Install the required packages
RUN pip install -r requirements.txt

# Install netcat for health check
RUN apt-get update && apt-get install -y netcat-openbsd

# Make sure the data directory exists
RUN mkdir -p data
RUN mkdir -p models

# Expose the port for the server
EXPOSE 8082

# Run the server
CMD ["python", "-u", "model_server.py"]

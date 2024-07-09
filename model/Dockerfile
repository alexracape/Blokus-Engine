# Image for the Python server for the hosting the model
FROM python:3.10-slim

# Set the working directory
WORKDIR /server

# Copy server code to the container
COPY . . 

# Install the required packages
RUN pip install -r requirements.txt

# Expose the port for the server
EXPOSE 8082

# Run the server
CMD ["python", "-u", "model_server.py"]

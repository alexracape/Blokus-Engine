
from concurrent import futures
import grpc
import model_pb2
import model_pb2_grpc

import numpy as np
import torch


class BlokusModelServicer(model_pb2_grpc.BlokusModelServicer):
    def __init__(self):
        self.model = None

    def Predict(self, request, context):
        
        print("Predicting")
        
        # state = np.array(request.state).reshape(1, 1, 14, 14)
        # state = torch.tensor(state, dtype=torch.float32)
        # with torch.no_grad():
        #     prediction = self.model(state).numpy()
        # return model_pb2.Prediction(value=prediction)
        policy, value = [0.5, 0.5], [0.5, 0.5]
        return model_pb2.Prediction(policy=policy, value=value)

    def Train(self, request, context):
        data = np.array(request.data).reshape(-1, 1, 14, 14)
        data = torch.tensor(data, dtype=torch.float32)
        target = np.array(request.target).reshape(-1, 1)
        target = torch.tensor(target, dtype=torch.float32)
        loss = self.model.train(data, target)
        return model_pb2.Status(value=loss)


def serve():
    print("Starting up server...")
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    model_pb2_grpc.add_BlokusModelServicer_to_server(BlokusModelServicer(), server)
    server.add_insecure_port("[::]:50051")
    server.start()
    server.wait_for_termination()


if __name__ == "__main__":
    serve()

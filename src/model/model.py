from xmlrpc.server import SimpleXMLRPCServer, SimpleXMLRPCRequestHandler
import socket

import numpy as np
import torch

PORT = 8000

class SimpleModelServer(SimpleXMLRPCServer):

    def __init__(self, addr, request_handler=SimpleXMLRPCRequestHandler):
        super().__init__(addr, requestHandler=request_handler)
        self.quit: bool = False

    def serve_forever(self):
        self.quit = False
        while not self.quit:
            self.handle_request()


class Model():
    """RPC Server to host the Model"""

    def __init__(self) -> None:
        self.model = None
        self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")



def main():

    addr = (socket.gethostbyname(socket.gethostname()), PORT)
    request_handler = SimpleXMLRPCRequestHandler
    with SimpleModelServer(addr, request_handler) as server:
        server.register_introspection_functions()
        server.register_instance(Model())
        server.register_function(lambda: server.quit = True, "quit")
        print(f"Server listening on {addr}")
        server.serve_forever()


if __name__ == "__main__":
    main()

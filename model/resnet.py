import torch


DIM = 20


class ResidualBlock(torch.nn.Module):

    def __init__(self, in_channels, out_channels):
        super(ResidualBlock, self).__init__()

        self.conv1 = torch.nn.Conv2d(in_channels, out_channels, kernel_size=3, padding=1)
        self.conv2 = torch.nn.Conv2d(out_channels, out_channels, kernel_size=3, padding=1)
        self.bn1 = torch.nn.BatchNorm2d(out_channels)
        self.bn2 = torch.nn.BatchNorm2d(out_channels)

    def forward(self, x):
        y = self.conv1(x)
        y = self.bn1(y)
        y = torch.relu(y)
        y = self.conv2(y)
        y = self.bn2(y)
        y += x

        return torch.relu(y)


class ResNet(torch.nn.Module):
    """ML model that will predict policy and value for game states
    
    AlphaZero uses a ResNet architecture for the model with 20 residual 
    blocks and 256 filters per block. 
    """

    def __init__(self, blocks, width):
        super(ResNet, self).__init__()
        self.blocks = blocks
        self.width = width
        
        

    def forward(self, boards):
        """Get the policy and value for the given board state

        For now, the board is represented by a 20x20x5 tensor where the first 4 channels are
        binary boards for each player's pieces on the board. The 5th channel is a binary board
        with the valid moves for the current player. For now, I'm just going to use the boards.
        It is unclear why the player color is needed in the state.
        """
        # print(board.shape)
        x = self.relu(self.conv1(boards))
        x = self.relu(self.conv2(x))
        x = self.relu(self.conv3(x))

        x = x.view(-1, DIM * DIM)
        x = self.relu(self.fc1(x))
        x = self.relu(self.fc2(x))

        policy = self.policy_head(x)
        value = self.value_head(x)

        if len(boards.shape) == 3:
            mask = boards[4].flatten()
        else:
            mask = boards[:, 4, :, :].view(boards.size(0), -1)
        policy = policy * mask

        return policy, value


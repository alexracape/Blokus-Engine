from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class StateRepresentation(_message.Message):
    __slots__ = ("boards", "player")
    BOARDS_FIELD_NUMBER: _ClassVar[int]
    PLAYER_FIELD_NUMBER: _ClassVar[int]
    boards: _containers.RepeatedScalarFieldContainer[bool]
    player: int
    def __init__(self, boards: _Optional[_Iterable[bool]] = ..., player: _Optional[int] = ...) -> None: ...

class Target(_message.Message):
    __slots__ = ("policy", "value")
    POLICY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    policy: _containers.RepeatedScalarFieldContainer[float]
    value: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, policy: _Optional[_Iterable[float]] = ..., value: _Optional[_Iterable[float]] = ...) -> None: ...

class Move(_message.Message):
    __slots__ = ("player", "tile")
    PLAYER_FIELD_NUMBER: _ClassVar[int]
    TILE_FIELD_NUMBER: _ClassVar[int]
    player: int
    tile: int
    def __init__(self, player: _Optional[int] = ..., tile: _Optional[int] = ...) -> None: ...

class ActionProb(_message.Message):
    __slots__ = ("action", "prob")
    ACTION_FIELD_NUMBER: _ClassVar[int]
    PROB_FIELD_NUMBER: _ClassVar[int]
    action: int
    prob: float
    def __init__(self, action: _Optional[int] = ..., prob: _Optional[float] = ...) -> None: ...

class Policy(_message.Message):
    __slots__ = ("probs",)
    PROBS_FIELD_NUMBER: _ClassVar[int]
    probs: _containers.RepeatedCompositeFieldContainer[ActionProb]
    def __init__(self, probs: _Optional[_Iterable[_Union[ActionProb, _Mapping]]] = ...) -> None: ...

class Game(_message.Message):
    __slots__ = ("history", "policies", "values")
    HISTORY_FIELD_NUMBER: _ClassVar[int]
    POLICIES_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    history: _containers.RepeatedCompositeFieldContainer[Move]
    policies: _containers.RepeatedCompositeFieldContainer[Policy]
    values: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, history: _Optional[_Iterable[_Union[Move, _Mapping]]] = ..., policies: _Optional[_Iterable[_Union[Policy, _Mapping]]] = ..., values: _Optional[_Iterable[float]] = ...) -> None: ...

class Status(_message.Message):
    __slots__ = ("code",)
    CODE_FIELD_NUMBER: _ClassVar[int]
    code: int
    def __init__(self, code: _Optional[int] = ...) -> None: ...

class Empty(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

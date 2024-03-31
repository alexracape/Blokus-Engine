from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class State(_message.Message):
    __slots__ = ("boards", "player")
    BOARDS_FIELD_NUMBER: _ClassVar[int]
    PLAYER_FIELD_NUMBER: _ClassVar[int]
    boards: _containers.RepeatedScalarFieldContainer[bool]
    player: int
    def __init__(self, boards: _Optional[_Iterable[bool]] = ..., player: _Optional[int] = ...) -> None: ...

class Prediction(_message.Message):
    __slots__ = ("policy", "value")
    POLICY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    policy: _containers.RepeatedScalarFieldContainer[float]
    value: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, policy: _Optional[_Iterable[float]] = ..., value: _Optional[_Iterable[float]] = ...) -> None: ...

class Data(_message.Message):
    __slots__ = ("states", "predictions")
    STATES_FIELD_NUMBER: _ClassVar[int]
    PREDICTIONS_FIELD_NUMBER: _ClassVar[int]
    states: _containers.RepeatedCompositeFieldContainer[State]
    predictions: _containers.RepeatedCompositeFieldContainer[Prediction]
    def __init__(self, states: _Optional[_Iterable[_Union[State, _Mapping]]] = ..., predictions: _Optional[_Iterable[_Union[Prediction, _Mapping]]] = ...) -> None: ...

class Status(_message.Message):
    __slots__ = ("code",)
    CODE_FIELD_NUMBER: _ClassVar[int]
    code: int
    def __init__(self, code: _Optional[int] = ...) -> None: ...

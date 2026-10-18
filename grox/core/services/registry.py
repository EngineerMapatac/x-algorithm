from abc import ABC, abstractmethod

import grpc

_REGISTRY: list[type["GrpcServicer"]] = []


class GrpcServicer(ABC):
    @classmethod
    def build(cls) -> "GrpcServicer | None":
        return cls()

    @abstractmethod
    def add_to_server(self, server: grpc.aio.Server) -> None:
        pass


def register_servicer[T: type[GrpcServicer]](cls: T) -> T:
    if not (isinstance(cls, type) and issubclass(cls, GrpcServicer)):
        raise TypeError(
            f"register_servicer expects a GrpcServicer subclass, got {cls!r}"
        )
    if cls not in _REGISTRY:
        _REGISTRY.append(cls)
    return cls


def registered_servicers() -> list[type[GrpcServicer]]:
    return list(_REGISTRY)

import inspect
import logging
from typing import Protocol

logger = logging.getLogger(__name__)


class EngineProcessor(Protocol):
    @classmethod
    def start(cls) -> None: ...

    @classmethod
    async def stop(cls, timeout: float = 5) -> None: ...


_REGISTRY: list[tuple[type, bool]] = []


def register_processor[T: type](cls: T, *, enabled: bool) -> T:
    if not callable(getattr(cls, "start", None)):
        raise TypeError(
            f"{cls.__name__} must define a callable start() to register as an engine processor"
        )
    stop = getattr(cls, "stop", None)
    if not callable(stop) or not inspect.iscoroutinefunction(stop):
        raise TypeError(
            f"{cls.__name__} must define an async stop() to register as an engine processor"
        )
    _REGISTRY.append((cls, enabled))
    return cls


def start_registered() -> None:
    for cls, enabled in _REGISTRY:
        if enabled:
            logger.info(f"Starting registered processor {cls.__name__}")
            cls.start()
        else:
            logger.info(
                f"Registered processor {cls.__name__} disabled in this environment"
            )


async def stop_registered() -> None:
    for cls, enabled in _REGISTRY:
        if enabled:
            await cls.stop()

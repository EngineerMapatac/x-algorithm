from grox.config.config import TaskGeneratorConfig
from grox.core.generators.task_generator import TaskGenerator

_REGISTRY: dict[str, type[TaskGenerator]] = {}


def register_task_generator(cls: type[TaskGenerator]) -> type[TaskGenerator]:
    if not (isinstance(cls, type) and issubclass(cls, TaskGenerator)):
        raise TypeError(
            f"register_task_generator expects a TaskGenerator subclass, got {cls!r}"
        )
    if not cls.TASK_GENERATOR_TYPE:
        raise ValueError(
            f"{cls.__name__} must set TASK_GENERATOR_TYPE to be registered"
        )
    key = cls.TASK_GENERATOR_TYPE
    existing = _REGISTRY.get(key)
    if existing is not None and existing is not cls:
        raise ValueError(
            f"Duplicate generator registration for {key!r}: {existing.__name__} vs {cls.__name__}"
        )
    _REGISTRY[key] = cls
    return cls


def build(cfg: TaskGeneratorConfig) -> TaskGenerator:
    cls = _REGISTRY.get(cfg.type)
    if cls is None:
        raise ValueError(
            f"No task generator registered for type {cfg.type!r}. Registered: {sorted(_REGISTRY)}"
        )
    return cls.from_config(cfg)


def registered_types() -> set[str]:
    return set(_REGISTRY)

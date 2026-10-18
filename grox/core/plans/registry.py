from grox.core.plans.plan import Plan

_REGISTRY: list[type[Plan]] = []


def register_plan(cls: type[Plan]) -> type[Plan]:
    if not (isinstance(cls, type) and issubclass(cls, Plan)):
        raise TypeError(f"register_plan expects a Plan subclass, got {cls!r}")
    if cls not in _REGISTRY:
        _REGISTRY.append(cls)
    return cls


def registered_plans() -> list[type[Plan]]:
    return list(_REGISTRY)

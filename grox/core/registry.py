from grox.core.generators.registry import register_task_generator
from grox.core.generators.task_generator import TaskGenerator
from grox.core.plans.plan import Plan
from grox.core.plans.registry import register_plan
from grox.core.services.registry import GrpcServicer, register_servicer


def register[T: type](cls: T) -> T:
    if isinstance(cls, type) and issubclass(cls, Plan):
        return register_plan(cls)
    if isinstance(cls, type) and issubclass(cls, TaskGenerator):
        return register_task_generator(cls)
    if isinstance(cls, type) and issubclass(cls, GrpcServicer):
        return register_servicer(cls)
    raise TypeError(
        f"@register expects a Plan, TaskGenerator, or GrpcServicer subclass, got {cls!r}"
    )

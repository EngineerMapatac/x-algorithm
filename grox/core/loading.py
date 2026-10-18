import importlib
import pkgutil
from types import ModuleType

import grox.flows
from grox.config.config import grox_config
from grox.core.generators.registry import _REGISTRY
from grox.core.plans.registry import registered_plans

_loaded = False

_FRAMEWORK_MODULES = {
    "grox.core.plans": {"plan", "plan_master", "registry"},
    "grox.core.generators": {"task_generator", "stream_generator", "registry"},
}


def _import_recursively(package: ModuleType) -> None:
    for mod in pkgutil.iter_modules(package.__path__):
        module = importlib.import_module(f"{package.__name__}.{mod.name}")
        if mod.ispkg:
            _import_recursively(module)


def _assert_framework_only() -> None:
    for pkg_name, allowed in _FRAMEWORK_MODULES.items():
        pkg = importlib.import_module(pkg_name)
        stray = {m.name for m in pkgutil.iter_modules(pkg.__path__)} - allowed
        if stray:
            raise ValueError(
                f"{pkg_name} is framework-only; move {sorted(stray)} into a grox/flows/ folder (flows are the only registration site)"
            )


def _validate_plan_keys() -> None:
    keys = [p.KEY for p in registered_plans()]
    duplicates = {k for k in keys if keys.count(k) > 1}
    if duplicates:
        raise ValueError(f"Duplicate Plan.KEYs: {sorted(duplicates)}")
    known = set(keys)
    for gen_type, gen_cls in _REGISTRY.items():
        unknown = set(gen_cls.PLANS_TO_INJECT) - known
        if unknown:
            raise ValueError(
                f"Generator {gen_type!r} injects unregistered plan keys: {sorted(unknown)}. Registered: {sorted(known)}"
            )


def _validate_generator_types() -> None:
    declared = {cfg.type for cfg in grox_config.dispatcher.task_generators}
    unknown = declared - set(_REGISTRY)
    if unknown:
        raise ValueError(
            f"Config declares unregistered task generator types: {sorted(unknown)}. Registered: {sorted(_REGISTRY)}"
        )


def load_all() -> None:
    global _loaded
    if _loaded:
        return
    _assert_framework_only()
    _import_recursively(grox.flows)
    _validate_plan_keys()
    _validate_generator_types()
    _loaded = True

from functools import cache
from pathlib import Path

from jinja2 import Environment, FileSystemLoader, StrictUndefined

_env = Environment(
    loader=FileSystemLoader(Path(__file__).parent / "templates"),
    undefined=StrictUndefined,
)

# As stated in README for the open source repo, prompts are excluded to reduce gameability of the system.


@cache
def reply_scoring_system_prompt(large_account_follower_threshold: int) -> str:
    return _env.get_template("reply_scoring_system.j2").render(
        large_account_follower_threshold=large_account_follower_threshold
    )


@cache
def reply_scoring_system_simple_prompt(large_account_follower_threshold: int) -> str:
    return _env.get_template("reply_scoring_system_simple.j2").render(
        large_account_follower_threshold=large_account_follower_threshold
    )


@cache
def coordinated_spam_system_prompt() -> str:
    return _env.get_template("coordinated_spam_system.j2").render()


@cache
def multi_step_reply_spam_system_prompt() -> str:
    return _env.get_template("multi_step_reply_spam_system.j2").render()

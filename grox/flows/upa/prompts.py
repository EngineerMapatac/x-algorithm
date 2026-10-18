from functools import cache
from pathlib import Path

from jinja2 import Environment, FileSystemLoader, StrictUndefined

_env = Environment(
    loader=FileSystemLoader(Path(__file__).parent / "templates"),
    undefined=StrictUndefined,
)

# As stated in README for the open source repo, prompts are excluded to reduce gameability of the system.


def banger_mini_vlm_screen_score_gemma_prompt(topics) -> str:
    return _env.get_template("banger_mini_vlm_screen_score_gemma.j2").render(
        topics=topics
    )


@cache
def post_safety_deluxe_prompt() -> str:
    return _env.get_template("post_safety_deluxe.j2").render()

from dataclasses import dataclass

from pydantic import BaseModel, Field


class CoordinatedSpamResult(BaseModel):
    spam_post_ids: list[str] = Field(default_factory=list)
    reason: str = ""


@dataclass
class CoordinatedSpamState:
    result: CoordinatedSpamResult | None = None

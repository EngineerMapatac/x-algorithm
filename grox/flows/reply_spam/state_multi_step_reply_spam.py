from dataclasses import dataclass

from pydantic import BaseModel, Field


class MultiStepReplySpamResult(BaseModel):
    spam_post_ids: list[str] = Field(default_factory=list)
    reason: str = ""


@dataclass
class MultiStepReplySpamState:
    result: MultiStepReplySpamResult | None = None

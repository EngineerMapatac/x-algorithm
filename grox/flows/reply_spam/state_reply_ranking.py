from dataclasses import dataclass

from pydantic import BaseModel


class ReplyScoreResult(BaseModel):
    score: float
    reason: str


@dataclass
class ReplyRankingState:
    result: ReplyScoreResult | None = None

from dataclasses import dataclass

from pydantic import BaseModel

from grox.flows.upa.models import TweetBoolMetadata


class PostSafetyScreenResult(BaseModel):
    tweet_bool_metadata: TweetBoolMetadata


@dataclass
class PostSafetyState:
    result: PostSafetyScreenResult | None = None

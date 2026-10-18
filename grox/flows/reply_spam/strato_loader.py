import asyncio
import logging

from strato_http.queries.data_types import (
    ReplyRankingScore,
    ReplyRankingScoreKafka,
)
from strato_http.queries.reply_ranking_score import StratoReplyRankingScore
from strato_http.queries.reply_ranking_score_cache import (
    StratoReplyRankingScoreCacheAtla,
    StratoReplyRankingScoreCachePdxa,
)
from strato_http.queries.reply_ranking_score_kafka_v2 import (
    StratoReplyRankingScoreV2Kafka,
)


logger = logging.getLogger(__name__)

_CACHE_FANOUT_SCORE_MAX = 1.0


class ReplyRankingScoreStratoLoader:
    strato = StratoReplyRankingScore()
    strato_cache_atla = StratoReplyRankingScoreCacheAtla()
    strato_cache_pdxa = StratoReplyRankingScoreCachePdxa()
    reply_ranking_v2_kafka_strato = StratoReplyRankingScoreV2Kafka()

    @classmethod
    async def fetch_reply_ranking_score(cls, post_id: str) -> ReplyRankingScore | None:
        return await cls.strato.fetch(int(post_id))

    @classmethod
    async def publish_reply_ranking_score(
        cls, post_id: str, reply_ranking_score: ReplyRankingScore
    ):
        if (
            reply_ranking_score.score is not None
            and reply_ranking_score.score <= _CACHE_FANOUT_SCORE_MAX
        ):
            await asyncio.gather(
                cls.strato_cache_atla.put(int(post_id), reply_ranking_score),
                cls.strato_cache_pdxa.put(int(post_id), reply_ranking_score),
            )
        await cls.reply_ranking_v2_kafka_strato.insert(
            int(post_id),
            ReplyRankingScoreKafka(
                postId=int(post_id),
                score=reply_ranking_score.score,
                reasoning=reply_ranking_score.reasoning,
            ),
        )
        await cls.strato.put(int(post_id), reply_ranking_score)

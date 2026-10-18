import logging

from protos.night_owl_search import night_owl_search_pb2

from grox.core.clients.nightowl_client import NightOwlClient

logger = logging.getLogger(__name__)


class DescendantIdProvider:
    def __init__(self, client: NightOwlClient):
        self._client = client

    async def fetch(self, post_id: int, num_results: int = 30) -> list[int]:
        request = night_owl_search_pb2.SearchRequest(
            old_query=f"include:spam include:protected in_reply_to_tweet_id:{post_id}",
            target=night_owl_search_pb2.SEARCH_TARGET_REALTIME,
            pagination=night_owl_search_pb2.Pagination(
                from_size=night_owl_search_pb2.FromSize(size=num_results)
            ),
            ranking=night_owl_search_pb2.Ranking(
                recency=night_owl_search_pb2.RecencyRanking()
            ),
            return_full_source=False,
        )
        response = await self._client.search(request)
        return [hit.doc_id for hit in response.hits]

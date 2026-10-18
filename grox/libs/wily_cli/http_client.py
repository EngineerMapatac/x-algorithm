import asyncio
import logging
import random
import time
from typing import Any
from urllib.parse import quote, unquote

import httpx

logger = logging.getLogger(__name__)


class WilyHttpClient:
    def __init__(
        self,
        wily_path: str = "",
        static_url: str = "",
        zone: str = "atla",
        name: str = "service",
        timeout: float = 30.0,
        role: str = "ads",
        client_name: str = "wily-http-client",
    ):
        if not wily_path and not static_url:
            raise ValueError(f"{name}: either wily_path or static_url must be set")
        self._wily_path = wily_path
        self._static_url = static_url
        self._zone = zone
        self._name = name
        self._timeout = timeout
        self._role = role
        self._client_name = client_name
        self._client: httpx.AsyncClient | None = None
        self._base_url: str = ""
        self._resolve_lock = asyncio.Lock()

    @property
    def base_url(self) -> str:
        return self._base_url

    @staticmethod
    def _encode_lookup_path(wily_path: str) -> str:
        path = unquote(wily_path.lstrip("/"))
        return quote(path, safe="/:")

    async def _resolve_wilyns(self) -> str:
        path = self._encode_lookup_path(self._wily_path)
        wilyns_url = f"https://wilyns-{self._zone}.twitter.biz"
        context = f"/p/static/{self._zone}/{self._role}/{self._client_name}/0/{int(time.time())}"
        async with httpx.AsyncClient() as tmp:
            resp = await tmp.get(
                f"{wilyns_url}/lookups/{path}",
                params={"context": context},
                timeout=10.0,
            )
            resp.raise_for_status()
            entries = resp.json().get("entries", [])
            if not entries:
                raise RuntimeError(f"WilyNS returned no entries for {self._wily_path}")
            entry = random.choice(entries)
            url = f"http://{entry['addr']}:{entry['port']}"
            return url

    async def _resolve(self) -> str:
        if self._static_url:
            logger.info("%s: using static URL %s", self._name, self._static_url)
            return self._static_url
        url = await self._resolve_wilyns()
        logger.info(
            "%s: resolved via WilyNS %s -> %s", self._name, self._wily_path, url
        )
        return url

    async def _build_client(self):
        old_client = self._client
        self._base_url = await self._resolve()
        self._client = httpx.AsyncClient(base_url=self._base_url, timeout=self._timeout)
        if old_client is not None:
            await old_client.aclose()

    async def start(self):
        await self._build_client()

    async def close(self):
        if self._client is not None:
            await self._client.aclose()
            self._client = None

    async def _request(self, method: str, path: str, **kwargs: Any) -> httpx.Response:
        if self._client is None:
            await self._build_client()
        try:
            return await self._client.request(method, path, **kwargs)
        except (httpx.ConnectError, httpx.ConnectTimeout):
            if self._static_url:
                raise
            async with self._resolve_lock:
                try:
                    return await self._client.request(method, path, **kwargs)
                except (httpx.ConnectError, httpx.ConnectTimeout):
                    logger.warning(
                        "%s: connection failure on %s %s, re-resolving %s",
                        self._name,
                        method,
                        path,
                        self._wily_path,
                    )
                    await self._build_client()
            return await self._client.request(method, path, **kwargs)

    async def get(self, path: str, **kwargs: Any) -> httpx.Response:
        return await self._request("GET", path, **kwargs)

    async def post(self, path: str, **kwargs: Any) -> httpx.Response:
        return await self._request("POST", path, **kwargs)

    async def put(self, path: str, **kwargs: Any) -> httpx.Response:
        return await self._request("PUT", path, **kwargs)

    async def delete(self, path: str, **kwargs: Any) -> httpx.Response:
        return await self._request("DELETE", path, **kwargs)

# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
import logging
from abc import ABC, abstractmethod
from typing import Any

from xai_configlib import Config as Config
from xai_configlib import configclass as configclass

logger = logging.getLogger(__name__)


@configclass
class Dataset(Config, ABC):
    @abstractmethod
    def make(
        self,
        *,
        batch_size: int,
        shard_index: int,
        num_shards: int,
        run_server: bool,
        server_hosts: list[str],
        server_port: int = 8898,
        skip_rows: int = 0,
    ) -> Any:
        raise NotImplementedError()

    @abstractmethod
    def example_data(self, batch_size: int) -> Any:
        raise NotImplementedError()

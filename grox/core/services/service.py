import logging

import grpc
from grpc._cython.cygrpc import CompressionLevel
from grpc_health.v1._async import HealthServicer
from grpc_health.v1.health_pb2_grpc import add_HealthServicer_to_server

from grox.config.config import grox_config
from grox.core.loading import load_all
from grox.core.services.registry import registered_servicers

logger = logging.getLogger(__name__)


class GrpcServer:
    def __init__(self):
        self.config = grox_config.grpc_server
        self.serving_addr = f"[::]:{self.config.port}"

    async def start(self):
        logger.info(f"Starting Grox server on {self.serving_addr}...")

        options = [
            ("grpc.keepalive_time_ms", 30000),
            ("grpc.keepalive_timeout_ms", 20000),
            ("grpc.keepalive_permit_without_calls", True),
            ("grpc.http2.max_pings_without_data", 0),
            ("grpc.http2.min_time_between_pings_ms", 10000),
            ("grpc.http2.min_ping_interval_without_data_ms", 5000),
            ("grpc.max_receive_message_length", 128 * 1024 * 1024),
            ("grpc.max_send_message_length", 128 * 1024 * 1024),
            ("grpc.http2.max_frame_size", 16 * 1024 * 1024),
            ("grpc.http2.bdp_probe", 1),
            ("grpc.so_reuseaddr", 1),
            ("grpc.default_compression_algorithm", grpc.Compression.Gzip),
            ("grpc.default_compression_level", CompressionLevel.medium),
        ]

        server = grpc.aio.server(options=options)
        server.add_insecure_port(self.serving_addr)
        add_HealthServicer_to_server(HealthServicer(), server)
        load_all()
        for cls in registered_servicers():
            servicer = cls.build()
            if servicer is None:
                logger.info(
                    f"gRPC servicer {cls.__name__} disabled by config; skipping"
                )
                continue
            servicer.add_to_server(server)
            logger.info(f"gRPC servicer {cls.__name__} registered")

        self.server = server
        await server.start()
        logger.info("Grox server started")

    async def stop(self):
        await self.server.stop(self.config.graceful_shutdown_timeout)

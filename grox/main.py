import signal
import asyncio
import logging

from kerberos_cli.kerberos import KerberosRenewer

from grox.core.engine import Engine
from grox.core.services.service import GrpcServer
from grox.core.dispatcher import Dispatcher
from grox.config.config import grox_config
from grox.core.schedules.init import init_metrics, init_proc
from grox.core.schedules.context import (
    cleanup,
    new_context,
    shutdown_context,
    queue_connection_shutdown_context,
)

logger = logging.getLogger(__name__)
shutdown = asyncio.Event()


def init_kerberos_renewer() -> KerberosRenewer | None:
    keytab_path = grox_config.kerberos.keytab_path
    principal = grox_config.kerberos.principal
    if keytab_path is None or principal is None:
        logger.warning("Kerberos is not enabled, skipping")
        return None
    return KerberosRenewer(keytab_path=keytab_path, principal=principal)


async def serve():
    await init_proc("main", defer_metrics=True)
    logger.info("Starting grox server...")
    context = new_context()
    kerberos_renewer = init_kerberos_renewer()
    engine = Engine(context)
    dispatcher = Dispatcher(context)
    grpc_server = GrpcServer()

    if kerberos_renewer is not None:
        await kerberos_renewer.renew()
        kerberos_renewer.start()

    await engine.start()
    await dispatcher.start()
    init_metrics("main")
    await grpc_server.start()

    logger.info("Grox server started")
    event_loop = asyncio.get_running_loop()
    event_loop.add_signal_handler(signal.SIGINT, lambda: shutdown.set())
    event_loop.add_signal_handler(signal.SIGTERM, lambda: shutdown.set())

    await shutdown.wait()
    logger.warning("Grox server shutting down...")
    queue_connection_shutdown_context(context)
    await asyncio.sleep(grox_config.shutdown_drain_timeout)

    shutdown_context(context)
    await asyncio.gather(
        grpc_server.stop(),
        dispatcher.stop(),
        engine.stop(),
    )
    if kerberos_renewer is not None:
        kerberos_renewer.stop()
    cleanup()
    logger.warning("Grox server stopped")


if __name__ == "__main__":
    asyncio.run(serve())

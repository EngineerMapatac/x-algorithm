import logging
import os
import ssl

logger = logging.getLogger(__name__)


def create_mtls_ssl_context() -> ssl.SSLContext:
    ca_file = os.getenv("SSL_CA_FILE", "/etc/ssl/internal-ca/ca-bundle.crt")
    if not ca_file or not os.path.exists(ca_file):
        raise FileNotFoundError(
            f"Kafka mTLS requires CA at {ca_file!r} (mount internal-ca ConfigMap or set SSL_CA_FILE)"
        )

    ssl_ctx = ssl.create_default_context(cafile=ca_file)
    ssl_ctx.verify_mode = ssl.CERT_REQUIRED
    ssl_ctx.check_hostname = False
    logger.info(f"Kafka mTLS: CERT_REQUIRED ca={ca_file} check_hostname=False")

    cert_file = os.getenv("SSL_CERT_FILE", "/etc/ssl/s2s/client/tls.crt")
    key_file = os.getenv("SSL_KEY_FILE", "/etc/ssl/s2s/client/tls.key")
    if not os.path.exists(cert_file) and os.path.exists("/certs/client.fullchain"):
        cert_file = "/certs/client.fullchain"
    if not os.path.exists(key_file) and os.path.exists("/certs/client.key"):
        key_file = "/certs/client.key"

    if os.path.exists(cert_file) and os.path.exists(key_file):
        ssl_ctx.load_cert_chain(certfile=cert_file, keyfile=key_file)
        logger.info(f"mTLS client certificate loaded: CERT={cert_file} KEY={key_file}")
    else:
        logger.warning(
            f"mTLS client cert files not found: CERT={cert_file} KEY={key_file}"
        )

    return ssl_ctx

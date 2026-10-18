import io
import os
import pickle


def kafka_ssl_config() -> dict:
    cfg = {"enable.ssl.certificate.verification": True}
    ca_location = os.environ.get("KAFKA_SSL_CA_LOCATION", "").strip()
    if ca_location:
        cfg["ssl.ca.location"] = ca_location
    endpoint_id = os.environ.get("KAFKA_SSL_ENDPOINT_IDENTIFICATION", "").strip()
    if endpoint_id:
        cfg["ssl.endpoint.identification.algorithm"] = endpoint_id
    return cfg


_ALLOWED_PICKLE_GLOBALS = {
    ("numpy", "ndarray"),
    ("numpy", "dtype"),
    ("numpy.core.multiarray", "_reconstruct"),
    ("numpy._core.multiarray", "_reconstruct"),
    ("numpy.core.multiarray", "scalar"),
    ("numpy._core.multiarray", "scalar"),
    ("numpy.core.numeric", "_frombuffer"),
    ("numpy._core.numeric", "_frombuffer"),
    ("_codecs", "encode"),
}


class _RestrictedUnpickler(pickle.Unpickler):
    def find_class(self, module, name):
        if (module, name) in _ALLOWED_PICKLE_GLOBALS:
            return super().find_class(module, name)
        raise pickle.UnpicklingError(
            f"forbidden global {module}.{name} in pipeline payload "
            "(only numpy array reconstruction is permitted)"
        )


def safe_pickle_loads(data: bytes):
    return _RestrictedUnpickler(io.BytesIO(data)).load()

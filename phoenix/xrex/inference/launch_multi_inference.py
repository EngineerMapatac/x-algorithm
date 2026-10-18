# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
import argparse
import logging
import os
import pathlib
import signal
import subprocess
import sys
import threading
import time

logger = logging.getLogger(__name__)

SHARED_EMB_TABLE_PATH = "/dev/shm/emb_table.dat"
SHARED_MM_EMB_PATH = "/dev/shm/mm_embeddings.dat"


def _read_version_file(path: str) -> int:
    try:
        return int(open(path).read().strip())
    except (FileNotFoundError, ValueError, OSError):
        return 0


class ReadinessSupervisor(threading.Thread):
    def __init__(
        self,
        num_workers: int,
        ready_event: threading.Event,
        shared_dir: str,
        poll_interval: float = 0.2,
        hotswap_enabled: bool = False,
    ) -> None:
        super().__init__(daemon=True, name="ReadinessSupervisor")
        self._num_workers = num_workers
        self._ready_event = ready_event
        self._shared_dir = shared_dir
        self._poll_interval = poll_interval
        self._hotswap_enabled = hotswap_enabled
        self._stop = threading.Event()
        self._was_ready: bool | None = None

    def stop(self) -> None:
        self._stop.set()

    def run(self) -> None:
        trigger_path = f"{self._shared_dir}/reload_trigger"
        while not self._stop.is_set():
            try:
                trigger = _read_version_file(trigger_path)
                done = [
                    _read_version_file(f"{self._shared_dir}/reload_done_w{w}")
                    for w in range(self._num_workers)
                ]
                all_done = self._hotswap_enabled or trigger == 0 or all(d >= trigger for d in done)
                if all_done:
                    if not self._ready_event.is_set():
                        self._ready_event.set()
                    if self._was_ready is False:
                        logger.info(
                            f"ReadinessSupervisor: trigger={trigger} done={done} -> pod READY"
                        )
                    self._was_ready = True
                else:
                    if self._ready_event.is_set():
                        self._ready_event.clear()
                    if self._was_ready is True:
                        logger.info(
                            f"ReadinessSupervisor: trigger={trigger} done={done} "
                            f"-> pod NOT-READY (workers still reloading)"
                        )
                    self._was_ready = False
            except Exception as e:
                logger.warning(f"ReadinessSupervisor: error reading shm: {e}")
            self._stop.wait(self._poll_interval)


def _generate_haproxy_config(listen_port, backend_ports, hostname="127.0.0.1"):
    servers = "\n".join(
        f"    server worker{i} {hostname}:{port}" for i, port in enumerate(backend_ports)
    )
    return f"""\
global
    maxconn 65536
    log stdout format raw local0 warning

defaults
    mode tcp
    timeout connect 5s
    timeout client  300s
    timeout server  300s
    no log

frontend grpc_in
    bind *:{listen_port}
    default_backend workers

backend workers
    balance roundrobin
{servers}
"""


class HaproxyProxy:
    def __init__(self, listen_port, backend_ports, hostname="127.0.0.1"):
        self.listen_port = listen_port
        self.backend_ports = backend_ports
        self.hostname = hostname
        self._proc = None
        self._config_path = "/tmp/haproxy_multi_inference.cfg"

    def start(self):
        cfg = _generate_haproxy_config(self.listen_port, self.backend_ports, self.hostname)
        with open(self._config_path, "w") as f:
            f.write(cfg)
        logger.info(f"haproxy proxy on port {self.listen_port} -> backends {self.backend_ports}")
        self._proc = subprocess.Popen(
            ["haproxy", "-f", self._config_path, "-db"], stderr=subprocess.PIPE
        )

    def check_alive(self):
        if self._proc and self._proc.poll() is not None:
            stderr = ""
            if self._proc.stderr:
                stderr = self._proc.stderr.read().decode(errors="replace").strip()
            raise RuntimeError(
                f"haproxy exited with code {self._proc.returncode} "
                f"(port {self.listen_port} may already be in use). "
                f"stderr: {stderr or '(empty)'}"
            )

    def wait(self):
        if self._proc:
            self._proc.wait()

    def stop(self):
        if self._proc and self._proc.poll() is None:
            self._proc.terminate()
            try:
                self._proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self._proc.kill()


def spawn_workers(
    num_workers,
    worker_cmd,
    shared_emb_path,
    base_worker_port=19988,
    base_coordinator_port=2345,
    base_metrics_port=19090,
    workers_per_gpu=1,
):
    workers = []
    num_gpus = num_workers // workers_per_gpu
    assert num_gpus * workers_per_gpu == num_workers, (
        f"num_workers={num_workers} must be a multiple of workers_per_gpu={workers_per_gpu}"
    )

    base_profile_path = None
    profile_arg_idx = None
    for idx, arg in enumerate(worker_cmd):
        if arg.startswith("--jax_profile_path="):
            base_profile_path = arg.split("=", 1)[1]
            profile_arg_idx = idx
            break
        if arg == "--jax_profile_path" and idx + 1 < len(worker_cmd):
            base_profile_path = worker_cmd[idx + 1]
            profile_arg_idx = idx
            break

    for i in range(num_workers):
        env = {
            **os.environ,
            "CUDA_VISIBLE_DEVICES": str(i % num_gpus),
            "NUM_PHYSICAL_DEVICES_PER_NODE": "1",
            "JAX_COORDINATOR_PORT": str(base_coordinator_port + i),
            "JAX_PLATFORMS": "",
        }
        if workers_per_gpu > 1:
            env["XLA_PYTHON_CLIENT_MEM_FRACTION"] = f"{0.8 / workers_per_gpu:.2f}"

        cmd = list(worker_cmd)
        if base_profile_path is not None:
            worker_profile = f"{base_profile_path}/worker_{i}"
            os.makedirs(worker_profile, exist_ok=True)
            if cmd[profile_arg_idx].startswith("--jax_profile_path="):
                cmd[profile_arg_idx] = f"--jax_profile_path={worker_profile}"
            else:
                cmd[profile_arg_idx + 1] = worker_profile

        cmd = cmd + [
            f"--grpc_port={base_worker_port + i}",
            f"--metrics_port={base_metrics_port + i}",
            "--num_devices_per_process=1",
            f"--worker_id={i}",
            f"--num_workers={num_workers}",
        ]
        logger.info(f"[Worker {i}] GPU={i % num_gpus} port={base_worker_port + i}")
        proc = subprocess.Popen(
            cmd,
            env=env,
            preexec_fn=lambda: __import__("ctypes").CDLL("libc.so.6").prctl(1, 9),
        )
        workers.append(proc)
    return workers


def wait_for_workers(workers, ports, autometrics_ports, timeout=600):
    import urllib.request

    start = time.time()
    ready = set()
    while len(ready) < len(ports):
        if time.time() - start > timeout:
            missing = [p for i, p in enumerate(ports) if i not in ready]
            raise TimeoutError(f"Workers on ports {missing} didn't start within {timeout}s")
        for i, port in enumerate(ports):
            if i in ready:
                continue
            if workers[i].poll() is not None:
                raise RuntimeError(
                    f"Worker {i} (port {port}) died with code {workers[i].returncode}."
                )
            try:
                metrics_port = autometrics_ports[i]
                url = f"http://127.0.0.1:{metrics_port}/metrics"
                resp = urllib.request.urlopen(url, timeout=1)
                text = resp.read().decode()
                if 'pod_status{status="serving"} 1.0' in text:
                    ready.add(i)
                    logger.info(f"[Worker {i}] Ready ({len(ready)}/{len(ports)})")
            except (ConnectionRefusedError, OSError):
                pass
        if len(ready) < len(ports):
            time.sleep(2)
    logger.info(f"All {len(ports)} workers ready")


def start_readiness_server(port, ready_event, on_shutdown=None):
    from http.server import BaseHTTPRequestHandler, HTTPServer

    from xrex.inference.status_server import (
        issue_shutdown_token,
        shutdown_request_authorized,
    )

    shutdown_token, _ = issue_shutdown_token(port)

    class Handler(BaseHTTPRequestHandler):
        def _shutdown_authorized(self):
            return shutdown_request_authorized(
                self.client_address[0],
                self.headers.get("Authorization", ""),
                shutdown_token,
            )

        def do_GET(self):
            if self.path == "/shutdown" and on_shutdown:
                if not self._shutdown_authorized():
                    logger.warning(
                        f"Rejected unauthorized shutdown request from {self.client_address[0]}"
                    )
                    self.send_response(403)
                    self.end_headers()
                    self.wfile.write(b"forbidden")
                    return
                on_shutdown()
                self.send_response(200)
                self.end_headers()
                self.wfile.write(b"shutting down")
                return
            if self.path == "/readyz":
                if ready_event.is_set():
                    self.send_response(200)
                    self.end_headers()
                    self.wfile.write(b"ready")
                else:
                    self.send_response(503)
                    self.end_headers()
                    self.wfile.write(b"not ready")
                return
            self.send_response(404)
            self.end_headers()

        def log_message(self, format, *args):
            pass

    try:
        httpd = HTTPServer(("0.0.0.0", port), Handler)
        thread = threading.Thread(target=httpd.serve_forever, daemon=True)
        thread.start()
        logger.info(f"Readiness server on port {port}")
    except OSError as e:
        logger.warning(f"Failed to start readiness server on port {port}: {e}")


def start_metrics_proxy(port, worker_metrics_ports, worker_autometrics_ports):
    import re
    import urllib.request
    from http.server import BaseHTTPRequestHandler, HTTPServer

    def _fetch_metrics_text(url):
        try:
            resp = urllib.request.urlopen(url, timeout=2)
            return resp.read().decode()
        except Exception:
            return None

    def _parse_and_aggregate(worker_ports, autometrics_ports):
        meta = {}
        values = {}
        order = []

        urls = [f"http://127.0.0.1:{wp}/metrics" for wp in worker_ports]

        for url in urls:
            text = _fetch_metrics_text(url)
            if text is None:
                continue

            current_type = "untyped"
            for line in text.splitlines():
                if line.startswith("# HELP"):
                    name = line.split()[2] if len(line.split()) > 2 else ""
                    if name not in meta:
                        meta[name] = {"help": line, "type_line": "", "type": "untyped"}
                        order.append(name)
                    continue
                if line.startswith("# TYPE"):
                    parts = line.split()
                    name = parts[2] if len(parts) > 2 else ""
                    current_type = parts[3] if len(parts) > 3 else "untyped"
                    if name in meta:
                        meta[name]["type_line"] = line
                        meta[name]["type"] = current_type
                    continue
                if line.startswith("#") or not line.strip():
                    continue

                match = re.match(r"^([^\s{]+(?:\{[^}]*\})?)\s+(.+)$", line)
                if not match:
                    continue
                key, val_str = match.group(1), match.group(2)
                try:
                    val = float(val_str)
                except ValueError:
                    continue

                if key not in values:
                    values[key] = []
                values[key].append(val)

        lines = []
        seen_keys = set()
        for name in order:
            m = meta[name]
            lines.append(m["help"])
            if m["type_line"]:
                lines.append(m["type_line"])
            mtype = m["type"]
            for key, vals in values.items():
                base = key.split("{")[0]
                if base != name and not base.startswith(name + "_"):
                    continue
                if key in seen_keys:
                    continue
                seen_keys.add(key)
                if mtype in ("counter", "histogram", "summary"):
                    agg = sum(vals)
                elif mtype == "gauge":
                    if base == "model_checkpoint_timestamp":
                        agg = min(vals)
                    else:
                        agg = max(vals)
                else:
                    agg = sum(vals)
                lines.append(f"{key} {agg}")

        return "\n".join(lines) + "\n"

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path not in ("/metrics", "/autometrics"):
                self.send_response(404)
                self.end_headers()
                return
            body = _parse_and_aggregate(worker_metrics_ports, worker_autometrics_ports).encode()
            self.send_response(200)
            self.send_header("Content-Type", "text/plain; charset=utf-8")
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, format, *args):
            pass

    try:
        httpd = HTTPServer(("0.0.0.0", port), Handler)
        thread = threading.Thread(target=httpd.serve_forever, daemon=True)
        thread.start()
        logger.info(
            f"Metrics proxy on port {port} -> "
            f"python /metrics ports {worker_metrics_ports}, "
            f"rust /autometrics ports {worker_autometrics_ports}"
        )
    except OSError as e:
        logger.warning(f"Failed to start metrics proxy on port {port}: {e}")


def build_worker_cmd(argv):
    filtered = []
    skip_next = False
    for arg in argv:
        if skip_next:
            skip_next = False
            continue
        if arg.startswith("--num_workers"):
            if "=" not in arg:
                skip_next = True
            continue
        if arg.startswith("--grpc_port") or arg.startswith("--metrics_port"):
            if "=" not in arg:
                skip_next = True
            continue
        if arg.startswith("--readiness_port"):
            if "=" not in arg:
                skip_next = True
            continue
        if arg.startswith("--workers_per_gpu"):
            if "=" not in arg:
                skip_next = True
            continue
        filtered.append(arg)

    script_idx = None
    for i, arg in enumerate(filtered):
        if "launch_multi_inference" in arg:
            script_idx = i
            break

    if script_idx is not None:
        filtered[script_idx] = filtered[script_idx].replace(
            "launch_multi_inference", "launch_inference"
        )

    return [sys.executable] + filtered


def main():
    parser = argparse.ArgumentParser(
        description="Multi-GPU inference: spawns N single-GPU workers behind a TCP proxy"
    )
    parser.add_argument("--num_workers", type=int, default=8, help="Number of GPU workers")
    parser.add_argument(
        "--workers_per_gpu",
        type=int,
        default=1,
        help="Co-located workers per GPU (num_workers must be a multiple).",
    )
    parser.add_argument("--grpc_port", type=int, default=9988, help="Public gRPC port")
    parser.add_argument(
        "--metrics_port", type=int, default=9090, help="Metrics port (unused, for compat)"
    )
    parser.add_argument("--readiness_port", type=int, default=9091, help="Readiness probe port")
    parser.add_argument(
        "--startup_timeout", type=float, default=1800, help="Worker startup timeout (seconds)"
    )
    parser.add_argument(
        "--enable_hotswap",
        type=lambda v: v.lower() in ("true", "1", "yes"),
        default=False,
        help="Enable hotswap (passed through to workers, also used by ReadinessSupervisor)",
    )
    parser.add_argument(
        "--jax_compilation_cache_dir",
        type=str,
        default=os.environ.get("JAX_COMPILATION_CACHE_DIR", ""),
        help=(
            "Directory used as the JAX persistent compilation cache. Empty "
            "(default) disables the cache. Forwarded to each worker."
        ),
    )
    parser.add_argument(
        "--deployment_name",
        type=str,
        default=os.environ.get("DEPLOYMENT_NAME", ""),
        help=(
            "Human-readable deployment identifier, logged for debuggability "
            "and forwarded to each worker."
        ),
    )

    args, remaining = parser.parse_known_args()

    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s %(levelname)s %(name)s: %(message)s",
    )

    if args.jax_compilation_cache_dir:
        logger.info(
            "JAX persistent compilation cache: dir=%s deployment=%s (forwarded to each worker)",
            args.jax_compilation_cache_dir,
            args.deployment_name or "<unset>",
        )
    else:
        logger.info(
            "JAX persistent compilation cache disabled (--jax_compilation_cache_dir is empty)."
        )

    num_workers = args.num_workers
    public_port = args.grpc_port
    base_worker_port = 19988
    base_autometrics_port = 19090
    shared_emb_path = SHARED_EMB_TABLE_PATH

    worker_cmd = build_worker_cmd(sys.argv)
    logger.info(f"Worker command: {' '.join(worker_cmd)}")

    import shutil

    for name in os.listdir("/dev/shm"):
        path = f"/dev/shm/{name}"
        try:
            if os.path.isdir(path):
                shutil.rmtree(path)
            else:
                os.remove(path)
        except Exception:
            pass

    logger.info(f"Creating shared emb_table at {shared_emb_path}")
    pathlib.Path(shared_emb_path).touch()

    worker_ports = [base_worker_port + i for i in range(num_workers)]
    workers = []

    ready_event = threading.Event()
    stop_event = threading.Event()

    proxy_holder = [None]

    def cleanup(signum=None, frame=None):
        logger.info("Shutting down...")
        ready_event.clear()
        if proxy_holder[0] is not None:
            proxy_holder[0].stop()
        for w in workers:
            try:
                w.terminate()
            except Exception:
                pass
        for w in workers:
            try:
                w.wait(timeout=10)
            except Exception:
                w.kill()
        for name in os.listdir("/dev/shm"):
            path = f"/dev/shm/{name}"
            try:
                if os.path.isdir(path):
                    shutil.rmtree(path)
                else:
                    os.remove(path)
            except Exception:
                pass
        logger.info("All workers stopped.")
        sys.exit(0)

    signal.signal(signal.SIGTERM, cleanup)
    signal.signal(signal.SIGINT, cleanup)

    start_readiness_server(
        args.readiness_port,
        ready_event,
        on_shutdown=lambda: stop_event.set(),
    )

    def _start_liveness_metrics_server(port: int, autometrics_ports: list[int]) -> None:
        import re
        import urllib.request
        from http.server import BaseHTTPRequestHandler, HTTPServer

        def _fetch_and_aggregate_autometrics():
            meta = {}
            values = {}
            order = []
            for ap in autometrics_ports:
                try:
                    resp = urllib.request.urlopen(f"http://127.0.0.1:{ap}/metrics", timeout=2)
                    text = resp.read().decode()
                except Exception:
                    continue
                for line in text.splitlines():
                    if line.startswith("# HELP"):
                        name = line.split()[2] if len(line.split()) > 2 else ""
                        if name not in meta:
                            meta[name] = {"help": line, "type_line": "", "type": "untyped"}
                            order.append(name)
                        continue
                    if line.startswith("# TYPE"):
                        parts = line.split()
                        name = parts[2] if len(parts) > 2 else ""
                        mtype = parts[3] if len(parts) > 3 else "untyped"
                        if name in meta:
                            meta[name]["type_line"] = line
                            meta[name]["type"] = mtype
                        continue
                    if line.startswith("#") or not line.strip():
                        continue
                    match = re.match(r"^([^\s{]+(?:\{[^}]*\})?)\s+(.+)$", line)
                    if not match:
                        continue
                    key, val_str = match.group(1), match.group(2)
                    try:
                        val = float(val_str)
                    except ValueError:
                        continue
                    if key not in values:
                        values[key] = []
                    values[key].append(val)
            lines = []
            seen = set()
            for name in order:
                m = meta[name]
                lines.append(m["help"])
                if m["type_line"]:
                    lines.append(m["type_line"])
                mtype = m["type"]
                for key, vals in values.items():
                    base = key.split("{")[0]
                    if base != name and not base.startswith(name + "_"):
                        continue
                    if key in seen:
                        continue
                    seen.add(key)
                    if mtype in ("counter", "histogram", "summary"):
                        agg = sum(vals)
                    elif mtype == "gauge":
                        agg = max(vals)
                    else:
                        agg = sum(vals)
                    lines.append(f"{key} {agg}")
            return "\n".join(lines) + "\n"

        class Handler(BaseHTTPRequestHandler):
            def do_GET(self):
                if self.path == "/metrics":
                    body = _fetch_and_aggregate_autometrics().encode()
                    self.send_response(200)
                    self.send_header("Content-Type", "text/plain; charset=utf-8")
                    self.end_headers()
                    self.wfile.write(body)
                else:
                    self.send_response(200)
                    self.end_headers()

            def log_message(self, format, *args):
                pass

        httpd = HTTPServer(("0.0.0.0", port), Handler)
        logger.info(
            f"Liveness + autometrics server on port {port} -> worker autometrics ports {autometrics_ports}"
        )
        httpd.serve_forever()

    threading.Thread(
        target=_start_liveness_metrics_server,
        args=(args.metrics_port, [base_autometrics_port + i for i in range(num_workers)]),
        daemon=True,
    ).start()

    worker_metrics_ports = [8000 + (base_worker_port + i - 9988) for i in range(num_workers)]
    worker_autometrics_ports = [base_autometrics_port + i for i in range(num_workers)]
    start_metrics_proxy(
        port=8000,
        worker_metrics_ports=worker_metrics_ports,
        worker_autometrics_ports=worker_autometrics_ports,
    )

    logger.info(f"Spawning {num_workers} workers on ports {worker_ports}...")
    workers = spawn_workers(
        num_workers, worker_cmd, shared_emb_path, workers_per_gpu=args.workers_per_gpu
    )

    time.sleep(2)
    for i, w in enumerate(workers):
        if w.poll() is not None:
            logger.error(f"Worker {i} died immediately with code {w.returncode}.")

    logger.info("Waiting for workers to start...")
    try:
        wait_for_workers(workers, worker_ports, worker_metrics_ports, timeout=args.startup_timeout)
    except (TimeoutError, RuntimeError) as e:
        logger.error(str(e))
        cleanup()

    proxy = HaproxyProxy(public_port, worker_ports)
    proxy_holder[0] = proxy
    proxy.start()

    time.sleep(0.5)
    try:
        proxy.check_alive()
    except RuntimeError as e:
        logger.error(str(e))
        cleanup()

    ready_event.set()
    logger.info("All workers ready - pod is serving")

    supervisor = ReadinessSupervisor(
        num_workers,
        ready_event,
        shared_dir="/dev/shm",
        hotswap_enabled=args.enable_hotswap,
    )
    supervisor.start()

    try:
        proxy.wait()
    except KeyboardInterrupt:
        pass
    finally:
        cleanup()


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Run inside nix develop: a headless host -> naruwm -> real shm client."""
import os
from pathlib import Path
import subprocess
import signal
import re
import tempfile
import time


def wait_for(predicate, process, label):
    deadline = time.monotonic() + 20
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"{label} exited early ({process.returncode})")
        if predicate():
            return
        time.sleep(0.1)
    raise RuntimeError(f"timed out waiting for {label}")


def stop(process):
    if process is not None and process.poll() is None:
        os.killpg(process.pid, signal.SIGTERM)
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()


with tempfile.TemporaryDirectory(prefix="naruwm-smoke-") as directory:
    root = Path(directory)
    env = dict(os.environ, XDG_RUNTIME_DIR=directory, LIBGL_ALWAYS_SOFTWARE="1", RUST_LOG="naruwm=info,smithay=warn")
    for key in ("WAYLAND_DISPLAY", "WAYLAND_SOCKET", "DISPLAY"):
        env.pop(key, None)
    host = compositor = None
    host_log = root / "host.log"
    client_log = root / "client.log"
    try:
        with host_log.open("w") as host_output, client_log.open("w") as client_output:
            host = subprocess.Popen(
                ["weston", "--backend=headless", "--renderer=pixman", "--socket=naru-host", "--idle-time=0", "--no-config"],
                env=env, start_new_session=True, stdout=host_output, stderr=subprocess.STDOUT,
            )
            wait_for(lambda: (root / "naru-host").exists(), host, "headless host")
            compositor = subprocess.Popen(
                ["target/debug/naruwm", "--", "env", "WAYLAND_DEBUG=1", "weston-terminal"],
                env=dict(env, WAYLAND_DISPLAY="naru-host", WINIT_UNIX_BACKEND="wayland"),
                start_new_session=True, stdout=client_output, stderr=subprocess.STDOUT,
            )
            # Only the terminal logs protocol messages, so these are inner-client frames.
            def rendered():
                trace = client_log.read_text()
                frames = re.findall(r"\.frame\(new id wl_callback#(\d+)\)", trace)
                delivered = any(f"wl_callback#{callback}.done(" in trace for callback in frames)
                return "wl_shm_pool" in trace and ".attach(" in trace and delivered and "naruwm ready" in trace
            wait_for(rendered, compositor, "client buffer and frame")
            time.sleep(1)
            if compositor.poll() is not None:
                raise RuntimeError("compositor exited after its first frame")
            print("PASS: headless host, naruwm socket, shm client, buffer attach, frame callback")
    except Exception:
        for log in (host_log, client_log):
            if log.exists():
                print(f"{log.name}:\n{log.read_text()[-12000:]}")
        raise
    finally:
        stop(compositor)
        stop(host)

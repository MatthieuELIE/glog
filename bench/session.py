"""Run a program in a pseudo-terminal, send it keys and wait for its frames."""
import ctypes
import os
import pty
import select
import subprocess
import sys
import termios
import time
from pathlib import Path

RUSAGE_INFO_V4 = 4

# ratatui hides the cursor as the last write of a draw, or shows it when the frame places it, as in the search input.
FRAME_ENDS = (b"\x1b[?25l", b"\x1b[?25h")


def start(binary, directory, columns, rows):
    master, slave = pty.openpty()
    termios.tcsetwinsize(slave, (rows, columns))
    process = subprocess.Popen([binary], cwd=directory, stdin=slave, stdout=slave, stderr=slave)
    os.close(slave)

    return process, master


def read_output(master):
    try:
        return os.read(master, 65536)
    except OSError:
        # Linux reports a closed pseudo-terminal as an error, macOS as an empty read.
        return b""


def wait_frame(master, timeout):
    """Read until a frame ends; false when none does within `timeout` seconds."""
    deadline = time.monotonic() + timeout
    seen = b""
    while not any(end in seen for end in FRAME_ENDS):
        readable, _, _ = select.select([master], [], [], max(0, deadline - time.monotonic()))
        if not readable:
            return False

        output = read_output(master)
        if not output:
            return False

        seen += output

    return True


def drain(master, quiet):
    """Discard the output until none comes for `quiet` seconds."""
    while select.select([master], [], [], quiet)[0]:
        if not read_output(master):
            return


def key_latency(master, key, timeout):
    """Seconds between a key and the end of the frame that answers it, or None."""
    drain(master, 0)
    start = time.monotonic()
    os.write(master, key)
    if not wait_frame(master, timeout):
        return None

    return time.monotonic() - start


def has_git_child(process):
    found = subprocess.run(["pgrep", "-P", str(process.pid), "-x", "git"], stdout=subprocess.DEVNULL)

    return found.returncode == 0


class RusageInfoV4(ctypes.Structure):
    # struct rusage_info_v4 of macOS's <sys/resource.h>, which /usr/bin/time -l reads for its peak memory footprint.
    _fields_ = [
        ("uuid", ctypes.c_uint8 * 16),
        ("counters", ctypes.c_uint64 * 28),
        ("lifetime_max_phys_footprint", ctypes.c_uint64),
        ("later_counters", ctypes.c_uint64 * 6),
    ]


def peak_footprint(pid):
    """Peak memory footprint of a running macOS process, in bytes."""
    libproc = ctypes.CDLL("libproc.dylib", use_errno=True)
    info = RusageInfoV4()
    if libproc.proc_pid_rusage(pid, RUSAGE_INFO_V4, ctypes.byref(info)) != 0:
        raise OSError(ctypes.get_errno(), f"proc_pid_rusage failed for process {pid}")

    return info.lifetime_max_phys_footprint


def peak_resident_kb(status):
    """The `VmHWM` field of a Linux /proc/<pid>/status text, in kilobytes."""
    for line in status.splitlines():
        if line.startswith("VmHWM:"):
            return int(line.split()[1])

    raise ValueError("no VmHWM line in the process status")


def memory(process):
    """Peak memory of the process, in megabytes: its peak footprint on macOS, its peak resident size on Linux."""
    if sys.platform == "darwin":
        return peak_footprint(process.pid) / 1024 / 1024

    status = Path(f"/proc/{process.pid}/status").read_text()

    return peak_resident_kb(status) / 1024


def stop(process, master):
    process.kill()
    process.wait()
    os.close(master)

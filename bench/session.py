"""Run a program in a pseudo-terminal, send it keys and wait for its frames."""
import os
import pty
import select
import subprocess
import termios
import time

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


def memory(process):
    """Resident memory of the process, in megabytes."""
    rss = subprocess.run(["ps", "-o", "rss=", "-p", str(process.pid)], capture_output=True, text=True, check=True)

    return int(rss.stdout) / 1024


def stop(process, master):
    process.kill()
    process.wait()
    os.close(master)

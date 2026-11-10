"""Create the synthetic repositories the bench measures."""
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path("/tmp/glog-bench")
GENERATOR = Path(__file__).with_name("gen.py")
SIZES = {
    "smoke": (300, 10),
    "s": (10_000, 100),
    "m": (100_000, 1_000),
    "l": (1_000_000, 5_000),
}


def generate(path, commits, branches):
    subprocess.run(["git", "init", "--quiet", "--bare", "--initial-branch=main", path], check=True)
    stream = subprocess.Popen([sys.executable, GENERATOR, str(commits), str(branches)], stdout=subprocess.PIPE)
    subprocess.run(["git", "fast-import", "--quiet"], cwd=path, stdin=stream.stdout, check=True)
    if stream.wait() != 0:
        raise subprocess.CalledProcessError(stream.returncode, stream.args)

    subprocess.run(["git", "commit-graph", "write", "--reachable"], cwd=path, check=True, stderr=subprocess.DEVNULL)


def ensure(size):
    path = ROOT / f"repo-{size}"
    if path.exists():
        return path

    partial = ROOT / f"repo-{size}.partial"
    shutil.rmtree(partial, ignore_errors=True)
    generate(partial, *SIZES[size])
    partial.rename(path)

    return path

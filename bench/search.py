"""Play searches in a running glog and time each key against its budget."""
import session

QUERIES = ["number 1 with", "zzzz"]
JUMPS = 20
KEY_TIMEOUT = 2
ENTER = b"\r"
ESCAPE = b"\x1b"


def script(query):
    """The keys of one search as (name, bytes) pairs: the input, `Enter`, then jumps and moves."""
    keys = [("/", b"/")]
    keys += [(f"character {char!r} of {query!r}", char.encode()) for char in query]
    keys.append(("Enter", ENTER))
    for _ in range(JUMPS):
        keys += [("n", b"n"), ("N", b"N")]

    keys += [("j", b"j"), ("k", b"k"), ("Esc", ESCAPE)]

    return keys


def play(master):
    """Time every key of every query: (name, milliseconds) pairs, the last one None when a key got no frame."""
    timings = []
    for query in QUERIES:
        for name, key in script(query):
            latency = session.key_latency(master, key, KEY_TIMEOUT)
            if latency is None:
                return timings + [(name, None)]

            timings.append((name, latency * 1000))

    return timings

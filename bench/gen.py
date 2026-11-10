"""Write a git fast-import stream: a main line with regular merges, plus open branches."""
import random
import sys

commits, branches = int(sys.argv[1]), int(sys.argv[2])
random.seed(1)
out = sys.stdout
mark = 0
clock = 1_500_000_000
AUTHORS = ["Ada Lovelace", "Linus T", "Grace Hopper", "Jean-Baptiste Emmanuel Zorg"]


def commit(ref, parent, merged=None):
    global mark, clock
    mark += 1
    clock += 60
    author = AUTHORS[mark % len(AUTHORS)]
    message = f"feat(scope{mark % 50}): change number {mark} with a subject of ordinary length"
    out.write(f"commit {ref}\nmark :{mark}\n")
    out.write(f"committer {author} <a@example.com> {clock} +0000\n")
    out.write(f"data {len(message)}\n{message}\n")
    if parent:
        out.write(f"from :{parent}\n")
    if merged:
        out.write(f"merge :{merged}\n")
    out.write("\n")
    return mark


main = []
tip = None
while len(main) < commits:
    if len(main) > 10 and len(main) % 20 == 0:
        side = main[-5]
        for _ in range(2):
            side = commit("refs/heads/side-tmp", side)
        tip = commit("refs/heads/main", tip, side)
    else:
        tip = commit("refs/heads/main", tip)
    main.append(tip)

for index in range(branches):
    parent = random.choice(main)
    for _ in range(3):
        parent = commit(f"refs/heads/feature/branch-{index}", parent)
    if index % 10 == 0:
        out.write(f"reset refs/tags/v0.{index}\nfrom :{parent}\n\n")

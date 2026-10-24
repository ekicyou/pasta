"""Leftover hunt for /kiro-next: what the specs that landed since the previous inventory left behind.

Usage (run at the worktree root):
  python -I .claude/skills/kiro-next/scripts/leftovers.py [--prev <sha>] [--base <sha>] [--out <file>]
  python -I .claude/skills/kiro-next/scripts/leftovers.py --self-test

--prev defaults to the base SHA of the roadmap's 棚卸 heading, --base to HEAD.
For each spec folder added under .kiro/specs/completed/ between the two, prints only the sections
whose heading names a leftover (Implementation Notes, 範囲外, 申し送り, リリース, …) and the lines
elsewhere that defer something, each with file:line (brief.md and requirements.md are skipped: they are the
spec's inputs). Then the mechanical checks: references to the
landed specs' old paths (.kiro/specs/<name>/ without completed/), the completed count, and the
manual's chapter count against the number the roadmap records (only when both exist). The last line says whether the
output is small enough to read directly ("read directly") or should go to an agent ("use an agent").
"""
import os
import re
import subprocess
import sys

sys.dont_write_bytecode = True  # keep the skill folder free of __pycache__
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from scan import ROADMAP, SPECS, git, prev_inventory  # noqa: E402

LEFTOVER_HEADING = re.compile(r"Implementation Notes|範囲外|申し送り|リリース|後回し|残|Deferred|Follow|Open Question|未決|開発者の判断|開発者の許可")
DEFERRAL = re.compile(r"後回し|別 ?spec|公開(の)?後|リリースノート|開発者の許可|手で確かめ|実機|TODO|未確認|申し送")
SUMMARY = "book/src/SUMMARY.md"
CHAPTERS = re.compile(r"(\d{4}-\d{2}-\d{2}) の時点で (\d+) 章")
AGENT_LINES = 400
# What the spec was given, not what it left: its brief (incoming handoffs) and requirements
INPUTS = {"brief.md", "requirements.md"}


def sections(text):
    """[(line number of the heading, heading, body lines)] for every markdown heading."""
    out, cur = [], None
    for n, l in enumerate(text.splitlines(), 1):
        if re.match(r"^#{1,6} ", l):
            cur = (n, l, [])
            out.append(cur)
        elif cur:
            cur[2].append((n, l))
    return out


def extract(path, text):
    """Leftover sections in full, then deferring lines outside them, each tagged with path:line."""
    out, inside = [], set()
    for n, h, body in sections(text):
        if LEFTOVER_HEADING.search(h):
            out.append(f"{path}:{n} {h}")
            out += [f"  {l}" for _, l in body if l.strip()]
            inside.update(m for m, _ in body)
    for n, l in enumerate(text.splitlines(), 1):
        if n not in inside and DEFERRAL.search(l) and not l.startswith("#"):
            out.append(f"{path}:{n} {l.strip()}")
    return out


def chapter_count(summary):
    return len(re.findall(r"\]\([^)]*\.md\)", summary))


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    if "--self-test" in sys.argv:
        return self_test()
    roadmap = open(ROADMAP, encoding="utf-8").read()
    arg = lambda k, d: sys.argv[sys.argv.index(k) + 1] if k in sys.argv else d
    prev = arg("--prev", (prev_inventory(roadmap.splitlines()) or (None, None))[1])
    base = arg("--base", "HEAD")
    if prev is None:
        print("FAIL no --prev and no 棚卸 heading", file=sys.stderr)
        return 1
    landed = sorted({p.split("/")[3] for p in git("diff", "--name-only", "--diff-filter=A", prev, base, "--", f"{SPECS}/completed/").split()})
    out = [f"landed since {prev[:8]} ({len(landed)}): {', '.join(landed) or 'none'}"]
    for s in landed:
        d = f"{SPECS}/completed/{s}"
        for f in sorted(os.listdir(d)):
            if f.endswith(".md") and f not in INPUTS:
                out += extract(f"{d}/{f}", open(f"{d}/{f}", encoding="utf-8").read())
    stale = []
    for s in landed:
        # .kiro/specs/completed/<name>/ does not contain .kiro/specs/<name>/, so every hit is an old path
        r = subprocess.run(["git", "grep", "-n", "-I", "-F", "-e", f".kiro/specs/{s}/", "--", ".", f":!{SPECS}/completed/"],
                           capture_output=True, text=True, encoding="utf-8")
        stale += r.stdout.splitlines()
    out.append(f"old paths of landed specs: {len(stale)}")
    out += [f"  {l}" for l in stale]
    out.append(f"completed: {len(os.listdir(SPECS + '/completed'))}")
    # Only for a repository with an mdBook manual whose chapter count the roadmap records
    m = CHAPTERS.search(roadmap)
    if m and os.path.exists(SUMMARY):
        n = chapter_count(open(SUMMARY, encoding="utf-8").read())
        out.append(f"chapters: {n} in {SUMMARY}, roadmap records {m.group(2)} ({m.group(1)})"
                   + ("" if int(m.group(2)) == n else " -> MISMATCH: check the tests that hard-code the chapter count"))
    out.append(f"{len(out)} lines: " + ("read directly" if len(out) <= AGENT_LINES else "use an agent"))
    text = "\n".join(out) + "\n"
    if "--out" in sys.argv:
        open(arg("--out", ""), "w", encoding="utf-8").write(text)
        print(out[0])
        print(out[-1])
    else:
        sys.stdout.write(text)
    return 0


DOC = """# Tasks
- [x] 1. 作る
## Implementation Notes
- 3 人のシェルは公開の後に確かめる

## 範囲外
- 別 spec で直す
## 本文
- ここは関係ない
- リリースノートに書く
"""


def self_test():
    got = extract("t.md", DOC)
    assert got[0] == "t.md:3 ## Implementation Notes", got
    assert "  - 3 人のシェルは公開の後に確かめる" in got and "t.md:6 ## 範囲外" in got, got
    assert got[-1] == "t.md:10 - リリースノートに書く", got
    assert not any("関係ない" in l for l in got), got
    assert chapter_count("- [a](a.md)\n  - [b](x/b.md)\n[c](https://x)\n") == 2
    m = CHAPTERS.search("章の数（2026-10-24 の時点で 63 章）")
    assert m and m.group(2) == "63"
    print("selftest: all passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())

"""Step 0-3 groundwork for /kiro-next: the facts an inventory needs, in a few lines.

Usage (run at the worktree root; the scratch directory must exist):
  python -I .claude/skills/kiro-next/scripts/scan.py --out <scratch> [--base <sha>]
  python -I .claude/skills/kiro-next/scripts/scan.py --self-test

Prints: the previous and current base, the specs that landed in between, branches that may be
in flight, the current goal (first version table whose tag vX.Y.Z is not on origin) and its
unfinished specs, the ledger-vs-folder check, counts, the roadmap headings with line ranges,
and the specs to re-measure with the reason for each. Writes <out>/diff-files.md (files changed
since the previous inventory) and <out>/facts-<spec>.md for each picked spec (the files its brief
names: tracked or not, lines, changed since the previous inventory).
Exit 1 when the roadmap has no previous inventory heading.
"""
import os
import re
import subprocess
import sys

ROADMAP = ".kiro/steering/roadmap.md"
SPECS = ".kiro/specs"
INVENTORY = re.compile(r"^## 棚卸（(\d{4}-\d{2}-\d{2})・main `([0-9a-f]{7,40})`")
VERSION = re.compile(r"\d+\.\d+\.\d+")
SPEC_NAME = re.compile(r"^[a-z0-9]+(-[a-z0-9]+)+$")
REMEASURE = "棚卸の再測定"
# Backticked tokens that look like a path: a slash, or a file extension
PATH_TOKEN = re.compile(r"`([^`\s]*(?:/|\.(?:rs|lua|md|toml|yml|yaml|ps1|mjs|js|json|pest|bat|py|lock))[^`\s]*)`")


def git(*args):
    r = subprocess.run(["git", *args], capture_output=True, text=True, encoding="utf-8")
    if r.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)}: {r.stderr.strip()}")
    return r.stdout


def spec_name(cell):
    """Leading spec-like name of a table cell or ledger head, or None."""
    m = re.match(r"`?([a-z0-9-]+)", cell.strip())
    return m.group(1) if m and SPEC_NAME.match(m.group(1)) else None


def prev_inventory(lines):
    for l in lines:
        m = INVENTORY.match(l)
        if m:
            return m.group(1), m.group(2)
    return None


def headings(lines):
    """(start, end, text) for every ## and ### heading; end is the last line of its section (1-based)."""
    idx = [(i, l) for i, l in enumerate(lines) if l.startswith("## ") or l.startswith("### ")]
    out = []
    for k, (i, l) in enumerate(idx):
        level = l.split(" ")[0]
        end = next((j for j, m in idx[k + 1:] if len(m.split(" ")[0]) <= len(level)), len(lines))
        out.append((i + 1, end, l))
    return out


def version_tables(lines):
    """[(version, [spec names])] for each table whose header starts | 順 | spec |, under a ## heading naming a version."""
    out, heading, i = [], "", 0
    while i < len(lines):
        l = lines[i]
        if l.startswith("## "):
            heading = l
        if l.startswith("|") and [c.strip() for c in l.strip().strip("|").split("|")][:2] == ["順", "spec"]:
            names, i = [], i + 2
            while i < len(lines) and lines[i].startswith("|"):
                cells = [c.strip() for c in lines[i].strip().strip("|").split("|")]
                n = spec_name(cells[1]) if len(cells) > 1 else None
                if n:
                    names.append(n)
                i += 1
            v = VERSION.search(heading)
            if v:
                out.append((v.group(0), names))
            continue
        i += 1
    return out


def ledger(lines):
    """{name: (done, deps)} from the - [ ] / - [x] rows of ## Specs (dependency order)."""
    rows, inside = {}, False
    for l in lines:
        if l.startswith("## "):
            inside = l.startswith("## Specs (dependency order)")
            continue
        m = re.match(r"^- \[( |x)\] ([a-z0-9-]+) -- (.*)$", l) if inside else None
        if m:
            d = m.group(3).rfind("Dependencies:")
            deps = re.findall(r"[a-z0-9]+(?:-[a-z0-9]+)+", m.group(3)[d:]) if d >= 0 else []
            rows.setdefault(m.group(2), (m.group(1) == "x", deps))
    return rows


def residents(lines):
    out, inside = [], False
    for l in lines:
        if l.startswith("#"):
            inside = l.startswith("### 常駐 spec")
            continue
        m = re.match(r"^- `([a-z0-9-]+)`", l) if inside else None
        if m:
            out.append(m.group(1))
    return out


def latest_remeasure(text):
    """The newest 棚卸の再測定 section of a brief (the last one), else the whole brief."""
    parts = re.split(r"(?m)^(?=## )", text)
    hits = [p for p in parts if REMEASURE in p.split("\n", 1)[0]]
    return hits[-1] if hits else text


def path_tokens(text):
    seen = []
    for t in PATH_TOKEN.findall(text):
        t = t.removeprefix("./").rstrip("、。,")
        if t and t not in seen and not t.startswith("http"):
            seen.append(t)
    return seen


# Changed files that never make a spec worth re-measuring: steering, specs and the workflow skills
# (allowed overlaps), and generated skill references
QUIET = re.compile(r"^(\.kiro/|\.claude/skills/kiro-|\.claude/skills/[^/]+/references/)")


def touches(token, path):
    """Does a brief's path token name this repository path (suffix match, or a folder prefix)?"""
    if token.endswith("/"):
        return f"/{token}" in f"/{path}"
    return path == token or path.endswith("/" + token)


def pick(goal_specs, unfinished, briefs, diff, landed, rows):
    """[(spec, reason)] to re-measure: unfinished goal specs, then specs whose latest
    measurement names a changed file, or whose functional predecessor landed."""
    out = [(s, "goal") for s in goal_specs if s in unfinished]
    diff = [p for p in diff if not QUIET.match(p)]
    for s in sorted(unfinished):
        if s in goal_specs:
            continue
        hit = next((p for t in path_tokens(latest_remeasure(briefs.get(s, ""))) for p in diff if touches(t, p)), None)
        dep = next((d for d in rows.get(s, (False, []))[1] if d in landed), None)
        if hit:
            out.append((s, f"diff: {hit}"))
        elif dep:
            out.append((s, f"dep landed: {dep}"))
    return out


def facts(tokens, tracked, diff):
    """One line per token: the tracked files it names, their lines, and whether they changed."""
    lines, seen = [], set()
    for t in tokens[:40]:
        files = [p for p in tracked if touches(t, p) and p not in seen and (t.startswith(".kiro") or not p.startswith(".kiro/"))]
        seen.update(files)
        if not files and any(touches(t, p) for p in tracked):
            continue
        if not files:
            lines.append(f"- `{t}`: not tracked (new file, or a stale name)")
        elif len(files) > 3:
            ch = sum(1 for p in files if p in diff)
            lines.append(f"- `{t}`: {len(files)} files, {ch} changed")
        else:
            for p in files:
                try:
                    n = sum(1 for _ in open(p, encoding="utf-8", errors="replace"))
                except OSError:
                    n = "?"
                lines.append(f"- `{p}`: {n} lines{', changed' if p in diff else ''}")
    return lines


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    if "--self-test" in sys.argv:
        return self_test()
    out = sys.argv[sys.argv.index("--out") + 1]
    base = sys.argv[sys.argv.index("--base") + 1] if "--base" in sys.argv else git("rev-parse", "HEAD").strip()
    lines = open(ROADMAP, encoding="utf-8").read().splitlines()
    prev = prev_inventory(lines)
    if prev is None:
        print("FAIL no previous inventory heading (## 棚卸（…・main `…`）)", file=sys.stderr)
        return 1
    date, prev_sha = prev
    landed = sorted({p.split("/")[3] for p in git("diff", "--name-only", "--diff-filter=A", prev_sha, base, "--", f"{SPECS}/completed/").split()})
    diff = git("diff", "--name-only", prev_sha, base).split()
    tracked = git("ls-files").split()
    folders = sorted(d for d in os.listdir(SPECS) if d != "completed" and os.path.isdir(f"{SPECS}/{d}"))
    res = residents(lines)
    unfinished = set(folders) - set(res)
    rows = ledger(lines)
    briefs = {s: open(f"{SPECS}/{s}/brief.md", encoding="utf-8").read() for s in unfinished if os.path.exists(f"{SPECS}/{s}/brief.md")}

    tags = set(re.findall(r"refs/tags/(v\d+\.\d+\.\d+)$", git("ls-remote", "--tags", "origin", "refs/tags/v*"), re.M))
    tables = version_tables(lines)
    goal = next(((v, names) for v, names in tables if f"v{v}" not in tags), None)
    goal_specs = [n for n in goal[1] if n in unfinished] if goal else sorted(unfinished)

    me = git("rev-parse", "--abbrev-ref", "HEAD").strip()
    branches = [b.strip() for b in git("branch", "-r", "--format=%(refname:short)").split() if b.startswith("origin/claude/")]
    branches += [l.split("refs/heads/")[1] for l in git("worktree", "list", "--porcelain").splitlines() if l.startswith("branch refs/heads/")]
    inflight = sorted({(s, b) for b in branches if me not in b for s in unfinished if s in b})

    print(f"prev {date} {prev_sha} -> base {base[:8]}")
    print(f"landed ({len(landed)}): {', '.join(landed) or 'none'}")
    print(f"diff: {len(diff)} files -> {out}/diff-files.md")
    print("in flight? " + ("; ".join(f"{s} ({b})" for s, b in inflight) or "none") + "  (name match only: check each)")
    print(f"goal: {goal[0] + ' (no tag)' if goal else 'none (ゴール未設定)'}; unfinished in goal: {', '.join(goal_specs) or 'none'}")
    open_rows = {n for n, (d, _) in rows.items() if not d}
    done_rows = sorted(n for n, (d, _) in rows.items() if d)
    print(f"ledger: {len(open_rows)} open rows / {len(unfinished)} folders"
          f"; rows without folder: {', '.join(sorted(open_rows - unfinished)) or 'none'}"
          f"; folders without row: {', '.join(sorted(unfinished - open_rows)) or 'none'}"
          f"; [x] rows to fold: {', '.join(done_rows) or 'none'}")
    print(f"counts: completed {len(os.listdir(SPECS + '/completed'))}, briefs {len(unfinished)}, resident {', '.join(res) or 'none'}")
    print("roadmap headings (line ranges):")
    for s, e, h in headings(lines):
        print(f"  L{s}-{e} {h}")
    picks = pick(goal_specs, unfinished, briefs, diff, set(landed), rows)
    print(f"re-measure ({len(picks)}):")
    for s, why in picks:
        print(f"  {s} -- {why}")
        sheet = facts(path_tokens(latest_remeasure(briefs.get(s, ""))), tracked, set(diff))
        with open(f"{out}/facts-{s}.md", "w", encoding="utf-8") as f:
            f.write(f"# {s} -- files its latest measurement names (base {base[:8]}, changed = since {prev_sha})\n" + "\n".join(sheet) + "\n")
    with open(f"{out}/diff-files.md", "w", encoding="utf-8") as f:
        f.write("\n".join(diff) + "\n")
    return 0


SAMPLE = """# Roadmap
### 常駐 spec（再実行型）
- `loop-spec` — 常駐
## 0.2.0 の範囲
| 順 | spec | 要件定義 |
| -- | ---- | -------- |
| 0 | done-one | Opus |
| 1 | `next-one`（注） | Opus |
| 2 | 0.2.0 のリリース | — |
## 棚卸（2026-01-02・main `abc1234`）
### ウェーブ（2026-01-02）
## Specs (dependency order)
- [x] done-one -- 終わった。Dependencies: none
- [ ] next-one -- 次。Dependencies: done-one
- [ ] far-one -- 遠い。Dependencies: next-one（注の other-x）
"""

BRIEF = """# Brief: far-one
古い `crates/a/old.rs`
## 2026-01-01 棚卸の再測定（main `aaa`）
- 触るファイル: `crates/a/old.rs`
## 2026-01-02 棚卸の再測定（main `bbb`）
- 触るファイル: `crates/b/new.rs`、`book/src/x/`、https://example.com/a.md は無視
"""


def self_test():
    lines = SAMPLE.splitlines()
    assert prev_inventory(lines) == ("2026-01-02", "abc1234")
    assert version_tables(lines) == [("0.2.0", ["done-one", "next-one"])], version_tables(lines)
    rows = ledger(lines)
    assert rows["next-one"] == (False, ["done-one"]) and rows["done-one"][0], rows
    assert rows["far-one"][1] == ["next-one", "other-x"], rows
    assert residents(lines) == ["loop-spec"]
    h = headings(lines)
    assert h[0][2].startswith("### 常駐") and any(x[2].startswith("## 棚卸") and x[1] == 11 for x in h), h
    sec = latest_remeasure(BRIEF)
    assert "new.rs" in sec and "old.rs" not in sec
    assert path_tokens(sec) == ["crates/b/new.rs", "book/src/x/"], path_tokens(sec)
    assert touches("b/new.rs", "crates/b/new.rs") and not touches("new.rs", "crates/b/renew.rs")
    assert touches("book/src/x/", "book/src/x/y.md") and not touches("src/x/", "book/src/xx/y.md")
    unfinished = {"next-one", "far-one", "idle-one"}
    briefs = {"far-one": BRIEF, "idle-one": "`crates/a/old.rs`"}
    got = pick(["next-one"], unfinished, briefs, ["crates/b/new.rs"], set(), rows)
    assert got == [("next-one", "goal"), ("far-one", "diff: crates/b/new.rs")], got
    assert pick([], unfinished, briefs, [".kiro/steering/x/a.md", ".claude/skills/kiro-next/old.rs"], set(), rows) == []
    got = pick([], unfinished, {}, [], {"next-one"}, rows)
    assert got == [("far-one", "dep landed: next-one")], got
    assert facts(["zz/none.rs"], ["crates/a.rs"], set()) == ["- `zz/none.rs`: not tracked (new file, or a stale name)"]
    assert facts(["a.rs", "crates/a.rs", "tech.md"], ["crates/a.rs", ".kiro/x/tech.md"], set()) == ["- `crates/a.rs`: ? lines"], facts(["a.rs", "crates/a.rs", "tech.md"], ["crates/a.rs", ".kiro/x/tech.md"], set())
    print("selftest: all passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())

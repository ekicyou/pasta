"""Read the current wave that /kiro-next wrote into roadmap.md.

Usage:
  MSYS_NO_PATHCONV=1 git show origin/main:.kiro/steering/roadmap.md | python -I read-wave.py
  (Git Bash rewrites "origin/main:..." as a Windows path without MSYS_NO_PATHCONV=1.)
  python -I read-wave.py --self-test

Prints one line per spec: "<model> <name>" (model is Fable or Opus), then
"date <YYYY-MM-DD>" for the wave. Warnings go to stderr as "WARN ...".
A spec is Fable when the code block lists it under Fable OR the wave table's
last column (要件定義) starts with Fable — the stricter of the two wins.
Exit 1 when no wave can be found.
"""
import sys

INVENTORY = "## 棚卸（"
WAVE = "### ウェーブ（"
FENCE = "```"


def read_wave(text):
    lines = text.splitlines()
    start = next((i for i, l in enumerate(lines) if l.startswith(INVENTORY)), None)
    if start is None:
        raise ValueError("no inventory section (## 棚卸（…）)")
    end = next((i for i in range(start + 1, len(lines)) if lines[i].startswith("## ")), len(lines))
    wave = next((i for i in range(start, end) if lines[i].startswith(WAVE)), None)
    if wave is None:
        raise ValueError("no wave subsection (### ウェーブ（…）) in the inventory section")
    date = lines[wave][len(WAVE):].split("）")[0]
    sub_end = next((i for i in range(wave + 1, end) if lines[i].startswith("### ")), end)
    sub = lines[wave + 1:sub_end]

    # Wave table: first column = spec name, last column = requirements model.
    table = {}
    for l in sub:
        cells = [c.strip() for c in l.strip().strip("|").split("|")]
        if l.startswith("|") and len(cells) >= 2 and cells[0] not in ("spec", "") and not set(cells[0]) <= set("-: "):
            table[cells[0]] = cells[-1]

    open_ = next((i for i, l in enumerate(sub) if l.strip() == FENCE), None)
    if open_ is None:
        raise ValueError("no start-command code block in the wave subsection")
    close = next((i for i in range(open_ + 1, len(sub)) if sub[i].strip() == FENCE), None)
    if close is None:
        raise ValueError("unterminated code block in the wave subsection")

    specs, warns, group = [], [], None
    for l in sub[open_ + 1:close]:
        s = l.strip()
        if s in ("Fable", "Opus"):
            group = s
        elif s.startswith("/kiro-start "):
            name = s.split()[1]
            model = group or "Opus"
            if table.get(name, "").startswith("Fable") and model != "Fable":
                warns.append(f"{name}: table says Fable, block says {model}; treated as Fable")
                model = "Fable"
            specs.append((model, name))
    for name in table:
        if name not in {n for _, n in specs}:
            warns.append(f"{name}: in the wave table but not in the code block; not started")
    if not specs:
        raise ValueError("the code block lists no /kiro-start command")
    return date, specs, warns


SAMPLE = """# Roadmap
## 棚卸（2026-01-02・main `abc`）
### ウェーブ（2026-01-02）
| spec | 子孫 | 要件定義 |
| ---- | ---- | -------- |
| a-spec | 1 | Opus |
| b-spec | 2 | Fable |
| c-spec | 0 | Fable |
```
Wave 3本
Fable
/kiro-start b-spec

Opus
/kiro-start a-spec
/kiro-start c-spec
```
### 次
## Phase 1
"""


def self_test():
    date, specs, warns = read_wave(SAMPLE)
    assert date == "2026-01-02", date
    assert specs == [("Fable", "b-spec"), ("Opus", "a-spec"), ("Fable", "c-spec")], specs
    assert len(warns) == 1 and warns[0].startswith("c-spec"), warns
    for bad in ("# none\n", "## 棚卸（x）\n### ウェーブ（x）\nno block\n"):
        try:
            read_wave(bad)
            raise AssertionError("expected ValueError")
        except ValueError:
            pass
    print("selftest: all passed")


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    sys.stderr.reconfigure(encoding="utf-8")
    if "--self-test" in sys.argv:
        self_test()
        return 0
    try:
        date, specs, warns = read_wave(sys.stdin.buffer.read().decode("utf-8"))
    except ValueError as e:
        print(f"FAIL {e}", file=sys.stderr)
        return 1
    for w in warns:
        print(f"WARN {w}", file=sys.stderr)
    print(f"date {date}")
    for model, name in specs:
        print(f"{model} {name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

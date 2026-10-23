"""Read the current wave that /kiro-next wrote into roadmap.md.

Usage:
  MSYS_NO_PATHCONV=1 git show origin/main:.kiro/steering/roadmap.md | python -I read-wave.py
  (Git Bash rewrites "origin/main:..." as a Windows path without MSYS_NO_PATHCONV=1.)
  ... | python -I read-wave.py --models <name>...   (requirements model of named specs)
  python -I read-wave.py --self-test

Prints one line per spec: "<model> <name>" (model is Fable or Opus), then
"date <YYYY-MM-DD>" for the wave. Warnings go to stderr as "WARN ...".
When the block is the release form ("/release <X.Y.Z>" and no /kiro-start), prints
"release <X.Y.Z>" instead of the spec lines.
A spec is Fable when the code block lists it under Fable OR the wave table's
last column (要件定義) starts with Fable — the stricter of the two wins.
Exit 1 when no wave can be found.
With --models, prints "<Fable|Opus|unknown> <name>" per named spec, looked up in
every table that has "spec" and "要件定義" columns (Fable wins on conflict).
"""
import re
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

    specs, warns, group, release = [], [], None, None
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
        elif s.split()[:1] == ["/release"]:
            if not re.fullmatch(r"/release \d+\.\d+\.\d+", s):
                raise ValueError(f"bad release line: {s}")
            release = s.split()[1]
    if release and specs:
        raise ValueError("the code block mixes /release with /kiro-start")
    for name in table:
        if name not in {n for _, n in specs}:
            warns.append(f"{name}: in the wave table but not in the code block; not started")
    if not specs and not release:
        raise ValueError("the code block lists no /kiro-start or /release command")
    return date, specs, warns, release


def spec_name(cell):
    name = ""
    for ch in cell:
        if not (ch.isascii() and (ch.isalnum() or ch == "-")):
            break
        name += ch
    return name


def read_models(text):
    """Requirements model per spec from every roadmap table that has both a
    "spec" column and a "要件定義" column. Fable wins over Opus on conflict."""
    models, cols = {}, None
    for l in text.splitlines():
        if not l.startswith("|"):
            cols = None
            continue
        cells = [c.strip() for c in l.strip().strip("|").split("|")]
        if cols is None:
            cols = (cells.index("spec"), cells.index("要件定義")) if "spec" in cells and "要件定義" in cells else False
            continue
        if not cols or set(cells[0]) <= set("-: ") or len(cells) <= max(cols):
            continue
        name, model = spec_name(cells[cols[0]]), cells[cols[1]]
        kind = "Fable" if model.startswith("Fable") else "Opus" if model.startswith("Opus") else None
        if name and kind and models.get(name) != "Fable":
            models[name] = kind
    return models


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
| 順 | spec | 要件定義 | 規模 |
| -- | ---- | -------- | ---- |
| 1 | d-spec（既存） | Fable | 15 |
| 2 | a-spec | Opus（確定） | 3 |
| 3 | e-spec | 未定 | 3 |
"""


SAMPLE_RELEASE = """## 棚卸（2026-01-03・main `def`）
### ウェーブ（2026-01-03）
```
Wave リリース（0.1.0 の spec は 0 本）
/release 0.1.0
```
"""


def self_test():
    date, specs, warns, release = read_wave(SAMPLE)
    assert release is None, release
    assert date == "2026-01-02", date
    assert specs == [("Fable", "b-spec"), ("Opus", "a-spec"), ("Fable", "c-spec")], specs
    assert len(warns) == 1 and warns[0].startswith("c-spec"), warns
    models = read_models(SAMPLE)
    assert models == {"a-spec": "Opus", "b-spec": "Fable", "c-spec": "Fable", "d-spec": "Fable"}, models
    rel = SAMPLE_RELEASE
    assert read_wave(rel) == ("2026-01-03", [], [], "0.1.0"), read_wave(rel)
    for bad in ("# none\n", "## 棚卸（x）\n### ウェーブ（x）\nno block\n",
                rel.replace("/release 0.1.0", "/release 0.1.0\n/kiro-start a"), rel.replace("/release 0.1.0", "/release v0.1.0")):
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
    if "--models" in sys.argv:
        models = read_models(sys.stdin.buffer.read().decode("utf-8"))
        for name in sys.argv[sys.argv.index("--models") + 1:]:
            print(f"{models.get(name, 'unknown')} {name}")
        return 0
    try:
        date, specs, warns, release = read_wave(sys.stdin.buffer.read().decode("utf-8"))
    except ValueError as e:
        print(f"FAIL {e}", file=sys.stderr)
        return 1
    for w in warns:
        print(f"WARN {w}", file=sys.stderr)
    print(f"date {date}")
    if release:
        print(f"release {release}")
    for model, name in specs:
        print(f"{model} {name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

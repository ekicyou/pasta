"""Step 7-8 checks for /kiro-next in one run, before the commit.

Usage (run at the worktree root):
  python -I .claude/skills/kiro-next/scripts/finish.py [--fold] [--overview <out.html>]
  python -I .claude/skills/kiro-next/scripts/finish.py --self-test

--fold      delete the "- [x] " ledger rows of roadmap.md (they are folded into 完了フェーズ by hand first).
--overview  build the overview page into that file (its self-test runs first when overview/ changed).
Always: line endings of every changed or new file (a file that was CRLF must stay CRLF: no lone LF,
no CR CR), the wave that read-wave.py reads from roadmap.md (its self-test runs when it changed).
Prints one line per check; exit 1 when any check fails.
"""
import os
import subprocess
import sys

sys.dont_write_bytecode = True  # keep the skill folder free of __pycache__
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from scan import ROADMAP, git  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
READ_WAVE = os.path.join(HERE, "..", "..", "kiro-wave", "scripts", "read-wave.py")
OVERVIEW = os.path.join(HERE, "..", "overview", "roadmap-overview.ps1")


def fold(text):
    """Drop the done ledger rows; keeps every other byte, line endings included."""
    lines = text.splitlines(keepends=True)
    kept = [l for l in lines if not l.startswith("- [x] ")]
    return "".join(kept), len(lines) - len(kept)


def endings(now, before, autocrlf=False):
    """Problem with the line endings of a file, or None. before = its committed bytes (None if new).
    With core.autocrlf=true the checkout of an LF blob is CRLF, so that is what the file must keep."""
    if b"\0" in now:
        return None  # binary
    crlf = now.count(b"\r\n")
    lone = now.count(b"\n") - crlf
    was_crlf = (before.count(b"\r\n") > 0 or autocrlf) if before is not None else crlf > lone
    if now.count(b"\r\r"):
        return "CR CR"
    if was_crlf and lone:
        return f"{lone} lone LF in a CRLF file"
    if not was_crlf and crlf and before is not None:
        return f"{crlf} CRLF in an LF file"
    return None


def run(cmd, stdin=None):
    r = subprocess.run(cmd, input=stdin, capture_output=True)
    return r.returncode, (r.stdout + r.stderr).decode("utf-8", "replace").strip()


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    if "--self-test" in sys.argv:
        return self_test()
    fails = 0
    if "--fold" in sys.argv:
        raw = open(ROADMAP, "rb").read().decode("utf-8")
        text, n = fold(raw)
        with open(ROADMAP, "w", encoding="utf-8", newline="") as f:
            f.write(text)
        print(f"fold: removed {n} [x] row(s)")
    changed = sorted(set(git("diff", "--name-only", "HEAD").split()) | set(git("ls-files", "--others", "--exclude-standard").split()))
    bad = []
    r = subprocess.run(["git", "config", "--get", "core.autocrlf"], capture_output=True, text=True)
    autocrlf = r.stdout.strip() == "true"
    for p in changed:
        if not os.path.isfile(p):
            continue
        r = subprocess.run(["git", "show", f"HEAD:{p}"], capture_output=True)
        why = endings(open(p, "rb").read(), r.stdout if r.returncode == 0 else None, autocrlf)
        if why:
            bad.append(f"{p}: {why}")
    print(f"line endings: {len(changed)} file(s), " + ("ok" if not bad else "FAIL " + "; ".join(bad)))
    fails += bool(bad)
    if not os.path.exists(READ_WAVE):
        print("read-wave: skipped (no kiro-wave skill)")
    else:
        if any("kiro-wave/scripts/" in p for p in changed):
            code, out = run([sys.executable, "-I", READ_WAVE, "--self-test"])
            print(f"read-wave self-test: {out.splitlines()[-1] if out else code}")
            fails += code != 0
        code, out = run([sys.executable, "-I", READ_WAVE], open(ROADMAP, "rb").read())
        print("read-wave: " + out.replace("\n", " / "))
        fails += code != 0
    if "--overview" in sys.argv:
        dest = sys.argv[sys.argv.index("--overview") + 1]
        if any("kiro-next/overview/" in p for p in changed):
            code, out = run(["pwsh", "-NoProfile", "-File", OVERVIEW, "-SelfTest"])
            print(f"overview self-test: {out.splitlines()[-1] if out else code}")
            fails += code != 0
        code, out = run(["pwsh", "-NoProfile", "-File", OVERVIEW, "-OutFile", dest])
        print("overview: " + out.replace("\n", " / "))
        fails += code != 0 or "WARN" in out
    print("finish: " + ("all ok" if not fails else f"{fails} check(s) failed"))
    return 1 if fails else 0


def self_test():
    text, n = fold("# R\r\n- [x] a -- x\r\n- [ ] b -- y\r\n- [x] c -- z")
    assert (text, n) == ("# R\r\n- [ ] b -- y\r\n", 2), (text, n)
    assert endings(b"a\r\nb\r\n", b"a\r\n") is None
    assert endings(b"a\r\nb\n", b"a\r\n") == "1 lone LF in a CRLF file"
    assert endings(b"a\nb\r\n", b"a\n") == "1 CRLF in an LF file"
    assert endings(b"a\r\r\n", None) == "CR CR"
    assert endings(b"a\r\nb\r\n", b"a\n", autocrlf=True) is None
    assert endings(b"a\nb\n", b"a\n", autocrlf=True) == "2 lone LF in a CRLF file"
    assert endings(b"\0\r\r", None) is None
    assert endings(b"a\r\nb\r\n", None) is None and endings(b"a\nb\n", None) is None
    print("selftest: all passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())

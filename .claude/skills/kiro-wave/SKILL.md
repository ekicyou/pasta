---
name: kiro-wave
description: 'Launch the wave that /kiro-next recommended: reads the current wave from .kiro/steering/roadmap.md on the remote default branch (the "### ウェーブ（日付）" subsection of the latest "## 棚卸（…）" section) and starts one separate Claude Code session per Opus spec, each in its own worktree, running /kiro-start <name>. Specs whose requirements model is Fable are NOT started: they are excluded with a warning and given as individual /kiro-start commands for the developer to run in a Fable session. Never edits files, commits, pushes or merges. Use when: /kiro-wave, ウェーブを立ち上げる, ウェーブを一括で起動, start the wave. DO NOT USE FOR: planning the wave (use /kiro-next), starting one spec here (use /kiro-start).'
disable-model-invocation: true
allowed-tools: Bash, Read, Grep, Glob, ToolSearch
argument-hint: '[spec names to limit to]'
---

# kiro-wave — start the recommended wave as separate sessions

<instructions>

## Core Task
Start the wave `/kiro-next` wrote into the roadmap, one session per spec, so the developer does not paste `/kiro-start` commands one by one. Fable specs are never started here (developer 2026-10-23: 「Fable を推奨するスペックがある場合はそのスペックを除外し、除外したことを警告し、個別にスタートコマンドを出して実行を促してください」). Sessions started from here can only use this session's model or a cheaper one, so a Fable spec would silently run on the wrong model.

`$ARGUMENTS` (optional): spec names to limit the launch to. A name that is not in the wave is refused (reported, not started); this skill never starts a spec outside the wave.

Invoking `/kiro-wave` is the developer's explicit request for separate sessions: use `initiation: "user_asked"`.

## Communication Language
Think in English. Every developer-facing message (progress, warnings, the report) uses the session's console language and persona (fall back to Japanese).

## Hard Rules
- Read-only in this checkout: no file edits, no commits, no push, no PR, no merge, no roadmap changes.
- Only specs listed in the wave's code block are candidates. Never start a Fable spec. Never start a spec twice.
- **Never print the remote URL** (it may carry a token). Mask with `sed -E 's#https://[^@]*@#https://***@#'`.
- Git Bash rewrites `origin/main:path` as a Windows path: prefix `git show <ref>:<path>` with `MSYS_NO_PATHCONV=1`.

## Step 1 — Read the wave from the remote default branch
1. Default branch from `git symbolic-ref refs/remotes/origin/HEAD` (fallback `main`). `git fetch origin <default>` (masked output). If the fetch fails, STOP and report it (a stale roadmap may start the wrong wave).
2. Run, from the repository root:
   ```bash
   MSYS_NO_PATHCONV=1 git show origin/<default>:.kiro/steering/roadmap.md | python -I .claude/skills/kiro-wave/scripts/read-wave.py
   ```
   Output: `date <YYYY-MM-DD>` then one `<Fable|Opus> <name>` line per spec. `WARN …` lines on stderr are relayed to the developer as they are. `FAIL …` (exit 1) → STOP and tell the developer to run `/kiro-next` first (or fix the wave subsection); start nothing.
   The script treats a spec as Fable when either the code block or the wave table says Fable.
3. If `$ARGUMENTS` names specs, keep only those (refuse names that are not in the wave, in one line each).

## Step 2 — Check each spec before starting it
For each remaining spec, against `origin/<default>` (use `git ls-tree` / `git show` with `MSYS_NO_PATHCONV=1`; the local checkout may be behind):
- `.kiro/specs/<name>/brief.md` exists → otherwise skip ("brief が無い").
- `.kiro/specs/completed/<name>/` does not exist → otherwise skip ("完了済み").
- `.kiro/specs/<name>/spec.json` does not exist → otherwise skip ("もう始まっている"; the developer continues it in its own session).
- Not already running: `git worktree list` and `git branch -r` show no worktree or branch whose name contains `kiro-start-<name>`, and `mcp__ccd_session_mgmt__list_sessions` (load with `ToolSearch` if deferred) shows no live session titled for this spec → otherwise skip ("作業中のセッションがある").
Skipped specs are reported, never started.

## Step 3 — Start the Opus specs
Load `mcp__ccd_session__start_session` and `mcp__ccd_session__mark_chapter` with one `ToolSearch` call if they are deferred.

1. If two or more sessions will start, call `mcp__ccd_session__mark_chapter` first (title like `ウェーブ <date> の起動`).
2. Right before each start, in the same message, write a bold title line and one sentence on what the session will do.
3. Start the **first** spec alone with:
   - `initiation`: `"user_asked"`, `context`: `"fresh"`, `use_worktree`: `true`
   - `title`: `<name> の要件定義`, `name`: the spec name cut to 24 characters (`[a-z0-9-]`), `branch_name`: `kiro-start-<name>` (the Step 2 duplicate check looks for this)
   - `prompt`: exactly `/kiro-start <name>` — nothing else, so the whole line reaches `/kiro-start` as its argument
   - `background`: the shared context below (≥80 characters)
   - do not pass `model` (it inherits this session's model, which must be Opus for these specs)
4. Then confirm it took: `mcp__ccd_session_mgmt__list_sessions` with `{"linked": true}` shows it starting or working. If it failed to start or shows an error, STOP: report the cause and give the remaining specs as individual commands (Step 4 format) instead of starting them.
5. Start the rest the same way, one call per spec (they may go in one message after the first is confirmed).

**Background (fill `<…>`; Japanese is fine, it is read by the new session):**
```
この依頼は /kiro-wave から来た。<date> の棚卸のウェーブ（<全部の spec 名>）の 1 本として、spec `<name>` の要件定義を始める。
- 最初の 1 行の /kiro-start <name> をそのまま実行する。/kiro-start は Step 0.5 で origin/<default> を取り込むので、手元の main が古くても構わない。
- 同じウェーブの spec と同時に走る。触ってよいファイルと触らないファイルは、`.kiro/steering/roadmap.md` の「棚卸（<date>…）」の「同じウェーブに置くときの約束」と、brief の最後の再測定の節に従う。
- 要件の話し合いは、開発者がこのセッションを開いて行う。push・PR・マージはしない（kiro-complete の仕事）。
```

## Step 4 — Report (persona language)
1. Started sessions: one line each, `[<title>](#<local_… id>)` with the id from the tool result (never invent one). Say that each session stops at the requirements discussion and waits for the developer there.
2. **If any Fable spec was in the wave — a clearly marked warning** (e.g. 「⚠ Fable の spec を除外しました」) naming each excluded spec and the model reason, then for each one its start command in its own untagged fenced block (one command per block, so it can be pasted alone; not `bash`, it is not a shell command), and an explicit prompt to run it: open a new session (a harness worktree), switch the model to Fable, and paste the command.
   ```
   /kiro-start <name>
   ```
3. Skipped specs with their reasons (Step 2), refused `$ARGUMENTS` names, and `WARN` lines from Step 1.
4. If nothing was started (all Fable or all skipped), say so plainly.

When a started session ends a turn, the app may continue this session with a note; then report in a line or two which sessions are waiting for the developer, and start nothing new.

</instructions>

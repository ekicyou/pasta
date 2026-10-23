---
name: kiro-wave
description: 'Launch the wave that /kiro-next recommended: reads the current wave from .kiro/steering/roadmap.md on the remote default branch (the "### ウェーブ（日付）" subsection of the latest "## 棚卸（…）" section) and starts one separate Claude Code session per Opus spec, each in its own worktree, running /kiro-start <name>. Specs whose requirements model is Fable are NOT started: they are excluded with a warning and given as individual /kiro-start commands for the developer to run in a Fable session. With spec names as arguments (e.g. /kiro-wave scene-exact-match) it starts only those specs, in or out of the wave, under the same Fable rule. Never edits files, commits, pushes or merges. Use when: /kiro-wave, ウェーブを立ち上げる, ウェーブを一括で起動, start the wave. DO NOT USE FOR: planning the wave (use /kiro-next), starting one spec here (use /kiro-start).'
disable-model-invocation: true
allowed-tools: Bash, Read, Grep, Glob, ToolSearch
argument-hint: '[spec names to start instead of the wave]'
---

# kiro-wave — start the recommended wave as separate sessions

<instructions>

## Core Task
Start the wave `/kiro-next` wrote into the roadmap, one session per spec, so the developer does not paste `/kiro-start` commands one by one. Fable specs are never started here (developer 2026-10-23: 「Fable を推奨するスペックがある場合はそのスペックを除外し、除外したことを警告し、個別にスタートコマンドを出して実行を促してください」). Sessions started from here can only use this session's model or a cheaper one, so a Fable spec would silently run on the wrong model.

Two modes, chosen by `$ARGUMENTS`:
- **Wave mode** (no arguments): start the current wave's specs.
- **Named mode** (`/kiro-wave scene-exact-match [more names…]`): start only the named specs, whether or not they are in the wave (developer 2026-10-23: 「指定されたスペックだけを、サブセッションとして『/kiro-start scene-exact-match』を立ち上げる」). The wave is not read for candidates; the Fable rule and the Step 2 checks still apply.

Invoking `/kiro-wave` is the developer's explicit request for separate sessions: use `initiation: "user_asked"`.

## Communication Language
Think in English. Every developer-facing message (progress, warnings, the report) uses the session's console language and persona (fall back to Japanese).

## Hard Rules
- Read-only in this checkout: no file edits, no commits, no push, no PR, no merge, no roadmap changes.
- Candidates are the wave's code block (wave mode) or exactly the named specs (named mode) — nothing else. Never start a Fable spec, or one whose model cannot be confirmed as Opus. Never start a spec twice.
- **Never print the remote URL** (it may carry a token). Mask with `sed -E 's#https://[^@]*@#https://***@#'`.
- Git Bash rewrites `origin/main:path` as a Windows path: prefix `git show <ref>:<path>` with `MSYS_NO_PATHCONV=1`.

## Step 1 — Read the candidates from the remote default branch
1. Default branch from `git symbolic-ref refs/remotes/origin/HEAD` (fallback `main`). `git fetch origin <default>` (masked output). If the fetch fails, STOP and report it (a stale roadmap may start the wrong wave).
2. **Named mode**: first refuse (report, do not start) any name that is not `[a-z0-9-]+` — names go into shell commands. Then get each named spec's requirements model, then go to Step 2.
   ```bash
   MSYS_NO_PATHCONV=1 git show origin/<default>:.kiro/steering/roadmap.md | python -I .claude/skills/kiro-wave/scripts/read-wave.py --models <name>...
   ```
   Output: one `<Fable|Opus|unknown> <name>` line each (looked up in every roadmap table with `spec` and `要件定義` columns; Fable wins on conflict). For `unknown`, read the brief on `origin/<default>` and take the last line that contains `要件定義のモデル` (the latest 棚卸の再測定 section, e.g. `- **要件定義のモデル**: Opus。…`): its value starting with `Opus` → Opus; `Fable`, missing, or unclear → treat as Fable (excluded in Step 3, with the reason "モデルを確かめられない"). Check the spec exists before trusting `unknown` (Step 2).
3. **Wave mode**: run, from the repository root:
   ```bash
   MSYS_NO_PATHCONV=1 git show origin/<default>:.kiro/steering/roadmap.md | python -I .claude/skills/kiro-wave/scripts/read-wave.py
   ```
   Output: `date <YYYY-MM-DD>` then one `<Fable|Opus> <name>` line per spec. `WARN …` lines on stderr are relayed to the developer as they are. `FAIL …` (exit 1) → STOP and tell the developer to run `/kiro-next` first (or fix the wave subsection); start nothing.
   The script treats a spec as Fable when either the code block or the wave table says Fable.

## Step 2 — Check each spec before starting it
For each candidate, against `origin/<default>` (use `git ls-tree` / `git show` with `MSYS_NO_PATHCONV=1`; the local checkout may be behind):
- `.kiro/specs/<name>/brief.md` exists → otherwise skip ("brief が無い").
- `.kiro/specs/completed/<name>/` does not exist → otherwise skip ("完了済み").
- `.kiro/specs/<name>/spec.json` does not exist → otherwise skip ("もう始まっている"; the developer continues it in its own session).
- Not already running: `git worktree list` and `git branch -r` show no worktree or branch whose name contains `kiro-start-<name>`, and `mcp__ccd_session_mgmt__list_sessions` (load with `ToolSearch` if deferred) shows no live session titled for this spec → otherwise skip ("作業中のセッションがある").
Skipped specs are reported, never started.

## Step 3 — Start the Opus specs
Load `mcp__ccd_session__start_session` and `mcp__ccd_session__mark_chapter` with one `ToolSearch` call if they are deferred.

1. If two or more sessions will start, call `mcp__ccd_session__mark_chapter` first (title like `ウェーブ <date> の起動`, or `指名した spec の起動` in named mode).
2. Right before each start, in the same message, write a bold title line and one sentence on what the session will do.
3. Start the **first** spec alone with:
   - `initiation`: `"user_asked"`, `context`: `"fresh"`, `use_worktree`: `true`
   - `title`: `<name> の要件定義`, `name`: the spec name cut to 24 characters (`[a-z0-9-]`), `branch_name`: `kiro-start-<name>` (the Step 2 duplicate check looks for this)
   - `prompt`: exactly `/kiro-start <name>` — nothing else, so the whole line reaches `/kiro-start` as its argument
   - `background`: the shared context below (≥80 characters)
   - do not pass `model` (it inherits this session's model, which must be Opus for these specs)
4. Then confirm it took: `mcp__ccd_session_mgmt__list_sessions` with `{"linked": true}` shows it starting or working. If it failed to start or shows an error, STOP: report the cause and give the remaining specs as individual commands (Step 4 format) instead of starting them.
5. Start the rest the same way, one call per spec (they may go in one message after the first is confirmed).

**Background (fill `<…>`; Japanese is fine, it is read by the new session).** Wave mode:
```
この依頼は /kiro-wave から来た。<date> の棚卸のウェーブ（<全部の spec 名>）の 1 本として、spec `<name>` の要件定義を始める。
- 最初の 1 行の /kiro-start <name> をそのまま実行する。/kiro-start は Step 0.5 で origin/<default> を取り込むので、手元の main が古くても構わない。
- 同じウェーブの spec と同時に走る。触ってよいファイルと触らないファイルは、`.kiro/steering/roadmap.md` の「棚卸（<date>…）」の「同じウェーブに置くときの約束」と、brief の最後の再測定の節に従う。
- 要件の話し合いは、開発者がこのセッションを開いて行う。push・PR・マージはしない（kiro-complete の仕事）。
```
Named mode:
```
この依頼は /kiro-wave で開発者が個別に指名した spec `<name>` の要件定義である（同時に指名した spec：<名前の一覧、無ければ「なし」>）。
- 最初の 1 行の /kiro-start <name> をそのまま実行する。/kiro-start は Step 0.5 で origin/<default> を取り込むので、手元の main が古くても構わない。
- 今のウェーブの外かもしれない。ほかの spec とファイルが重ならないかを brief の最後の再測定の節と roadmap の Dependencies で確かめ、重なりがあれば要件の話し合いで開発者に知らせる。
- 要件の話し合いは、開発者がこのセッションを開いて行う。push・PR・マージはしない（kiro-complete の仕事）。
```

## Step 4 — Report (persona language)
1. Started sessions: one line each, `[<title>](#<local_… id>)` with the id from the tool result (never invent one). Say that each session stops at the requirements discussion and waits for the developer there.
2. **If any candidate was Fable (or its model could not be confirmed) — a clearly marked warning** (e.g. 「⚠ Fable の spec を除外しました」) naming each excluded spec and the model reason (where the model came from: the wave, a roadmap table, or the brief), then for each one its start command in its own untagged fenced block (one command per block, so it can be pasted alone; not `bash`, it is not a shell command), and an explicit prompt to run it: open a new session (a harness worktree), switch the model to Fable, and paste the command.
   ```
   /kiro-start <name>
   ```
3. Skipped specs with their reasons (Step 2) and `WARN` lines from Step 1. In named mode, add one line that the named specs outside the current wave were not checked for file overlap with running specs here (each new session checks it in its requirements discussion).
4. If nothing was started (all Fable or all skipped), say so plainly.

When a started session ends a turn, the app may continue this session with a note; then report in a line or two which sessions are waiting for the developer, and start nothing new.

</instructions>

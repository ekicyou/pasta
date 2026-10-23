---
name: kiro-wave
description: 'Launch the wave that /kiro-next recommended: reads the current wave from .kiro/steering/roadmap.md on the remote default branch (the "### ウェーブ（日付）" subsection of the latest "## 棚卸（…）" section) and starts one separate Claude Code session per Opus spec, each in its own worktree, running /kiro-start <name>. Specs whose requirements model is Fable are NOT started: they are excluded with a warning and given as individual /kiro-start commands for the developer to run in a Fable session. With spec names as arguments (e.g. /kiro-wave scene-exact-match) it starts only those specs, in or out of the wave, under the same Fable rule. Because the new sessions read the remote default branch, it first lands this worktree's own changes (uncommitted edits or unmerged commits, e.g. the /kiro-next inventory) with PR → squash merge, after one approval dialog; otherwise it edits nothing. Use when: /kiro-wave, ウェーブを立ち上げる, ウェーブを一括で起動, start the wave. DO NOT USE FOR: planning the wave (use /kiro-next), starting one spec here (use /kiro-start).'
disable-model-invocation: true
allowed-tools: Bash, Read, Grep, Glob, ToolSearch, AskUserQuestion
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
- The only writes are Step 0 (commit → push → PR → squash merge of this worktree's own changes, after the approval dialog). Otherwise read-only: no file edits, no roadmap changes.
- Never start a session before this worktree's changes are on the remote default branch: the new sessions read it, so an unmerged inventory would start the wrong wave (developer 2026-10-23: 「kiro-wave の実施には『PR→スクワッシュマージ』が必須」).
- Candidates are the wave's code block (wave mode) or exactly the named specs (named mode) — nothing else. Never start a Fable spec, or one whose model cannot be confirmed as Opus. Never start a spec twice.
- **Never print the remote URL** (it may carry a token). Mask with `sed -E 's#https://[^@]*@#https://***@#'`.
- Git Bash rewrites `origin/main:path` as a Windows path: prefix `git show <ref>:<path>` with `MSYS_NO_PATHCONV=1`.

## Step 0 — Land this worktree's changes first (PR → squash merge)
1. Default branch from `git symbolic-ref refs/remotes/origin/HEAD` (fallback `main`). `git fetch origin <default>` (masked output). If the fetch fails, STOP and report it (a stale roadmap may start the wrong wave).
2. Find what is not landed yet:
   - **uncommitted**: `git status --porcelain`. Files whose diff is line endings only (`git diff --ignore-cr-at-eol --quiet -- <file>` exits 0, e.g. the `sample.generated.lua` that the pasta_lua tests rewrite) are restored with `git checkout -- <file>` and not counted.
   - **unmerged commits**: `git log --oneline origin/<default>..HEAD`. Commits already squash-merged still show here, so when the list is not empty, check `gh pr list --head <branch> --state merged --json number,headRefOid`: if a merged PR's `headRefOid` equals `git rev-parse HEAD`, the commits are landed.
3. Nothing left → say so in one line and go to Step 1.
4. If the current branch is the default branch and something is left → STOP: changes on the default branch cannot go through a PR; ask the developer to move them to a worktree branch. Start nothing.
5. Otherwise ask **once with `AskUserQuestion`** (the approval dialog; load it with `ToolSearch` if deferred). Question (persona language): this worktree has changes that are not on `origin/<default>`, and the new sessions need them there — land them with PR → squash merge and continue? Put the facts in the option descriptions: the branch, the unmerged commit subjects, the uncommitted files (at most ~10, then "ほか N 件"), and an open PR if `gh pr list --head <branch> --state open` finds one. Options:
   - `PR→スクワッシュマージして続ける (Recommended)` — land, then start the sessions
   - `やめる` — start nothing
   Anything other than the first option → STOP and start nothing (say the wave can be started after the changes are merged).
6. On approval, without asking again (developer 2026-10-10: no second confirmation between PR and merge):
   1. If there are uncommitted changes: `git add -A` and commit with a subject that says what changed (from the diff, conventional-commit style like the repository's history) and the session's attribution trailer.
   2. `git push -u origin <branch>` (masked output).
   3. Reuse the open PR if there is one; otherwise `gh pr create --base <default> --head <branch>` with the commit subject as the title (the latest subject if several, or a one-line summary) and a short bullet body ending with the attribution line. Avoid double backslashes in `gh` arguments.
   4. `gh pr merge <number> --squash --delete-branch`. Do not wait for CI. The local-branch deletion warning ("checked out in the current worktree") is not a failure.
   5. Verify with `gh pr view <number> --json state,mergeCommit`: `MERGED` → `git fetch origin <default>` and report the PR (as a full link) and the merge commit in one line. Anything else, or a refused merge → STOP and report it; do not route around the refusal with another tool, and start nothing.

## Step 1 — Read the candidates from the remote default branch
1. Use the default branch and the fetch from Step 0 (after a merge in Step 0, the fetch there already includes it).
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
   A `release <X.Y.Z>` line instead of spec lines means the inventory found nothing left in the current goal and recommends the release (areka developer 2026-10-23: 「ウェーブの対象が０本となり、リリースできるなら、/releaseを開始コマンドとせよ」): start nothing — `/release` runs only when the developer types it — and tell the developer to type `/release <X.Y.Z>` (one untagged fenced block), then stop.
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
Before the report, detach every session confirmed as starting or working in Step 3 with `mcp__ccd_session_mgmt__detach_session` (load with `ToolSearch` if deferred), one call per session id. Otherwise the app wakes this session each time a started session ends a turn on its own, e.g. every time a `/kiro-impl` implementer or reviewer comes back (developer 2026-10-23: monitoring the started sessions is not needed). A refused detach is reported in one line; it does not stop the report.

1. Started sessions: one line each, `[<title>](#<local_… id>)` with the id from the tool result (never invent one). Say that each session stops at the requirements discussion and waits for the developer there, and that, once detached, it no longer reports back here (follow it from the sidebar).
2. **If any candidate was Fable (or its model could not be confirmed) — a clearly marked warning** (e.g. 「⚠ Fable の spec を除外しました」) naming each excluded spec and the model reason (where the model came from: the wave, a roadmap table, or the brief), then for each one its start command in its own untagged fenced block (one command per block, so it can be pasted alone; not `bash`, it is not a shell command), and an explicit prompt to run it: open a new session (a harness worktree), switch the model to Fable, and paste the command.
   ```
   /kiro-start <name>
   ```
3. Skipped specs with their reasons (Step 2) and `WARN` lines from Step 1. In named mode, add one line that the named specs outside the current wave were not checked for file overlap with running specs here (each new session checks it in its requirements discussion).
4. If nothing was started (all Fable or all skipped), say so plainly.

If a session could not be detached and the app later continues this session with a note on it, report in a line or two which sessions are waiting for the developer, and start nothing new.

</instructions>

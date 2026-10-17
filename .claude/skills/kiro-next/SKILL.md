---
name: kiro-next
description: 'Roadmap inventory ("棚卸") and next-wave planning for the pasta Kiro workflow, run after main has advanced. Syncs the harness worktree branch with remote main, identifies in-flight specs and reserves the files they touch, re-measures every unfinished brief against current main with parallel subagents (each brief gets a "棚卸の再測定" section), applies trivial doc/comment/path fixes immediately, splits oversized specs, promotes brief-less backlog items into briefs, builds a strict dependency tree, ranks its head specs (descendant count → kind bug/feature → filing order), picks up to 8 mutually non-overlapping specs as the next wave, rewrites .kiro/steering/roadmap.md (and stale bits of product.md), commits on the worktree branch WITHOUT pushing, and reports the wave as a Fable/Opus /kiro-start code block. PR creation and squash merge happen only behind an explicit developer approval gate. Use when: /kiro-next, 棚卸, mainが進んだので棚卸, 次のウェーブ, next wave. DO NOT USE FOR: starting a spec (use /kiro-start), filing one new idea (use /kiro-discovery), completing a spec (use /kiro-complete).'
disable-model-invocation: true
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Agent, SendMessage, WebFetch, ToolSearch, AskUserQuestion
argument-hint: '[extra instructions]'
---

# kiro-next — roadmap inventory and next wave

<instructions>

## Core Task
Run one full roadmap inventory ("棚卸") on the current harness worktree branch and propose the next parallel wave. The developer's standing instruction is fixed and is the default behaviour of this skill (no arguments needed):

1. main has advanced → inventory the roadmap, deep-dive the specs, brief thoroughly.
2. Apply roadmap-listed trivial fixes (no brief needed) immediately and close them.
3. Of the items that exist only in the roadmap (no brief), launch briefs for those that warrant one; adjust granularity; split specs whose load is too high.
4. Build a strict dependency tree, rank its head specs, and pick at most 8 as the next wave — only from specs inside the current goal (the next release milestone the roadmap sets). Specs beyond the goal never enter the wave.
5. Report the wave in the fixed code-block format, split into Fable and Opus.

`$ARGUMENTS` (optional) carries extra developer instructions for this run; they override the defaults where they conflict.

## Communication Language
- **Think in English.** All internal reasoning, planning and subagent prompts are English.
- **Every developer-facing message uses the session's console language and persona** (the language/persona configured for the session; fall back to Japanese). This includes progress notes, questions, the final report and the approval gate. Reports are always Japanese-side, even right after reading English subagent reports.
- Text written into project files (briefs, roadmap, steering) uses the spec language (`spec.json.language`; this repository: Japanese) in plain everyday words — no project jargon, no abbreviations; cite code by "what it is" plus path.
- Refer to specs by name. `#数字` only as `PR#123`.

## Hard Rules
- **Never on the default branch.** If the current branch is the repository default branch, STOP and ask the developer to re-run inside a harness worktree.
- **No push, no PR, no merge without the approval gate (Step 11).** Commits stay local on the worktree branch.
- **Never touch files reserved by in-flight specs** except the allowed overlaps (Step 6).
- **Never print the remote URL** (it may carry a token). Mask with `sed -E 's#https://[^@]*@#https://***@#'`.
- **Preserve line endings per file.** Briefs are CRLF; `.kiro/steering/*.md` may be LF. Check each file with `file` before editing. git-bash `sed -i` strips CR — use the Edit tool, `perl -pi`, or Python with `newline=''`. Verify with `file` and by counting `b'\r\r'` / lone `\n`.
- **Git Bash has no `$TMPDIR`.** Always use the absolute scratchpad path in POSIX form for temp files.
- **Never run `cargo` for this skill** unless a step says so; the inventory is Markdown work (Grep/Glob/Read only for measurement). Heavy builds pollute other sessions' work.
- Do not use `spawn_task` chips for out-of-scope findings; they become briefs or roadmap backlog items here.

## Step 0 — Resolve context
- Repository root = current worktree. Steering: `.kiro/steering/` (`product.md`, `tech.md`, `structure.md`, `workflow.md`, `roadmap.md`). Specs: `.kiro/specs/` (completed under `completed/`). Default branch from `git symbolic-ref refs/remotes/origin/HEAD` (fallback `main`).
- Scratch directory = the session scratchpad (never `/tmp`, never `C:\`).
- Load memory relevant to inventories: parallel-spec report format (Fable need flagged at the top), merge-without-waiting-CI, line-ending notes.
- Name the session `<repository name>棚卸` (e.g. `pasta棚卸`). The repository name is the main checkout's folder name: `basename "$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")"` (the same `R` as the desk step below). Set it with `mcp__ccd_session_mgmt__set_session_title` (load it with `ToolSearch` if it is deferred); skip if the title is already that. If the tool is missing or refuses, say so in one line and go on (developer 2026-10-17: 「スキル名を『{リポジトリ名}棚卸』としてください」).
- Clear this repository's desk records once (last item of Step 0, before the sync in Step 1 and before any inventory work). At inventory time nobody works in this repository, so leftover records of dead sessions are removed here. Run this one Bash call in the foreground, exactly once:

```bash
R=$(basename "$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")")
if [ -z "$AREKA_IMPL_WATCH_HOME" ]; then echo "desk: skip no-home"
elif [ ! -f "$AREKA_IMPL_WATCH_HOME/areka-impl-watch.exe" ]; then echo "desk: skip no-exe"
elif ! [[ "$R" =~ ^[A-Za-z0-9._-]{1,100}$ ]]; then echo "desk: skip bad-name $R"
else "$AREKA_IMPL_WATCH_HOME/areka-impl-watch.exe" clear --repo "$R" 2>&1; echo "exit=$?"; fi
```

  - Reading the result: `exit=0` → relay the output's one line as is (`cleared repo=<name>; left: <ids>` or `left: none`). `desk: skip …`, `exit=1` (including a refused environment variable or home folder), or `exit=2` (an exe that does not have `clear --repo` yet) → tell the developer one line and continue the inventory. In no case does this step stop the inventory.
  - Never call the full `clear` (no arguments) and never touch another repository's records. Do not start `watch` (do not become a desk participant) and do not read `status`.
  - If the owner of a cleared record was in fact still running, that session's own desk rules re-establish its watch and apply again; this step adds no rule of its own.

## Step 1 — Sync with remote main
1. Call `mcp__ccd_host__sync_with_base_branch` (base = default branch).
2. If it refuses (e.g. credentials in the origin URL), do it yourself: `git fetch origin <default>` (masked output) → `git merge --ff-only FETCH_HEAD`; if the branch has its own commits, `git merge origin/<default>`. Resolve conflicts yourself; never force.
3. Verify: `git ls-remote origin refs/heads/<default>` equals the merge base you expect (no more than the local branch's own commits ahead). Record the base SHA (`<BASE>`) for every section heading below.

## Step 2 — Lightweight scan
- `.kiro/steering/roadmap.md` is the single ledger (~30 KB). Read it whole: 概要 / 運用ルール / 前提として確定している方針 / 完了フェーズ（要約）/ 常駐 spec / 棚卸 section / each active Phase section (境界戦略・ウェーブ構成) / `## Specs (dependency order)` / バックログ.
- Inventory: `ls .kiro/specs/` (non-completed folders — each should hold `brief.md`; note any with `spec.json`), `ls .kiro/specs/completed | wc -l`. Cross-check that every unchecked `- [ ]` ledger row has a folder and vice versa.
- **In-flight specs**: `git worktree list` and `git branch -r --sort=-committerdate`. For each in-flight spec branch (other worktrees, recent `origin/claude/*` branches not yet merged): `git diff --name-only <default>...<branch> | grep -v '^\.kiro/'` → save to `<scratch>/inflight-files.md`. Also read each in-flight spec's brief/design for files it still plans to touch. All of these are **reserved**. A spec whose branch is already merged is not in flight.
- Determine the inventory date (today) and whether a previous "棚卸（YYYY-MM-DD）" section exists to supersede.
- **Current goal**: find the roadmap's milestone sections (e.g. `## 0.4.0 までの道筋` and the goal set for the version after it). The current goal is the nearest unreleased milestone; its scope is the set of specs its section lists (in its table or text). Record that scope and the section's own order and priority marks (e.g. 高優先度). If no milestone is set, note "ゴール未設定" for Step 10 and treat every unfinished spec as in scope.

## Step 3 — Parallel re-measurement (subagents)
1. Write `<scratch>/remeasure-common.md` (template below).
2. Partition every unfinished, not-in-flight spec into groups of ~4–6 following the Phase sections of the roadmap (e.g. Phase 11 bug lane, Phase 11 feature lane, Phase 12 manual/guide, release-ci and other cross-cutting). If there are ≤ 6 such specs, use one group. Every spec in exactly one group — check the count adds up. Re-measure specs inside the current goal; re-measure a spec beyond the goal only when a spec landed since the previous inventory changed its files or premises.
3. Launch all groups plus one "fixes & brief-less items" agent **in one message, in the background** (`Agent`, `model: "opus"`). Each group prompt = "read the common file and follow it" + its spec list + 3–5 lines of focus (which landed specs changed its premises, which in-flight branches touch its files, near-cap files, suspected splits).
4. As each report arrives, append a compact summary to `<scratch>/results.md` (name | kind | size | model | head? | blocking overlaps | split). Do not keep raw reports in context.
5. If agents die (e.g. HTTP 429) and the developer says resume: run `git status` and `grep -l '棚卸の再測定' .kiro/specs/*/brief.md` to detect partial writes, then relaunch the same prompts.

**Common instruction template** (fill `<…>`):
```
# Common instructions — roadmap inventory re-measurement
Worktree: <path> (run everything here). Base: <default> <BASE> (<date>). Previous inventory: <date of the previous 棚卸 section>. Specs landed since then: <list> (see .kiro/specs/completed/<name>/).
In-flight specs (NOT candidates; their files are reserved): <list with one-line notes>. Changed files: <scratch>/inflight-files.md. Also read their briefs for planned files.
Read first: .kiro/steering/roadmap.md (運用ルール, boundary strategy and wave tables of the active Phases); each assigned brief (and requirements/design/tasks if present).
For EACH spec: (1) verify the brief against current main with Grep/Glob/Read only — no cargo, no heavy commands; note stale claims and what landed specs already did. (2) touched source files on main (mark new files; note files near 1,000 lines with wc -l). (3) size in tasks (cap 20); if >20 propose a concrete split (names, boundary, files per half) — never re-split a spec already carved once just for barely crossing 20. (4) dependencies: functional (unfinished only) and file overlap with other unfinished specs including in-flight ones (any shared source file blocks the same wave; same-page different-section manual edits are allowed). (5) kind: バグ / 機能 / 文書 / 基盤, with one-phrase evidence (bug = wrong current behaviour; 機能 = new capability; also mark "developer asked for it" when the brief cites the developer's request as origin). (6) requirements model: Opus by default. Fable ONLY when BOTH hold: (i) a hard-to-reverse foundation decision — new DSL syntax/semantics, a persisted data format, or a public Lua API contract that existing ghosts or later specs will build on; AND (ii) that decision is still deep: three or more interlocking open questions with no recommendation yet, or a canon (Ukagaka/SSP/SHIORI) ambiguity the docs do not settle, or timing/concurrency semantics across Rust/Lua/SHIORI. Never Fable for these alone: many crates touched, crossing the Lua/Rust boundary, large size, many agenda items, a breaking change with a clear migration path, choices already narrowed to two with a recommendation. When answering Fable, name the specific decision and which of (ii) applies; if you cannot, or it is borderline, answer Opus. (7) immediate-fix candidates (stale comments, broken paths, doc errors) with file:line — do NOT fix code. (8) APPEND to the end of each brief.md a section "## <date> 棚卸の再測定（main <BASE>）" in Japanese plain words, ≤25 lines: 前提の変化 / 触るファイル / 規模 / 先に要るもの / 種別 / 要件定義のモデル / 分割の案 / 見つけた穴・古くなった記述. Preserve line endings. Edit no other file. Do not commit.
Return (English, compact), one block per spec:
<name> | kind (evidence) | size | model
 touches: … | functional deps: … | file-overlap with: spec(file), … | split: … | stale/holes: … | immediate fixes: …
Last line: "briefs written: N/N".
```

**Fixes & brief-less agent** (read-only, may WebFetch one crate version): classify each `バックログ` item and each "人手の確認が残っている項目" as (a) trivially fixable now with exact file:line and change, (b) needs a brief (S+ or a decision that can be a requirements agenda item), (c) still blocked (developer decision / missing consumer / manual verification); scan Implementation Notes and research "範囲外" of specs completed since the last inventory for orphaned leftovers; find stale statements in steering (`product.md` Phase list, `structure.md`, `tech.md`), `README.md`; grep `.kiro/specs/<name>/` paths in `crates/`, `book/`, `.claude/skills/` that moved to `completed/`; list `.rs` files ≥ 950 lines; count completed entries and brief folders.

## Step 4 — Immediate fixes (controller, right away)
- Apply only class-(a) fixes: doc/comment/path/count/date corrections and wrong pointers in the roadmap. Use the Edit tool.
- Before editing a file an in-flight branch also edits, diff that branch to confirm a different hunk; if a fix lands in a reserved file, hand it to the owning spec's brief instead.
- Never "fix" anything that changes behaviour, security posture, bytes sent to SHIORI, or needs a design decision — those stay in the backlog or become briefs.
- If a fix touches `book/` or the generated skill references, run `node book/tools/gen-skill-refs.mjs` (never hand-edit generated files) and the checks in Step 8.

## Step 5 — Granularity: splits and new briefs (subagents)
- **Split** a spec when the re-measurement says >20 tasks and it was never carved out before (also when a human wait, e.g. an external review, would sit inside one PR). Do not split specs already carved once for barely crossing 20; mention a parallelism-only split as a developer decision instead.
- **Promote** backlog items classed (b). Keep (c) items in the backlog with their reason.
- Launch two background agents (Opus): one performs the splits (append "## <date> 棚卸の分割" to the original — In/Out, what moved where, order, size, files — and create the new brief), one writes the promoted briefs. House style: `# Brief: <name>`, Problem / Current State / Desired Outcome / Approach / Scope (In/Out) / Boundary Candidates / Out of Boundary / Upstream / Downstream / Existing Spec Touchpoints / Constraints, first line naming the origin (棚卸 date, split from / backlog item), and a closing measurement section (files, size, deps, kind, model, agenda). Canon claims (Ukagaka/SSP/SHIORI behaviour) must be checked with the ukagaka-docs MCP (`search_docs` takes ONE word). New briefs are CRLF and go in `.kiro/specs/<name>/brief.md`.
- Never move or delete brief folders.

## Step 6 — Dependency tree and the next wave
**Edges** (strict): functional dependencies + any shared source file (other unfinished specs and in-flight specs) + the roadmap's wave order. Allowed overlaps that do NOT create edges: steering files; generated files (`.claude/skills/*/references/` regenerated by `gen-skill-refs.mjs`); different sections of the same manual page; different rows of the same table. A conditional overlap ("only if …") counts as an edge unless the wave row states a promise that removes it. Hot files the roadmap names in its 境界戦略 (currently `crates/pasta_lua/src/code_gen/element_gen.rs`, `crates/pasta_lua/pasta_scripts/pasta/act.lua`) belong to one spec per wave.
**Single-seat resources** (one spec per wave): `Cargo.lock` / `tech.md` dependency additions; `.github/workflows/release.yml` and `release.ps1` (the release-ci seat); the `release` skill (`.claude/skills/release/`) during any release.
**Excluded from the tree**: 却下・保留・据え置きの spec and `常駐 spec` (neither candidates nor counted as descendants). **Not parallelised**: specs whose core work is measurement (latency/perf/load).
**Goal gate**: only specs inside the current goal (Step 2) are wave candidates. A spec beyond the goal never enters the wave — not when it is a head, not when it has the most descendants, not when the roadmap says it may run "alongside" the milestone. The only exception is a bug the developer or the roadmap marks 致命的 (fatal). Extra developer instructions for this run do not lift the gate by themselves: if they name a beyond-goal spec, report that the milestone section must first be changed to include it (a Step 10 decision item). Beyond-goal specs still count as descendants.
**Procedure**:
1. Heads = specs with no unfinished predecessor (in-flight specs count as unfinished predecessors).
2. Descendant count = number of unfinished specs transitively depending on the head (approximate is fine; state "約").
3. Drop heads beyond the goal. Sort the rest by the goal section's own order and priority marks → descendant count desc → kind (developer-requested / バグ before 機能 / 文書) → filing order (higher in the ledger).
4. Greedily take heads from the top whose touched files do not overlap any already-taken head; stop at 8.
5. For every chosen spec write its touched files and the **same-wave promises** (files it must not touch). List the in-goal heads not chosen with one-line reasons, and outline the following wave's candidates (in-goal only). Beyond-goal heads go on one separate "ゴールの後" line with their order only.
6. Model per chosen spec from the re-measurement (Fable / Opus).

## Step 7 — Write the roadmap
Edit `.kiro/steering/roadmap.md` (Edit tool or one Python script in the scratch directory; assert every anchor exists):
- **Fold** finished specs: every `[x]` ledger row and every completed Phase section collapses into `完了フェーズ（要約）` (one row per Phase: 主題 and spec names). Old wave tables and superseded 棚卸 sections are deleted (git history keeps them; nothing is archived to a separate file).
- Replace the previous "棚卸（YYYY-MM-DD）" section with the new one: 即時修正 (what was fixed and closed, what was handed to specs), 統合・分割・改名, promotions and what stayed in the backlog (with reason), the wave and why, the Fable/Opus split, broken same-wave promises, holes found, 人手の確認が残っている項目, decisions needed. In the wave subsection `### ウェーブ（<date>）`, right after its table (first column = spec name), write the wave's start commands as the subsection's only fenced code block — a one-line caption, then the block exactly as in Step 10 item 4 (leave out an empty Fable group). The overview page (Step 8) reads the first fenced block of this subsection and shows it with a copy button (areka developer 2026-10-16 「ウェーブのコード表示をコピペボタン付きで」).
- Keep the shapes the overview page reads (its markers live in `overview/roadmap-overview.strings.json`): the heading `## 棚卸（YYYY-MM-DD・main …）`, the version tables whose header starts `| 順 | spec |` (one table per version, under a `## ` heading that names the version), `## Specs (dependency order)`, and the first table of `## 完了フェーズ（要約）`. If you change one of these shapes, change the markers and the fixture in the same commit.
- Active Phase sections: updated `境界戦略` and `ウェーブ構成` (rules, the new wave row with touched files and promises, next-wave candidates, 保留).
- `## Specs (dependency order)`: unfinished specs only, in dependency order, `- [ ] <name> -- <summary>. Dependencies: <list|none>`; split notes on the original rows, new rows inserted in dependency position.
- `バックログ` minus promoted items. Update the header `概要` if the current theme changed.
- The current goal's milestone section: update each in-scope spec's status (完了 / 開発中 / this wave / waiting on what). Do not add specs to its scope yourself — scope changes are developer decisions.
- Then stale bits of `product.md` (Phase list / 現在地) and `structure.md` / `tech.md`. Grep for old wording of changed rules and for references to moved sections.
- Verify: `git diff --numstat` proportional to the edit (a whole-file diff means broken line endings); `b'\r\r'` count 0 and no lone `\n` in CRLF files.

## Step 8 — Verify and commit
- If `book/`, `.claude/skills/pasta-ghost-authoring/` or `.claude/skills/pasta-lua-coding/` changed, run both and require exit 0:
  ```
  node book/tools/gen-skill-refs.mjs --check
  node book/tools/link-check.mjs
  ```
  Otherwise nothing needs to run (Markdown-only inventory).
- Commit on the worktree branch, no push: `docs(roadmap): 棚卸＝<what landed>の後の再測定・<n> 本を分割・<m> 本を起票・ウェーブ <k> 本を組む`, with a short bullet body and the required attribution trailer.
- **Overview page** (ported from areka's `/kiro-next` 2026-10-16; developer: 「html で俯瞰したい」): after the commit, build the page from the committed roadmap with `pwsh -NoProfile -File <worktree>\.claude\skills\kiro-next\overview\roadmap-overview.ps1 -OutFile <scratch>\roadmap-overview-latest.html` (expect `OK wrote …`; `WARN` lines are roadmap mismatches — fix the roadmap and amend, or mention them in the report; `FAIL` means no page — report it and go on). If you touched anything under `overview/`, run the same script with `-SelfTest` first and require `selftest: all passed`. The file stays in the scratch directory and is never committed. Read the whole generated file before publishing (the Artifact rule for files Claude did not write; the data line is long, so read it in parts), then publish it with the `Artifact` tool as a new private artifact: `file_path` = that file, `title` = `roadmap-overview-latest`, `icon` = `map`, `description` = `棚卸（<date>・main <BASE>）の時点のロードマップの俯瞰`. Keep the returned URL for the report. If publishing is refused or fails, say so in the report and go on.

## Step 9 — Memory
- Write an inventory memory in the project memory directory (base SHA, counts, wave, splits, new briefs, pending items, branch/commit) and add its one-line pointer to `MEMORY.md`; update an existing inventory memory instead of duplicating. Record any new standing rule the developer gave during the run.

## Step 10 — Report (persona language, conclusions only)
1. Immediate fixes done (and what was handed to specs).
2. Briefing: re-measured count, splits, new briefs, resulting brief count.
3. Dependency-tree summary: the current goal, its largest heads and why they were or were not taken; beyond-goal heads in one line.
4. The wave, exactly in this form (one code block, Fable first; no per-spec notes inside). Start the report section with which specs need Fable for requirements, each with the specific decision that meets the Step 3 (6) rule ("なし" if none — the usual outcome — with the next-closest candidate and why it stays Opus):
```
Wave <k>本
Fable
/kiro-start <name>

Opus
/kiro-start <name>
```
   Follow the project preference of one command per block when the developer wants to paste them one by one; the combined block above is the default summary.
5. Decisions needed from the developer (none of them block the wave) and any manual steps — including "ゴール未設定" and any beyond-goal spec the roadmap or the developer wanted to run early (it needs the milestone scope changed first).
6. The overview page URL from Step 8, as one Markdown link whose text carries the inventory date and base SHA: `[ロードマップの俯瞰（棚卸 <date>・main <BASE>）](<url>)`. Never a bare URL or a code block (developer 2026-10-17: 「URLリンクにすればよい」).

Write this whole report as ordinary text **before** stopping at the approval gate. The gate never replaces the report (areka developer 2026-10-16: 「ウェーブを教えてもらってないのに？」 — the wave was missing when only the approval box was shown).

## Step 11 — Approval gate (STOP here)
- After the report, STOP. Do not push, open a PR or merge.
- Only when the developer explicitly instructs it (e.g. 「PRしてスクワッシュマージ」) push the branch, open the PR with `gh pr create` (body ends with the required attribution line; avoid double backslashes in `gh` arguments), then show the PR number, title and changed-file summary and ask for confirmation once more before `gh pr merge --squash --delete-branch`. Do not wait for CI on a Markdown-only inventory. After merging, report the main commit. If the merge is refused, report it; do not route around the refusal with another tool.
- If the run was a dry run or the developer says not to merge the inventory, move the commit to a local backup branch (`git branch <name> <sha>`) instead of leaving it on the branch that will be merged.

</instructions>

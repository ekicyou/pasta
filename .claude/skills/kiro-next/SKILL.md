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
4. Build a strict dependency tree, rank its head specs, and pick at most 8 as the next wave.
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

## Step 1 — Sync with remote main
1. Call `mcp__ccd_host__sync_with_base_branch` (base = default branch).
2. If it refuses (e.g. credentials in the origin URL), do it yourself: `git fetch origin <default>` (masked output) → `git merge --ff-only FETCH_HEAD`; if the branch has its own commits, `git merge origin/<default>`. Resolve conflicts yourself; never force.
3. Verify: `git ls-remote origin refs/heads/<default>` equals the merge base you expect (no more than the local branch's own commits ahead). Record the base SHA (`<BASE>`) for every section heading below.

## Step 2 — Lightweight scan
- `.kiro/steering/roadmap.md` is the single ledger (~30 KB). Read it whole: 概要 / 運用ルール / 前提として確定している方針 / 完了フェーズ（要約）/ 常駐 spec / 棚卸 section / each active Phase section (境界戦略・ウェーブ構成) / `## Specs (dependency order)` / バックログ.
- Inventory: `ls .kiro/specs/` (non-completed folders — each should hold `brief.md`; note any with `spec.json`), `ls .kiro/specs/completed | wc -l`. Cross-check that every unchecked `- [ ]` ledger row has a folder and vice versa.
- **In-flight specs**: `git worktree list` and `git branch -r --sort=-committerdate`. For each in-flight spec branch (other worktrees, recent `origin/claude/*` branches not yet merged): `git diff --name-only <default>...<branch> | grep -v '^\.kiro/'` → save to `<scratch>/inflight-files.md`. Also read each in-flight spec's brief/design for files it still plans to touch. All of these are **reserved**. A spec whose branch is already merged is not in flight.
- Determine the inventory date (today) and whether a previous "棚卸（YYYY-MM-DD）" section exists to supersede.

## Step 3 — Parallel re-measurement (subagents)
1. Write `<scratch>/remeasure-common.md` (template below).
2. Partition every unfinished, not-in-flight spec into groups of ~4–6 following the Phase sections of the roadmap (e.g. Phase 11 bug lane, Phase 11 feature lane, Phase 12 manual/guide, release-ci and other cross-cutting). If there are ≤ 6 such specs, use one group. Every spec in exactly one group — check the count adds up.
3. Launch all groups plus one "fixes & brief-less items" agent **in one message, in the background** (`Agent`, `model: "opus"`). Each group prompt = "read the common file and follow it" + its spec list + 3–5 lines of focus (which landed specs changed its premises, which in-flight branches touch its files, near-cap files, suspected splits).
4. As each report arrives, append a compact summary to `<scratch>/results.md` (name | kind | size | model | head? | blocking overlaps | split). Do not keep raw reports in context.
5. If agents die (e.g. HTTP 429) and the developer says resume: run `git status` and `grep -l '棚卸の再測定' .kiro/specs/*/brief.md` to detect partial writes, then relaunch the same prompts.

**Common instruction template** (fill `<…>`):
```
# Common instructions — roadmap inventory re-measurement
Worktree: <path> (run everything here). Base: <default> <BASE> (<date>). Previous inventory: <date of the previous 棚卸 section>. Specs landed since then: <list> (see .kiro/specs/completed/<name>/).
In-flight specs (NOT candidates; their files are reserved): <list with one-line notes>. Changed files: <scratch>/inflight-files.md. Also read their briefs for planned files.
Read first: .kiro/steering/roadmap.md (運用ルール, boundary strategy and wave tables of the active Phases); each assigned brief (and requirements/design/tasks if present).
For EACH spec: (1) verify the brief against current main with Grep/Glob/Read only — no cargo, no heavy commands; note stale claims and what landed specs already did. (2) touched source files on main (mark new files; note files near 1,000 lines with wc -l). (3) size in tasks (cap 20); if >20 propose a concrete split (names, boundary, files per half) — never re-split a spec already carved once just for barely crossing 20. (4) dependencies: functional (unfinished only) and file overlap with other unfinished specs including in-flight ones (any shared source file blocks the same wave; same-page different-section manual edits are allowed). (5) kind: バグ / 機能 / 文書 / 基盤, with one-phrase evidence (bug = wrong current behaviour; 機能 = new capability; also mark "developer asked for it" when the brief cites the developer's request as origin). (6) requirements model: Fable if canon ambiguity, cross-engine architecture, developer-decision forks, timing/concurrency, or new syntax/semantics that later specs build on; else Opus. (7) immediate-fix candidates (stale comments, broken paths, doc errors) with file:line — do NOT fix code. (8) APPEND to the end of each brief.md a section "## <date> 棚卸の再測定（main <BASE>）" in Japanese plain words, ≤25 lines: 前提の変化 / 触るファイル / 規模 / 先に要るもの / 種別 / 要件定義のモデル / 分割の案 / 見つけた穴・古くなった記述. Preserve line endings. Edit no other file. Do not commit.
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
**Single-seat resources** (one spec per wave): `Cargo.lock` / `tech.md` dependency additions; `.github/workflows/release.yml` and `release.ps1` (the release-ci seat); the release-workflow standing spec during any release.
**Excluded from the tree**: 却下・保留・据え置きの spec and `常駐 spec` (neither candidates nor counted as descendants). **Not parallelised**: specs whose core work is measurement (latency/perf/load).
**Procedure**:
1. Heads = specs with no unfinished predecessor (in-flight specs count as unfinished predecessors).
2. Descendant count = number of unfinished specs transitively depending on the head (approximate is fine; state "約").
3. Sort heads by descendant count desc → kind (developer-requested / バグ before 機能 / 文書) → filing order (higher in the ledger).
4. Greedily take heads from the top whose touched files do not overlap any already-taken head; stop at 8.
5. For every chosen spec write its touched files and the **same-wave promises** (files it must not touch). List the heads not chosen with one-line reasons, and outline the following wave's candidates.
6. Model per chosen spec from the re-measurement (Fable / Opus).

## Step 7 — Write the roadmap
Edit `.kiro/steering/roadmap.md` (Edit tool or one Python script in the scratch directory; assert every anchor exists):
- **Fold** finished specs: every `[x]` ledger row and every completed Phase section collapses into `完了フェーズ（要約）` (one row per Phase: 主題 and spec names). Old wave tables and superseded 棚卸 sections are deleted (git history keeps them; nothing is archived to a separate file).
- Replace the previous "棚卸（YYYY-MM-DD）" section with the new one: 即時修正 (what was fixed and closed, what was handed to specs), 統合・分割・改名, promotions and what stayed in the backlog (with reason), the wave and why, the Fable/Opus split, broken same-wave promises, holes found, 人手の確認が残っている項目, decisions needed.
- Active Phase sections: updated `境界戦略` and `ウェーブ構成` (rules, the new wave row with touched files and promises, next-wave candidates, 保留).
- `## Specs (dependency order)`: unfinished specs only, in dependency order, `- [ ] <name> -- <summary>. Dependencies: <list|none>`; split notes on the original rows, new rows inserted in dependency position.
- `バックログ` minus promoted items. Update the header `概要` if the current theme changed.
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

## Step 9 — Memory
- Write an inventory memory in the project memory directory (base SHA, counts, wave, splits, new briefs, pending items, branch/commit) and add its one-line pointer to `MEMORY.md`; update an existing inventory memory instead of duplicating. Record any new standing rule the developer gave during the run.

## Step 10 — Report (persona language, conclusions only)
1. Immediate fixes done (and what was handed to specs).
2. Briefing: re-measured count, splits, new briefs, resulting brief count.
3. Dependency-tree summary: largest heads and why they were or were not taken.
4. The wave, exactly in this form (one code block, Fable first; no per-spec notes inside). Start the report section with which specs need Fable for requirements ("なし" if none, with the next-closest candidate):
```
Wave <k>本
Fable
/kiro-start <name>

Opus
/kiro-start <name>
```
   Follow the project preference of one command per block when the developer wants to paste them one by one; the combined block above is the default summary.
5. Decisions needed from the developer (none of them block the wave) and any manual steps.

## Step 11 — Approval gate (STOP here)
- After the report, STOP. Do not push, open a PR or merge.
- Only when the developer explicitly instructs it (e.g. 「PRしてスクワッシュマージ」) push the branch, open the PR with `gh pr create` (body ends with the required attribution line; avoid double backslashes in `gh` arguments), then show the PR number, title and changed-file summary and ask for confirmation once more before `gh pr merge --squash --delete-branch`. Do not wait for CI on a Markdown-only inventory. After merging, report the main commit. If the merge is refused, report it; do not route around the refusal with another tool.
- If the run was a dry run or the developer says not to merge the inventory, move the commit to a local backup branch (`git branch <name> <sha>`) instead of leaving it on the branch that will be merged.

</instructions>

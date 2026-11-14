---
name: kiro-start
description: 'Post-discovery single-spec entry point. Precondition: /kiro-discovery has already created {specs-root}/{feature-name}/brief.md, so the feature name is already confirmed. Acts as a lightweight orchestrator that drives a confirmed spec from init through requirements, gap analysis, and an interactive requirements discussion on the harness worktree branch. The controller handles deterministic resolution, the remote default-branch sync, the precondition gate, the default-branch guard, gap-doc path resolution, the chat-window discussion, and commits; it delegates spec init + requirements generation and the gap analysis to subagents. No questions are asked before the discussion phase: the subagents draft best-effort with labelled assumptions and return OPEN QUESTIONS, which become discussion topics. Portable across repos: deterministically resolves the specs root, skill base directory, git remote, and default branch (fixed priority orders, no guessing). Does NOT create branches or worktrees — feature branches are supplied by the Claude Code harness worktree. When the current branch is the repository default branch it STOPs and asks the developer to re-run inside the harness worktree (a non-default working branch); otherwise it initializes the spec via kiro-spec-init (which consumes brief.md), generates requirements (kiro-spec-requirements), runs the gap analysis (kiro-validate-gap), then conducts the requirements discussion (kiro-requirements-discussion) interactively in the chat window, committing each phase to the current branch without pushing. Use when: 要件ディスカバリ後に（ハーネスのワークツリー上で）仕様を開始したい, kiro-start, spec開始, start a confirmed spec on the harness worktree branch. DO NOT USE FOR: 複数仕様の一括生成 (use /kiro-spec-batch), 既存 spec の追加要件生成のみ (use /kiro-spec-requirements directly).'
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Agent, WebSearch, WebFetch
argument-hint: <feature-name>
---

# Spec Start (Init → Requirements → Gap Analysis → Requirements Discussion, post-discovery)

<instructions>
## Core Task
Start a single specification end-to-end **after `/kiro-discovery`**. Discovery has already created `{specs-root}/{feature-name}/brief.md`, so the **feature name is already confirmed and equals `$ARGUMENTS`**. This skill does **not** create branches or worktrees: feature branches are supplied by the Claude Code (harness) worktree feature. Before anything else it brings in the latest remote default branch (Step 0.5). When the current branch is the repository's default branch, **STOP** and ask the developer to re-run inside the harness worktree (a non-default working branch). Otherwise (already on a non-default branch), drive the spec through three phases on the current branch, committing each without pushing:
1. **Init + Requirements** — `kiro-spec-init` (consumes brief.md) then `kiro-spec-requirements` (delegated to a subagent; best-effort draft, never blocks on user input).
2. **Gap analysis** — `kiro-validate-gap` (delegated to a subagent): analyzes the requirements against the existing codebase and writes a gap-analysis document.
3. **Requirements discussion** — `kiro-requirements-discussion` (run **interactively in the controller / chat window**): ingests the requirements and gap-analysis documents plus the subagent's OPEN QUESTIONS, collects/classifies issues, explains the spec's purpose with concrete examples, presents all developer-facing topics at once with recommendations, and settles them in the chat window (bulk approval, or individual discussion of only the topics the developer names).

For multi-spec generation, use `/kiro-spec-batch`. This skill stops **before** design (`/kiro-design`).

This skill acts as a **lightweight orchestrator**. The controller (main context) performs deterministic, state-sensitive orchestration that cannot be safely delegated — portable-context resolution, the remote sync, the precondition gate, the default-branch guard, gap-document path resolution, the interactive requirements discussion, and commits. It never creates, deletes, resets, or pushes branches. The two non-interactive heavy phases (init + requirements; gap analysis) are **delegated to subagents** via the Agent tool so the controller context stays small. Subagents never interact with the user; any genuine clarification is recorded as an OPEN QUESTION and **carried forward to the discussion phase** — it is never asked before then.

**The requirements discussion is the one heavy phase that is NOT delegated.** It is inherently interactive — it requires developer dialogue (bulk approval or per-topic discussion) — and the developer's Q&A MUST happen in the chat window. A subagent cannot talk to the user, so the controller runs this phase inline by reading the `kiro-requirements-discussion` SKILL.md and following it directly (see Step 7).

This skill is designed to be **portable across repositories**. It does not hard-code skill paths, the remote name, or the default branch; instead it resolves each one with a **deterministic, ordered detection procedure** (Step 0). Each resolution yields exactly one result or a hard failure — never an ambiguous guess.

## Communication Language
- **Think in English, report in the user's language.** Internal reasoning, planning, and tool orchestration may be in English, but every message surfaced to the developer MUST be written in the target language configured for this spec.
- **Resolve the report language** from `{specs-root}/{feature-name}/spec.json` (`language` field). If `spec.json` does not exist yet (before Step 3), fall back to the language of the user's input (default `ja` for this repository). Use this same language for the discussion phase and the final Output Description.
- This applies to ALL developer-facing text emitted by the controller: orchestration progress narration ("dispatching subagent", "verifying outputs", "starting discussion"), warnings, **and the entire interactive requirements discussion in Step 7**.
- The Step 3 and Step 6 **subagent prompts themselves stay in English** (they are internal instructions, not user-facing). Translate only the controller's own narration and the Step 7 discussion into the spec's target language.
- **No questions before the discussion phase.** The controller does NOT ask the developer any question during Steps 0–6. The first (and only) interactive phase is Step 7 (requirements discussion), conducted in the chat window; never use `AskUserQuestion`.
- `$ARGUMENTS` is the **confirmed kiro feature name** produced by `/kiro-discovery` (it matches the existing directory `{specs-root}/{feature-name}/`, which already contains `brief.md`). Pass it **verbatim** as the first parameter of `kiro-spec-init`, `kiro-spec-requirements`, `kiro-validate-gap`, and `kiro-requirements-discussion`. Do NOT ask clarifying questions or re-derive a name in this wrapper.
- This skill does not derive, create, or switch branches. It operates on whatever branch the harness worktree provides; the only branch-related decision is the default-branch guard in Step 2 (STOP vs. proceed).

## Execution Steps

### Step 0: Resolve portable context (deterministic)
Resolve these values once, in order. Each has a single deterministic outcome; if a required value cannot be resolved, fail as specified.

1. **Specs root** (`{specs-root}`): Use the first existing directory in this fixed priority order:
   1. `.kiro/specs`
   If `.kiro/specs` does not exist, STOP (hard fail): this repository is not cc-sdd initialized.

2. **Skill base directory** (`{skill-base}`): Locate the directory that contains `kiro-spec-init/SKILL.md`, checking this fixed priority order and taking the FIRST match:
   1. `.claude/skills`
   2. `.agents/skills`
   3. `.github/skills`
   Resolve `kiro-spec-requirements/SKILL.md`, `kiro-validate-gap/SKILL.md`, and `kiro-requirements-discussion/SKILL.md` under the **same** `{skill-base}`. If any of these four sibling skills is not found under the chosen base, STOP (hard fail): the required kiro skills are not installed.

3. **Default remote** (`{remote}`): Run `git remote`. Apply this fixed rule:
   - If `origin` is present → `{remote}` = `origin`.
   - Else if exactly one remote exists → `{remote}` = that remote.
   - Else (no remotes, or multiple without `origin`) → `{remote}` = none; treat all remote operations as skipped (warn once).

4. **Default branch** (`{default-branch}`): Determine deterministically:
   - If `{remote}` is set, read `git symbolic-ref --quiet --short refs/remotes/{remote}/HEAD` and strip the `"{remote}/"` prefix.
   - If that yields nothing and a local `main` branch exists → `{default-branch}` = `main`.
   - Else if a local `master` branch exists → `{default-branch}` = `master`.
   - Else → `{default-branch}` = the current branch (in this degenerate case current == default, so the Step 2 guard would STOP; this is acceptable since a non-default harness worktree branch is expected).

   Record `{default-branch}` as a single concrete name before proceeding. Do not re-evaluate it later.

### Step 0.5: Bring in the remote default branch (prevents stale-base accidents)
Run this before the precondition gate: the discovery `brief.md` may exist only on the remote default branch (discovery lands through a PR), and the spec must start from the latest base.
1. If `{remote}` is none → warn once and skip this step.
2. `git fetch {remote} {default-branch}`. If the fetch fails, STOP and report the error (do not continue on a possibly stale base).
3. If the current branch (`git branch --show-current`) equals `{default-branch}`: `git merge --ff-only {remote}/{default-branch}` — never a merge commit on the default branch. If it cannot fast-forward, STOP and report (Step 2 would STOP on the default branch anyway).
4. On any other branch: `git merge --no-edit {remote}/{default-branch}`. If it conflicts, resolve the conflicts and complete the merge; never force, reset, or discard local changes. If the merge cannot start (e.g. uncommitted changes in the way) or a conflict cannot be resolved with confidence, run `git merge --abort` when a merge is in progress, then STOP and report the cause and the files involved.
5. Confirm the merge really took (areka developer 2026-10-23: `/kiro-wave` squash-merges the inventory just before it starts this session, and the spec must be read only after that merge is in): `git merge-base --is-ancestor {remote}/{default-branch} HEAD` must exit 0. Otherwise STOP and report it; nothing about the spec has been read yet.
6. Report the result in one line: how many commits came in (or `Already up to date`), with the `{remote}/{default-branch}` short SHA.

### Step 1: Verify post-discovery precondition (hard gate)
This skill is deterministic about its precondition: the spec folder created by `/kiro-discovery` MUST already exist.
1. Treat `$ARGUMENTS` as the confirmed feature name. Verify the spec folder and discovery brief exist:
   ```powershell
   Test-Path "{specs-root}/{feature-name}"
   Test-Path "{specs-root}/{feature-name}/brief.md"
   ```
2. **If the spec folder `{specs-root}/{feature-name}/` does not exist: STOP.** Do NOT create the folder, do NOT create a branch, do NOT run init/requirements. Report the failure: the feature name was not found, `/kiro-start` runs only after `/kiro-discovery`, and suggest running `/kiro-discovery "<idea>"` first (or check the feature name spelling against existing folders under `{specs-root}/`).
3. **If the folder exists but `brief.md` is missing:** the discovery brief is incomplete. STOP and report that `brief.md` is missing; recommend re-running `/kiro-discovery` for this feature. Do not fabricate a brief.
4. Only when both exist, proceed.

### Step 2: Default-branch guard (no branch creation)
This skill never creates branches or worktrees — the feature branch is supplied by the Claude Code (harness) worktree. This step is purely a guard: it either STOPs (on the default branch) or lets the flow proceed on the current non-default branch.

1. Determine the current branch:
   ```powershell
   $branch = git branch --show-current
   ```
2. **If `$branch` equals `{default-branch}` (resolved in Step 0): STOP.**
   - Do NOT create a branch, do NOT run any phase, do NOT commit, do NOT push.
   - Report that `/kiro-start` does not run on the default branch (`{default-branch}`): spec work must happen on a non-default working branch. Ask the developer to re-run `/kiro-start {feature-name}` inside the Claude Code harness worktree (a non-default working branch). The discovery artifacts (`brief.md`) remain intact for the re-run.
3. **If `$branch` does NOT equal `{default-branch}`:**
   - Proceed directly to the spec initialization phase (Step 3) on the current branch. Do not create or switch branches, do not push (the remote default branch was already merged in Step 0.5).

### Step 3: Delegate spec init + requirements to a subagent (orchestration)
Dispatch **one** non-interactive subagent via the Agent tool to perform the init → requirements phase so the controller context stays lightweight. Pass the resolved values from Step 0 (`{specs-root}`, `{skill-base}`, `{feature-name}` = `$ARGUMENTS`) into the prompt.

Use this subagent prompt:
```
You are completing the init + requirements phase for the confirmed kiro feature "{feature-name}".
The feature name is FINAL — do not re-derive or change it. brief.md already exists.

1. Read {specs-root}/{feature-name}/brief.md for confirmed problem, approach, scope, and boundary candidates.
2. Initialize the spec: read {skill-base}/kiro-spec-init/SKILL.md and follow it, passing "{feature-name}" verbatim.
   It reuses the existing {specs-root}/{feature-name}/ directory and writes spec.json and requirements.md from templates.
3. Generate requirements: read {skill-base}/kiro-spec-requirements/SKILL.md and follow every step
   (context load, EARS rules, any parallel research subagents it directs, draft, and the automated review gate).
   Write {specs-root}/{feature-name}/requirements.md and update spec.json metadata only after the review gate passes.
4. Ground every decision in brief.md. DO NOT ask the user anything — you cannot interact with the user, and ALL
   clarification is deferred to the interactive requirements-discussion phase that the controller runs later.
   When a scope ambiguity, contradiction, or open decision remains that brief.md cannot resolve, DO NOT block and
   DO NOT guess silently: draft the requirement best-effort with an explicit, clearly-labelled assumption, and record
   the item as an OPEN QUESTION for the discussion phase. ALWAYS produce a complete best-effort draft.

Return a structured report:
- STATUS: DRAFTED (always produce a best-effort draft; never block on user input)
- Created/updated files (full paths)
- Requirements summary (3-5 bullets) and review-gate result
- OPEN QUESTIONS: numbered list of items to resolve in the requirements-discussion phase, each with the assumption drafted for it (empty if none)
```

### Step 4: Verify requirements (orchestration, no questions)
Do NOT ask the developer anything here — open questions are deferred to Step 7.
1. Verify the outputs in the controller:
   ```powershell
   Test-Path "{specs-root}/{feature-name}/spec.json"
   Test-Path "{specs-root}/{feature-name}/requirements.md"
   ```
   Confirm `spec.json` has `phase: "requirements-generated"` and `approvals.requirements.generated: true`.
2. If a required file is missing or metadata was not updated, report the failure (do not commit); suggest re-running `/kiro-start {feature-name}`.
3. Retain the subagent's **OPEN QUESTIONS** list (if any) in the controller context — these are fed into Step 7 as discussion topics.

### Step 5: Commit the requirements baseline to the current branch (no push)
The normal path always reaches this step on a **non-default** branch (the default-branch case STOPped in Step 2). Commit the generated requirements baseline to the **current branch**. Do **not** push.
```powershell
git add -A
git commit -m "chore({feature-name}): initialize spec (spec.json, requirements.md)"
```
- This is an intentional checkpoint that locks the requirements baseline before gap analysis and discussion modify it.
- Never push here. Remote sync is the responsibility of `kiro-complete` (PR-based), not `kiro-start`.
- If the commit fails (e.g., nothing to commit, hook failure), report the error; the generated files remain staged/present on the current branch.

### Step 6: Delegate gap analysis to a subagent (orchestration)
Gap analysis is non-interactive heavy work, so it is delegated. Dispatch **one** subagent via the Agent tool to run `kiro-validate-gap` against the requirements. Pass the resolved Step 0 values into the prompt.

Use this subagent prompt:
```
You are completing the gap-analysis (validate-gap) phase for the confirmed kiro feature "{feature-name}".
requirements.md and spec.json are already drafted. DO NOT modify requirements.md or spec.json.

1. Read {skill-base}/kiro-validate-gap/SKILL.md and follow every step it specifies.
2. Inputs: {specs-root}/{feature-name}/spec.json (language/metadata), {specs-root}/{feature-name}/requirements.md,
   {specs-root}/{feature-name}/brief.md, and the steering files under .kiro/steering/.
3. Analyze the gap between the requirements and the EXISTING codebase using Grep/Glob/Read; research
   external dependencies only when needed (WebSearch/WebFetch). Evaluate multiple implementation approaches.
4. Write the gap-analysis document to disk exactly where the kiro-validate-gap skill directs, then READ IT BACK to
   verify it was written. Report the exact full path you wrote.
5. DO NOT ask the user anything — you cannot interact with the user. Provide analysis and options, not final decisions.

Return a structured report:
- STATUS: DONE | FAILED
- Gap-analysis document path (full path) and confirm it was written and read back
- Analysis summary (3-5 bullets): existing patterns, missing capabilities, candidate approaches, research flags
- Design-decision items surfaced (numbered; these feed the requirements discussion)
```

After the subagent returns:
1. **If `STATUS: FAILED`** or no structured report: report the failure. The requirements baseline (Step 5) is already committed and intact; suggest re-running `/kiro-start {feature-name}` (it will resume from gap analysis since requirements are already drafted) or running `/kiro-validate-gap {feature-name}` standalone. Do NOT proceed to discussion.
2. **If `STATUS: DONE`**, resolve the **gap-document path** (`{gap-doc}`) deterministically — take the FIRST existing file in this fixed order inside `{specs-root}/{feature-name}/`:
   1. the exact path the subagent reported writing
   2. `research.md`
   3. `gap-analysis.md`
   ```powershell
   Test-Path "{specs-root}/{feature-name}/research.md"
   Test-Path "{specs-root}/{feature-name}/gap-analysis.md"
   ```
   This bridges any naming difference between what `kiro-validate-gap` writes and what `kiro-requirements-discussion` expects to read. If none of these exists, treat it as `STATUS: FAILED` (handle as in 6.1).
3. Commit the gap-analysis document (no push):
   ```powershell
   git add -A
   git commit -m "docs({feature-name}): add gap analysis"
   ```
   If nothing changed (e.g., the subagent appended to an existing file with no net change), skip the commit and note it.

### Step 7: Requirements discussion — interactive, in the chat window (controller)
This phase is **NOT delegated**: it requires turn-by-turn developer dialogue, and all Q&A happens in the **chat window** (natural conversational turns), NOT via `AskUserQuestion` popups. The controller runs it inline.

1. **Ingest the upstream documents** (the requirements and gap-analysis outputs):
   - Read `{specs-root}/{feature-name}/requirements.md` in full.
   - Read the resolved `{gap-doc}` from Step 6 in full.
   - Read `{specs-root}/{feature-name}/spec.json` (confirm the discussion language) and the steering files under `.kiro/steering/`.
2. **Follow the discussion skill inline:** read `{skill-base}/kiro-requirements-discussion/SKILL.md` and execute its workflow (Phase 1–5) directly in the controller. When that skill references `gap-analysis.md`, substitute the resolved `{gap-doc}` path from Step 6 (this is the "取り込み" of the validate-gap output). When it references `requirements.md`, use the file ingested above.
3. **Seed the discussion with the OPEN QUESTIONS** retained from Step 4 — add them as Category C (developer-confirmation) topics, each recommending the assumption the subagent drafted (or a better one), so every assumption is explicitly confirmed or revised.
4. **Conduct the developer dialogue in the chat window:**
   - Category A (obvious fixes) and Category B (design decisions deferred) are handled exactly as the discussion skill directs (Phase 2), committing per its convention (`docs({feature-name}): ...`).
   - Category C (developer-facing questions) follows the skill's batch-proposal flow (Phase 3–4): first explain the spec's purpose with a concrete example or code, then write **all** topics at once in the chat, each with a recommendation and its rationale. Ask for the first bulk approval the way its Phase 3 says: through the desktop app's next-step box (`mcp__ccd_turn__wrap_up` with one `ask` and the two reply buttons 「推奨どおり承認」「一部を議論」) when that tool exists, with the full agenda in the closing message after that call; otherwise end the message with the same one question. The developer may click a button or answer in chat. If the developer approves in bulk, settle every topic on its recommendation in one commit. If the developer names specific topics to discuss, settle the unnamed topics on their recommendations (one commit), then discuss only the named topics one by one in the chat, updating the documents, committing per the skill's convention, and showing the progress summary after each. Never use `AskUserQuestion`.
   - If the developer stops responding or defers, summarize the remaining open topics and stop without guessing. Do not fabricate answers.
5. The discussion skill commits its own changes per category. The controller does not duplicate those commits.

### Step 8: Final verification and safety commit (no push)
1. Verify the spec state:
   ```powershell
   Test-Path "{specs-root}/{feature-name}/requirements.md"
   Test-Path "{specs-root}/{feature-name}/{gap-doc basename}"
   git status --porcelain
   ```
2. If `git status` shows any uncommitted spec changes left over from the discussion, commit them as a safety net (no push):
   ```powershell
   git add -A
   git commit -m "docs({feature-name}): finalize requirements discussion"
   ```
   If the tree is already clean, skip this commit.
3. Never push. Remote sync is the responsibility of `kiro-complete` (PR-based), not `kiro-start`.

## Important Constraints
- **Lightweight orchestration with one interactive exception**: The controller (main context) runs Step 0 (resolution), Step 0.5 (merging the remote default branch), Step 1 (precondition gate), Step 2 (default-branch guard), Step 4 (verification), Step 5/6/8 (commits + gap-doc resolution), and Step 7 (the interactive requirements discussion). Init + requirements (Step 3) and gap analysis (Step 6) are delegated to subagents. The requirements discussion (Step 7) is intentionally NOT delegated because it needs chat-window developer dialogue a subagent cannot perform. Do NOT run kiro-spec-init, kiro-spec-requirements, or kiro-validate-gap inline in the controller.
- **No questions before discussion**: The controller asks the developer NOTHING during Steps 0–6. The subagents never ask the user; their open questions are carried forward and resolved only in Step 7.
- **Discussion Q&A in the chat window**: All developer-facing topics in Step 7 are written at once in the chat, each with a recommendation; the first bulk approval is asked through the next-step box when it exists (otherwise one question at the end of the message); bulk approval settles them all, otherwise only the topics the developer names are discussed one by one. Never use `AskUserQuestion`.
- **No branch/worktree creation, ever**: This skill does not create, switch, delete, reset, or force-update branches, and does not create worktrees. Feature branches are supplied by the Claude Code (harness) worktree feature.
- **Subagents never interact with the user**: The Step 3 and Step 6 subagents must not ask questions. The Step 3 subagent always returns a best-effort DRAFTED report; genuine ambiguities are recorded as OPEN QUESTIONS for the discussion phase.
- **Report in the user's language**: Think in English internally, but write every developer-facing message (progress narration, warnings, and the entire Step 7 discussion) in the spec's target language. Keep the internal subagent prompts (Step 3, Step 6) in English.
- **Deterministic resolution**: Step 0 resolves specs root, skill base, remote, and default branch with fixed priority orders; Step 6 resolves the gap-document path with a fixed priority order. Never guess; if a required value is unresolved, hard-fail as specified. Resolve each value once and reuse it.
- This skill is **post-discovery only**: if `{specs-root}/{feature-name}/` does not exist, FAIL deterministically (do not create anything).
- Do NOT generate design or tasks. This skill stops after the requirements discussion.
- Do NOT re-derive or change the feature name; it is fixed by discovery (`$ARGUMENTS`).
- **STOP on the default branch**: If the current branch equals the resolved default branch, STOP in Step 2 (no phase, no commit) and ask the developer to re-run inside the harness worktree. Never push.
- Do NOT use this skill for `/kiro-spec-batch` (multi-spec) flows.
</instructions>

## Output Description
Provide output in the language specified in `spec.json` with the following structure:

1. **Feature Name**: `feature-name` (confirmed by discovery; equals the argument)
2. **Project Summary**: Brief summary (1 sentence, sourced from brief.md)
3. **Created/Updated Files**: Bullet list with full paths (`spec.json`, `requirements.md`, the gap-analysis document `{gap-doc}`)
4. **Branch Status**: On the normal path, report the current branch and that each phase was committed to it without pushing, e.g. `Committed spec to current branch "{branch}" (no push)`. (No branch was created — feature branches come from the harness worktree.) If the run STOPped because the current branch is the default branch, report only the STOP instead (see Safety & Fallback).
5. **Phase Status**:
   - Requirements: confirm `requirements.md` was generated and the subagent's automated review gate passed.
   - Gap analysis: confirm the gap-analysis document was written (give its path).
   - Discussion: summarize the discussion outcome — counts for Category A (obvious fixes), Category B (design decisions deferred), and Category C (developer-resolved), or note "no issues found" / "stopped with open topics".
6. **Next Step**: Command block showing `/kiro-design <feature-name>` (gap analysis and the requirements discussion are already complete), followed by the discussion's design-model recommendation (**Opus** by default, **Fable** only when a strict criterion is named, with its 1–2 line reason; criteria in kiro-requirements-discussion Phase 5).

**Format Requirements**:
- Use Markdown headings (##, ###)
- Wrap commands in code blocks
- Keep total output concise (under 320 words)
- Use clear, professional language per `spec.json.language`

## Safety & Fallback
- **Specs Root Unresolved (hard fail)**: If `.kiro/specs` does not exist, STOP and report that the repository is not cc-sdd initialized.
- **Skills Unresolved (hard fail)**: If any of `kiro-spec-init`, `kiro-spec-requirements`, `kiro-validate-gap`, or `kiro-requirements-discussion` SKILL.md is not found under `.claude/skills`, `.agents/skills`, or `.github/skills` (in that order), STOP and report that the required kiro skills are not installed.
- **Missing Spec Folder (hard fail)**: If `{specs-root}/{feature-name}/` does not exist, STOP immediately and report the error. Do not create the folder, branch, or any spec files. Suggest running `/kiro-discovery "<idea>"` first, or verifying the feature name against existing folders under `{specs-root}/`.
- **Missing Brief (hard fail)**: If the folder exists but `brief.md` is absent, STOP and report that the discovery brief is incomplete; recommend re-running `/kiro-discovery` for this feature.
- **Init Delegation**: All init-level fallbacks (missing templates, write failure) are handled by `kiro-spec-init` inside the Step 3 subagent. Honor its results as reported by the subagent.
- **Requirements Delegation**: All requirements-level behavior (steering load, EARS rules, automated review gate) is handled by `kiro-spec-requirements` inside the Step 3 subagent. Honor its results as reported by the subagent.
- **Subagent Failure (Step 3)**: If the subagent errors out or returns no structured report, do NOT commit. Report the failure and suggest re-running `/kiro-start {feature-name}` on the same branch. No branch was created, so there is nothing to clean up.
- **Clarifications Deferred (no questions before discussion)**: The Step 3 subagent never blocks on user input; it drafts best-effort with labelled assumptions and returns OPEN QUESTIONS. The controller does NOT ask these in Step 4 — it carries them into Step 7 and resolves them in the discussion.
- **Missing Outputs After DRAFTED**: If the subagent reports DRAFTED but `spec.json`/`requirements.md` are missing or metadata is not updated (Step 4 verification fails), report the failure and do not commit.
- **Gap Analysis Delegation**: All gap-analysis behavior (codebase research, multi-option evaluation, document write) is handled by `kiro-validate-gap` inside the Step 6 subagent. Honor its results as reported by the subagent.
- **Gap Analysis Failure (Step 6)**: If the Step 6 subagent returns `STATUS: FAILED`, errors out, or no gap-document path can be resolved, report the failure. The requirements baseline (Step 5) is committed and intact. Do NOT proceed to the discussion; suggest re-running `/kiro-start {feature-name}` (resumes from gap analysis) or `/kiro-validate-gap {feature-name}` standalone.
- **Discussion Skill Behavior (Step 7)**: All issue collection, classification (A/B/C), obvious-fix application, design-decision deferral, and per-category commits are handled by `kiro-requirements-discussion` followed inline. The controller substitutes the resolved `{gap-doc}` path for any `gap-analysis.md` reference and seeds Category C with the Step 3 OPEN QUESTIONS. All developer Q&A is conducted in the chat window (never `AskUserQuestion`).
- **Discussion: No Gap Doc**: If gap analysis somehow produced no readable document but the flow reached Step 7, warn once and run the discussion against `requirements.md` only (the discussion skill supports running without a gap document).
- **Discussion: Developer Unresponsive / Open Topics**: If the developer stops responding or defers Category C topics, summarize the remaining open topics and stop without guessing. Already-resolved topics and their commits remain on the branch; the rest can be resumed any time via `/kiro-requirements-discussion {feature-name}`.
- **On Default Branch (STOP)**: If the current branch equals `{default-branch}`, STOP in Step 2 before any phase/commit/push. Report that spec work does not run on the default branch and ask the developer to re-run `/kiro-start {feature-name}` inside the Claude Code harness worktree (a non-default working branch). Do not create a branch or modify any files; `brief.md` is preserved for the re-run.
- **Non-Default Branch (normal path)**: Proceed through all three phases on the current branch and commit each (no push). Remote sync is deferred to `kiro-complete`.
- **Remote Sync Failure (Step 0.5)**: If the fetch fails, the default branch cannot fast-forward, or a merge cannot be completed, abort any in-progress merge, STOP, and report. Nothing else has run; re-run `/kiro-start {feature-name}` after it is resolved.
- **No Remote**: If `{remote}` is none, warn once and skip Step 0.5; all phases and the local commits still proceed (kiro-start never pushes; its only remote operation is the Step 0.5 fetch).
- **Commit Failure**: Report the error with the current branch name; the generated files remain staged/present on the current branch.

---
name: kiro-start
description: 'Post-discovery single-spec entry point. Precondition: /kiro-discovery has already created {specs-root}/{feature-name}/brief.md, so the feature name is already confirmed. Acts as an orchestrator: the controller handles deterministic resolution, the precondition gate, the default-branch guard, the initial commit, and an interactive requirements discussion, while delegating spec init + requirements generation + gap analysis to a single non-interactive subagent. Portable across repos: deterministically resolves the specs root, skill base directory, git remote, and default branch (fixed priority orders, no guessing). Does NOT create branches or worktrees — feature branches are supplied by the Claude Code harness worktree. When the current branch is the repository default branch it STOPs and asks the developer to re-run inside the harness worktree (a non-default working branch); otherwise it initializes the spec via kiro-spec-init (which consumes brief.md), runs kiro-spec-requirements, runs kiro-validate-gap, commits the generated spec to the current branch without pushing, and then runs kiro-requirements-discussion interactively. No questions are asked before the discussion phase. Use when: 要件ディスカバリ後に（ハーネスのワークツリー上で）仕様を開始したい, kiro-start, spec開始, start a confirmed spec on the harness worktree branch. DO NOT USE FOR: 複数仕様の一括生成 (use /kiro-spec-batch), 既存 spec の追加要件生成のみ (use /kiro-spec-requirements directly).'
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Agent, WebSearch, WebFetch, AskUserQuestion
argument-hint: <feature-name>
---

# Spec Start (Init + Requirements + Gap + Discussion, post-discovery)

<instructions>
## Core Task
Start a single specification end-to-end **after `/kiro-discovery`**, taking it from init through requirements, gap analysis, and an interactive requirements discussion. Discovery has already created `{specs-root}/{feature-name}/brief.md`, so the **feature name is already confirmed and equals `$ARGUMENTS`**. This skill does **not** create branches or worktrees: feature branches are supplied by the Claude Code (harness) worktree feature. When the current branch is the repository's default branch, **STOP** and ask the developer to re-run inside the harness worktree (a non-default working branch). Otherwise (already on a non-default branch), it:
1. Initializes the spec via `kiro-spec-init` (consumes the existing brief.md, skips clarification),
2. Runs `kiro-spec-requirements` (best-effort draft; never blocks on user input),
3. Runs `kiro-validate-gap` (gap analysis written to `research.md`),
4. Commits the generated spec to the current branch **without pushing**, and
5. Runs `kiro-requirements-discussion` **interactively** to refine the requirements.

For multi-spec generation, use `/kiro-spec-batch`. This skill stops **before** design (`/kiro-spec-design`).

This skill acts as an **orchestrator**. The controller (main context) performs deterministic, state-sensitive orchestration that cannot be safely delegated — portable-context resolution, the precondition gate, the default-branch guard, the initial commit — **plus** the interactive requirements discussion, which is inherently conversational and therefore runs inline in the controller (a subagent cannot talk to the developer). All other heavy work (spec initialization, requirements research and review gate, and gap analysis) is **delegated to a single non-interactive subagent** via the Agent tool so the controller context stays small. That subagent never interacts with the user; any genuine clarification is recorded as an OPEN QUESTION and **carried forward to the discussion phase** — it is never asked before then.

This skill is designed to be **portable across repositories**. It does not hard-code skill paths, the remote name, or the default branch; instead it resolves each one with a **deterministic, ordered detection procedure** (Step 0). Each resolution yields exactly one result or a hard failure — never an ambiguous guess.

## Communication Language
- **Think in English, report in the user's language.** Internal reasoning, planning, and tool orchestration may be in English, but every message surfaced to the developer MUST be written in the target language configured for this spec.
- **Resolve the report language** from `{specs-root}/{feature-name}/spec.json` (`language` field). If `spec.json` does not exist yet (before Step 3), fall back to the language of the user's input (default `ja` for this repository). Use this same language for the discussion phase and the final Output Description.
- This applies to ALL developer-facing text emitted by the controller: orchestration progress narration ("dispatching subagent", "verifying outputs", "committing spec"), warnings, and the entire interactive discussion in Step 6.
- The Step 3 **subagent prompt itself stays in English** (it is internal instruction, not user-facing). Translate only the controller's own narration and the discussion you conduct with the developer.
- **No questions before the discussion phase.** The controller does NOT ask the developer any question during Steps 0–5. The first (and only) interactive phase is Step 6 (requirements discussion). Conduct Step 6 as **free-form conversational chat** (自由応答) — present one topic at a time and await the developer's natural-language reply; prefer plain conversational turns over structured `AskUserQuestion` widgets, reserving the latter only for a strictly constrained either/or choice.
- `$ARGUMENTS` is the **confirmed kiro feature name** produced by `/kiro-discovery` (it matches the existing directory `{specs-root}/{feature-name}/`, which already contains `brief.md`). Pass it **verbatim** as the first parameter of `kiro-spec-init`, `kiro-spec-requirements`, `kiro-validate-gap`, and `kiro-requirements-discussion`. Do NOT ask clarifying questions or re-derive a name in this wrapper.
- This skill does not derive, create, or switch branches. It operates on whatever branch the harness worktree provides; the only branch-related decision is the default-branch guard in Step 2 (STOP vs. proceed).

## Execution Steps

### Step 0: Resolve portable context (deterministic)
Resolve these four values once, in order. Each has a single deterministic outcome; if a required value cannot be resolved, fail as specified.

1. **Specs root** (`{specs-root}`): Use the first existing directory in this fixed priority order:
   1. `.kiro/specs`
   If `.kiro/specs` does not exist, STOP (hard fail): this repository is not cc-sdd initialized.

2. **Skill base directory** (`{skill-base}`): Locate the directory that contains `kiro-spec-init/SKILL.md`, checking this fixed priority order and taking the FIRST match:
   1. `.claude/skills`
   2. `.agents/skills`
   3. `.github/skills`
   Resolve `kiro-spec-requirements/SKILL.md`, `kiro-validate-gap/SKILL.md`, and `kiro-requirements-discussion/SKILL.md` under the **same** `{skill-base}`. If `kiro-spec-init` or `kiro-spec-requirements` is not found under any of the three bases, STOP (hard fail): the required kiro skills are not installed. If `kiro-validate-gap` or `kiro-requirements-discussion` is missing under the resolved base, warn once and skip the corresponding phase (gap analysis and/or discussion) rather than hard-failing the whole flow.

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
   - Do NOT create a branch, do NOT run init/requirements/gap/discussion, do NOT commit, do NOT push.
   - Report that `/kiro-start` does not run on the default branch (`{default-branch}`): spec initialization must happen on a non-default working branch. Ask the developer to re-run `/kiro-start {feature-name}` inside the Claude Code harness worktree (a non-default working branch). The discovery artifacts (`brief.md`) remain intact for the re-run.
3. **If `$branch` does NOT equal `{default-branch}`:**
   - Proceed directly to the spec initialization phase (Step 3) on the current branch. Do not create or switch branches, do not pull, do not push.

### Step 3: Delegate spec init + requirements + gap analysis to a subagent (orchestration)
Dispatch **one** non-interactive subagent via the Agent tool to perform the entire heavy phase (init → requirements → gap analysis) so the controller context stays lightweight. Pass the resolved values from Step 0 (`{specs-root}`, `{skill-base}`, `{feature-name}` = `$ARGUMENTS`) into the prompt. If `kiro-validate-gap` was reported missing in Step 0, drop step 4 from the prompt and note that gap analysis is skipped.

Use this subagent prompt:
```
You are completing the init + requirements + gap-analysis phase for the confirmed kiro feature "{feature-name}".
The feature name is FINAL — do not re-derive or change it. brief.md already exists.

1. Read {specs-root}/{feature-name}/brief.md for confirmed problem, approach, scope, and boundary candidates.
2. Initialize the spec: read {skill-base}/kiro-spec-init/SKILL.md and follow it, passing "{feature-name}" verbatim.
   It reuses the existing {specs-root}/{feature-name}/ directory and writes spec.json and requirements.md from templates.
3. Generate requirements: read {skill-base}/kiro-spec-requirements/SKILL.md and follow every step
   (context load, EARS rules, any parallel research subagents it directs, draft, and the automated review gate).
   Write {specs-root}/{feature-name}/requirements.md and update spec.json metadata only after the review gate passes.
4. Run gap analysis: read {skill-base}/kiro-validate-gap/SKILL.md and follow it, passing "{feature-name}" verbatim.
   Analyze the gap between the generated requirements and the existing codebase. The skill writes the gap analysis to
   {specs-root}/{feature-name}/research.md — that file IS the gap analysis artifact for the downstream discussion phase.
5. Ground every decision in brief.md. DO NOT ask the user anything — you cannot interact with the user, and ALL
   clarification is deferred to the interactive requirements-discussion phase that the controller runs next.
   When a scope ambiguity, contradiction, or open decision remains that brief.md cannot resolve, DO NOT block and
   DO NOT guess silently: draft the requirement best-effort with an explicit, clearly-labeled assumption, and record
   the item as an OPEN QUESTION for the discussion phase. ALWAYS produce a complete best-effort draft.

Return a structured report:
- STATUS: DRAFTED (always produce a best-effort draft; never block on user input)
- Created/updated files (full paths: spec.json, requirements.md, research.md)
- Requirements summary (3-5 bullets) and review-gate result
- Gap-analysis summary (3-5 bullets: key gaps, integration challenges, candidate approaches) — or "skipped" if step 4 was dropped
- OPEN QUESTIONS: numbered list of items to resolve in the requirements-discussion phase (empty if none)
```

### Step 4: Verify outputs (orchestration, no questions)
Do NOT ask the developer anything here — open questions are deferred to Step 6.
1. Verify the outputs in the controller:
   ```powershell
   Test-Path "{specs-root}/{feature-name}/spec.json"
   Test-Path "{specs-root}/{feature-name}/requirements.md"
   Test-Path "{specs-root}/{feature-name}/research.md"
   ```
   Confirm `spec.json` has `phase: "requirements-generated"` and `approvals.requirements.generated: true`. `research.md` may be absent if gap analysis was skipped (Step 0 warning); in that case warn once and continue.
2. If `spec.json` or `requirements.md` is missing or metadata was not updated, report the failure (do not commit); suggest re-running `/kiro-start {feature-name}`.
3. Retain the subagent's **OPEN QUESTIONS** list (if any) in the controller context — these are fed into Step 6 as discussion topics.

### Step 5: Commit the generated spec to the current branch (no push)
The normal path always reaches this step on a **non-default** branch (the default-branch case STOPped in Step 2). Commit the generated spec **before** starting the interactive discussion so the discussion's per-topic commits layer cleanly on top. Do **not** push.
```powershell
git add -A
git commit -m "chore({feature-name}): initialize spec and gap analysis (spec.json, requirements.md, research.md)"
```
- Never push here. Remote sync is the responsibility of `kiro-complete` (PR-based), not `kiro-start`.
- If the commit fails (e.g., nothing to commit, hook failure), report the error; the generated files remain staged/present on the current branch. Still proceed to Step 6 unless the failure indicates the spec files are absent.

### Step 6: Interactive requirements discussion (controller, inline — first questions here)
This is the **only** interactive phase. Run `kiro-requirements-discussion` **inline in the controller** (it is conversational and a subagent cannot talk to the developer). If `kiro-requirements-discussion` was reported missing in Step 0, skip this step and report that the discussion was skipped (recommend running `/kiro-requirements-discussion {feature-name}` manually).

1. Read `{skill-base}/kiro-requirements-discussion/SKILL.md` and follow it, passing `{feature-name}` (= `$ARGUMENTS`) verbatim.
2. **Gap-analysis source mapping:** `kiro-validate-gap` wrote the gap analysis to `{specs-root}/{feature-name}/research.md`. `kiro-requirements-discussion` refers to the gap analysis as `gap-analysis.md`; treat **`research.md` as that gap-analysis input**. Load `research.md` for the discussion's context (Phase 1) instead of failing on a missing `gap-analysis.md`. If `research.md` is also absent (gap skipped), proceed with requirements only, per the discussion skill's fallback.
3. **Seed the discussion with the OPEN QUESTIONS** retained from Step 4 — add them as Category C (developer-confirmation) topics so the assumptions drafted by the subagent are explicitly confirmed or revised.
4. **Conduct it as free-form conversational chat** in the spec's language: present one topic at a time (per the discussion skill's Phase 6 — topic number/total, the referenced requirement, the problem, options, and a recommendation), then await the developer's natural-language reply. Prefer plain conversation over `AskUserQuestion` widgets.
5. Let the discussion skill apply its own edits and commits per its commit conventions (`docs({feature}): ...`). Do **not** push.
6. When the discussion completes (all issues processed), proceed to the Output Description.

> Note: Once Step 6 begins, `/kiro-start` is **interactive and multi-turn** — it does not finish in a single autonomous pass. This is intentional.

## Important Constraints
- **Orchestration split**: The controller (main context) runs Step 0 (resolution), Step 1 (precondition gate), Step 2 (default-branch guard), Step 4 (verification), Step 5 (commit), and Step 6 (interactive discussion, inline). The init + requirements + gap work is delegated to a single non-interactive subagent (Step 3). Do NOT run kiro-spec-init, kiro-spec-requirements, or kiro-validate-gap inline in the controller; do NOT delegate the interactive discussion to a subagent.
- **No questions before discussion**: The controller asks the developer NOTHING during Steps 0–5. The subagent never asks the user; its open questions are carried forward and resolved only in Step 6. The discussion (Step 6) is the first and only interactive phase, conducted as free-form chat.
- **No branch/worktree creation, ever**: This skill does not create, switch, delete, reset, or force-update branches, and does not create worktrees. Feature branches are supplied by the Claude Code (harness) worktree feature.
- **Subagent never interacts with the user**: The Step 3 subagent must not ask questions. It always returns a best-effort DRAFTED report; genuine ambiguities are recorded as OPEN QUESTIONS for the discussion phase.
- **Report in the user's language**: Think in English internally, but write every developer-facing message (progress narration, warnings, and the entire Step 6 discussion) in the spec's target language (see Communication Language). Keep the internal Step 3 subagent prompt in English.
- **Deterministic resolution**: Step 0 resolves specs root, skill base, remote, and default branch with fixed priority orders. Never guess; if a required value is unresolved, hard-fail as specified. Resolve each value once and reuse it.
- This skill is **post-discovery only**: if `{specs-root}/{feature-name}/` does not exist, FAIL deterministically (do not create anything).
- Do NOT generate design or tasks. This skill stops after the requirements discussion (Step 6), before design.
- Do NOT re-derive or change the feature name; it is fixed by discovery (`$ARGUMENTS`).
- **STOP on the default branch**: If the current branch equals the resolved default branch, STOP in Step 2 (no init, no gap, no commit, no discussion) and ask the developer to re-run inside the harness worktree. Never push.
- Do NOT use this skill for `/kiro-spec-batch` (multi-spec) flows.
</instructions>

## Output Description
Provide output in the language specified in `spec.json` with the following structure:

1. **Feature Name**: `feature-name` (confirmed by discovery; equals the argument)
2. **Project Summary**: Brief summary (1 sentence, sourced from brief.md)
3. **Created Files**: Bullet list with full paths (`spec.json`, `requirements.md`, `research.md`)
4. **Branch Status**: On the normal path, report the current branch and that the spec was committed to it without pushing, e.g. `Committed spec to current branch "{branch}" (no push)`. (No branch was created — feature branches come from the harness worktree.) If the run STOPped because the current branch is the default branch, report only the STOP instead (see Safety & Fallback).
5. **Requirements Status**: Confirm `requirements.md` was generated for `{feature-name}` and the subagent's automated review gate passed
6. **Gap Analysis Status**: Confirm `research.md` was produced (or note it was skipped)
7. **Discussion Status**: Summarize the requirements discussion outcome — counts of obvious fixes applied (Category A), items deferred to design (Category B), and developer-confirmed resolutions (Category C); or note the discussion was skipped
8. **Next Step**: Command block showing `/kiro-spec-design <feature-name>` (or `/kiro-spec-design <feature-name> -y` to auto-approve requirements). Gap analysis and the requirements discussion are already complete.

**Format Requirements**:
- Use Markdown headings (##, ###)
- Wrap commands in code blocks
- Keep total output concise (under 300 words)
- Use clear, professional language per `spec.json.language`

## Safety & Fallback
- **Specs Root Unresolved (hard fail)**: If `.kiro/specs` does not exist, STOP and report that the repository is not cc-sdd initialized.
- **Skills Unresolved (hard fail)**: If `kiro-spec-init/SKILL.md` or `kiro-spec-requirements/SKILL.md` is not found under `.claude/skills`, `.agents/skills`, or `.github/skills` (in that order), STOP and report that the required kiro skills are not installed.
- **Optional Skills Missing (soft fail)**: If `kiro-validate-gap` is missing, warn once and skip gap analysis (Step 3 step 4 dropped, `research.md` not produced). If `kiro-requirements-discussion` is missing, warn once and skip Step 6, recommending the developer run `/kiro-requirements-discussion {feature-name}` manually. Neither aborts the rest of the flow.
- **Missing Spec Folder (hard fail)**: If `{specs-root}/{feature-name}/` does not exist, STOP immediately and report the error. Do not create the folder, branch, or any spec files. Suggest running `/kiro-discovery "<idea>"` first, or verifying the feature name against existing folders under `{specs-root}/`.
- **Missing Brief (hard fail)**: If the folder exists but `brief.md` is absent, STOP and report that the discovery brief is incomplete; recommend re-running `/kiro-discovery` for this feature.
- **Init Delegation**: All init-level fallbacks (missing templates, write failure) are handled by `kiro-spec-init` inside the Step 3 subagent. Honor its results as reported by the subagent.
- **Requirements Delegation**: All requirements-level behavior (steering load, EARS rules, automated review gate) is handled by `kiro-spec-requirements` inside the Step 3 subagent. Honor its results as reported by the subagent.
- **Gap-Analysis Delegation**: All gap-analysis behavior (codebase analysis, dependency research, `research.md` output) is handled by `kiro-validate-gap` inside the Step 3 subagent. If `research.md` is absent after a DRAFTED report, warn once; the discussion (Step 6) can still proceed with requirements only.
- **Subagent Failure (Step 3)**: If the subagent errors out or returns no structured report, do NOT commit. Report the failure and suggest re-running `/kiro-start {feature-name}` on the same branch. No branch was created, so there is nothing to clean up.
- **Clarifications Deferred (no questions before discussion)**: The Step 3 subagent never blocks on user input; it drafts best-effort with labeled assumptions and returns OPEN QUESTIONS. The controller does NOT ask these in Step 4 — it carries them into Step 6 and resolves them conversationally during the discussion.
- **Missing Outputs After DRAFTED**: If the subagent reports DRAFTED but `spec.json`/`requirements.md` are missing or metadata is not updated (Step 4 verification fails), report the failure and do not commit.
- **Discussion Interruption (Step 6)**: The discussion is interactive and multi-turn. If the developer ends or abandons it, the Step 5 commit (initial spec + gap) is preserved on the current branch; remaining topics can be resumed any time via `/kiro-requirements-discussion {feature-name}`.
- **On Default Branch (STOP)**: If the current branch equals `{default-branch}`, STOP in Step 2 before any init/gap/commit/discussion/push. Report that spec initialization does not run on the default branch and ask the developer to re-run `/kiro-start {feature-name}` inside the Claude Code harness worktree (a non-default working branch). Do not create a branch or modify any files; `brief.md` is preserved for the re-run.
- **Non-Default Branch (normal path)**: Proceed with init + requirements + gap on the current branch, commit the generated spec to it (no push), then run the interactive discussion. Remote sync is deferred to `kiro-complete`.
- **No Remote**: If `{remote}` is none, warn once; spec initialization, the local commit, and the discussion still proceed (kiro-start never pushes or performs remote operations).
- **Commit Failure**: Report the error with the current branch name; the generated files remain staged/present on the current branch.

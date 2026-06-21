---
name: kiro-design
description: 'Design-phase orchestrator for a single spec. Precondition: requirements.md already exists for {feature-name} (produced by /kiro-start or /kiro-spec-requirements). Acts as an orchestrator: the controller handles deterministic resolution, the precondition gate, the default-branch guard, the initial commit, and an interactive design discussion, while delegating design generation + design validation to a single non-interactive subagent. Portable across repos: deterministically resolves the specs root, skill base directory, git remote, and default branch (fixed priority orders, no guessing). Does NOT create branches or worktrees — feature branches are supplied by the Claude Code harness worktree. When the current branch is the repository default branch it STOPs and asks the developer to re-run inside the harness worktree (a non-default working branch); otherwise it runs kiro-spec-design (with -y), runs kiro-validate-design non-interactively, commits the generated design to the current branch without pushing, and then runs kiro-design-discussion interactively. No questions are asked before the discussion phase. Use when: 要件確定後に設計フェーズを開始したい, kiro-design, 設計開始, design開始, start the design phase for a spec. DO NOT USE FOR: 要件生成や要件ディスカッション (use /kiro-start or /kiro-spec-requirements), タスク生成 (use /kiro-spec-tasks), 実装 (use /kiro-impl).'
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Agent, WebSearch, WebFetch, AskUserQuestion
argument-hint: <feature-name>
---

# Spec Design (Design + Validate + Discussion)

<instructions>
## Core Task
Run the design phase of a single specification end-to-end, taking it from design generation through design validation and an interactive design discussion. The **feature name equals `$ARGUMENTS`** and must already have `requirements.md` (produced by `/kiro-start` or `/kiro-spec-requirements`). This skill does **not** create branches or worktrees: feature branches are supplied by the Claude Code (harness) worktree feature. When the current branch is the repository's default branch, **STOP** and ask the developer to re-run inside the harness worktree (a non-default working branch). Otherwise (already on a non-default branch), it:
1. Generates the design via `kiro-spec-design` with `-y` (auto-approves requirements; writes `design.md` and `research.md`),
2. Runs `kiro-validate-design` **non-interactively** (quality review → GO/NO-GO + critical issues report; no edits, no questions),
3. Commits the generated design to the current branch **without pushing**, and
4. Runs `kiro-design-discussion` **interactively** to refine the design.

This skill stops **before** task generation (`/kiro-spec-tasks`).

This skill acts as an **orchestrator**. The controller (main context) performs deterministic, state-sensitive orchestration that cannot be safely delegated — portable-context resolution, the precondition gate, the default-branch guard, the initial commit — **plus** the interactive design discussion, which is inherently conversational and therefore runs inline in the controller (a subagent cannot talk to the developer). All other heavy work (design generation, discovery/synthesis, the design review gate, and design validation) is **delegated to a single non-interactive subagent** via the Agent tool so the controller context stays small. That subagent never interacts with the user; any genuine clarification or validation concern is recorded as an OPEN QUESTION and **carried forward to the discussion phase** — it is never asked before then.

This skill is designed to be **portable across repositories**. It does not hard-code skill paths, the remote name, or the default branch; instead it resolves each one with a **deterministic, ordered detection procedure** (Step 0). Each resolution yields exactly one result or a hard failure — never an ambiguous guess.

## Communication Language
- **Think in English, report in the user's language.** Internal reasoning, planning, and tool orchestration may be in English, but every message surfaced to the developer MUST be written in the target language configured for this spec.
- **Resolve the report language** from `{specs-root}/{feature-name}/spec.json` (`language` field). If `spec.json` does not specify a language, fall back to the language of the user's input (default `ja` for this repository). Use this same language for the discussion phase and the final Output Description.
- This applies to ALL developer-facing text emitted by the controller: orchestration progress narration ("dispatching subagent", "verifying outputs", "committing design"), warnings, and the entire interactive discussion in Step 6.
- The Step 3 **subagent prompt itself stays in English** (it is internal instruction, not user-facing). Translate only the controller's own narration and the discussion you conduct with the developer.
- **No questions before the discussion phase.** The controller does NOT ask the developer any question during Steps 0–5, and the validation in Step 3 runs non-interactively. The first (and only) interactive phase is Step 6 (design discussion). Conduct Step 6 as **free-form conversational chat** (自由応答) — present one topic at a time and await the developer's natural-language reply; prefer plain conversational turns over structured `AskUserQuestion` widgets, reserving the latter only for a strictly constrained either/or choice.
- `$ARGUMENTS` is the **confirmed kiro feature name** (it matches an existing directory `{specs-root}/{feature-name}/` that already contains `requirements.md`). Pass it **verbatim** as the first parameter of `kiro-spec-design`, `kiro-validate-design`, and `kiro-design-discussion`. Do NOT ask clarifying questions or re-derive a name in this wrapper.
- This skill does not derive, create, or switch branches. It operates on whatever branch the harness worktree provides; the only branch-related decision is the default-branch guard in Step 2 (STOP vs. proceed).

## Execution Steps

### Step 0: Resolve portable context (deterministic)
Resolve these four values once, in order. Each has a single deterministic outcome; if a required value cannot be resolved, fail as specified.

1. **Specs root** (`{specs-root}`): Use the first existing directory in this fixed priority order:
   1. `.kiro/specs`
   If `.kiro/specs` does not exist, STOP (hard fail): this repository is not cc-sdd initialized.

2. **Skill base directory** (`{skill-base}`): Locate the directory that contains `kiro-spec-design/SKILL.md`, checking this fixed priority order and taking the FIRST match:
   1. `.claude/skills`
   2. `.agents/skills`
   3. `.github/skills`
   Resolve `kiro-validate-design/SKILL.md` and `kiro-design-discussion/SKILL.md` under the **same** `{skill-base}`. If `kiro-spec-design` is not found under any of the three bases, STOP (hard fail): the required kiro skill is not installed. If `kiro-validate-design` or `kiro-design-discussion` is missing under the resolved base, warn once and skip the corresponding phase (validation and/or discussion) rather than hard-failing the whole flow.

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

### Step 1: Verify design precondition (hard gate)
The design phase requires that requirements already exist for this feature.
1. Treat `$ARGUMENTS` as the confirmed feature name. Verify the spec folder and requirements document exist:
   ```powershell
   Test-Path "{specs-root}/{feature-name}"
   Test-Path "{specs-root}/{feature-name}/requirements.md"
   ```
2. **If the spec folder `{specs-root}/{feature-name}/` does not exist: STOP.** Do NOT create the folder, do NOT create a branch, do NOT run design. Report the failure: the feature name was not found; suggest running `/kiro-start "<feature>"` (post-discovery) or checking the feature name spelling against existing folders under `{specs-root}/`.
3. **If the folder exists but `requirements.md` is missing:** the requirements phase is incomplete. STOP and report that `requirements.md` is missing; recommend running `/kiro-start {feature-name}` or `/kiro-spec-requirements {feature-name}` first. Do not fabricate requirements.
4. Only when both exist, proceed.

### Step 2: Default-branch guard (no branch creation)
This skill never creates branches or worktrees — the feature branch is supplied by the Claude Code (harness) worktree. This step is purely a guard: it either STOPs (on the default branch) or lets the flow proceed on the current non-default branch.

1. Determine the current branch:
   ```powershell
   $branch = git branch --show-current
   ```
2. **If `$branch` equals `{default-branch}` (resolved in Step 0): STOP.**
   - Do NOT create a branch, do NOT run design/validation/discussion, do NOT commit, do NOT push.
   - Report that `/kiro-design` does not run on the default branch (`{default-branch}`): design must happen on a non-default working branch. Ask the developer to re-run `/kiro-design {feature-name}` inside the Claude Code harness worktree (a non-default working branch). The existing artifacts (`requirements.md`) remain intact for the re-run.
3. **If `$branch` does NOT equal `{default-branch}`:**
   - Proceed directly to the design generation phase (Step 3) on the current branch. Do not create or switch branches, do not pull, do not push.

### Step 3: Delegate design generation + validation to a subagent (orchestration)
Dispatch **one** non-interactive subagent via the Agent tool to perform the entire heavy phase (design → validation) so the controller context stays lightweight. Pass the resolved values from Step 0 (`{specs-root}`, `{skill-base}`, `{feature-name}` = `$ARGUMENTS`) into the prompt. If `kiro-validate-design` was reported missing in Step 0, drop step 3 from the prompt and note that validation is skipped.

Use this subagent prompt:
```
You are completing the design + validation phase for the confirmed kiro feature "{feature-name}".
The feature name is FINAL — do not re-derive or change it. requirements.md already exists.

1. Read {specs-root}/{feature-name}/requirements.md and {specs-root}/{feature-name}/research.md (if present)
   for the requirements and any prior gap analysis / discovery findings.
2. Generate the design: read {skill-base}/kiro-spec-design/SKILL.md and follow every step, passing
   "{feature-name}" with the -y flag (the -y auto-approves requirements). Run discovery/synthesis, pass the
   design review gate, then write {specs-root}/{feature-name}/design.md and update
   {specs-root}/{feature-name}/research.md. Update spec.json to phase "design-generated" only after the
   review gate passes.
3. Validate the design: read {skill-base}/kiro-validate-design/SKILL.md and follow it, passing "{feature-name}".
   Run it NON-INTERACTIVELY — you cannot interact with the user, so DO NOT ask any questions. Produce the quality
   review as a REPORT only (analysis, up to 3 critical issues, strengths, and a GO/NO-GO decision with rationale).
   Do NOT modify design.md based on the review — any fixes happen later in the interactive discussion phase.
4. Ground every decision in requirements.md and research.md. DO NOT ask the user anything — ALL clarification is
   deferred to the interactive design-discussion phase that the controller runs next. When a design ambiguity, an
   open trade-off, or a validation concern remains that the inputs cannot resolve, DO NOT block and DO NOT guess
   silently: draft the design best-effort with an explicit, clearly-labeled assumption, and record the item as an
   OPEN QUESTION for the discussion phase. ALWAYS produce a complete best-effort design draft and a validation report.

Return a structured report:
- STATUS: DRAFTED (always produce a best-effort draft; never block on user input)
- Created/updated files (full paths: design.md, research.md, spec.json)
- Design summary (3-5 bullets: architecture, key components, boundary)
- Design review-gate result (from kiro-spec-design)
- Validation result (from kiro-validate-design): GO or NO-GO + up to 3 critical issues + strengths — or "skipped" if step 3 was dropped
- OPEN QUESTIONS: numbered list of items to resolve in the design-discussion phase (empty if none)
```

### Step 4: Read subagent results and verify outputs (orchestration, no questions)
Do NOT ask the developer anything here — open questions and validation concerns are deferred to Step 6.
1. Read the subagent's structured report. Retain its **validation result** (GO/NO-GO + critical issues) and **OPEN QUESTIONS** in the controller context — these seed Step 6.
2. Verify the outputs in the controller:
   ```powershell
   Test-Path "{specs-root}/{feature-name}/design.md"
   Test-Path "{specs-root}/{feature-name}/research.md"
   ```
   Confirm `spec.json` has `phase: "design-generated"` and `approvals.design.generated: true`. `research.md` should normally exist; if absent, warn once and continue.
3. If `design.md` is missing or metadata was not updated, report the failure (do not commit); suggest re-running `/kiro-design {feature-name}`.

### Step 5: Commit the generated design to the current branch (no push)
The normal path always reaches this step on a **non-default** branch (the default-branch case STOPped in Step 2). Commit the generated design **before** starting the interactive discussion so the discussion's per-topic commits layer cleanly on top. Do **not** push.
```powershell
git add -A
git commit -m "chore({feature-name}): generate design (design.md, research.md)"
```
- Never push here. Remote sync is the responsibility of `kiro-complete` (PR-based), not `kiro-design`.
- If the commit fails (e.g., nothing to commit, hook failure), report the error; the generated files remain staged/present on the current branch. Still proceed to Step 6 unless the failure indicates the design files are absent.

### Step 6: Interactive design discussion (controller, inline — first questions here)
This is the **only** interactive phase. Run `kiro-design-discussion` **inline in the controller** (it is conversational and a subagent cannot talk to the developer). If `kiro-design-discussion` was reported missing in Step 0, skip this step and report that the discussion was skipped (recommend running `/kiro-design-discussion {feature-name}` manually).

1. Read `{skill-base}/kiro-design-discussion/SKILL.md` and follow it, passing `{feature-name}` (= `$ARGUMENTS`) verbatim.
2. **Discovery/gap source:** the gap analysis and discovery findings live in `{specs-root}/{feature-name}/research.md` (written by `/kiro-validate-gap` and `/kiro-spec-design`). `kiro-design-discussion` already loads `research.md` (and `gap-analysis.md` if present) in its Phase 1 — load whichever exists; do not fail on a missing `gap-analysis.md`.
3. **Seed the discussion** with the **validation critical issues** and **OPEN QUESTIONS** retained from Step 4 — add them as Category B (developer-confirmation) topics so the validation concerns and the assumptions drafted by the subagent are explicitly confirmed or revised.
4. **Conduct it as free-form conversational chat** in the spec's language: present one topic at a time (per the discussion skill's Phase 5 — topic number/total, the referenced design section, the problem, options, and a recommendation), then await the developer's natural-language reply. Prefer plain conversation over `AskUserQuestion` widgets.
5. Let the discussion skill apply its own edits and commits per its commit conventions (`docs({feature}): ...`). Do **not** push.
6. When the discussion completes (all issues processed), proceed to the Output Description.

> Note: Once Step 6 begins, `/kiro-design` is **interactive and multi-turn** — it does not finish in a single autonomous pass. This is intentional.

## Important Constraints
- **Orchestration split**: The controller (main context) runs Step 0 (resolution), Step 1 (precondition gate), Step 2 (default-branch guard), Step 4 (read results + verification), Step 5 (commit), and Step 6 (interactive discussion, inline). The design generation + validation work is delegated to a single non-interactive subagent (Step 3). Do NOT run kiro-spec-design or kiro-validate-design inline in the controller; do NOT delegate the interactive discussion to a subagent.
- **No questions before discussion**: The controller asks the developer NOTHING during Steps 0–5; the validation in Step 3 runs non-interactively (report only). The subagent never asks the user; its open questions and validation concerns are carried forward and resolved only in Step 6. The discussion (Step 6) is the first and only interactive phase, conducted as free-form chat.
- **No branch/worktree creation, ever**: This skill does not create, switch, delete, reset, or force-update branches, and does not create worktrees. Feature branches are supplied by the Claude Code (harness) worktree feature.
- **Subagent never interacts with the user**: The Step 3 subagent must not ask questions and must run kiro-validate-design non-interactively. It always returns a best-effort DRAFTED report; genuine ambiguities and validation concerns are recorded as OPEN QUESTIONS / critical issues for the discussion phase.
- **Report in the user's language**: Think in English internally, but write every developer-facing message (progress narration, warnings, and the entire Step 6 discussion) in the spec's target language (see Communication Language). Keep the internal Step 3 subagent prompt in English.
- **Deterministic resolution**: Step 0 resolves specs root, skill base, remote, and default branch with fixed priority orders. Never guess; if a required value is unresolved, hard-fail as specified. Resolve each value once and reuse it.
- This skill is **post-requirements only**: if `{specs-root}/{feature-name}/requirements.md` does not exist, FAIL deterministically (do not create anything).
- Do NOT generate tasks or implement. This skill stops after the design discussion (Step 6), before task generation.
- Do NOT re-derive or change the feature name; it is fixed by `$ARGUMENTS`.
- **STOP on the default branch**: If the current branch equals the resolved default branch, STOP in Step 2 (no design, no validation, no commit, no discussion) and ask the developer to re-run inside the harness worktree. Never push.
</instructions>

## Output Description
Provide output in the language specified in `spec.json` with the following structure:

1. **Feature Name**: `feature-name` (equals the argument)
2. **Design Summary**: Brief summary (2-3 sentences, sourced from the generated design)
3. **Created Files**: Bullet list with full paths (`design.md`, `research.md`)
4. **Branch Status**: On the normal path, report the current branch and that the design was committed to it without pushing, e.g. `Committed design to current branch "{branch}" (no push)`. (No branch was created — feature branches come from the harness worktree.) If the run STOPped because the current branch is the default branch, report only the STOP instead (see Safety & Fallback).
5. **Design Status**: Confirm `design.md` was generated for `{feature-name}` and the subagent's design review gate passed
6. **Validation Status**: Report the GO/NO-GO decision and the critical issues from `kiro-validate-design` (or note it was skipped)
7. **Discussion Status**: Summarize the design discussion outcome — counts of obvious fixes applied (Category A) and developer-confirmed resolutions (Category B); or note the discussion was skipped
8. **Next Step**: Command block showing `/kiro-spec-tasks <feature-name>` (or `/kiro-spec-tasks <feature-name> -y` to auto-approve the design). Design validation and the design discussion are already complete.

**Format Requirements**:
- Use Markdown headings (##, ###)
- Wrap commands in code blocks
- Keep total output concise (under 300 words)
- Use clear, professional language per `spec.json.language`

## Safety & Fallback
- **Specs Root Unresolved (hard fail)**: If `.kiro/specs` does not exist, STOP and report that the repository is not cc-sdd initialized.
- **Skill Unresolved (hard fail)**: If `kiro-spec-design/SKILL.md` is not found under `.claude/skills`, `.agents/skills`, or `.github/skills` (in that order), STOP and report that the required kiro skill is not installed.
- **Optional Skills Missing (soft fail)**: If `kiro-validate-design` is missing, warn once and skip validation (Step 3 step 3 dropped). If `kiro-design-discussion` is missing, warn once and skip Step 6, recommending the developer run `/kiro-design-discussion {feature-name}` manually. Neither aborts the rest of the flow.
- **Missing Spec Folder (hard fail)**: If `{specs-root}/{feature-name}/` does not exist, STOP immediately and report the error. Do not create the folder, branch, or any spec files. Suggest running `/kiro-start "<feature>"` first, or verifying the feature name against existing folders under `{specs-root}/`.
- **Missing Requirements (hard fail)**: If the folder exists but `requirements.md` is absent, STOP and report that the requirements phase is incomplete; recommend running `/kiro-start {feature-name}` or `/kiro-spec-requirements {feature-name}` first.
- **Design Delegation**: All design-level behavior (discovery, synthesis, design review gate, requirement-ID validation, template fallbacks) is handled by `kiro-spec-design` inside the Step 3 subagent. Honor its results as reported by the subagent. If the subagent reports a real spec gap surfaced during the design review gate, do NOT commit; report it and recommend fixing `requirements.md` (e.g. via `/kiro-requirements-discussion {feature-name}`) before re-running `/kiro-design {feature-name}`.
- **Validation Delegation**: All validation behavior (quality review, GO/NO-GO) is handled by `kiro-validate-design` inside the Step 3 subagent, run non-interactively. A NO-GO decision does NOT abort the flow — its critical issues become discussion topics in Step 6.
- **Subagent Failure (Step 3)**: If the subagent errors out or returns no structured report, do NOT commit. Report the failure and suggest re-running `/kiro-design {feature-name}` on the same branch. No branch was created, so there is nothing to clean up.
- **Clarifications Deferred (no questions before discussion)**: The Step 3 subagent never blocks on user input; it drafts best-effort with labeled assumptions and returns OPEN QUESTIONS plus validation critical issues. The controller does NOT ask these in Step 4 — it carries them into Step 6 and resolves them conversationally during the discussion.
- **Missing Outputs After DRAFTED**: If the subagent reports DRAFTED but `design.md` is missing or metadata is not updated (Step 4 verification fails), report the failure and do not commit.
- **Discussion Interruption (Step 6)**: The discussion is interactive and multi-turn. If the developer ends or abandons it, the Step 5 commit (generated design) is preserved on the current branch; remaining topics can be resumed any time via `/kiro-design-discussion {feature-name}`.
- **On Default Branch (STOP)**: If the current branch equals `{default-branch}`, STOP in Step 2 before any design/validation/commit/discussion/push. Report that design does not run on the default branch and ask the developer to re-run `/kiro-design {feature-name}` inside the Claude Code harness worktree (a non-default working branch). Do not create a branch or modify any files; existing artifacts are preserved for the re-run.
- **Non-Default Branch (normal path)**: Proceed with design + validation on the current branch, commit the generated design to it (no push), then run the interactive discussion. Remote sync is deferred to `kiro-complete`.
- **No Remote**: If `{remote}` is none, warn once; design generation, the local commit, and the discussion still proceed (kiro-design never pushes or performs remote operations).
- **Commit Failure**: Report the error with the current branch name; the generated files remain staged/present on the current branch.

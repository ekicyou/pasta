---
name: kiro-design
description: 'Post-requirements design entry point. Precondition: requirements.md is already generated (typically after /kiro-start and the requirements discussion). Acts as a lightweight orchestrator that drives a confirmed spec through design generation, design validation, and an interactive design discussion on the harness worktree branch. The controller handles deterministic resolution, the precondition gate, the default-branch guard, validation-report path resolution, the chat-window discussion, and commits; it delegates design generation and design validation to subagents. No questions are asked before the discussion phase: the subagents draft best-effort with labelled assumptions and return OPEN QUESTIONS, which become discussion topics. Portable across repos: deterministically resolves the specs root, skill base directory, git remote, and default branch (fixed priority orders, no guessing). Does NOT create branches or worktrees — feature branches are supplied by the Claude Code harness worktree. When the current branch is the repository default branch it STOPs and asks the developer to re-run inside the harness worktree (a non-default working branch); otherwise it generates the design via kiro-spec-design (with -y to auto-approve requirements), validates it via kiro-validate-design (subagent, non-interactive, report persisted to disk), then conducts the design discussion (kiro-design-discussion) interactively in the chat window, committing each phase to the current branch without pushing. Use when: 要件確定後に（ハーネスのワークツリー上で）設計フェーズを開始したい, kiro-design, 設計開始, start the design phase on the harness worktree branch. DO NOT USE FOR: 要件生成・要件ディスカッション (use /kiro-start), タスク生成 (use /kiro-spec-tasks), 設計生成のみ (use /kiro-spec-design directly), 設計バリデーションのみ (use /kiro-validate-design directly).'
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Agent, WebSearch, WebFetch
argument-hint: <feature-name>
---

# Spec Design (Design → Validate → Design Discussion, post-requirements)

<instructions>
## Core Task
Drive the design phase of a single specification end-to-end **after requirements are generated** (typically after `/kiro-start` and the requirements discussion). The **feature name equals `$ARGUMENTS`** and matches an existing directory `{specs-root}/{feature-name}/` that already contains `requirements.md`. This skill does **not** create branches or worktrees: feature branches are supplied by the Claude Code (harness) worktree feature. When the current branch is the repository's default branch, **STOP** and ask the developer to re-run inside the harness worktree (a non-default working branch). Otherwise (already on a non-default branch), drive the design through three phases on the current branch, committing each without pushing:
1. **Design generation** — `kiro-spec-design {feature} -y` (delegated to a subagent; best-effort draft, never blocks on user input): translates requirements into architecture and writes the design document. `-y` auto-approves requirements so design can proceed.
2. **Design validation** — `kiro-validate-design {feature}` (delegated to a subagent, run **non-interactively**): produces a GO/NO-GO quality review and **persists the report to disk** so the discussion can ingest it.
3. **Design discussion** — `kiro-design-discussion {feature}` (run **interactively in the controller / chat window**): ingests the design document, the validation report, the research/decision log, and the subagent's OPEN QUESTIONS, collects/classifies issues, explains the spec's purpose with concrete examples, presents all developer-facing topics at once with recommendations, and settles them in the chat window (bulk approval, or individual discussion of only the topics the developer names).

This skill stops **before** task generation (`/kiro-spec-tasks`).

This skill acts as a **lightweight orchestrator**. The controller (main context) performs deterministic, state-sensitive orchestration that cannot be safely delegated — portable-context resolution, the precondition gate, the default-branch guard, validation-report path resolution, the interactive design discussion, and commits. It never creates, deletes, resets, or pushes branches. The two non-interactive heavy phases (design generation; design validation) are **delegated to subagents** via the Agent tool so the controller context stays small. Subagents never interact with the user; any genuine clarification or validation concern is recorded as an OPEN QUESTION or critical issue and **carried forward to the discussion phase** — it is never asked before then.

**The design discussion is the one heavy phase that is NOT delegated.** It is inherently interactive — it requires developer dialogue (bulk approval or per-topic discussion) — and the developer's Q&A MUST happen in the chat window. A subagent cannot talk to the user, so the controller runs this phase inline by reading the `kiro-design-discussion` SKILL.md and following it directly (see Step 7).

This skill is designed to be **portable across repositories**. It does not hard-code skill paths, the remote name, or the default branch; instead it resolves each one with a **deterministic, ordered detection procedure** (Step 0). Each resolution yields exactly one result or a hard failure — never an ambiguous guess.

## Communication Language
- **Think in English, report in the user's language.** Internal reasoning, planning, and tool orchestration may be in English, but every message surfaced to the developer MUST be written in the target language configured for this spec.
- **Resolve the report language** from `{specs-root}/{feature-name}/spec.json` (`language` field). If `spec.json` lacks a language, fall back to the language of the user's input (default `ja` for this repository). Use this same language for the discussion phase and the final Output Description.
- This applies to ALL developer-facing text emitted by the controller: orchestration progress narration ("dispatching subagent", "verifying outputs", "starting discussion"), warnings, **and the entire interactive design discussion in Step 7**.
- The Step 3 and Step 6 **subagent prompts themselves stay in English** (they are internal instructions, not user-facing). Translate only the controller's own narration and the Step 7 discussion into the spec's target language.
- **No questions before the discussion phase.** The controller does NOT ask the developer any question during Steps 0–6, and the validation in Step 6 runs non-interactively. The first (and only) interactive phase is Step 7 (design discussion), conducted in the chat window; never use `AskUserQuestion`.
- `$ARGUMENTS` is the **confirmed kiro feature name** (it matches the existing directory `{specs-root}/{feature-name}/`, which already contains `requirements.md`). Pass it **verbatim** as the first parameter of `kiro-spec-design`, `kiro-validate-design`, and `kiro-design-discussion`. Do NOT ask clarifying questions or re-derive a name in this wrapper.
- This skill does not derive, create, or switch branches. It operates on whatever branch the harness worktree provides; the only branch-related decision is the default-branch guard in Step 2 (STOP vs. proceed).

## Execution Steps

### Step 0: Resolve portable context (deterministic)
Resolve these values once, in order. Each has a single deterministic outcome; if a required value cannot be resolved, fail as specified.

1. **Specs root** (`{specs-root}`): Use the first existing directory in this fixed priority order:
   1. `.kiro/specs`
   If `.kiro/specs` does not exist, STOP (hard fail): this repository is not cc-sdd initialized.

2. **Skill base directory** (`{skill-base}`): Locate the directory that contains `kiro-spec-design/SKILL.md`, checking this fixed priority order and taking the FIRST match:
   1. `.claude/skills`
   2. `.agents/skills`
   3. `.github/skills`
   Resolve `kiro-validate-design/SKILL.md` and `kiro-design-discussion/SKILL.md` under the **same** `{skill-base}`. If any of these three sibling skills is not found under the chosen base, STOP (hard fail): the required kiro skills are not installed.

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

### Step 1: Verify precondition (hard gate)
The design phase requires generated requirements.
1. Treat `$ARGUMENTS` as the confirmed feature name. Verify the spec folder and requirements exist:
   ```powershell
   Test-Path "{specs-root}/{feature-name}"
   Test-Path "{specs-root}/{feature-name}/requirements.md"
   ```
2. **If the spec folder `{specs-root}/{feature-name}/` does not exist: STOP.** Do NOT create the folder, do NOT run design. Report that the feature was not found; suggest running `/kiro-start <feature>` (post-discovery) or checking the feature name against existing folders under `{specs-root}/`.
3. **If the folder exists but `requirements.md` is missing:** STOP and report that requirements have not been generated; recommend running `/kiro-start {feature-name}` (or `/kiro-spec-requirements {feature-name}`) first. Do not fabricate requirements.
4. Read `{specs-root}/{feature-name}/spec.json` and confirm `approvals.requirements.generated: true`. If requirements are not marked generated, warn but proceed (the Step 3 `-y` flag auto-approves requirements). If `design.md` already exists, proceed anyway — `kiro-spec-design` operates in merge mode (existing design used as reference).
5. Only when the spec folder and `requirements.md` both exist, proceed.

### Step 2: Default-branch guard (no branch creation)
This skill never creates branches or worktrees — the feature branch is supplied by the Claude Code (harness) worktree. This step is purely a guard: it either STOPs (on the default branch) or lets the flow proceed on the current non-default branch.

1. Determine the current branch:
   ```powershell
   $branch = git branch --show-current
   ```
2. **If `$branch` equals `{default-branch}` (resolved in Step 0): STOP.**
   - Do NOT create a branch, do NOT run any phase, do NOT commit, do NOT push.
   - Report that `/kiro-design` does not run on the default branch (`{default-branch}`): design work must happen on a non-default working branch. Ask the developer to re-run `/kiro-design {feature-name}` inside the Claude Code harness worktree (a non-default working branch). The existing artifacts (`requirements.md`) remain intact for the re-run.
3. **If `$branch` does NOT equal `{default-branch}`:**
   - Proceed directly to the design generation phase (Step 3) on the current branch. Do not create or switch branches, do not pull, do not push.

### Step 3: Delegate design generation to a subagent (orchestration)
Dispatch **one** non-interactive subagent via the Agent tool to perform the design-generation phase so the controller context stays lightweight. Pass the resolved Step 0 values (`{specs-root}`, `{skill-base}`, `{feature-name}` = `$ARGUMENTS`) into the prompt.

Use this subagent prompt:
```
You are completing the design-generation phase for the confirmed kiro feature "{feature-name}".
requirements.md is generated. The feature name is FINAL — do not re-derive or change it.

1. Read {skill-base}/kiro-spec-design/SKILL.md and follow every step it specifies, passing the feature name
   "{feature-name}" with the -y (auto-approve requirements) flag. The -y flag auto-approves requirements in spec.json so
   the design phase may proceed.
2. Inputs: {specs-root}/{feature-name}/spec.json, requirements.md, research.md (gap analysis + recorded design
   decisions, if present), brief.md (if present), the steering files under .kiro/steering/, and the design template
   under .kiro/settings/templates/specs/.
3. Execute discovery and synthesis as the skill directs (it MAY dispatch its own research subagents), generate the design
   draft, pass the design review gate (at most 2 repair passes), then write {specs-root}/{feature-name}/design.md and
   update research.md. Update spec.json metadata (phase=design-generated, approvals.design.generated=true,
   approvals.requirements.approved=true) ONLY after the review gate passes.
4. Ground every decision in requirements.md and research.md. DO NOT ask the user anything — you cannot interact with
   the user, and ALL clarification is deferred to the interactive design-discussion phase that the controller runs later.
   When a design ambiguity or an open trade-off remains that the inputs cannot resolve, DO NOT block and DO NOT guess
   silently: draft the design best-effort with an explicit, clearly-labelled assumption, and record the item as an
   OPEN QUESTION for the discussion phase. ALWAYS produce a complete best-effort design draft.
5. Exception: if the design review exposes a GENUINE requirements gap or contradiction (the requirements themselves
   are missing or contradict each other, not merely a design choice), DO NOT paper over it in design.md. Return
   STATUS: SPEC_GAP and describe the gap.

Environment note (Windows + harness): if a direct .md Write/Edit fails, write the file content via a PowerShell
here-string (Set-Content) or equivalent so the final files are valid.

Return a structured report:
- STATUS: DRAFTED | SPEC_GAP
- Created/updated files (full paths)
- Discovery type (full/light/minimal) and a 3-5 bullet design summary; design review-gate result
- OPEN QUESTIONS: numbered list of items to resolve in the design-discussion phase, each with the assumption drafted for it (empty if none)
- SPEC GAP: the requirements gap or contradiction (only when STATUS=SPEC_GAP)
```

### Step 4: Verify the design (orchestration, no questions)
Do NOT ask the developer anything here — open questions are deferred to Step 7.
1. **If the subagent returns `STATUS: SPEC_GAP`**: do NOT commit. Report the requirements gap and recommend fixing `requirements.md` (e.g. via `/kiro-requirements-discussion {feature-name}`) before re-running `/kiro-design {feature-name}`. Stop here.
2. **If the subagent returns `STATUS: DRAFTED`**, verify the outputs in the controller:
   ```powershell
   Test-Path "{specs-root}/{feature-name}/design.md"
   ```
   Confirm `spec.json` has `phase: "design-generated"`, `approvals.design.generated: true`, and `approvals.requirements.approved: true`.
3. If a required file is missing or metadata was not updated, report the failure (do not commit); suggest re-running `/kiro-design {feature-name}`.
4. Retain the subagent's **OPEN QUESTIONS** list (if any) in the controller context — these are fed into Step 7 as discussion topics.

### Step 5: Commit the design baseline to the current branch (no push)
The normal path always reaches this step on a **non-default** branch (the default-branch case STOPped in Step 2). Commit the generated design baseline to the **current branch**. Do **not** push.
```powershell
git add -A
git commit -m "docs({feature-name}): generate design (design.md)"
```
- This is an intentional checkpoint that locks the design baseline before validation and discussion modify it.
- Never push here. Remote sync is the responsibility of `kiro-complete` (PR-based), not `kiro-design`.
- If the commit fails (e.g., nothing to commit, hook failure), report the error; the generated files remain present on the current branch.

### Step 6: Delegate design validation to a subagent (orchestration)
Design validation is delegated, and it runs **non-interactively** (the source skill is interactive by default, but a subagent cannot talk to the user, so the subagent produces the verdict directly and persists it to disk). Dispatch **one** subagent via the Agent tool. Pass the resolved Step 0 values into the prompt.

Use this subagent prompt:
```
You are completing the design-validation phase for the confirmed kiro feature "{feature-name}".
design.md and requirements.md are drafted. DO NOT modify design.md, requirements.md, research.md, or spec.json.

1. Read {skill-base}/kiro-validate-design/SKILL.md and follow its REVIEW process
   (Analysis -> Critical Issues -> Strengths -> GO/NO-GO).
2. Inputs: {specs-root}/{feature-name}/spec.json (language/metadata), requirements.md, design.md, research.md (if present),
   and the steering files under .kiro/steering/.
3. IMPORTANT: this runs NON-INTERACTIVELY. DO NOT ask the user anything (no AskUserQuestion). Produce the verdict directly:
   at most 3 critical issues, 1-2 strengths, and a clear GO / NO-GO decision with rationale.
4. Persist the review to disk: write the full review report to {specs-root}/{feature-name}/design-validation.md
   (Markdown, in the spec.json language), then READ IT BACK to verify it was written. Report the exact full path you wrote.

Environment note (Windows + harness): if a direct .md Write/Edit fails, write the file content via a PowerShell
here-string (Set-Content) or equivalent so the final file is valid.

Return a structured report:
- STATUS: DONE | FAILED
- Validation report path (full path) and confirm it was written and read back
- Decision: GO | NO-GO and a one-line rationale
- Critical issues: numbered list (max 3; these feed the design discussion), empty if none
```

After the subagent returns:
1. **If `STATUS: FAILED`** or no structured report: report the failure. The design baseline (Step 5) is already committed and intact; suggest re-running `/kiro-design {feature-name}` (it will resume from validation since design is already drafted) or running `/kiro-validate-design {feature-name}` standalone. Do NOT proceed to discussion.
2. **If `STATUS: DONE`**, resolve the **validation-report path** (`{validation-doc}`) deterministically — take the FIRST existing file in this fixed order inside `{specs-root}/{feature-name}/`:
   1. the exact path the subagent reported writing
   2. `design-validation.md`
   ```powershell
   Test-Path "{specs-root}/{feature-name}/design-validation.md"
   ```
   If none exists, warn once and treat the validation report as unavailable (the discussion can still run against design.md + requirements.md + research.md; the validation findings reported by the subagent are passed into the discussion verbatim instead).
3. A **NO-GO** decision does NOT stop the flow: the validation's critical issues are exactly what the design discussion (Step 7) resolves. Carry them forward.
4. Commit the validation report (no push):
   ```powershell
   git add -A
   git commit -m "docs({feature-name}): add design validation report"
   ```
   If nothing changed, skip the commit and note it.

### Step 7: Design discussion — interactive, in the chat window (controller)
This phase is **NOT delegated**: it requires turn-by-turn developer dialogue, and all Q&A happens in the **chat window** (natural conversational turns), NOT via `AskUserQuestion` popups. The controller runs it inline.

1. **Ingest the upstream documents** (the design-generation and design-validation outputs):
   - Read `{specs-root}/{feature-name}/design.md` in full.
   - Read `{specs-root}/{feature-name}/requirements.md` in full (for requirements-coverage checks).
   - Read the resolved `{validation-doc}` from Step 6 in full (or use the validation findings the subagent returned, if no file was written).
   - Read `{specs-root}/{feature-name}/research.md` in full (if present) and `{specs-root}/{feature-name}/spec.json` (confirm the discussion language) and the steering files under `.kiro/steering/`.
2. **Follow the discussion skill inline:** read `{skill-base}/kiro-design-discussion/SKILL.md` and execute its workflow (Phase 1–5) directly in the controller. When that skill references `gap-analysis.md`, substitute `{specs-root}/{feature-name}/research.md` (this repository records gap analysis and design decisions there). Seed the skill's Phase 2 issue collection with the critical issues from `{validation-doc}` so the validation findings are explicitly triaged.
3. **Seed the discussion with the OPEN QUESTIONS** retained from Step 4 — add them as Category B (developer-confirmation) topics, each recommending the assumption the subagent drafted (or a better one), so every assumption is explicitly confirmed or revised.
4. **Conduct the developer dialogue in the chat window:**
   - Category A (obvious fixes) is handled exactly as the discussion skill directs (Phase 2), committing per its convention (`docs({feature-name}): fix obvious issues in design`).
   - Category B (developer-facing questions) follows the skill's batch-proposal flow (Phase 3–4): first explain the spec's purpose with a concrete example or code (interface signatures, data structures, usage sketches — not implementation bodies), then write **all** topics at once in the chat, each with the current design, a recommendation and its rationale. Ask for the first bulk approval the way its Phase 3 says: through the desktop app's next-step box (`mcp__ccd_turn__wrap_up` with one `ask` and the two reply buttons 「推奨どおり承認」「一部を議論」) when that tool exists, with the full agenda in the closing message after that call; otherwise end the message with the same one question. The developer may click a button or answer in chat. If the developer approves in bulk, settle every topic on its recommendation in one commit (`docs({feature-name}): 全設計議題を推奨案で確定`). If the developer names specific topics to discuss, settle the unnamed topics on their recommendations (one commit), then discuss only the named topics one by one in the chat, updating the documents, committing per the skill's convention (`docs({feature-name}): resolve design discussion #{n} - {topic}`), and showing the progress summary after each. Never use `AskUserQuestion`.
   - If the developer stops responding or defers, summarize the remaining open topics and stop without guessing. Do not fabricate answers.
5. The discussion skill commits its own changes per category. The controller does not duplicate those commits.

### Step 8: Final verification and safety commit (no push)
1. Verify the spec state:
   ```powershell
   Test-Path "{specs-root}/{feature-name}/design.md"
   git status --porcelain
   ```
2. If `git status` shows any uncommitted spec changes left over from the discussion, commit them as a safety net (no push):
   ```powershell
   git add -A
   git commit -m "docs({feature-name}): finalize design discussion"
   ```
   If the tree is already clean, skip this commit.
3. Never push. Remote sync is the responsibility of `kiro-complete` (PR-based), not `kiro-design`.

## Important Constraints
- **Lightweight orchestration with one interactive exception**: The controller (main context) runs Step 0 (resolution), Step 1 (precondition gate), Step 2 (default-branch guard), Step 4 (verification), Step 5/6/8 (commits + validation-report resolution), and Step 7 (the interactive design discussion). Design generation (Step 3) and design validation (Step 6) are delegated to subagents. The design discussion (Step 7) is intentionally NOT delegated because it needs chat-window developer dialogue a subagent cannot perform. Do NOT run kiro-spec-design or kiro-validate-design inline in the controller.
- **No questions before discussion**: The controller asks the developer NOTHING during Steps 0–6. The subagents never ask the user; their open questions and validation concerns are carried forward and resolved only in Step 7.
- **Validation runs non-interactively**: `kiro-validate-design` is interactive by default, but the Step 6 subagent must NOT ask the user anything; it produces the GO/NO-GO verdict directly and persists it to disk so the discussion can ingest it.
- **Discussion Q&A in the chat window**: All developer-facing topics in Step 7 are written at once in the chat, each with a recommendation; the first bulk approval is asked through the next-step box when it exists (otherwise one question at the end of the message); bulk approval settles them all, otherwise only the topics the developer names are discussed one by one. Never use `AskUserQuestion`.
- **No branch/worktree creation, ever**: This skill does not create, switch, delete, reset, or force-update branches, and does not create worktrees. Feature branches are supplied by the Claude Code (harness) worktree feature.
- **Subagents never interact with the user**: The Step 3 and Step 6 subagents must not ask questions. The Step 3 subagent always returns a best-effort DRAFTED report (or SPEC_GAP for a genuine requirements gap); design ambiguities are recorded as OPEN QUESTIONS for the discussion phase.
- **Report in the user's language**: Think in English internally, but write every developer-facing message (progress narration, warnings, and the entire Step 7 discussion) in the spec's target language. Keep the internal subagent prompts (Step 3, Step 6) in English.
- **Deterministic resolution**: Step 0 resolves specs root, skill base, remote, and default branch with fixed priority orders; Step 6 resolves the validation-report path with a fixed priority order. Never guess; if a required value is unresolved, hard-fail as specified. Resolve each value once and reuse it.
- This skill is **post-requirements only**: if `{specs-root}/{feature-name}/requirements.md` does not exist, FAIL deterministically (do not create anything).
- Do NOT generate tasks. This skill stops after the design discussion.
- Do NOT re-derive or change the feature name; it is fixed (`$ARGUMENTS`).
- **STOP on the default branch**: If the current branch equals the resolved default branch, STOP in Step 2 (no phase, no commit) and ask the developer to re-run inside the harness worktree. Never push.
</instructions>

## Output Description
Provide output in the language specified in `spec.json` with the following structure:

1. **Feature Name**: `feature-name` (equals the argument)
2. **Project Summary**: Brief summary (1 sentence, sourced from requirements.md / design.md)
3. **Created/Updated Files**: Bullet list with full paths (`design.md`, the validation report `{validation-doc}`, `research.md` if updated)
4. **Branch Status**: On the normal path, report the current branch and that each phase was committed to it without pushing, e.g. `Committed design to current branch "{branch}" (no push)`. If the run STOPped because the current branch is the default branch, report only the STOP instead (see Safety & Fallback).
5. **Phase Status**:
   - Design: confirm `design.md` was generated and the subagent's design review gate passed.
   - Validation: confirm the validation report was written (give its path) and report the GO/NO-GO decision.
   - Discussion: summarize the discussion outcome — counts for Category A (obvious fixes) and Category B (developer-resolved), or note "no issues found" / "stopped with open topics".
6. **Next Step**: Command block showing `/kiro-spec-tasks <feature-name>` (or `/kiro-spec-tasks <feature-name> -y` to auto-approve the design). Design generation, validation, and the discussion are already complete.

**Format Requirements**:
- Use Markdown headings (##, ###)
- Wrap commands in code blocks
- Keep total output concise (under 320 words)
- Use clear, professional language per `spec.json.language`

## Safety & Fallback
- **Specs Root Unresolved (hard fail)**: If `.kiro/specs` does not exist, STOP and report that the repository is not cc-sdd initialized.
- **Skills Unresolved (hard fail)**: If any of `kiro-spec-design`, `kiro-validate-design`, or `kiro-design-discussion` SKILL.md is not found under `.claude/skills`, `.agents/skills`, or `.github/skills` (in that order), STOP and report that the required kiro skills are not installed.
- **Missing Spec Folder (hard fail)**: If `{specs-root}/{feature-name}/` does not exist, STOP immediately and report the error. Do not create the folder, branch, or any spec files. Suggest running `/kiro-start <feature>` first, or verifying the feature name against existing folders under `{specs-root}/`.
- **Missing Requirements (hard fail)**: If the folder exists but `requirements.md` is absent, STOP and report that requirements have not been generated; recommend running `/kiro-start {feature-name}` (or `/kiro-spec-requirements {feature-name}`) first. Do not fabricate requirements.
- **Design Generation Delegation**: All design-level behavior (discovery, synthesis, design review gate, research.md updates, spec.json metadata) is handled by `kiro-spec-design` inside the Step 3 subagent. Honor its results as reported by the subagent.
- **Subagent Failure (Step 3)**: If the subagent errors out or returns no structured report, do NOT commit. Report the failure and suggest re-running `/kiro-design {feature-name}` on the same branch. No branch was created, so there is nothing to clean up.
- **Clarifications Deferred (no questions before discussion)**: The Step 3 subagent never blocks on user input; it drafts best-effort with labelled assumptions and returns OPEN QUESTIONS. The controller does NOT ask these in Step 4 — it carries them into Step 7 and resolves them in the discussion.
- **Spec Gap (Step 4)**: If the subagent returns `STATUS: SPEC_GAP` (a genuine requirements gap or contradiction surfaced by the design review gate), do NOT commit; report it and recommend fixing `requirements.md` (e.g. via `/kiro-requirements-discussion {feature-name}`) before re-running `/kiro-design {feature-name}`.
- **Missing Outputs After DRAFTED**: If the subagent reports DRAFTED but `design.md` is missing or metadata is not updated (Step 4 verification fails), report the failure and do not commit.
- **Design Validation Delegation**: All review behavior (analysis, critical issues, GO/NO-GO) is handled by `kiro-validate-design` inside the Step 6 subagent, run non-interactively with the report persisted to disk. Honor its results as reported by the subagent.
- **Validation Failure (Step 6)**: If the Step 6 subagent returns `STATUS: FAILED` or errors out, report the failure. The design baseline (Step 5) is committed and intact. You MAY still proceed to the discussion against design.md + requirements.md + research.md, or suggest re-running `/kiro-validate-design {feature-name}` standalone — state which you chose.
- **NO-GO Decision**: A NO-GO verdict does not stop the flow; its critical issues feed the design discussion (Step 7), which is where they are resolved (and design.md updated).
- **Discussion Skill Behavior (Step 7)**: All issue collection, classification (A/B), obvious-fix application, and per-category commits are handled by `kiro-design-discussion` followed inline. The controller substitutes `research.md` for any `gap-analysis.md` reference and seeds issue collection with the validation report's critical issues and the Step 3 OPEN QUESTIONS. All developer Q&A is conducted in the chat window (never `AskUserQuestion`).
- **Discussion: Developer Unresponsive / Open Topics**: If the developer stops responding or defers Category B topics, summarize the remaining open topics and stop without guessing. Already-resolved topics and their commits remain on the branch; the rest can be resumed any time via `/kiro-design-discussion {feature-name}`.
- **On Default Branch (STOP)**: If the current branch equals `{default-branch}`, STOP in Step 2 before any phase/commit/push. Report that design work does not run on the default branch and ask the developer to re-run `/kiro-design {feature-name}` inside the Claude Code harness worktree (a non-default working branch). Do not create a branch or modify any files.
- **Non-Default Branch (normal path)**: Proceed through all three phases on the current branch and commit each (no push). Remote sync is deferred to `kiro-complete`.
- **No Remote**: If `{remote}` is none, warn once; all phases and the local commits still proceed (kiro-design never pushes or performs remote operations).
- **Commit Failure**: Report the error with the current branch name; the generated files remain present on the current branch.

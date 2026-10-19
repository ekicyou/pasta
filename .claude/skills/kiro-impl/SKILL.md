---
name: kiro-impl
description: Implement approved tasks using TDD with native subagent dispatch. Runs all pending tasks autonomously or selected tasks manually.
disable-model-invocation: true
allowed-tools: Read, Write, Edit, MultiEdit, Bash, Glob, Grep, Agent, WebSearch, WebFetch
argument-hint: <feature-name> [task-numbers] [--review required|inline|off]
---

# kiro-impl Skill

## Role
You operate in two modes:
- **Autonomous mode** (no task numbers): Dispatch a fresh subagent per task, with independent review after each
- **Manual mode** (task numbers provided): Execute selected tasks directly in the main context

## Core Mission
- **Success Criteria**:
  - All tests written before implementation code
  - Code passes all tests with no regressions
  - Tasks marked as completed in tasks.md
  - Implementation aligns with design and requirements
  - Task completion follows the selected review mode

## Review Mode
- Default review mode is `required`
- Accept explicit forms: `--review required|inline|off`
- Also accept clear natural-language opt-outs such as `skip review` or `without review` as `off`
- If the request is ambiguous, keep `required`

> **Branch & commit model (compatibility note):** kiro-impl operates on the single feature working branch supplied by the Claude Code harness worktree. It **only commits to the current branch** and does NOT create branches or push — branch creation is the harness's responsibility, and push happens later in the kiro-complete PR flow. This note documents existing behavior; it introduces no new branch/push logic.
>
> If `tasks.md` was not committed in a prior phase, there is **no dedicated task-phase commit**: the uncommitted `tasks.md` is picked up by the per-task commits below, which already include `tasks.md` in their selective staging (changed files + `tasks.md`). This stays within the existing selective-staging convention — it does **not** authorize `git add -A` or `git add .`.

## Execution Steps

### Step 1: Gather Context

Reuse steering/spec context already available from conversation; load missing context below.
Select skills for the current task even when steering/spec context is already available:
- `.kiro/specs/{feature}/spec.json` and `tasks.md` for approvals, task selection, and dependencies
- Referenced sections of `requirements.md` and `design.md`, expanding to related contracts as needed for the selected task
- Core steering context: `product.md`, `tech.md`, `structure.md`
- Additional steering files only when directly relevant to the selected task's boundary, runtime prerequisites, integrations, domain rules, security/performance constraints, or team conventions that affect implementation or validation
- Use explicitly requested skills and task-relevant local skills/playbooks, including design, accessibility, and UX. Select by description and read only needed guidance, even for small tasks; preserve required checks and host/project rules.

#### Preflight

**Validate approvals**:
- Verify tasks are approved in spec.json (stop if not, see Safety & Fallback)

**Discover validation commands**:
- Inspect repository-local sources of truth in this order: project scripts/manifests (`package.json`, `pyproject.toml`, `go.mod`, `Cargo.toml`, app manifests), task runners (`Makefile`, `justfile`), CI/workflow files, existing e2e/integration configs, then `README*`
- Derive a canonical validation set for this repo: `TEST_COMMANDS`, `BUILD_COMMANDS`, and `SMOKE_COMMANDS`
- Prefer commands already used by repo automation over ad hoc shell pipelines
- For `SMOKE_COMMANDS`, choose the lightest trustworthy runtime-liveness check for the app shape (for example: root URL load, Electron launch, CLI `--help`, service health endpoint, mobile simulator/e2e harness if one already exists)
- Keep the full command set in the parent context, and pass only the task-relevant subset to implementer and reviewer subagents

**Establish repo baseline**:
- Run `git status --porcelain` and note any pre-existing uncommitted changes

**Desk preflight and watch** (last Preflight step, both modes): Desk timing 1 in "Desk coordination (areka-impl-watch)" below.

### Step 2: Select Tasks & Determine Mode

**Parse arguments**:
- Extract feature name from first argument
- If task numbers provided (e.g., "1.1" or "1,2,3"): **manual mode**
- If no task numbers: **autonomous mode** (all pending tasks)
- Determine review mode from the invocation:
  - `--review required` or omitted → `required`
  - `--review inline` → `inline`
  - `--review off`, `skip review`, or `without review` → `off`

**Build task queue**:
- Read tasks.md, identify actionable sub-tasks (X.Y numbering like 1.1, 2.3)
- Major tasks (1., 2.) are grouping headers, not execution units
- Skip tasks with `_Blocked:_` annotation
- For each selected task, check `_Depends:_` annotations -- verify referenced tasks are `[x]`
- If prerequisites incomplete, execute them first or warn the user
- Use `_Boundary:_` annotations to understand the task's component scope

### Step 3: Execute Implementation

#### Autonomous Mode (subagent dispatch)

**Iteration discipline**: Process exactly ONE sub-task (e.g., 1.1) per iteration. Do NOT batch multiple sub-tasks into a single subagent dispatch. Each iteration follows the full cycle: dispatch implementer → review → commit → re-read tasks.md → next.

**Context management**: Re-read `tasks.md` before each iteration. Carry forward the task outcome, commit/evidence references, unresolved constraints, and relevant Implementation Notes; do not repeat completed worker transcripts in later handoffs.

**Delegation scope**: Use native subagents for these task-local steps, passing the focused inputs below rather than the full parent conversation. Independent feature/PR chats and their worktrees are managed through the host; do not create a standalone chat for each implementation or review step.

For each task (one at a time):

**Desk** (see "Desk coordination (areka-impl-watch)"): before each task apply Desk timing 2 (stop request, load-test detection by the desk criteria, desk-free tasks while waiting); on a `MISSING: ... desk` answer, a granted turn or a `DESK_RUN_DONE` answer Desk timing 3; on a stop request Desk timing 4, then Desk timing 5 on resume; Desk timings 8 and 9 whenever they occur.

**a) Dispatch implementer**:
- Read `templates/implementer-prompt.md` from this skill's directory
- Construct a prompt by combining the template with task-specific context:
  - Task description and boundary scope
  - Paths to spec files: requirements.md, design.md, tasks.md
  - Exact requirement and design section numbers this task must satisfy (using source numbering, NOT invented `REQ-*` aliases)
  - Task-relevant steering context and parent-discovered validation commands (tests/build/smoke as relevant)
  - Selected skill/playbook paths and concise task-relevant guidance, including required checks; inline necessary guidance when the worker cannot access those paths
  - Whether the task is behavioral (Feature Flag Protocol) or non-behavioral
  - **Previous learnings**: Include any `## Implementation Notes` entries from tasks.md that are relevant to this task's boundary or dependencies (e.g., "better-sqlite3 requires separate rebuild for Electron"). This prevents the same mistakes from recurring.
- The implementer subagent will read the spec files and build its own Task Brief (acceptance criteria, completion definition, design constraints, verification method) before implementation
- Preserve this task context, including selected skill guidance, on every implementer re-dispatch (context requests, review remediation, and debug retries); append the new context or feedback.
- Append the two desk sentences of "Desk coordination (areka-impl-watch)" verbatim to the end of every implementer prompt (only the first while desk steps are off)
- Dispatch via **Agent tool** as a fresh subagent

**Effort escalation** (local customization — keep when regenerating this skill): this skill explicitly sets the Agent tool's `effort` parameter as follows. Escalate only on these triggers; never lower effort below the session default.
- First implementer dispatch, `NEEDS_CONTEXT` re-dispatch, status/verdict format re-dispatch, and reviewer dispatch → omit `effort` (session default)
- Implementer re-dispatch after `REJECTED` (round 1-2) → `effort: "high"`
- Debug subagent (section g) → `effort: "xhigh"`
- New implementer after `NEXT_ACTION: RETRY_TASK` → `effort: "high"`

**b) Handle implementer status**:
- Parse implementer status only from the exact `## Status Report` block and `- STATUS:` field.
- If `STATUS` is missing, ambiguous, or replaced with prose, re-dispatch the implementer once requesting the exact structured status block only. Do NOT proceed to review without a parseable `READY_FOR_REVIEW | BLOCKED | NEEDS_CONTEXT` value.
- **READY_FOR_REVIEW** → proceed to review
- **BLOCKED** → dispatch debug subagent (see section below); do NOT immediately skip
- **NEEDS_CONTEXT** with `MISSING: exclusive-test desk` or `MISSING: load-test desks` → request the desk and continue the same subagent per Desk timing 3 (not counted as the one re-dispatch below, no effort escalation)
- **NEEDS_CONTEXT** with `DESK_RUN_DONE` → return the desk at once and, if red, continue fixing without it per Desk timing 3 (not counted as the one re-dispatch below, no effort escalation)
- **NEEDS_CONTEXT** → re-dispatch once with the requested additional context; if still unresolved → dispatch debug subagent

**c) Review the task**:
- If review mode is `required`:
  - Read `templates/reviewer-prompt.md` from this skill's directory
  - Resolve `../kiro-review/SKILL.md` relative to this skill's directory and pass its absolute path as `REVIEW_PROTOCOL_PATH`
  - Construct a review prompt with:
    - The task description and relevant spec section numbers
    - Paths to spec files (requirements.md, design.md) so the reviewer can read them directly
    - The implementer's status report (for reference only — reviewer must verify independently)
  - The reviewer must apply the `kiro-review` protocol to this task-local review.
  - Preserve the existing task-specific context: task text, spec refs, `_Boundary:_` scope, validation commands, implementer report, and the actual `git diff` as the primary source of truth.
  - Validation commands for the reviewer leave out UI-test and load-test steps unless the main conversation holds the desk for them at this handoff, and the reviewer prompt then carries the no-desk sentence ("Test commands for the reviewer" in "Desk coordination (areka-impl-watch)")
  - The reviewer subagent will run `git diff` itself to read the actual code changes and verify against the spec
  - Dispatch via **Agent tool** as a fresh subagent
- If review mode is `inline`:
  - Apply `kiro-review` in the parent context using the same task evidence and the actual `git diff`
- If review mode is `off`:
  - Skip task-local review
  - Record in the parent context that task-local review was skipped for this task

**d) Handle reviewer verdict**:
- If review mode is `off`:
  - Do not fabricate a reviewer verdict
  - Before marking the task `[x]` or making any success claim, apply `kiro-verify-completion` using fresh evidence from the current code state; then mark task `[x]` in tasks.md and perform selective git commit
- Otherwise:
  - Parse reviewer verdict only from the exact `## Review Verdict` block and `- VERDICT:` field.
  - If `VERDICT` is missing, ambiguous, or replaced with prose, re-dispatch the reviewer once requesting the exact structured verdict only. Do NOT mark the task complete, commit, or continue to the next task without a parseable `APPROVED | REJECTED` value.
  - **APPROVED** → before marking the task `[x]` or making any success claim, apply `kiro-verify-completion` using fresh evidence from the current code state; then mark task `[x]` in tasks.md and perform selective git commit
  - **REJECTED (round 1-2)** → re-dispatch implementer with review feedback (`effort: "high"`, see Effort escalation)
  - **REJECTED (round 3)** → dispatch debug subagent (see section below)

**e) Commit** (parent-only, selective staging):
- Stage only the files actually changed for this task, plus tasks.md
- **NEVER** use `git add -A` or `git add .`
- Use `git add <file1> <file2> ...` with explicit file paths
- Commit message format: `feat(<feature-name>): <task description>`

**f) Record learnings**:
- If this task revealed cross-cutting insights, append a one-line note to the `## Implementation Notes` section at the bottom of tasks.md

**g) Debug subagent** (triggered by BLOCKED, NEEDS_CONTEXT unresolved, or REJECTED after 2 remediation rounds):

- Read `templates/debugger-prompt.md` from this skill's directory
- Resolve `../kiro-debug/SKILL.md` relative to this skill's directory and pass its absolute path as `DEBUG_PROTOCOL_PATH`
- Supply the task brief/boundary, exact spec references, failure output, reviewer findings, current changed files/diff, and relevant Implementation Notes/runtime constraints
- Include attempted fixes and their observed results concisely, without copying the failed workers' conversation history
- Append the two desk sentences of "Desk coordination (areka-impl-watch)" verbatim to the end of the prompt (only the first while desk steps are off)
- Dispatch via **Agent tool** as a fresh subagent with `effort: "xhigh"` (see Effort escalation); it reads the canonical `kiro-debug` procedure itself

**Handle debug report**:
- Require a `## Debug Report` and parse its exact `- NEXT_ACTION:` field (`RETRY_TASK | BLOCK_TASK | STOP_FOR_HUMAN`). If the protocol or a valid report is missing, stop this feature (Desk timing 7) and report the missing input; do not guess a next action or dispatch another implementer.
- If `NEXT_ACTION: STOP_FOR_HUMAN` → append `_Blocked: <ROOT_CAUSE>_` to tasks.md, stop the feature run (Desk timing 7), and report that human review is required before continuing
- If `NEXT_ACTION: BLOCK_TASK` → append `_Blocked: <ROOT_CAUSE>_` to tasks.md, skip to next task
- If `NEXT_ACTION: RETRY_TASK` → preserve the current worktree; do NOT reset or discard unrelated changes. Spawn a **new** implementer subagent (`effort: "high"`, see Effort escalation) with the original task context (including selected skill guidance), the debug report's `FIX_PLAN`, `NOTES`, and the current `git diff`, and require it to repair the task with explicit edits only
  - If the new implementer succeeds (READY_FOR_REVIEW → reviewer APPROVED) → normal flow
  - If the new implementer also fails → repeat debug cycle (max 2 debug rounds total). After 2 failed debug rounds → append `_Blocked: debug attempted twice, still failing — <ROOT_CAUSE>_` to tasks.md, skip
- **Max 2 debug rounds per task**. Each round: fresh debug subagent → fresh implementer. If still failing after 2 rounds, the task is blocked.
- Record debug findings in `## Implementation Notes` (this helps subsequent tasks avoid the same issue)

**`(P)` markers**: Tasks marked `(P)` in tasks.md indicate they have no inter-dependencies and could theoretically run in parallel. However, kiro-impl processes them sequentially (one at a time) to avoid git conflicts and simplify review. The `(P)` marker is informational for task planning, not an execution directive.

**Completion check**: If all remaining tasks are BLOCKED, run Desk timing 7 (`leave`), then stop and report blocked tasks with reasons to the user.

#### Manual Mode (main context)

For each selected task (the same Desk timing 2, 3, 4, 5, 8 and 9 as in Autonomous mode apply; with no implementer, the main conversation requests the load-test desks before starting a task that contains a load test (timing 2) and the exclusive-test desk before entering a UI test that stands up areka, runs the step once after the turn, and runs `return` as soon as that run ends, per the Manual-mode bullet of timing 3):

**1. Build Task Brief**:
Before writing any code, read the relevant sections of requirements.md and design.md for this task and clarify:
- What observable behaviors must be true when done (acceptance criteria)
- What files/functions/tests must exist (completion definition)
- What technical decisions to follow from design.md (design constraints)
- How to confirm the task works (verification method)

**2. Execute TDD cycle** (Kent Beck's RED → GREEN → REFACTOR):
- **RED**: Write test for the next small piece of functionality based on the acceptance criteria. Test should fail.
- **GREEN**: Implement simplest solution to make test pass, following the design constraints.
- **REFACTOR**: Improve code structure, remove duplication. All tests must still pass.
- **VERIFY**: All tests pass (new and existing), no regressions. Confirm verification method passes.
- **REVIEW**:
  - `required`: Apply `kiro-review` before marking the task complete. If the host supports fresh subagents in manual mode, use a fresh reviewer; otherwise perform the review in the main context using the `kiro-review` protocol. Do NOT continue until the verdict is parseably `APPROVED`.
  - `inline`: Apply `kiro-review` in the main context before marking the task complete.
  - `off`: Skip task-local review, but note that `kiro-validate-impl` becomes the primary quality gate before any feature-level completion claim.
- **MARK COMPLETE**:
  - `required|inline`: Only after review returns `APPROVED`, apply `kiro-verify-completion`, then update the checkbox from `- [ ]` to `- [x]` in tasks.md.
  - `off`: Apply `kiro-verify-completion`, then update the checkbox from `- [ ]` to `- [x]` in tasks.md.

### Step 4: Final Validation

**Desk**: before this step apply Desk timing 6 (not stopped, holding no desk). When the run ends in any way (GO, NO-GO stop, MANUAL_VERIFY_REQUIRED stop, STOP_FOR_HUMAN, missing debug report stop, BLOCKED, developer abandon, or all tasks BLOCKED) apply Desk timing 7 (`leave`), in both modes.

**Autonomous mode**:
- After all tasks complete, run `/kiro-validate-impl {feature}` as a GO/NO-GO gate
- If validation returns GO → before reporting feature success, apply `kiro-verify-completion` to the feature-level claim using the validation result and fresh supporting evidence
- If validation returns NO-GO:
  - Fix only concrete findings from the validation report
  - Cap remediation at 3 rounds; if still NO-GO, stop (Desk timing 7) and report remaining findings
- If validation returns MANUAL_VERIFY_REQUIRED → stop (Desk timing 7) and report the missing verification step

**Manual mode**:
- Suggest running `/kiro-validate-impl {feature}` but do not auto-execute
- If review mode is `off`, treat `/kiro-validate-impl {feature}` as mandatory before any feature-level success claim

## Feature Flag Protocol

For tasks that add or change behavior, enforce RED → GREEN with a feature flag:

1. **Add flag** (OFF by default): Introduce a toggle appropriate to the codebase (env var, config constant, boolean, conditional -- agent chooses the mechanism)
2. **RED -- flag OFF**: Write tests for the new behavior. Run tests → must FAIL. If tests pass with flag OFF, the tests are not testing the right thing. Rewrite.
3. **GREEN -- flag ON + implement**: Enable the flag, write implementation. Run tests → must PASS.
4. **Remove flag**: Make the code unconditional. Run tests → must still PASS.

**Skip this protocol for**: refactoring, configuration, documentation, or tasks with no behavioral change.

## Desk coordination (areka-impl-watch)

Parallel sessions share the desks of the console app `areka-impl-watch.exe`: the **implementation desk** (one per machine; every working participant is on it, and a load test can hold it alone as the **sole desk**, which stops everyone else), the **exclusive-test desk** (one per machine; one holder at a time; in pasta taken only for UI tests that stand up areka; it never stops anyone) and the **merge desk** (one per repository; `kiro-complete` takes it, this skill never does). The app's state is the source of truth; this section is the convention for calling it. The app and its runbook live in the areka repository (runbook: `C:\home\maz\git\areka\doc\impl-watch.md`); pasta only calls the installed exe.

- **Only the main conversation calls desk commands.** Implementer, reviewer, debugger and validator subagents never run `areka-impl-watch.exe` (one session = one participant).
- **One session per worktree uses the desk.** Two sessions in the same worktree get the same id and remove each other's watch.
- The desk is a helper. When it cannot be used, skip the desk steps for this run only and keep working; never let the desk stop implementation.
- Hold the exclusive-test desk only while a step that needs it runs, never while implementing or fixing. Other sessions keep implementing while you wait for it or hold it.

### Desk criteria (pasta's version of section 1 "判定の基準" of `C:\home\maz\git\areka\doc\impl-watch.md`)

pasta narrows the first row to UI tests that stand up areka: pasta's own full test (`cargo test --all`) takes no exclusive-test desk (developer decision 2026-10-19).

| Step | Desk to take |
|---|---|
| A UI test that stands up areka or its windows on the real desktop | exclusive-test desk (`--desk test`) |
| Quiet-CPU measurement, deliberate CPU-load reproduction, timing comparison (a load test) | exclusive-test desk and sole desk (`--desk test,sole`) |
| Everything else: `cargo test --all`, `cargo test -p <crate>`, the full test run by `/kiro-validate-impl`, SSP real-machine checks, clippy, format checks, builds | none (they run on the implementation desk) |

Decide from the task text, its `_Boundary:_` and the design's verification description whether a task contains a step of the first or the second row.

### Desk preflight (one foreground Bash, once at the start of the run)

```bash
ID=$(basename "$(git rev-parse --show-toplevel)"); R=$(basename "$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")")
if [ -z "$AREKA_IMPL_WATCH_HOME" ]; then echo "desk: skip no-home"
elif [ ! -f "$AREKA_IMPL_WATCH_HOME/areka-impl-watch.exe" ]; then echo "desk: skip no-exe"
elif ! [[ "$ID" =~ ^[A-Za-z0-9._-]{1,100}$ && "$R" =~ ^[A-Za-z0-9._-]{1,100}$ ]]; then echo "desk: skip bad-name $ID $R"
else echo "desk: id=$ID repo=$R"; fi
```

- `<ID>` (`--id`) is the name of the worktree's top folder, so a new Claude session in the same worktree gets the same id.
- `<R>` (`--repo`) is the name of the folder that holds the common git folder (`.git`); it is the same from every worktree of the repository. Do not use the `origin` URL (it may carry credentials).
- `--name` (and `merge --spec`) is the spec name `<feature>`.
- `desk: skip no-home|no-exe|bad-name`: do not make up other values. Remember the reason, tell the developer one Japanese line, keep it in the final report, and call no desk command for the rest of the run:
  - `no-home`: `机の段を飛ばしました（環境変数 AREKA_IMPL_WATCH_HOME が空）`
  - `no-exe`: `机の段を飛ばしました（置き場所に areka-impl-watch.exe が無い）。置くには: <install command below>`
  - `bad-name`: `机の段を飛ばしました（作業木かリポジトリの名前が識別の形に合わない: <ID> <R>）`
- `desk: id=<ID> repo=<R>`: remember both values and write them literally into every later command (Bash variables do not survive between commands). If they are lost to context compaction, run the preflight again (it prints the same values).

### Call form (every desk command)

```bash
"$AREKA_IMPL_WATCH_HOME/areka-impl-watch.exe" <command> <args> 2>&1; echo "exit=$?"
```

- `2>&1` puts the one stderr line (`gone:`, `not applied:`, `already running`, refusals) into the same output. Read only the last `exit=<n>` and the one or two lines before it.
- **Background** (Bash `run_in_background`): `watch`, `merge --wait`, `request --wait`, `stopped --wait`, `resume`. When starting one, remember its kind and its run id ("own waits").
- **Foreground**: `leave`, `merged`, `return`, `cancel`, `loadrunning`, and `status` only when investigating a reason.
- The desk requests and the return this skill makes:
  ```bash
  "$AREKA_IMPL_WATCH_HOME/areka-impl-watch.exe" request --id <ID> --repo <R> --desk test --purpose "<what>" --wait 2>&1; echo "exit=$?"
  "$AREKA_IMPL_WATCH_HOME/areka-impl-watch.exe" request --id <ID> --repo <R> --desk test,sole --purpose "<what, roughly how many minutes>" --wait 2>&1; echo "exit=$?"
  "$AREKA_IMPL_WATCH_HOME/areka-impl-watch.exe" return --id <ID> 2>&1; echo "exit=$?"
  ```
  The turn comes as `granted test` or `granted test,sole; stopped: <ids>` (`stopped: none` when nobody had to stop). `return` without `--desk` returns every desk you hold but the merge desk (this skill never holds one) and prints `returned <desks>`.

### How each ending is handled

| Ended | Output | Do |
|---|---|---|
| watch | `exit=0`, `stop requested by <X>` | Stop request. Run `stopped --wait` at the stop boundary (Desk timing 4) |
| watch | `exit=0`, `already stopped; ...` | Do not record "stopped" again; run `resume` in the background |
| watch | `exit=1`, `AREKA_IMPL_WATCH_HOME ...` | Stop the desk steps for this run; one line with the stderr line as is; keep it in the final report |
| watch | `exit=1`, `a watch wait for <ID> is already running` | If your own watch is running, do nothing. If you have no record of one, run the leftover-watch removal one-liner below in the background and tell one line. If you held a request, request again before that one-liner ends |
| watch | `exit=1`, anything else | Start the watch again once, at once. If the restart ends with the same `exit=1` line, follow the exit-1 rows below; if still broken, stop the desk steps for this run and keep it in the final report |
| watch | `exit=3` | If it is not the current watch (the last watch run started), do nothing. If it is the current watch and participation has ended (run-end `leave`), do nothing. Otherwise tell one line, start the watch again and re-submit any request you held (if you held a desk, Desk timing 8) |
| resume wait (`stopped --wait`, `resume`) | `exit=0`, `resumed` | Start the watch again, then continue from the step after where you stopped (Desk timing 5) |
| resume wait | `exit=0`, `stop requested again by <X>` | Do not go back to work; run `stopped --wait` again |
| resume wait | `exit=3` | Start the watch again and continue |
| resume wait | `exit=1`, `a resume wait for <ID> is already running` | The earlier resume wait is still running. Do nothing |
| `request --wait` | `exit=0`, `granted test` / `granted test,sole; stopped: ...` | The turn came (Desk timing 3) |
| `request --wait` | `exit=3`, `not applied: desks are held; only a merge holder may add the test desk` | You requested while holding a desk. The state did not change. Do not read `status`: `return` the desks you hold, then request again; tell one line |
| `request --wait` | `exit=3`, `not applied: a different desk request is already waiting` | A request for another desk set is waiting under this id. The state did not change. Do not read `status`; `return` removes no request. If you remember starting that request yourself (e.g. the load-test request of Desk timing 2), do not request again (it is refused again): wait for its turn, finish that turn per Desk timing 3 including its `return`, then request again; tell one line. If you have no record of one, it is a leftover of an earlier session in this worktree (same id) and its turn would hold the desk with nobody to return it: `cancel --id <ID>` in the foreground, request again at once, one line (`cancel` also drops any merge desk or merge request under this id; this skill holds none, and only one session per worktree uses the desk) |
| `request --wait` | `exit=3`, `not applied: a merge request is waiting; ...` | This skill never requests the merge desk, so another session in this worktree or a leftover record is waiting for it under this id. The state did not change. Do not `cancel` it (that would drop the other session's merge request); tell one line and wait for the developer's instruction |
| `request --wait` | `exit=3`, `gone: request gone` | The request was dropped (cancel, run end, reclaimed, cleared). If you ran `cancel` yourself, do nothing. Otherwise tell one line, start the watch again first, then request again (if you held a desk, Desk timing 8) |
| `request --wait` | `exit=1`, `a request wait for <ID> is already running` | If your own wait is running, do nothing. If you have no record of one: `cancel --id <ID>`, request again, one line |
| `return` | `exit=3`, `not applied: not the holder of that desk` / `not applied: holds no desk to return` | The desk is no longer yours (e.g. the record was cleared). Do not return it again; tell one line and continue |
| `return` | `exit=3`, `not applied: a running load test returns test and sole together` | Call `return --id <ID>` again without `--desk` |
| `loadrunning` | `exit=3`, `not applied: the test desk or the sole desk already has a holder` / `not applied: a merge request is waiting; ...` | You did not become the holder. Tell one line and wait for the developer's instruction without recording that measurement |
| `leave`, `merged`, `return`, `cancel`, `loadrunning` | `exit=1`, other than the two rows below | Tell the stderr line as is and follow the exit-code-1 table in section 5 of `C:\home\maz\git\areka\doc\impl-watch.md`. If still broken, stop the desk steps for this run and keep it in the final report |
| any | `exit=1`, `state.lock is busy ...` | Call once more right away (the exe already waits 10 seconds before refusing; do not wait on the skill side). If it repeats, stop the desk steps for this run and keep it in the final report |
| any | `exit=2` | Usage error, i.e. an error in this skill's text. Stop the desk steps for this run and keep it in the final report |

- Restarts and re-submissions happen once each, even when two waits end back to back for the same event. A second one ending in `already running` is harmless.
- Read endings by their output text. Your memory of "is this my watch / wait / request" decides only the own-or-leftover split of `already running` and `a different desk request is already waiting`.

**Removing a leftover watch**: a leftover watch from an earlier session keeps holding this id's watch lock until its next poll (every second) sees that its record is gone after `leave`; an immediate restart would hit that lock. So remove, wait and restart in one background Bash, and wake the session only once at its end:

```bash
"$AREKA_IMPL_WATCH_HOME/areka-impl-watch.exe" leave --id <ID> 2>&1; sleep 3; "$AREKA_IMPL_WATCH_HOME/areka-impl-watch.exe" watch --id <ID> --repo <R> --name <feature> 2>&1; echo "exit=$?"
```

- `sleep 3` is three poll intervals. Read only the last `exit=` and the watch lines (not the `leave` line). Remember this run as the current watch.
- This `leave` does not count as ending participation (only the run-end `leave` does).
- If this one-liner ends in `already running` again, do not repeat it; stop the desk steps for this run and keep it in the final report.

### What the conversation remembers

- Desk steps `on` (`id`, `repo`) or `off` (reason).
- The run id of the current watch: the last watch run started (including the leftover-removal one-liner). An `exit=3` of an ended watch is handled only when it is the current watch; endings of older watches are read and ignored.
- Stop state (none / stop requested / stopped), the exclusive-test desk (none / waiting / held) and the sole desk (none / waiting / held).
- Which subagent (or Manual-mode step) a desk request is for, so the turn goes back to it, and the subagents still waiting for that desk behind it (Desk timing 3).
- Whether participation has ended (after the run-end `leave`; the `leave` inside the leftover-removal one-liner does not count).
- The app's state is the source of truth. If this memory is lost, the preflight and the ending table are enough to recover; only the own-or-leftover split of `already running` and `a different desk request is already waiting` is lost, and then treat it as a leftover.

### Desk timings (Autonomous and Manual)

The execution steps point here as "Desk timing N". Every command uses the call form above. While desk steps are `off`, skip all of them.

1. **End of Preflight**: run the desk preflight. On `desk: skip ...`, follow its bullet above and go on without the desk. On `desk: id=<ID> repo=<R>`, before the first task start the watch in the background:
   ```bash
   "$AREKA_IMPL_WATCH_HOME/areka-impl-watch.exe" watch --id <ID> --repo <R> --name <feature> 2>&1; echo "exit=$?"
   ```
2. **Before each task**:
   - If a stop request has come, do not start the next task; go to timing 4.
   - Decide with the desk criteria whether the task contains a load test (second row). If it does, request `--desk test,sole --purpose "<what, roughly how many minutes>" --wait` in the background and do not hand over (dispatch, or in Manual mode start) that task until the turn comes.
   - If the task contains only first-row steps (UI tests that stand up areka), hand it over without requesting: no desk is held while implementing. The implementer stops before such a step and asks for the desk (timing 3).
   - While waiting for a turn, continue tasks whose dependencies are done and that need no desk. A task needs no desk only if, by the desk criteria, it contains neither a first-row step (UI test that stands up areka) nor a second-row step (load test): a task that may come back with `MISSING: exclusive-test desk` is not desk-free while a request is waiting (its request would be refused with `a different desk request is already waiting`). If none are left, tell one line `負荷テストの机の番待ち` (or `排他テストの机の番待ち` when waiting for `--desk test`) and end the turn.
3. **Desk requests and turns**:
   - Implementer or debugger returns `NEEDS_CONTEXT` with `MISSING: exclusive-test desk: <what>`: request `--desk test --purpose "<what>" --wait` in the background. On `granted test`, tell the same subagent to continue (SendMessage) with the line `The exclusive-test desk is held.`; if it cannot be continued, re-dispatch with the original prompt and that line added.
   - `MISSING: load-test desks: <what>`: the same with `--desk test,sole --purpose "<what, roughly how many minutes>"` and the line `The exclusive-test desk and the sole desk are held.`
   - Load-test turn requested in timing 2 (`granted test,sole; stopped: ...`): hand that task over with the line `The exclusive-test desk and the sole desk are held.` added to the prompt.
   - A `granted test,sole` that came while you were stopped: start the watch again once before handing over.
   - As soon as the subagent holding a desk comes back, with `DESK_RUN_DONE: <result>` or any other answer, pass or fail, run `return --id <ID>` in the foreground. Never keep a desk while fixing. If the run was red, tell the same subagent to continue fixing without the desk (SendMessage; re-dispatch only if it cannot be continued); the rerun asks for the desk again through `MISSING`. Each red run costs two round trips.
   - One desk request at a time: a `MISSING` that comes while a request or a held desk is already for another subagent is not requested yet (a same-set request would be accepted without a turn of its own); remember that subagent as waiting, and request for it right after the first one's `return`.
   - These requests, continuations and re-dispatches count neither as the one `NEEDS_CONTEXT` re-dispatch nor as an effort escalation.
   - Manual mode (the main conversation runs the task): the load-test desks are requested before the task (timing 2); request the exclusive-test desk before entering a first-row step. Run the step once after the turn comes, and `return` as soon as the run ends, pass or fail; fix without the desk and request again before rerunning.
4. **Stop request** (watch `stop requested by <X>`): carry the current task through implementer → review → verification → `[x]` → commit; never cut a running test or subagent. A first-row step inside that task still takes the exclusive-test desk through timing 3. At that commit boundary, if you hold the exclusive-test desk, `return` it first; then run `stopped --id <ID> --wait` in the background, tell one line `負荷テストのために止まった（停止要請: <X>）` and end the turn. If the next thing to do is a load-test request, run `stopped --wait` first and request after it. If no work is in progress (e.g. the turn ended waiting for a desk), you are already at the boundary: run `stopped --wait` at once.
5. **Resume**: per the ending table. On `resumed`, start the watch again, then go on to the next task without waiting for developer input.
6. **Step 4 (final validation)**: before it, confirm you are not stopped (no stop request pending) and hold no desk; otherwise finish timing 3 or 4 first. Then call `/kiro-validate-impl` without a desk (in pasta its full test is not a first-row step).
7. **Run end**, any end of the run (GO, NO-GO stop, MANUAL_VERIFY_REQUIRED stop, STOP_FOR_HUMAN, missing debug report stop, BLOCKED, developer abandon, or all tasks BLOCKED): run `leave` in the foreground (it also drops any desk and request you still have). An `exit=3` of a watch or wait that ends after it needs nothing. While waiting for completion approval the session is not a participant.
   ```bash
   "$AREKA_IMPL_WATCH_HOME/areka-impl-watch.exe" leave --id <ID> 2>&1; echo "exit=$?"
   ```
8. **`exit=3` of the watch or the turn wait while holding the exclusive-test desk or the sole desk**: do not start the next run; start the watch, request the same desks again, and continue only after the turn comes.
9. **Only when the developer says to record a load test that is already running**: `loadrunning --id <ID> --repo <R> --purpose "<what>"` in the foreground, and `return --id <ID>` when it is done.

**Test commands for the reviewer**: give the reviewer a first- or second-row step in `TEST_COMMANDS` / `SMOKE_COMMANDS` only if the main conversation holds the exclusive-test desk (or both desks) at the time of that handoff. Normally leave those steps out; the reviewer reads the evidence of the implementer's desk-held run (`DESK_RUN_DONE`). When you hold no desk at that handoff, add this sentence to the reviewer prompt (`kiro-review` stays unchanged): `Do not run UI tests that stand up areka, or load tests; the controller holds no desk for them now, so read the implementer's DESK_RUN_DONE evidence instead.`

**Two sentences for the implementer and the debugger**: on every dispatch (re-dispatches included), append them verbatim to the end of the prompt. While desk steps are `off`, append only the first: no desk exists this run, so UI tests that stand up areka and load tests go ahead without one. The templates stay unchanged.
- `Do not run areka-impl-watch.exe; the controller owns the desk commands.`
- `Run a UI test that stands up areka or its windows on the real desktop only when this prompt says the exclusive-test desk is held, and a load test (quiet-CPU measurement, deliberate CPU-load reproduction, timing comparison) only when it says the exclusive-test desk and the sole desk are held; everything else (cargo test --all, crate tests, SSP checks, clippy, format checks, builds) needs no desk. If the task needs a desk that is not held, stop before that step and report NEEDS_CONTEXT with "MISSING: exclusive-test desk: <what>" or "MISSING: load-test desks: <what and roughly how long>". When a desk is said to be held, run the step that needs it once and come back at once, pass or fail: even when it is red, do not fix and rerun while holding the desk, but report NEEDS_CONTEXT with "DESK_RUN_DONE: <result summary>".`

### Never

- Never set, change or rewrite `AREKA_IMPL_WATCH_HOME`: no `/c/...` conversion, no `cd`, no `AREKA_IMPL_WATCH_HOME=...` prefix before a command. Use the inherited value as is.
- Never build, copy or replace the exe during desk steps. Never edit or delete files under the desk folder by hand.
- Never stop another session's watch or wait processes (no `kill` by guess). Leftovers are removed from the record with `leave` / `cancel` and end on their own.
- Never call `clear` (the whole-state clear).
- No polling, no timed loops, no periodic `status`. In the normal flow read neither `status` nor `status.md`; the session wakes only when a wait command ends.
- One line to the developer per desk event. Waiting for a desk turn or for a resume is not a stop for approval.

### When the exe is missing (`desk: skip no-exe`)

Point the developer at the install tool. It lives in the areka repository (not in pasta), so write its fixed absolute path:

```
pwsh -NoProfile -File C:\home\maz\git\areka\tools\impl-watch-install.ps1
```

Add that it should be run in an areka working copy that has taken in the latest default branch. The skill only shows this command and never runs it.

## Critical Constraints
- **Strict Handoff Parsing**: Never infer implementer `STATUS` or reviewer `VERDICT` from surrounding prose; only the exact structured fields count
- **No Destructive Reset**: Never use `git checkout .`, `git reset --hard`, or similar destructive rollback inside the implementation loop
- **Selective Staging**: NEVER use `git add -A` or `git add .`; always stage explicit file paths
- **Bounded Review Rounds**: Max 2 implementer re-dispatch rounds per reviewer rejection, then debug
- **Bounded Debug**: Max 2 debug rounds per task (debug + re-implementation per round); if still failing → BLOCKED
- **Bounded Remediation**: Cap final-validation remediation at 3 rounds

## Output Description

**Autonomous mode**: For each task, report:
1. Task ID, implementer status, reviewer verdict
2. Files changed, commit hash
3. After all tasks: final validation result (GO/NO-GO)

**Manual mode**:
1. Tasks executed: task numbers and test results
2. Status: completed tasks marked in tasks.md, remaining tasks count

**Format**: Concise, in the language specified in spec.json.

## Safety & Fallback

### Error Scenarios

**Tasks Not Approved or Missing Spec Files**:
- **Stop Execution**: All spec files must exist and tasks must be approved
- **Suggested Action**: "Complete previous phases: `/kiro-spec-requirements`, `/kiro-design`, `/kiro-spec-tasks`"

**Test Failures**:
- **Stop Implementation**: Fix failing tests before continuing
- **Action**: Debug and fix, then re-run

**All Tasks Blocked**:
- Run Desk timing 7 (`leave`), then stop and report all blocked tasks with reasons
- Human review needed to resolve blockers

**Spec Conflicts with Reality**:
- If a requirement or design conflicts with reality (API doesn't exist, platform limitation), block the task with `_Blocked: <reason>_` -- do not silently work around it

**Upstream Ownership Detected**:
- If review, debug, or validation shows that the root cause belongs to an upstream, foundation, shared-platform, or dependency spec, do not patch around it inside the downstream feature
- Route the fix back to the owning upstream spec, keep the downstream task blocked until that contract is repaired, and re-run validation/smoke for dependent specs after the upstream fix lands

**Task Plan Invalidated During Implementation**:
- If debug returns `NEXT_ACTION: STOP_FOR_HUMAN` because of task ordering, boundary, or decomposition problems, stop and return for human review of `tasks.md` or the approved plan instead of forcing a code workaround

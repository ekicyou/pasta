---
name: kiro-tasks
description: 'Tasks-phase entry point that wraps /kiro-spec-tasks {feature} -y and consolidates the completed spec-planning work onto the default branch, then opens a fresh implementation branch. Acts as a lightweight orchestrator: the controller handles deterministic resolution, the precondition gate, output verification, the commit, the squash-merge integration, and impl-branch creation, while delegating tasks generation to a single subagent. Portable across repos: deterministically resolves the specs root, skill base directory, git remote, and default branch (fixed priority orders, no guessing). Generates tasks.md via kiro-spec-tasks with auto-approve, commits the work, and — when the current branch is NOT the default branch — squashes branch A into a new branch B at the merge-base, merges the default branch into B, fast-forwards the default branch to B, pushes, deletes A and B, and finally creates the implementation branch impl/{feature}. Use when: 設計承認後にタスクを生成して実装ブランチを切りたい, kiro-tasks, タスク生成して実装に入る, tasks生成しimplブランチ作成, generate tasks and start implementation. DO NOT USE FOR: 設計やrequirementsの生成 (use /kiro-spec-design or /kiro-spec-requirements), 実装そのもの (use /kiro-impl), spec完了アーカイブ (use /kiro-complete), 複数仕様の一括生成 (use /kiro-spec-batch).'
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Agent, AskUserQuestion
argument-hint: <feature-name>
---

# Spec Tasks (Tasks + Squash-Integrate + Impl Branch)

<instructions>
## Core Task
Finish the planning phase of a single specification and hand off to implementation. Generate `tasks.md` by wrapping `kiro-spec-tasks {feature} -y` (auto-approve), commit the resulting work, and — when the current branch is **not** the repository's default branch — consolidate the entire spec branch into a single commit on the default branch via a squash-merge integration, then create the implementation branch `impl/{feature-name}`. The feature name equals `$ARGUMENTS` and is passed **verbatim** to `kiro-spec-tasks`.

This skill acts as a **lightweight orchestrator**, sharing its design strategy with `kiro-start`:
- **Think in English, report in the user's language** (see Communication Language).
- **Portable across repositories** — never hard-code skill paths, remote name, or default branch; resolve each via a deterministic, ordered procedure (Step 0). Each resolution yields exactly one result or a hard failure — never an ambiguous guess.
- **Deterministic over heuristic** — resolve each value once and reuse it; do not re-evaluate later.
- **Controller stays lightweight** — the controller performs only deterministic, state-sensitive orchestration (resolution, precondition gate, verification, git operations, user clarification). The heavy tasks-generation work is delegated to a single subagent (Step 2) so the controller context stays small. The subagent never interacts with the user.

## Communication Language
- **Think in English, report in the user's language.** Internal reasoning, planning, and tool orchestration may be in English, but every message surfaced to the developer MUST be written in the target language configured for this spec.
- **Resolve the report language** from `{specs-root}/{feature-name}/spec.json` (`language` field). If unreadable, fall back to the language of the user's input (default `ja` for this repository). Use this same language for the final Output Description.
- This applies to ALL developer-facing text emitted by the controller: orchestration progress narration ("tasks generated", "squashing branch", "integrating into main", "impl branch created"), warnings, and any `AskUserQuestion` prompts/options.
- The Step 2 **subagent prompt itself stays in English** (it is internal instruction, not user-facing). Translate only the controller's own narration.
- `$ARGUMENTS` is the **confirmed kiro feature name**. Pass it **verbatim** as the first parameter of `kiro-spec-tasks`. Do NOT re-derive or change it.

## Execution Steps

### Step 0: Resolve portable context (deterministic)
Resolve these four values once, in order. Each has a single deterministic outcome; if a required value cannot be resolved, fail as specified.

1. **Specs root** (`{specs-root}`): Use the first existing directory in this fixed priority order:
   1. `.kiro/specs`
   If `.kiro/specs` does not exist, STOP (hard fail): this repository is not cc-sdd initialized.

2. **Skill base directory** (`{skill-base}`): Locate the directory that contains `kiro-spec-tasks/SKILL.md`, checking this fixed priority order and taking the FIRST match:
   1. `.claude/skills`
   2. `.agents/skills`
   3. `.github/skills`
   If `kiro-spec-tasks/SKILL.md` is not found under any of the three bases, STOP (hard fail): the required kiro skill is not installed.

3. **Default remote** (`{remote}`): Run `git remote`. Apply this fixed rule:
   - If `origin` is present → `{remote}` = `origin`.
   - Else if exactly one remote exists → `{remote}` = that remote.
   - Else (no remotes, or multiple without `origin`) → `{remote}` = none; treat all remote operations as skipped (warn once).

4. **Default branch** (`{default-branch}`): Determine deterministically:
   - If `{remote}` is set, read `git symbolic-ref --quiet --short refs/remotes/{remote}/HEAD` and strip the `"{remote}/"` prefix.
   - If that yields nothing and a local `main` branch exists → `{default-branch}` = `main`.
   - Else if a local `master` branch exists → `{default-branch}` = `master`.
   - Else → `{default-branch}` = the current branch (the squash-integration in Step 5 will be skipped because current == default).

   Record `{default-branch}` as a single concrete name before proceeding. Do not re-evaluate it later.

### Step 1: Verify precondition (hard gate)
`kiro-spec-tasks` requires an existing spec with requirements and design. Verify before doing anything mutating:
```powershell
Test-Path "{specs-root}/{feature-name}/spec.json"
Test-Path "{specs-root}/{feature-name}/requirements.md"
Test-Path "{specs-root}/{feature-name}/design.md"
```
- **If any of the three is missing: STOP.** Do NOT generate tasks, commit, or touch branches. Report which artifact is missing and recommend the correct upstream command (`/kiro-spec-requirements {feature}` or `/kiro-spec-design {feature}`).
- Only when all three exist, proceed.

### Step 2: Delegate tasks generation to a subagent (orchestration)
Dispatch **one** subagent via the Agent tool to run `kiro-spec-tasks` with auto-approve so the controller context stays lightweight. Pass the resolved values from Step 0 (`{specs-root}`, `{skill-base}`, `{feature-name}` = `$ARGUMENTS`) into the prompt.

Use this subagent prompt:
```
You are running the tasks-generation phase for the confirmed kiro feature "{feature-name}".
The feature name is FINAL — do not re-derive or change it.

1. Read {specs-root}/{feature-name}/spec.json, requirements.md, and design.md for confirmed scope, contracts, and architecture.
2. Generate tasks: read {skill-base}/kiro-spec-tasks/SKILL.md and follow every step, passing "{feature-name}" verbatim
   with the auto-approve flag (-y). Honor its rules files, its task-plan review gate, and its task-graph sanity review.
   Write {specs-root}/{feature-name}/tasks.md and update spec.json metadata (phase: "tasks-generated",
   approvals.tasks.generated: true, approvals.tasks.approved: true, approvals.requirements.approved: true,
   approvals.design.approved: true) only after the review gates pass.
3. DO NOT ask the user anything — you cannot interact with the user. DO NOT commit, branch, push, or run any git command;
   the controller owns all git operations.
4. If a real requirements/design gap or contradiction blocks task generation, DO NOT invent filler tasks. Write tasks.md
   only up to the unambiguous point (or not at all) and return the specific blocking issue.

Return a structured report:
- STATUS: FINALIZED | RETURN_TO_DESIGN
- Created/updated files (full paths)
- Tasks summary (task count, major groups, parallel markers)
- BLOCKING ISSUE: the exact requirements/design gap (empty if STATUS=FINALIZED)
```

### Step 3: Verify outputs (orchestration)
1. **If the subagent returns `STATUS: RETURN_TO_DESIGN`**: STOP. Do NOT commit or touch branches. Surface the blocking issue to the user and point them back to `/kiro-spec-design {feature}` (or requirements). Do not guess.
2. **If the subagent returns `STATUS: FINALIZED`**, verify in the controller:
   ```powershell
   Test-Path "{specs-root}/{feature-name}/tasks.md"
   ```
   Confirm `spec.json` has `phase: "tasks-generated"` and `approvals.tasks.approved: true`.
3. If `tasks.md` is missing or metadata was not updated, report the failure (do not commit); suggest re-running `/kiro-tasks {feature-name}`.

### Step 4: Commit the spec work
Stage and commit the generated tasks (and any other uncommitted spec-phase changes for this feature). Confirm what will be staged first:
```powershell
git status --short
```
Then commit:
```bash
git add -A
git commit -m "chore({feature-name}): generate and approve tasks (tasks.md)"
```
> If `git status --short` shows nothing to commit (tasks unchanged), skip the commit and warn once, then continue.

### Step 5: Squash-integrate into the default branch (only when current != `{default-branch}`)
Determine the current branch:
```powershell
$branchA = git branch --show-current
```

- **If `$branchA` equals `{default-branch}`:** Skip the squash integration **and skip Step 6** (no impl branch is created when already on the default branch). Instead, sync with the remote: pull then push.
  ```powershell
  git pull {remote} {default-branch}
  git push {remote} {default-branch}
  ```
  (If `{remote}` is none or a pull/push fails, warn and continue.) This is the terminal step for this case — the spec work is committed directly on the default branch and synced; report and stop.

- **If `$branchA` does NOT equal `{default-branch}`:** Consolidate branch A into a single commit on `{default-branch}` and clean up, then proceed to Step 6. **Proceed step by step, confirming each succeeds; if any step fails (especially an unresolved conflict), STOP and do NOT delete any branch.**

  1. **Update the default branch:**
     ```powershell
     git checkout {default-branch}
     git pull {remote} {default-branch}
     ```
     If `{remote}` is none or the pull fails (offline / no upstream), warn and continue (do not abort).

  2. **Create squash branch B at the merge-base and collapse A into one commit:**
     ```powershell
     $mergeBase = git merge-base {default-branch} $branchA
     $branchB = "$branchA-squash"
     git checkout -b $branchB $mergeBase
     git merge --squash $branchA
     ```
     Build the commit message from the **actual** `merge-base..$branchA` history (do NOT use a boilerplate line). Read it first:
     ```powershell
     git log --oneline "$mergeBase..$branchA"
     ```
     The message MUST include, at minimum:
     - the **feature name** (`{feature-name}`) in the subject and body, and
     - the **contract this spec established** (the planning deliverables and the key decisions they fixed — e.g. service/topic names, component names, API signatures, data contracts, or the scope the requirements/design/tasks locked in).
     ```bash
     git commit -m "feat({feature-name}): <one-line summary of what this spec planned>

     契約: <the concrete contract this spec fixed (required)>

     - <key change 1 (summarized from merge-base..A history)>
     - <key change 2>
     - tasks.md 生成・spec 計画フェーズ確定"
     ```

  3. **Merge the default branch into B (resolve conflicts here):**
     ```powershell
     git merge --no-ff {default-branch} -m "Merge {default-branch} into $branchB"
     ```
     If conflicts occur, resolve them semantically (interpret both sides; combine divergent evolutions of the same file rather than blindly picking one side), then `git commit` to finalize the merge. **If conflicts cannot be resolved, STOP here and do not delete branches.** When in doubt, verify the build/tests before committing the merge.

  4. **Fast-forward the default branch to B:**
     ```powershell
     git checkout {default-branch}
     git merge {branchB}
     ```
     B already contains `{default-branch}`, so this is a fast-forward. Verify with `git log --oneline {default-branch} -3`.

  5. **Push:**
     ```bash
     git push {remote} {default-branch}
     ```
     (If `{remote}` is none, skip and warn.)

  6. **Delete branches A and B (only after steps 1–5 all succeeded):**
     ```powershell
     git branch -D $branchA $branchB
     git push {remote} --delete $branchA   # only if remote A exists
     ```
     Use `-D` (uppercase): squashed branches are reported as "unmerged" by `-d`. Check `git ls-remote --heads {remote} $branchA` first; skip the remote delete if A is not on the remote. Branch B is local-only — no remote delete. Before deleting, confirm with `git log --oneline {default-branch}` that A's work is present in the default branch.

### Step 6: Create the implementation branch (only after a squash integration ran)
> Run this step ONLY when Step 5 performed the squash integration (i.e., the current branch was NOT the default branch). When the current branch was already the default branch, Step 5 is terminal and no impl branch is created.

From the up-to-date default branch, create and switch to `impl/{feature-name}`:
```powershell
git checkout {default-branch}
```
Resolve name conflicts: if `impl/{feature-name}` already exists, append a numeric suffix (`impl/{feature-name}-2`, `-3`, …) until unused. Check with:
```powershell
git show-ref --verify --quiet "refs/heads/impl/{feature-name}"
```
(exit code 0 = exists). Notify the user when a suffix is applied. Then:
```powershell
git switch -c "impl/{resolved-name}"
```
Leave `impl/{feature-name}` **local only** — do NOT push it on creation (same strategy as `kiro-start`; implementation will push it later). Report that the branch is ready for `/kiro-impl {feature-name}`.

## Important Constraints
- **Lightweight orchestration**: The controller runs Step 0 (resolution), Step 1 (precondition gate), Step 3 (verification), Step 4 (commit), Step 5 (integration or remote sync), and Step 6 (impl branch). The tasks-generation work is delegated to a single subagent (Step 2). Do NOT run kiro-spec-tasks inline in the controller.
- **Impl branch only off a feature branch**: Create `impl/{feature-name}` ONLY when Step 5 ran the squash integration (current branch was not the default branch). When already on the default branch, sync with the remote (pull + push) and stop — do NOT create an impl branch.
- **Impl branch stays local**: Never push `impl/{feature-name}` on creation (kiro-start strategy); implementation pushes it later.
- **Subagent never interacts with the user and never runs git**: The Step 2 subagent only generates tasks and updates spec metadata. All commits, branches, merges, and pushes are owned by the controller.
- **Report in the user's language**: Think in English internally, but write every developer-facing message in the spec's target language (see Communication Language). Keep the internal Step 2 subagent prompt in English.
- **Deterministic resolution**: Step 0 resolves specs root, skill base, remote, and default branch with fixed priority orders. Never guess; if a required value is unresolved, hard-fail. Resolve each value once and reuse it.
- **Integration only off the default branch**: The squash-integration (Step 5) runs ONLY when the current branch is not the default branch. Never delete, reset, or force-update branches except the specified `git branch -D` of A and B after a successful push.
- **No branch deletion before push success**: Delete A and B only after `git push {remote} {default-branch}` succeeds and the default branch contains A's work.
- **Squash message must be real**: Build it from `merge-base..A` history and include the feature name and the contract the spec fixed. Do NOT ship a boilerplate message.
- Do NOT re-derive or change the feature name; it is fixed by `$ARGUMENTS`.
- Do NOT generate requirements or design, perform implementation, or archive the spec (those are other skills).
</instructions>

## Output Description
Provide output in the language specified in `spec.json` with the following structure:

1. **Feature Name**: `feature-name` (equals the argument)
2. **Tasks Status**: Confirm `tasks.md` was generated and auto-approved (task count, major groups, parallel markers); the subagent's review gates passed
3. **Commit**: The chore commit hash/subject for the tasks work (or a note that there was nothing to commit)
4. **Integration Status**: One of:
   - `Squashed branch "{branchA}" into one commit and integrated into {default-branch}; pushed; branches A/B deleted`, or
   - `Integration skipped — current branch was already the default branch "{default-branch}"; synced with remote (pull + push)`
5. **Implementation Branch**: One of:
   - `Created and switched to impl/{resolved-name}` (note any numeric suffix applied), or
   - `Not created — work was committed directly on the default branch "{default-branch}"`
6. **Next Step**: Command block showing `/kiro-impl <feature-name>`

**Format Requirements**:
- Use Markdown headings (##, ###)
- Wrap commands in code blocks
- Keep total output concise (under 300 words)
- Use clear, professional language per `spec.json.language`

## Safety & Fallback
- **Specs Root Unresolved (hard fail)**: If `.kiro/specs` does not exist, STOP and report that the repository is not cc-sdd initialized.
- **Skill Unresolved (hard fail)**: If `kiro-spec-tasks/SKILL.md` is not found under `.claude/skills`, `.agents/skills`, or `.github/skills` (in that order), STOP and report that the required kiro skill is not installed.
- **Missing Upstream Artifacts (hard fail)**: If `spec.json`, `requirements.md`, or `design.md` is missing, STOP. Do not generate tasks, commit, or branch. Point to `/kiro-spec-requirements {feature}` or `/kiro-spec-design {feature}`.
- **Tasks Delegation**: All tasks-level behavior (rules, review gate, sanity review, template) is handled by `kiro-spec-tasks` inside the Step 2 subagent. Honor its results as reported.
- **Return To Design**: If the subagent returns `STATUS: RETURN_TO_DESIGN`, surface the blocking issue and stop; do not guess or commit.
- **Missing Outputs After FINALIZED**: If the subagent reports FINALIZED but `tasks.md` is missing or metadata is not updated (Step 3 verification fails), report the failure and do not commit.
- **Nothing To Commit (Step 4)**: If `git status --short` is empty, skip the commit, warn once, and continue.
- **No Remote / Pull / Push Failure**: If `{remote}` is none or a `git pull`/`git push` fails, warn and continue; never block integration progress on remote operations — but still do NOT delete branches if the push that proves integration did not succeed.
- **Merge Conflict (Step 5.3)**: Resolve conflicts semantically (combine divergent evolutions of the same file; do not blindly pick one side). Verify build/tests before committing the merge when in doubt. If unresolved, STOP and do not delete branches.
- **Branch Already Exists**: For `impl/{feature-name}`, append a numeric suffix and notify the user (do not switch to a pre-existing branch). For the squash branch `{branchA}-squash`, if it already exists from a prior aborted run, report it and stop rather than overwriting.
- **Premature Branch Deletion**: Delete A and B only after `git push {remote} {default-branch}` succeeds and `git log --oneline {default-branch}` shows A's work integrated.
- **Default Branch Is Current**: Skip the squash integration AND skip impl-branch creation. Sync with the remote (`git pull` then `git push`) and stop; the spec work stays committed directly on the default branch.

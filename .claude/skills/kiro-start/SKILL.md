---
name: kiro-start
description: 'Post-discovery single-spec entry point. Precondition: /kiro-discovery has already created {specs-root}/{feature-name}/brief.md, so the feature name is already confirmed. Portable across repos: deterministically resolves the specs root, skill base directory, git remote, and default branch (fixed priority orders, no guessing). Creates the development branch feat/{feature-name} only when the current branch is the repository default branch, initializes the spec via kiro-spec-init (which consumes brief.md), then runs kiro-spec-requirements. Use when: 要件ディスカバリ後に仕様を開発ブランチ付きで開始したい, kiro-start, spec開始, feat ブランチを切って spec を始める, start a confirmed spec with a feature branch. DO NOT USE FOR: 複数仕様の一括生成 (use /kiro-spec-batch), 既存 spec の追加要件生成のみ (use /kiro-spec-requirements directly).'
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Agent, WebSearch, WebFetch, AskUserQuestion
argument-hint: <feature-name>
---

# Spec Start (Branch + Init + Requirements, post-discovery)

<instructions>
## Core Task
Start a single specification end-to-end **after `/kiro-discovery`**. Discovery has already created `{specs-root}/{feature-name}/brief.md`, so the **feature name is already confirmed and equals `$ARGUMENTS`**. Create the development branch `feat/{feature-name}` when the current branch is the repository's default branch, initialize the spec via `kiro-spec-init` (which consumes the existing brief.md and skips clarification), then run `kiro-spec-requirements`. For multi-spec generation, use `/kiro-spec-batch` (this skill does not create branches for batch flows).

This skill is designed to be **portable across repositories**. It does not hard-code skill paths, the remote name, or the default branch; instead it resolves each one with a **deterministic, ordered detection procedure** (Step 0). Each resolution yields exactly one result or a hard failure — never an ambiguous guess.

## Parameter Passing
- `$ARGUMENTS` is the **confirmed kiro feature name** produced by `/kiro-discovery` (it matches the existing directory `{specs-root}/{feature-name}/`, which already contains `brief.md`). Pass it **verbatim** as the first parameter of BOTH `kiro-spec-init` and `kiro-spec-requirements`. Do NOT ask clarifying questions or re-derive a name in this wrapper.
- Because the feature name is already determined, the branch name `feat/{$ARGUMENTS}` is known upfront. No name generation or resolution occurs.

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
   Resolve `kiro-spec-requirements/SKILL.md` under the **same** `{skill-base}`. If neither sibling skill is found under any of the three bases, STOP (hard fail): the required kiro skills are not installed.

3. **Default remote** (`{remote}`): Run `git remote`. Apply this fixed rule:
   - If `origin` is present → `{remote}` = `origin`.
   - Else if exactly one remote exists → `{remote}` = that remote.
   - Else (no remotes, or multiple without `origin`) → `{remote}` = none; treat all remote operations as skipped (warn once).

4. **Default branch** (`{default-branch}`): Determine deterministically:
   - If `{remote}` is set, read `git symbolic-ref --quiet --short refs/remotes/{remote}/HEAD` and strip the `"{remote}/"` prefix.
   - If that yields nothing and a local `main` branch exists → `{default-branch}` = `main`.
   - Else if a local `master` branch exists → `{default-branch}` = `master`.
   - Else → `{default-branch}` = the current branch (branch creation will be skipped because current == default is trivially handled; see Step 2 note).

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

### Step 2: Update default branch and create the development branch (only when current == `{default-branch}`)
1. Determine the current branch:
   ```powershell
   $branch = git branch --show-current
   ```
2. **If `$branch` equals `{default-branch}` (resolved in Step 0):**
   - If `{remote}` is set, refresh before branching:
     ```powershell
     git pull {remote} {default-branch}
     ```
     If the pull fails (offline, no upstream, conflicts) or `{remote}` is none, **do NOT abort**. Warn and continue.
   - The target branch name is `feat/{feature-name}`. Resolve branch-name conflicts: if it already exists, append a numeric suffix (`feat/{feature-name}-2`, `-3`, …) until unused. Check existence with:
     ```powershell
     git show-ref --verify --quiet "refs/heads/feat/{feature-name}"
     ```
     (exit code 0 = exists). Notify the user when a suffix is applied.
   - Create and switch (the existing untracked `brief.md` follows to the new branch):
     ```powershell
     git switch -c "feat/{resolved-name}"
     ```
3. **If `$branch` does NOT equal `{default-branch}`:**
   - Skip the pull and branch creation. Do not commit.
   - **Warn the user** that the development branch was not created because the current branch (`$branch`) is not the default branch (`{default-branch}`).

### Step 3: Initialize the specification
1. Read and execute the procedure in `{skill-base}/kiro-spec-init/SKILL.md` (resolved in Step 0), passing `$ARGUMENTS` (the confirmed feature name) verbatim as the first parameter.
2. `kiro-spec-init` reads the existing `brief.md`, reuses the `{specs-root}/{feature-name}/` directory, and writes `spec.json` and `requirements.md`.

### Step 4: Generate requirements
1. Read and execute the procedure in `{skill-base}/kiro-spec-requirements/SKILL.md` (resolved in Step 0), passing `$ARGUMENTS` (the same confirmed feature name) as the first parameter.
2. This regenerates `{specs-root}/{feature-name}/requirements.md` with EARS-format requirements.

### Step 5: Commit the generated spec (only when a branch was created)
- **If a `feat/{...}` branch was created in Step 2** (i.e., current branch was the default branch), auto-commit on that branch:
  ```powershell
  git add -A
  git commit -m "chore({feature-name}): initialize spec (spec.json, requirements.md)"
  ```
- **Otherwise** (not on the default branch): leave the generated spec files uncommitted on the current branch.

## Important Constraints
- **Deterministic resolution**: Step 0 resolves specs root, skill base, remote, and default branch with fixed priority orders. Never guess; if a required value is unresolved, hard-fail as specified. Resolve each value once and reuse it.
- This skill is **post-discovery only**: if `{specs-root}/{feature-name}/` does not exist, FAIL deterministically (do not create anything).
- Do NOT generate design or tasks. This skill stops after requirements.
- Do NOT re-derive or change the feature name; it is fixed by discovery (`$ARGUMENTS`).
- Do NOT create a branch unless the current branch equals the resolved default branch. Never delete, reset, or force-update branches here.
- Do NOT use this skill for `/kiro-spec-batch` (multi-spec) flows.
</instructions>

## Output Description
Provide output in the language specified in `spec.json` with the following structure:

1. **Feature Name**: `feature-name` (confirmed by discovery; equals the argument)
2. **Project Summary**: Brief summary (1 sentence, sourced from brief.md)
3. **Created Files**: Bullet list with full paths (`spec.json`, `requirements.md`)
4. **Branch Status**: One of:
   - `Created and switched to feat/{resolved-name}` (initial spec files committed), or
   - `Branch creation skipped — current branch "{branch}" is not the default branch "{default-branch}"`
5. **Requirements Status**: Confirm `requirements.md` was generated for `{feature-name}`
6. **Next Step**: Command block showing `/kiro-spec-design <feature-name>` (or `/kiro-validate-gap <feature-name>` for existing codebases)

**Format Requirements**:
- Use Markdown headings (##, ###)
- Wrap commands in code blocks
- Keep total output concise (under 300 words)
- Use clear, professional language per `spec.json.language`

## Safety & Fallback
- **Specs Root Unresolved (hard fail)**: If `.kiro/specs` does not exist, STOP and report that the repository is not cc-sdd initialized.
- **Skills Unresolved (hard fail)**: If `kiro-spec-init/SKILL.md` is not found under `.claude/skills`, `.agents/skills`, or `.github/skills` (in that order), STOP and report that the required kiro skills are not installed.
- **Missing Spec Folder (hard fail)**: If `{specs-root}/{feature-name}/` does not exist, STOP immediately and report the error. Do not create the folder, branch, or any spec files. Suggest running `/kiro-discovery "<idea>"` first, or verifying the feature name against existing folders under `{specs-root}/`.
- **Missing Brief (hard fail)**: If the folder exists but `brief.md` is absent, STOP and report that the discovery brief is incomplete; recommend re-running `/kiro-discovery` for this feature.
- **Init Delegation**: All init-level fallbacks (missing templates, write failure) are handled by `kiro-spec-init`. Honor its results.
- **Requirements Delegation**: All requirements-level behavior (steering load, EARS rules, review gate) is handled by `kiro-spec-requirements`. Honor its results.
- **No Remote / Pull Failure**: If `{remote}` is none or `git pull` fails, warn and continue; never block spec initialization on remote operations.
- **Branch Already Exists**: Append a numeric suffix and notify the user (do not switch to a pre-existing branch).
- **Not on Default Branch**: Skip branch creation and commit; warn. Spec files remain uncommitted on the current branch.
- **Commit Failure**: Report the error with the branch name; the branch remains created/checked out with files staged.

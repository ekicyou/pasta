# Agentic SDLC and Spec-Driven Development

Kiro-style Spec-Driven Development on an agentic SDLC

## Project Context

### Paths
- Steering: `.kiro/steering/`
- Specs: `.kiro/specs/`

### Steering vs Specification

**Steering** (`.kiro/steering/`) - Guide AI with project-wide rules and context
**Specs** (`.kiro/specs/`) - Formalize development process for individual features

### Active Specifications
- Check `.kiro/specs/` for active specifications
- Use `/kiro-spec-status [feature-name]` to check progress

## Development Guidelines
- Think in English, generate responses in Japanese. All Markdown content written to project files (e.g., requirements.md, design.md, tasks.md, research.md, validation reports) MUST be written in the target language configured for this specification (see spec.json.language).

## Branch & Merge Strategy (PR-based)
- **Merge into `main` happens only via Pull Request.** Never push directly to the default branch.
- **1 feature = 1 branch = 1 PR**: a spec's whole lifecycle (requirements → design → tasks → implementation) runs on a single Claude Code (harness) worktree branch; integration is one squash-merged PR at completion.
- **Branches come from the harness worktree** — skills do NOT create/switch/delete branches. `/kiro-start` (entry) and `/kiro-complete` (exit) operate on the supplied worktree branch.
- Authority: `.kiro/steering/workflow.md` (branch strategy + completion procedure).

## Minimal Workflow
- Phase 0 (optional): `/kiro-steering`, `/kiro-steering-custom`
- Discovery: `/kiro-discovery "idea"` — determines action path, writes brief.md + roadmap.md for multi-spec projects
- Phase 1 (Specification):
  - Post-discovery single-spec entry: `/kiro-start {feature}` — on the harness worktree branch, drives init → requirements (kiro-spec-requirements) → gap analysis (kiro-validate-gap, subagent) → requirements discussion (kiro-requirements-discussion, interactive in the chat window), committing each phase (no push). Asks no questions before the discussion; the discussion's first approval is asked through the app's next-step box. STOPs on the default branch.
    - `/kiro-validate-gap {feature}` (standalone re-run; already performed by `/kiro-start`)
    - `/kiro-requirements-discussion {feature}` (standalone re-run; already performed by `/kiro-start`)
  - Design phase entry: `/kiro-design {feature}` — on the same branch, drives design generation (kiro-spec-design -y) → design validation (kiro-validate-design, subagent) → design discussion (kiro-design-discussion, interactive in the chat window), committing each phase (no push). Asks no questions before the discussion; the discussion's first approval is asked through the app's next-step box. STOPs on the default branch.
    - Then step by step on the same branch:
      - `/kiro-spec-design {feature} [-y]` (standalone re-run; already performed by `/kiro-design`)
      - `/kiro-validate-design {feature}` (standalone re-run; already performed by `/kiro-design`)
      - `/kiro-design-discussion {feature}` (standalone re-run; already performed by `/kiro-design`)
      - `/kiro-spec-tasks {feature} [-y]`
  - Without discovery / quick path: `/kiro-spec-quick {feature} [--auto]` or `/kiro-spec-init "description"` → `/kiro-spec-requirements {feature}`
  - Multi-spec: `/kiro-spec-batch` — creates all specs from roadmap.md in parallel by dependency wave
- Phase 2 (Implementation): `/kiro-impl {feature} [tasks] [--review required|inline|off]`
  - Without task numbers: autonomous mode (subagent per task + independent review + final validation)
  - With task numbers: manual mode (selected tasks in main context, still reviewer-gated before completion)
  - `--review off` skips task-local review; use it intentionally and keep `/kiro-validate-impl {feature}` as the final quality gate
  - `/kiro-validate-impl {feature}` (standalone re-validation)
- Completion (explicit approval required): `/kiro-complete {feature}` — static DoD gate → archive to `completed/` → one full test run after the archive (docs updated in parallel while it runs) → final commit → **PR create + squash merge** (the only path into `main`). Use only when the developer explicitly approves.
- Release (only when the developer types it): `/release [X.Y.Z]` — runs one release from the skill's own procedure without approval stops (version bump PR → squash merge → tag `vX.Y.Z` → watch the release CI (`release.yml`) and rerun failed jobs → final report with the highlights). Not `/kiro-complete`.
- Roadmap inventory and next wave: `/kiro-next [extra instructions]` — after `main` advances, re-measures the current version's briefs (and the fillers a spare seat may take) against current `main` with the skill's scan scripts, rebuilds the dependency tree, and picks the next wave (up to 8 non-overlapping specs, current version first) plus an outline of the next three waves (ending at `/release`); commits on the worktree branch, PR + squash merge only after the developer approves
- Wave launch: `/kiro-wave [spec names]` — starts one separate session (own worktree, `/kiro-start <name>`) per Opus spec of the wave `/kiro-next` recommended; Fable specs are excluded with a warning and handed back as `/kiro-start` commands; first lands this worktree's own changes by PR → squash merge
- Progress check: `/kiro-spec-status {feature}` (use anytime)

## Skills Structure
Skills are located in `.claude/skills/*/SKILL.md` (mostly `kiro-*`)
- Each skill is a directory with a `SKILL.md` file
- Skills run inline with access to conversation context
- Skills may delegate parallel research to subagents for efficiency
- Additional files (templates, examples) can be added to skill directories
- `kiro-start` — post-discovery single-spec entry (init → requirements → gap analysis (subagent) → requirements discussion (interactive, chat window) on the harness worktree branch; no branch creation, no push)
- `kiro-design` — post-requirements design-phase entry (design generation (kiro-spec-design -y, subagent) → design validation (kiro-validate-design, subagent, non-interactive, report to disk) → design discussion (kiro-design-discussion, interactive, chat window) on the harness worktree branch; no branch creation, no push)
- `kiro-complete` — spec completion exit (static DoD gate → archive → one full test run with parallel doc work → PR-based squash merge into `main`; the only path into the default branch)
- `kiro-review` — task-local adversarial review protocol used by reviewer subagents
- `kiro-debug` — root-cause-first debug protocol used by debugger subagents
- `kiro-verify-completion` — fresh-evidence gate before success or completion claims
- `release` — one release (`/release [X.Y.Z]`); the procedure lives in the skill (`SKILL.md` + `procedure.md`) and runs in the main context without approval stops; it stops only on a failed check, a red run, a state the procedure does not cover, or a refused command
- `update` — dependency update (`/update [--check]`, only when the developer types it): surveys the Rust crates and the npm packages (`editors/vscode`, `book`), proposes compatible updates / raised minimums / major updates in one batch with recommendations, then commits the approved ones per category (majors per dependency), verifies them (no desk) and lands them by PR → squash merge; never during `/release`
- `kiro-next` — roadmap inventory ("棚卸") and next-wave planning (rewrites `roadmap.md` / `roadmap-history.md` / `focus.md` on the worktree branch; PR + squash merge only behind an explicit approval)
- `kiro-wave` — launches the wave `/kiro-next` recommended as separate sessions (Opus specs only; Fable specs excluded with a warning); lands this worktree's changes by PR → squash merge first
- `pasta-check` — reference for the `pasta_check` CLI (ghost release build `release`, later check commands)
- `pasta-ghost-authoring` — Pasta DSL grammar and dictionary-writing patterns for a ghost's `.pasta` files
- `pasta-lua-coding` — pasta.dll Lua runtime API reference and coding conventions (ghost `scripts/`, Lua blocks in Pasta DSL)
- Use skills explicitly requested by the user and skills relevant to the task's domain, including design, accessibility, and UX.
- Select skills from their descriptions or metadata first, then read only the selected skills and the references needed for the task.
- Follow explicit host and project rules and retain required workflow checks. Do not skip relevant skills just because the task is small.

## Development Rules
- 3-phase approval workflow: Requirements → Design → Tasks → Implementation
- Human review required each phase; use `-y` only for intentional fast-track
- Keep steering current and verify alignment with `/kiro-spec-status`
- Follow the user's instructions precisely, and within that scope act autonomously: gather the necessary context and complete the requested work end-to-end in this run, asking questions only when essential information is missing or the instructions are critically ambiguous.

## Steering Configuration
- For spec and implementation work, load the core steering files below from `.kiro/steering/`. Reuse current context rather than rereading unchanged files.
- Load additional steering only when required by project rules or relevant to the task.
- Default files: `product.md`, `tech.md`, `structure.md`
- Custom files are supported (managed via `/kiro-steering-custom`)

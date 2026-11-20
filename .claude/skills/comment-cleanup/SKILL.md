---
name: comment-cleanup
description: Clean up Rust code comments — remove AI-generated over-commenting, narration, change history and references to `.kiro` specs, while preserving comments that carry real context. Use whenever the user asks to clean, tidy, prune, fix, audit, or review comments, mentions "too many comments", "verbose comments", "comment pass", "de-AI the comments", or asks to apply the CLAUDE.md `## Comments` rules to a file, diff, or PR. Also use after an implementation when the user asks to tidy up before committing. With no target given, works on the current uncommitted diff.
---

# Comment Cleanup

A comment-only editing pass. Walk every comment in scope, keep the few that carry
context the code cannot, delete or rewrite the rest. The rules mirror the
`## Comments` section of `CLAUDE.md`; this skill is the post-hoc half, `CLAUDE.md`
is the pre-hoc half.

## Scope — read this first

**Edit comments only. Never change code in this pass.** No renames, no
extractions, no logic changes, no "improvements" — even when a comment exists
only because the code is unclear. In that case the comment may survive the pass
(it is doing real work, badly) and the situation goes in the summary as a flag
for the user to act on. Refactors are a different task.

Whitespace left behind by deleted comments should be tidied (no orphaned blank
lines), but the code itself must be byte-identical. Attributes (`#[allow]`,
`#[expect]`, `#[cfg]`) are not comments and are out of scope.

**Target.** If the user names files, a PR, or a range, use that. If no target is
given, use the current uncommitted diff (`git diff` + `git diff --cached` +
untracked files), **not the whole repository**. Within a diff, touch only
comments that were added or changed in it; do not reopen untouched old comments
unless the user asked for a whole-file pass.

## The core principle

Code already shows **how**. A comment earns its place only by carrying **why**
— context the code cannot: a non-obvious constraint, a deliberate deviation, a
gotcha, a workaround, the reason a tempting simpler version is wrong. When in
doubt about a *why* comment, keep it; when in doubt about a *how* comment,
delete it. Wrongly deleting a warning that does real work costs far more than
leaving one mediocre comment behind.

## Rust specifics

- `// SAFETY:` comments are contracts. **Never delete them.** Trim only
  restatement; keep every invariant the `unsafe` block relies on. If an invariant
  looks wrong, flag it rather than rewording.
- Rustdoc (`///`, `//!`) on public items is API surface. Keep `# Safety`,
  `# Errors` and `# Panics` sections and intra-doc links.
- Code blocks inside doc comments are doctests, i.e. code. **Never edit or
  delete them.**
- Attributes (`#[allow]`, `#[expect]`, `#[cfg]`) are not comments and are out of
  scope.
- ukadoc and the SHIORI/SSTP specifications are external standards: durable
  breadcrumbs under rule 3. `.kiro/` specs, requirement IDs and task numbers are
  ephemeral.
- If no target is given, use the current uncommitted diff, not the whole
  repository.

## The pass — apply to every comment, in order

**1. Is it narration?** Restates the code, describes the steps ("loop over the
accounts", "parse the request body"), restates names/types/signatures, or marks
block ends (`} // end if`). → **Delete.** This is the bulk of agent-generated
comment noise. The code below it was fine all along; do not touch it.

**2. Is it change narration?** Addressed to the reviewer of a diff rather than
a reader of the file: "updated to use the v2 client", "removed the old
fallback", "added per feedback", "fixed the off-by-one", "now uses", "changed
from". → **Delete.** Test: would the sentence make sense to someone reading the
file fresh, a year from now, who never saw any PR? If not, it belongs in a
commit message.

**3. Does it point at a moving target?** "See spec section 3", "per the
requirements doc", "requirement 4.2", "task 7". → **Delete** if the annotated
point didn't need a comment at all (spec compliance itself never does), or
**rewrite** to encode the substance of the requirement directly. What counts as
durable vs ephemeral:

- **Durable (allowed, as breadcrumbs):** ukadoc and the SHIORI/SSTP
  specifications, RFCs and standards, issue/ticket IDs, permalinked sources, and
  a **maintained repo doc/README at a stable path** (e.g. `doc/impl-watch.md`),
  which lives and updates alongside the code. A README is *not* a rotting spec —
  don't lump the two.
- **Ephemeral (rewrite or delete):** `.kiro/` specs (`requirements.md`,
  `design.md`, `tasks.md`), requirement IDs, task numbers, "the spec", "the
  requirements", any design doc (a point-in-time artifact that gets superseded),
  and **any section number of any document** — "spec §7", "section 4.2" — the
  canonical ephemeral form. A section number is never durable, no matter what
  document it indexes into. (A heading of an *external standard*, quoted with its
  URL, is a breadcrumb; the bare number is not.)

This rule applies **inside doc comments exactly as in `//` comments** — a
"per spec §11" buried in a module `//!` is the same defect.

Even a durable reference is a *breadcrumb*, never the explanation itself
— the address survives, but the content behind it can change. The comment must
stand on its own with the link removed:

```rust
// Bad:  per requirements doc section 3.2
// Bad:  see ukadoc
// Good: SHIORI/3.0 header lines end in CRLF; a bare LF is rejected by SSP,
//       so do not "normalise" it. ukadoc: https://ssp.shillest.net/ukadoc/...
```

**4. Is it a *why* that's bloated or misplaced?** For comments that do carry
real rationale, in sequence:

- *Does the rationale belong in a doc, not here?* System-level "why it's built
  this way" narrative — an architecture decision, a benchmark result, a
  lifecycle overview — reads better in a maintained doc/README than in a long
  inline block. If a durable doc **already** carries it → delete the inline copy;
  do **not** leave a signpost. A comment that only restates a decision the code
  already reflects is dead weight *even pointed at the doc* — the doc is the
  first place anyone questioning the value looks. If no such doc exists yet,
  keep the comment but **flag it** as extraction-worthy (writing the doc is out
  of scope for a comment-only pass). Extraction is *pure*: once the narrative
  lives in the doc, what survives inline is only the local invariant a reader
  needs *at that line* — a non-obvious constraint or a cross-file sync
  obligation — not a pointer to the doc. A comment that is *nothing but* a
  pointer generally shouldn't exist.
- *Is all of it necessary?* Trim throat-clearing, restated context the reader
  already has, and anything the code itself shows. Keep every part that a
  future maintainer would need to avoid breaking or "simplifying" the code.
- *Does it belong in one block?* A long header block covering several distinct
  points often serves better broken up, with each short comment placed adjacent
  to the line it governs. Proximity is what keeps comments true.
- *Is the remaining length earned?* Some rationale is irreducibly multi-part
  (a workaround plus the bug it works around plus the removal condition).
  Length alone is not a defect — unearned length is. Never shorten a comment
  at the cost of the information in it.

The trap this rule exists to catch: **"carries a real *why*" and "is worded
minimally" are independent judgments.** A comment can carry genuine rationale
*and* still be three times too long. Apply Occam's razor to *every surviving
comment*: keep the single non-obvious fact a reader needs at that line, in the
fewest words, and cut everything else — the mechanism the code already shows,
where a value is consumed downstream, the consequence-of-the-consequence,
restated names/types, and justification of the justification. The razor's answer
is sometimes zero: delete. A five-line block almost never survives intact;
suspect it on sight. (Exception: `// SAFETY:` — see Rust specifics.)

**5. Is it stale?** Describes code that no longer exists or behavior that has
changed. → **Delete** (or correct it, if the underlying point still holds and
is worth keeping). A wrong comment is worse than no comment.

**6. None of the above?** It explains a non-obvious constraint, a surprising
but essential line, a contract the signature can't show (units, ranges,
side effects, failure modes), an edge case, or the source of a copied
algorithm. → **Keep.** Do not reword comments that are already fine — churn on
good comments is itself noise in the diff. But "already fine" means *already
minimal*, not merely correct: a correct-but-bloated comment is a rule-4 target,
not a keep.

Three keep categories that are easy to misread as deletable — they are not:

- **Cross-file consistency pointers** — "mirrors the other platform's table",
  "keep in sync with X". The *link itself* is the why: it prevents two copies
  drifting apart.
- **Data-literal semantics** — comments giving the meaning, order, or units of
  a literal (`// (width, height) in px, portrait`, `// 100 ns ticks, not ms`).
  The literal cannot show its own convention.
- **Format contracts** — comments pinning output or wire format to an external
  expectation.

**Structural section banners** (`// --- Name ---` dividers) are navigation, not
narration. Keep them unless the user asks for their removal. If a banner carries
substance beyond its label, that substance is protected like any other *why*.

## TODOs

Keep `TODO`/`FIXME` comments. They do not need issue IDs. Two exceptions:

- A TODO describing work that has visibly been done → stale, delete (rule 5).
- A TODO that is deferral disguised as a marker — e.g. `// TODO: handle errors`
  on a path that plausibly should already handle them → keep it, but flag it
  in the summary; deleting it would hide real incompleteness.

## Doc comments

All six rules apply inside `///` and `//!`, including rule 3's reference rules,
**except** that the doc is API surface: a doc comment that only re-emits the
item name and its types is narration → delete or reduce; one that documents the
contract the signature can't show (units, valid ranges, side effects, what
happens on failure, invariants) → keep. Condensing to a one-liner is valid only
when nothing beyond signature restatement is lost. A brief one-line summary on a
public item is legitimate even when short. `# Safety`, `# Errors`, `# Panics`
sections, intra-doc links and every code block (doctest) are never removed or
edited.

## Output

After editing, report (in Japanese, per the project's rule for user-facing text):

1. **Counts** — comments deleted / rewritten / kept, per file.
2. **Flags** — anything the pass could not fix within scope:
   - comments compensating for genuinely unclear code (candidate refactors)
   - deferral TODOs (possible unfinished work)
   - `// SAFETY:` invariants that look wrong
   - comments whose correctness couldn't be verified (kept conservatively)
3. **Judgment calls** — any borderline keep/delete decisions, one line each,
   so the user can overrule.

Keep the report brief. Do not list every deleted narration comment — they are
the point of the pass, not news. Afterwards confirm the code is untouched
(e.g. the diff outside comments is empty, and the crate still builds if cheap to
check).

## Worked examples

**Delete — narration on clean code. Remove every comment; the code is untouched:**

```rust
fn load(path: &Path) -> Result<Config> {
    // Read the file into a string
    let text = std::fs::read_to_string(path)?;
    // Parse the string as TOML, returning an error on failure
    let cfg: Config = toml::from_str(&text)?;
    Ok(cfg)
}
```

**Rewrite — block broken up and trimmed, substance kept, placed at point of use:**

```rust
// Before: one header block
// This function converts the time. Note that SSP counts in 100 ns ticks while
// our scheduler counts in ms, which caused drift. Also the value must be
// converted before any arithmetic. Finally the result is compared against
// the original to detect drift.

// After: two adjacent comments, narration dropped
let ticks = ms * 10_000; // SSP uses 100 ns ticks; convert before arithmetic
// Round up: rounding down fires the timer one tick early and the event is dropped.
let due = round_up(ticks, GRANULARITY);
```

**Keep — surprising but essential, exactly the comment that must survive:**

```rust
// SAFETY: `hwnd` was created on this thread and is destroyed only by
// `Drop`, which cannot run while `&self` is alive.
unsafe { SetWindowPos(self.hwnd, None, x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER) }?;
```

# Brief: License Compliance & Documentation

## Problem
- Pasta project is publicly released under MIT license
- However, the current dependency tree (Rust crates and related libraries) contains mixed licenses including Apache 2.0 and possibly others
- There is ambiguity about whether the overall MIT declaration accurately reflects the true license obligations imposed by all dependencies
- No comprehensive LICENSES or COPYING documentation exists that clearly enumerates all transitive dependencies and their licenses

## Current State
- Project declares MIT in LICENSE file and documentation
- Cargo.toml and Cargo.lock contain dependency specifications but no automated license audit
- README.md links to LICENSE but does not link to a comprehensive dependency license document
- No LICENSES or similar documentation file exists to satisfy SPDX or common open-source practices
- deny.toml exists but may not be comprehensively configured for license auditing

## Desired Outcome
- A comprehensive license audit completed for all direct and transitive dependencies
- A machine-readable and human-readable license inventory document (e.g., LICENSES.md or similar)
- Clarification of license compliance status (whether MIT-only is sufficient, or if Apache 2.0 + MIT "dual license" language is needed)
- README.md updated with a link to the license inventory from the top-level landing page
- Establish a repeatable process or CI/CD check to keep license compliance audits current as dependencies evolve

## Approach
- Audit all Rust crate dependencies using tools like `cargo-deny`, `cargo-license`, and related utilities
- Cross-reference with SPDX License List to ensure accuracy
- Document findings in a structured format (recommend REUSE Specification compliance or similar)
- Create a LICENSES.md file at the project root listing all dependencies, their licenses, and any required attribution or modifications
- Update README.md to link to the license inventory
- (Optional) Configure CI/CD to validate license compliance on future dependency updates

## Scope

### In
- Audit all Rust workspace crates (pasta_dsl, pasta_core, pasta_lua, pasta_shiori, pasta_lsp, pasta_check, pasta_sample_ghost)
- Enumerate all direct and transitive dependencies with their licenses
- Identify any license compatibility issues or conflicts
- Create/update LICENSES.md with comprehensive inventory
- Update README.md to link to license documentation
- Verify deny.toml configuration is appropriate for ongoing compliance
- Document the audit methodology and tools used for reproducibility

### Out
- Modifying or replacing actual dependency versions (unless license audit reveals a must-replace situation)
- Legal interpretation beyond scope of SPDX License List classification
- Retroactive changes to historical commit messages or past releases
- Tooling for automated license scanning beyond basic Cargo.toml parsing (e.g., SBOM generation, though could be explored as future work)

## Boundary Candidates
- **Dependency audit & inventory** — Data gathering on all licenses
- **Documentation & publication** — Creating the LICENSES.md and README updates
- **CI/CD integration** — Setting up ongoing validation (could be separate follow-up work)

## Out of Boundary
- Licensing decisions about future dependency additions (belongs with architecture/dependency strategy)
- Re-licensing the Pasta project itself (policy decision, not this spec)
- Audit of third-party binaries or non-Rust dependencies not declared in Cargo.toml

## Upstream / Downstream
- **Upstream**: Depends on stable Cargo.lock snapshot and deny.toml configuration
- **Downstream**: Future dependency updates will reference this license audit; CI/CD checks may depend on LICENSES.md existing; users evaluating Pasta for their own projects will reference the license documentation

## Existing Spec Touchpoints
- **Adjacent**: `release-workflow` spec may need to be aware of license compliance checks for release readiness
- **No direct dependency** on other active specs

## Constraints
- Audit must reflect the current stable main branch state (Cargo.lock in git)
- Output documentation should be human-readable and suitable for publication on README or in release notes
- Must comply with common open-source practices (SPDX, REUSE, or similar)
- Should be repeatable and maintainable as dependencies change

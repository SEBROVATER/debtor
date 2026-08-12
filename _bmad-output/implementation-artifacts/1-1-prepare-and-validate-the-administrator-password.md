---
baseline_commit: 1bbfbc3eac091f97c8ad3a17f2ce5e24f7d8d74d
---

# Story 1.1: Prepare and Validate the Administrator Password

Status: done

## Story

As the administrator,
I want to generate and validate the configured password hash before Debtor touches external state,
so that invalid credentials cannot start a partially initialized service.

## Acceptance Criteria

1. **Given** the independent `tools/password-hash` workspace is run through its protected input flow
   **When** the administrator supplies a password
   **Then** it emits a canonical Argon2id v19 PHC hash using exactly `m=19,456`, `t=2`, `p=1`, a 16-byte OS-generated salt, and a 32-byte output
   **And** neither password nor generated hash is written to logs, fixtures, or committed files.

2. **Given** `APP_ADMIN_PASSWORD_HASH` is absent, exceeds 256 encoded bytes, is structurally noncanonical, is not Argon2id v19, or contains parameters other than exactly `m`, `t`, and `p`
   **When** startup configuration is validated
   **Then** validation fails before database connection, migration, socket admission, or password KDF work
   **And** the sanitized error reveals no credential or hash content.

3. **Given** an Argon2id v19 PHC value is structurally canonical
   **When** its bounded parameters are validated
   **Then** memory must be `19,456..=65,536` KiB, iterations `2..=5`, parallelism `1..=4`, decoded salt length `16..=64` bytes, and output length `32..=64` bytes
   **And** any out-of-range value is rejected before external side effects.

4. **Given** the helper implementation is complete
   **When** its independent formatting, locked Clippy with warnings denied, and locked tests run through manifest-path commands
   **Then** every check passes without folding the helper into the production workspace
   **And** production code contains no unsafe Rust or credential-revealing diagnostics.

## Tasks / Subtasks

- [x] Define one reusable infrastructure-owned password-hash policy (AC: 2, 3)
  - [x] In `debtor-infra/src/auth/password.rs`, cheaply reject encoded values longer than 256 bytes and structural/canonical violations before invoking `PasswordHash::new` or any KDF operation.
  - [x] Retain acceptance only for Argon2id v19 PHC with exactly the unique `m`, `t`, and `p` parameters and the prescribed bounded values.
  - [x] Keep all policy failures mapped to `ApplicationError::Configuration(ConfigurationError::InvalidPasswordHash)` without carrying the rejected text.
  - [x] Preserve the runtime `PasswordVerifier` port and two-permit `spawn_blocking` verification path; it is not the subject of this story but must keep working.

- [x] Make root configuration reject the complete policy before startup side effects (AC: 2, 3)
  - [x] Update `src/config.rs` and, only if required by the resulting ownership boundary, `src/composition.rs` so invalid values cannot advance to SQLite connection, migration, or socket binding.
  - [x] Keep Argon2 and PHC parsing types out of `debtor-application` and `debtor-domain`; root may orchestrate a safe infra-provided validation result, while concrete cryptographic parsing remains in infra.
  - [x] Preserve generic root error conversion to `StartupError::Configuration`; no error, log, span, or test assertion may interpolate a password or hash.

- [x] Strengthen helper and operator evidence without changing its boundary (AC: 1, 4)
  - [x] Keep `tools/password-hash` as its own Cargo workspace using hidden terminal input and `Zeroizing` for password, confirmation, salt, and rendered hash.
  - [x] Keep the fixed helper profile: Argon2id v19, `m=19456,t=2,p=1`, 16-byte `SysRng` salt, and 32-byte output.
  - [x] Add direct tests for the helper's decoded salt/output lengths and profile, without committing a generated administrator hash or logging secrets.
  - [x] Synchronize `.env.example` and `README.md` only where needed to state the exact bounded startup contract and safe helper invocation. Do not claim superseded scaffold behavior as current product behavior.

- [x] Add invariant-owning tests and validate both workspaces (AC: 1-4)
  - [x] Add infra tests for each inclusive parameter boundary plus invalid length, algorithm, version, extra/duplicate/missing parameter, malformed/noncanonical PHC, decoded salt, and output cases.
  - [x] Add root configuration/composition tests proving absent and every invalid policy class fail with no SQLite file/database connection/migration/socket side effect and a sanitized error.
  - [x] Confirm valid hashes at both allowed policy extremes remain accepted; do not restrict production to the helper's one fixed profile.
  - [x] Run the required production and helper validation commands listed below.

### Review Findings

- [x] [Review][Patch] Cover parseable noncanonical PHCs in the no-side-effect startup test [src/main.rs:155]
- [x] [Review][Patch] Update the required targeted test command after the test rename [_bmad-output/implementation-artifacts/1-1-prepare-and-validate-the-administrator-password.md:127]

## Dev Notes

### Scope And Boundaries

- This is the first Epic 1 story. It establishes only secure hash generation and startup policy validation.
- Do not implement SQLite startup, migrations, socket admission, provider-independent startup, or manifest/workspace restructuring. Those are Story 1.2.
- Do not implement restart/shutdown behavior (Story 1.3), any Login page/session/CSRF/submission-token route (Stories 1.4-1.7), login rate limiting, or session promotion. Keep existing runtime behavior intact.
- No UX registry ID applies because this story adds no rendered surface. The only user-visible condition is a sanitized startup configuration failure.
- This is a pre-release project: remove superseded permissive validation instead of adding a compatibility path.

### Existing Implementation To Extend

| Path | Current state | Required change / preservation |
| --- | --- | --- |
| `debtor-infra/src/auth/password.rs` | `ArgonPasswordGate::new` parses a PHC hash, checks algorithm/version, unique `m/t/p`, parameter ranges, decoded salt/output sizes, and maps failure safely. `verify` uses a global two-permit semaphore and `spawn_blocking`. | Add the missing cheap encoded-length and canonical-structure admission before PHC parse/KDF. Preserve safe error mapping and runtime verification concurrency. |
| `src/config.rs` | Only requires a present, non-whitespace hash; retains arbitrary text in `Config`. | Make complete password policy validation part of accepted startup configuration or otherwise expose a single, direct, testable pre-side-effect validation boundary. Existing dummy `"hash"` fixtures must become valid test PHC values if config validates eagerly. |
| `src/composition.rs` | Validates `ArgonPasswordGate` before `db::connect` and migrations, then composes all adapters. | Preserve the pre-DB ordering. Avoid duplicate parsing/validation; call the shared policy rather than maintaining divergent root and infra rules. |
| `src/main.rs` | Converts config/composition errors into `StartupError::Configuration`, then binds only after `build_app` succeeds. Includes one malformed-hash no-database-side-effect test. | Expand startup ordering evidence to all required rejected classes while retaining generic diagnostics. |
| `tools/password-hash/src/main.rs` | Independent helper already uses `rpassword`, `Zeroizing`, OS RNG, and the required fixed Argon2 profile; it prints a shell assignment for the operator. | Retain the intentional stdout assignment as the helper's sole operator output. Do not treat it as an application diagnostic or write its value to fixtures/logs/files. Add profile evidence only. |
| `.env.example`, `README.md` | Document helper usage and a blank required environment variable. README's status paragraph still describes superseded brownfield behavior. | Keep secrets blank; clarify exact policy if necessary. Update stale status language only if touched, without broad unrelated documentation work. |

### Password Policy Contract

- `APP_ADMIN_PASSWORD_HASH` is required and must be at most **256 encoded bytes**. Do not trim or normalize a supplied hash into acceptance; reject invalid text.
- Validation must reject cheaply, before database connection, migration, socket admission, password verification, or KDF work.
- Accepted PHC values are canonical Argon2id version 19 with exactly the parameter names `m`, `t`, and `p`, each appearing once. No extra PHC parameters are allowed.
- Accepted ranges: `m=19_456..=65_536` KiB, `t=2..=5`, `p=1..=4`, decoded salt length `16..=64` bytes, output length `32..=64` bytes.
- The helper emits the fixed allowed minimum profile, but the validator must accept every value within the full production ranges.
- Treat canonicality as an explicit security property of the original input. Do not rely solely on semantic field checks if the parser accepts alternate textual encodings. Use the pinned RustCrypto APIs to parse only after bounded structural admission, then prove the parsed representation corresponds to the required canonical original format.
- `PasswordHash::new` is a parser for PHC text and exposes `algorithm`, `version`, `params`, `salt`, and `hash`; `Salt::decode_b64` determines decoded salt size. [Source: Context7 `/websites/rs_argon2`, Argon2 0.5.3 `PasswordHash` documentation]

### Architecture Compliance

- Preserve `debtor (root) -> debtor-web / debtor-infra -> debtor-application -> debtor-domain`.
- `debtor-infra` owns Argon2, PHC parsing, cryptographic implementation, and the concrete `ArgonPasswordGate`. Application retains only its narrow `PasswordVerifier` port; domain stays cryptography-free.
- Root owns environment lookup, startup ordering, and conversion into safe `StartupError` categories. It must not expose raw PHC parser or Argon2 diagnostics.
- Use `thiserror`-based typed application/adapter errors; keep `anyhow` only in root process/config orchestration.
- No `unsafe`; avoid production `unwrap` and `expect`; keep test-only lint allowances narrowly scoped.
- Do not change SQL, migrations, persistence, routes, or application/domain data models. SQLx metadata preparation is not needed unless scope changes to SQL/migrations.

### Security And Regression Guardrails

- Never log, return, assert against, serialize, fixture, or commit the configured hash or administrator password. Test strings and synthetic PHC values may exist only as non-secret test data and must not be emitted in diagnostics.
- Keep `StartupError::Configuration` generic (`"invalid application configuration"`) and source-free. Verify no rejected hash content leaks through `Display`, tracing, or error chains.
- Invalid configuration must cause no SQLite path creation, migration, database connection, socket binding, or password KDF/verification.
- Preserve `.env` in `.gitignore`; `.env.example` must contain only a commented blank value.
- Preserve the independent helper workspace declaration (`[workspace]` in its own manifest). Do not add it to production `workspace.members` or share its lockfile.
- Preserve fixed Rust toolchain `1.97.1`, edition 2024, resolver 3, locked dependencies, and existing versions including Argon2 `0.5.3`.

### Testing Requirements

- Infra unit tests own PHC-policy parsing and parameter boundary behavior.
- Root tests own configuration/startup ordering and no-external-side-effect evidence. Use a temporary database path and prove it remains absent after each invalid input.
- Helper tests own its exact generation profile and password input validation. Do not automate its interactive terminal flow by placing a real password in shell history or fixture files.
- Reuse existing tests in `debtor-infra/src/auth/password.rs`, `src/config.rs`, `src/main.rs` composition tests, and `tools/password-hash/src/main.rs`; do not create a new test framework.
- Required targeted commands, run from `debtor/`:

```bash
cargo test -p debtor-infra auth::password
cargo test -p debtor config::tests
cargo test --bin debtor composition_tests::invalid_password_hashes_have_no_database_side_effect
cargo test --manifest-path tools/password-hash/Cargo.toml --locked
```

- Required full validation, run from `debtor/`:

```bash
cargo fmt --all -- --check
cargo run --bin architecture-check --locked
cargo check --workspace --all-features --locked
SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked

cargo fmt --manifest-path tools/password-hash/Cargo.toml -- --check
cargo clippy --manifest-path tools/password-hash/Cargo.toml --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path tools/password-hash/Cargo.toml --locked
```

- Run `cargo deny check` only if dependencies or dependency-policy files change. Never use `cargo build --release` for validation.

### Project Structure Notes

- The repository root is the Rust workspace; implementation paths in this story are relative to that root.
- Planning artifacts are at repository root under `_bmad-output/`; the nested `debtor/_bmad-output/` copy is older. Use the root-level story plan and this story file as the current assignment source.
- No prior story exists. Recent commits are planning/agent artifacts, not implementation precedent; rely on the existing nested codebase patterns documented above.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 1.1: Prepare and Validate the Administrator Password`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Cross-Cutting Story Rule`]
- [Source: `specs/design.md#Security`]
- [Source: `_bmad-output/project-context.md#Technology Stack & Versions`]
- [Source: `_bmad-output/project-context.md#Framework-Specific Rules`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md#Authentication, Sessions, And CSRF`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-1 - Inward dependency direction [ADOPTED]`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-14 - Bounded operational envelope [ADOPTED]`]
- [Source: `debtor/debtor-infra/src/auth/password.rs`]
- [Source: `debtor/src/config.rs`]
- [Source: `debtor/src/composition.rs`]
- [Source: `debtor/tools/password-hash/src/main.rs`]
- [Source: Context7 `/websites/rs_argon2`, Argon2 0.5.3 `PasswordHash` documentation]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-terra

### Debug Log References

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Red tests added for oversized and noncanonical PHC input, then made green with shared infrastructure validation and root configuration admission.
- Full production and helper validation passed: formatting, Clippy, tests, and architecture fitness.

### Completion Notes List

- Status set to `ready-for-dev`.
- Story scope and guardrails verified against the current epics, design contract, project context, codebase, recent git history, and current Argon2 documentation.
- Implemented canonical bounded Argon2id v19 validation before root configuration acceptance and preserved pre-database composition validation.
- Added canonical-form, encoded-length, safe-error, no-database-side-effect, and helper salt-length test coverage.
- Validation passed: `cargo fmt --all -- --check`, `cargo check --workspace --all-features --locked`, `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`, `cargo run --bin architecture-check --locked`, plus helper fmt, Clippy, and tests.

### File List

- `_bmad-output/implementation-artifacts/1-1-prepare-and-validate-the-administrator-password.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `.env.example`
- `debtor-infra/src/auth.rs`
- `debtor-infra/src/auth/password.rs`
- `src/config.rs`
- `src/main.rs`
- `tools/password-hash/src/main.rs`

### Change Log

- 2026-08-12: Implemented bounded canonical administrator password-hash validation and startup admission tests; status moved to review.

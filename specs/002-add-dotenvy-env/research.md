# Research: dotenvy Environment Variable Integration

## Research Tasks

### Task 1: dotenvy runtime API behavior

**Decision**: Use `dotenvy::dotenv()` as the primary runtime loader
**Rationale**: `dotenvy::dotenv()` loads `.env` from the current directory (or parent directories) into the process environment. It does NOT override existing env vars (system env takes precedence). Returns `Result<PathBuf>` — errors if file exists but can't be read/parsed. `dotenvy::from_filename()` is an alternative for explicit path, but `dotenv()` searches upward and is the standard convention.
**Alternatives considered**:
- `dotenvy::from_filename(".env")` — explicit path, but loses parent-directory search behavior
- `dotenvy::from_path()` — even more explicit, but unnecessary for standard `.env` at project root

### Task 2: dotenvy_macro compile-time behavior

**Decision**: Use `dotenvy_macro::dotenv!()` for compile-time env var inclusion
**Rationale**: `dotenv!()` reads from `.env` at compile time and embeds the value as a `&'static str`. It panics if the variable is missing. This is suitable for baking config into the binary. The macro reads the `.env` file at the project root during compilation.
**Alternatives considered**:
- `std::env!()` — built-in Rust macro, but doesn't read `.env` file
- Custom build.rs — more complex, same result

### Task 3: Integration pattern with existing AppConfig::from_env()

**Decision**: Call `dotenvy::dotenv()` in `main()` before `AppConfig::from_env()`
**Rationale**: The existing `AppConfig::from_env()` uses `std::env::var()` which reads from the process environment. `dotenvy::dotenv()` populates the process environment from `.env`. So the integration is a single call at startup — no changes to `AppConfig::from_env()` needed. This preserves all existing tests and behavior.
**Alternatives considered**:
- Wrapping `dotenvy::dotenv()` inside `AppConfig::from_env()` — couples config struct to dotenvy dependency
- Using `dotenvy::vars()` iterator to build config — unnecessary complexity, duplicates existing logic

### Task 4: Error handling strategy

**Decision**: Use `dotenvy::dotenv().unwrap_or_else()` to fail fast with clear error if `.env` exists but is malformed/unreadable
**Rationale**: `dotenvy::dotenv()` returns `Err(DotenvError)` only if the file was found but couldn't be processed. If no `.env` exists, it returns `Err(NotPresent)` which we should ignore (graceful fallback). We need to distinguish between "file missing" (OK) and "file exists but broken" (fail fast).
**Alternatives considered**:
- Using `dotenvy::dotenv_iter()` for fine-grained error control — over-engineered for this use case
- Silently ignoring all errors — violates FR-007 fail-fast requirement

### Task 5: .gitignore negation pattern

**Decision**: Add `!.env.example` after existing `.env*` in `.gitignore`
**Rationale**: Git processes `.gitignore` rules in order. The negation `!` re-includes a previously excluded file. Since `.env*` excludes `.env.example`, the negation `!.env.example` restores it. This is the standard Git pattern for selectively including files within wildcard exclusions.
**Alternatives considered**:
- Changing `.env*` to `.env` — simpler but loses protection for `.env.local`, `.env.test`, etc.
- Using a separate `.gitignore` in subdirectory — unnecessary complexity

### Task 6: .env.example content structure

**Decision**: Include all 6 variables with placeholder values, comments explaining each, and `export` prefix convention
**Rationale**: The `.env.example` serves as both documentation and a copy-paste template. Comments explain the purpose and valid values for each variable. Placeholder values are safe defaults (not secrets).
**Alternatives considered**:
- Minimal template with just keys — less helpful for onboarding
- Separate README for config docs — splits documentation, harder to maintain

## Summary

All technical unknowns resolved. The integration is straightforward:
1. Add `dotenvy` + `dotenvy_macro` to `Cargo.toml`
2. Add `dotenvy::dotenv()` call in `main.rs` (with error handling)
3. Create `.env.example` template
4. Update `.gitignore` with negation rule
5. Write tests first (TDD per constitution VI)

# ADR 0002: Long-Term Foundation Hardening

- Status: Accepted
- Date: 2026-08-04
- Scope: Post-foundation maintainability, dependency governance, and edge protocol policy

## Context

The foundation is intentionally single-process, single-administrator, local SQLite behind a sanitizing reverse proxy. The existing architecture is strong, but long-lived operation requires application-owned spending policy, bounded authenticated sessions and caches, scalable ordinary reads, reproducible dependencies, and a fitness check that cannot silently accept missing production crates.

## Decisions

1. Preserve the one-process, one-admin, local-SQLite topology. Horizontal scaling, shared sessions, external writers, and persistent migration compatibility remain out of scope before first release.
2. Pin the tested Rust toolchain to `1.97.1`. CI uses the pinned toolchain rather than moving stable.
3. Keep `thiserror` for typed domain/application failures and `anyhow` only at the root process boundary. No error-framework replacement is justified.
4. Move raw spending parsing, payer/share policy, allocation construction, and eligibility orchestration into application commands. Web decodes transport fields only.
5. Cap authenticated sessions at 32 without eviction. A full-cap correct-password login flushes the anonymous session and returns retryable `503`; existing authenticated sessions are never evicted.
6. Cap stable historical rate contexts at 4,096 with deterministic standard-library LRU indexing. Cache eviction can refetch only.
7. Use 25-item keyset pagination for ordinary spending history and direct complete aggregate reads for detail/edit/delete. Full snapshots remain reserved for debt calculation.
8. Keep HTTP/3/QUIC at the reverse proxy. The application remains a private TCP HTTP backend; unsafe early data must not reach mutations.
9. Trim dependency features before isolated Askama, reqwest, tower-sessions, and SQLx upgrades. `cargo-deny` enforces advisories, sources, and reviewed permissive licensing; Dependabot groups weekly patch/minor updates and leaves majors isolated.
10. Architecture fitness checks required package presence and normal/build dependency direction from `cargo metadata`. Responsibility boundaries are enforced by targeted compile/integration tests rather than brittle source-token scans.

## Consequences

- Financial policy has one reusable application authority for HTML and future adapters.
- Authenticated memory is bounded without invalidating existing administrator sessions.
- Ordinary pages remain bounded as history grows while debt calculations retain snapshot semantics.
- HTTP/3 can be adopted at the edge without introducing application QUIC/TLS complexity.
- Dependency upgrades remain reviewable and reversible, at the cost of several isolated migration stages.
- Pre-release local databases may still need recreation after index/schema changes.

## Superseded Decisions

This ADR does not supersede ADR 0001's topology, exact accounting, SQLite durability, session restart invalidation, or proxy-owned TLS decision. It refines application policy ownership and long-term operational bounds.

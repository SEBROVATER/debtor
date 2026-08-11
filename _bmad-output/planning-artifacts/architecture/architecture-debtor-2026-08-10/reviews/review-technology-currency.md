# Technology Currency Review

**Artifact reviewed:** `../ARCHITECTURE-SPINE.md`  
**Review date:** 2026-08-10  
**Verdict:** PASS WITH LOW-RISK OBSERVATIONS

## Scope And Method

This review inventoried every committed named implementation technology in the spine, including technologies named outside the Stack table. Exact versions were checked against the committed toolchain and lockfile, the live crates.io index, installed CLI versions, official package registries, official project documentation, and a locked workspace compatibility build. Claims about relevant SQLite defaults were checked against current SQLite documentation and the actual connection configuration.

Evidence commands completed successfully:

- `rustc --version` and `cargo --version`: Rust/Cargo `1.97.1`.
- `cargo sqlx --version`: `sqlx-cli-sqlx 0.9.0`.
- `cargo deny --version`: `cargo-deny 0.20.2`.
- `cargo search` against crates.io for Axum, Askama, Tokio, SQLx, sqlx-cli, reqwest, rust_decimal, tower-sessions, Tower, tower-http, Argon2, and cargo-deny.
- `cargo tree -i libsqlite3-sys@0.37.0 -e features --locked`: proves SQLx's `sqlite-bundled` path activates `libsqlite3-sys/bundled`.
- `cargo check --workspace --all-features --locked`: passed.
- `cargo deny check`: advisories, bans, licenses, and sources all passed.

## Findings

### LOW - Research provenance is not auditable from the spine itself

The spine's front matter cites only `specs/design.md`, while `.memlog.md` says the versions were verified but records no URLs, retrieval dates, registry responses, or source revisions. The technology claims are correct as of this review, but a reader cannot distinguish researched facts from assertions using the spine and its declared sources alone. This review supplies the missing evidence trail; no spine correctness change is required.

### INFO - The bundled SQLite binding crate is behind its latest release, but is the newest compatible SQLx 0.9 line

`Cargo.lock` selects `libsqlite3-sys 0.37.0`; crates.io currently offers `0.38.1`. This is not an accidental stale or incompatible selection: SQLx `0.9.0` declares `libsqlite3-sys >=0.30.1, <0.38.0`, so `0.38.1` is excluded. The active `bundled` feature is proven by `cargo tree`, and the vendored amalgamation in `libsqlite3-sys 0.37.0` identifies itself as SQLite `3.51.3`. SQLite's official WAL documentation says the rare WAL-reset corruption defect is fixed in `3.51.3` and later. Therefore the spine's "bundled SQLite" choice is compatible and patched, although the indirect SQLite version should remain lockfile-monitored.

### INFO - HTMX has two materially different "latest" channels; the spine correctly chooses stable

The official npm `latest` package is `htmx.org 2.0.10`. GitHub's latest-release endpoint currently returns `v4.0.0-beta5` even though the tag itself is beta and the API record has `prerelease: false`. The official `htmx-ext-response-targets 2.0.4` npm package depends on `htmx.org ^2.0.2`, which accepts `2.0.10` and does not accept `4.0.0-beta5`. The spine's `2.0.10 + 2.0.4` pair is therefore current, stable, and compatible. Future automated checks must not treat GitHub's latest-release endpoint as the stable channel without inspecting the semantic version.

## Verification Matrix

| Spine commitment | Evidence and current status | Result |
| --- | --- | --- |
| Rust `1.97.1`, edition 2024 | `rust-toolchain.toml` pins `1.97.1`; installed compiler is `rustc 1.97.1`; the official stable distribution manifest is dated 2026-07-16 and identifies `1.97.1`; `Cargo.toml` uses edition 2024 and resolver 3. | Confirmed current stable |
| Axum `0.8.9` | `Cargo.lock` selects `0.8.9`; live crates.io search reports `0.8.9`; locked all-feature workspace check passes. | Confirmed current stable and compatible |
| Askama `0.16.0` | `Cargo.lock` selects `0.16.0`; live crates.io search reports `0.16.0`; locked all-feature workspace check passes. | Confirmed current stable and compatible |
| Tokio `1.53.1` | `Cargo.lock` selects `1.53.1`; live crates.io search reports `1.53.1`; locked all-feature workspace check passes. | Confirmed current stable and compatible |
| SQLx `0.9.0` | `Cargo.lock` selects SQLx and its component crates at `0.9.0`; live crates.io search reports `0.9.0`; official 0.9 docs retain checked `query!` macros and `.sqlx` offline preparation; build passes. | Confirmed current stable and fit |
| sqlx-cli `0.9.0` | Installed `cargo sqlx` reports `0.9.0`; live crates.io search reports `0.9.0`; official SQLx 0.9 CLI manifest supplies `cargo-sqlx` and the `prepare` command. | Confirmed current stable and fit |
| Bundled SQLite | SQLx 0.9 officially exposes `sqlite-bundled`; active feature tree reaches `libsqlite3-sys/bundled`; locked binding `0.37.0` vendors SQLite `3.51.3`. | Confirmed compatible; indirect version monitored |
| SQLite WAL, `synchronous=FULL`, foreign keys, five-second busy timeout | Official SQLite defaults are rollback journal (`DELETE`) and foreign keys off, so explicit configuration is required. `debtor-infra/src/db.rs` explicitly sets WAL, FULL, foreign keys on, and five seconds; adapter tests assert WAL/FULL/5000 ms. SQLite documents FULL as syncing WAL commits and WAL snapshots as stable read end marks. | Confirmed; no reliance on unsafe defaults |
| reqwest `0.13.4`, rustls | `Cargo.lock` selects `0.13.4`; live crates.io search reports `0.13.4`; manifest disables defaults and enables `rustls`; feature tree includes reqwest rustls, rustls `0.23.43`, and AWS-LC; official reqwest docs identify `rustls` as the Rust TLS backend. | Confirmed current stable and compatible |
| rust_decimal `1.42.1` | `Cargo.lock` selects `1.42.1`; live crates.io search reports `1.42.1`; official docs provide fixed-precision `Decimal`, exact parsing, normalization, and checked arithmetic. The project enables `serde-with-arbitrary-precision`, which activates serde_json `arbitrary_precision`; the provider test proves 28-digit lexical decoding without `f64`. | Confirmed current stable and fit within Decimal's documented 28-scale bound |
| HTMX `2.0.10` | Official npm `latest` is `2.0.10`; package metadata provides the self-hostable distribution files and integrity hash. | Confirmed current stable |
| HTMX response-targets `2.0.4` | Official npm `latest` is `2.0.4`; its dependency is `htmx.org ^2.0.2`, compatible with `2.0.10`; repository and homepage point to the official htmx extensions project. | Confirmed current stable and compatible |
| tower-sessions | `Cargo.lock` selects `0.15.0`; live crates.io search reports `0.15.0`; description explicitly identifies Tower/Axum middleware; locked build passes with Axum `0.8.9`. | Confirmed current stable and compatible |
| Argon2 / Argon2id v19 | `Cargo.lock` selects stable `argon2 0.5.3`; crates.io currently exposes `0.6.0-rc.8` as a prerelease, not a stable replacement; official 0.5.3 docs state `Argon2::default()` is Argon2id v19 and supports PHC verification. | Confirmed current stable; no prerelease adopted |
| cargo-deny | Installed `0.20.2` matches the live crates.io release; committed `deny.toml` enforces advisories, registries, licenses, and bans; `cargo deny check` passes. | Confirmed current and operational |
| HTTP/1.1, HTTP/2, HTTP/3/QUIC, TLS, `Alt-Svc` | These are protocol/topology responsibilities, not pinned application libraries. The reverse-proxy vendor is explicitly deferred, so there is no vendor version or default claim to validate yet. | Confirmed not falsely pinned; deployment-time verification remains deferred |

## Prerelease, Outdated, Incompatible, And Unconfirmed Scan

- **Pinned prereleases:** none.
- **Incompatible committed pairs:** none.
- **Outdated direct Stack-table pins:** none as of 2026-08-10.
- **Outdated transitive components:** `libsqlite3-sys 0.37.0` trails `0.38.1`, but SQLx 0.9 excludes 0.38 and the selected crate bundles patched SQLite 3.51.3; this is a justified compatibility hold, not a spine defect.
- **Unconfirmed named technology claims:** none after this review.
- **Advisory or source-policy failures:** none reported by `cargo deny check`.

## Official Sources

- Rust stable distribution manifest: https://static.rust-lang.org/dist/channel-rust-stable.toml
- Rust 1.97.1 dated manifest: https://static.rust-lang.org/dist/2026-07-16/channel-rust-1.97.1.toml
- crates.io registry/API: https://crates.io/ (Axum, Askama, Tokio, SQLx, sqlx-cli, reqwest, rust_decimal, tower-sessions, Argon2, cargo-deny, libsqlite3-sys)
- SQLx 0.9 checked query and offline mode docs: https://docs.rs/sqlx/0.9.0/sqlx/macro.query.html
- SQLx 0.9 manifests and SQLite feature bounds: https://github.com/launchbadge/sqlx/tree/v0.9.0
- reqwest 0.13.4 TLS backend docs: https://docs.rs/reqwest/0.13.4/reqwest/tls/index.html
- rust_decimal 1.42.1 Decimal docs: https://docs.rs/rust_decimal/1.42.1/rust_decimal/struct.Decimal.html
- Argon2 0.5.3 docs: https://docs.rs/argon2/0.5.3/argon2/
- HTMX npm stable metadata: https://registry.npmjs.org/htmx.org/latest
- response-targets npm stable metadata: https://registry.npmjs.org/htmx-ext-response-targets/latest
- HTMX GitHub latest release metadata: https://api.github.com/repos/bigskysoftware/htmx/releases/latest
- SQLite PRAGMA defaults and semantics: https://www.sqlite.org/pragma.html
- SQLite WAL semantics and WAL-reset fix: https://www.sqlite.org/wal.html

## Conclusion

Every committed named technology and exact version in the spine now has local or official-current evidence. The selected stack is current, stable, available, mutually compatible, and operational under the locked workspace. No high-, medium-, or correctness-blocking technology-currency issue was found. The only actionable weakness is evidence provenance: future architecture version decisions should record official URLs and retrieval dates rather than only asserting that a check occurred.

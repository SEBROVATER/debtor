# Security Architecture Review - Round 2

## Verdict

**REJECT pending residual security closure.** The second pass finds one high-severity lifecycle contradiction and four narrower boundary gaps. All other findings from the first security review are sufficiently closed by the combined architecture set and are intentionally omitted.

## Review Basis

- Reviewed artifact: `ARCHITECTURE-SPINE.md`, updated 2026-08-10.
- Normative baseline: `specs/design.md`.
- Synchronized decision record: `specs/adr/0001-foundation-architecture.md`.
- Retest scope: Argon policy, trusted-client resolution, cookies and sessions, unsafe-request ordering, submission-token state, mutation shutdown registry, early data, startup barrier, diagnostics, and HTMX provenance.
- Reporting rule: unresolved architecture holes only; no implementation findings and no repeated closed findings.

## Findings

### SEC2-01 - HIGH - The mutation registry does not reconcile definitive completion with bounded shutdown

**Evidence:** Spine AD-14 says HTTP drains for at most ten seconds, then the root mutation registry reaches each dispatched mutation's "intrinsically bounded terminal result" before checkpoint and pool close (`ARCHITECTURE-SPINE.md:176`). No fixed post-dispatch bound is defined or proved. Composite mutation work is explicitly placed after dispatch by AD-10, including archival snapshot, potentially many bounded-but-batched provider calls, calculation, write-gate acquisition, and commit (`ARCHITECTURE-SPINE.md:138`). The ledger size and number of rate contexts are not fixed, so individual I/O timeouts do not create a fixed end-to-end shutdown bound. Meanwhile, the normative design and ADR still require shutdown to drain for at most ten seconds and then checkpoint and close, without the registry barrier (`specs/design.md:116`; `specs/adr/0001-foundation-architecture.md:65`).

**Architecture hole:** A lifecycle implementation can either honor the ten-second shutdown contract and interrupt or outlive a dispatched mutation, or honor the registry barrier and exceed the normative shutdown bound. The phrase "intrinsically bounded" does not provide a duration, admission proof, or terminalization mechanism for a stalled task. Independently implemented lifecycle and mutation epics therefore still cannot satisfy one unambiguous contract.

**Required closure:** Choose and synchronize one rule. Either impose and verify a fixed post-dispatch upper bound that fits inside the shutdown budget, or explicitly limit ten seconds to HTTP draining and require the process to remain alive without checkpoint/pool close until the registry is empty. Define registry terminalization for panic/task failure and make the chosen total-shutdown semantics binding in `specs/design.md`, ADR 0001, and AD-14.

### SEC2-02 - MEDIUM - The Argon policy still lacks a total encoded-input bound and cheap rejection contract

**Evidence:** Spine AD-14 now fixes Argon2id v19, `m/t/p`, salt, and output bounds, but does not cap the total PHC string length or require length/grammar/parameter rejection before allocation, decoding, or KDF work (`ARCHITECTURE-SPINE.md:176`). The normative design and ADR continue to say only "bounded" or "complete bounded" policy (`specs/design.md:87`; `specs/adr/0001-foundation-architecture.md:65`).

**Architecture hole:** A validator can satisfy every listed decoded-field bound while accepting an arbitrarily large encoded configuration value and performing disproportionate parsing or decoding before discovering that a decoded field exceeds its limit. Separate startup, verification, and helper implementations can also disagree about noncanonical PHC encodings while claiming the same parameter policy.

**Required closure:** Add one maximum encoded-byte length and a canonical PHC grammar to the named policy, require length and structural rejection before decoding or KDF work, and bind startup validation, runtime verification, the password helper, and policy tests to that same contract. Synchronize the concrete policy into the normative design and ADR.

### SEC2-03 - MEDIUM - Production trusted-proxy configuration admission remains implicit

**Evidence:** Spine AD-12 now defines the resolver, socket-peer trust, one header mode, right-to-left chain walking, malformed-input rejection, direct-peer fallback, and sole ownership (`ARCHITECTURE-SPINE.md:150,170`). AD-14 says startup validates "proxy policy" but does not define which semantically unsafe configurations fail startup (`ARCHITECTURE-SPINE.md:176`). The design requires production to run behind a sanitizing proxy and says only configured proxy CIDRs may supply forwarding input, but does not require a nonempty production trust set or reject a selected forwarding mode that cannot match the deployed topology (`specs/design.md:41,89,93,98`).

**Architecture hole:** An implementation may treat an empty production CIDR set as valid. Every proxied request then resolves to the shared edge address, collapsing the login limiter into one attacker-exhaustible identity while still satisfying the resolver's direct-peer fallback. Generic "proxy policy" validation does not make this deployment-invalid state testable.

**Required closure:** Define production configuration validity: the trusted-proxy CIDR set must be nonempty, the selected header mode must be recognized and singular, and configuration that cannot represent the required edge path must fail before socket admission. Keep explicit direct-peer fallback for debug/local or genuinely unproxied supported modes only.

### SEC2-04 - MEDIUM - Successful login does not explicitly persist promotion before redirect

**Evidence:** AD-14 requires anonymous-session persistence before login HTML and says correct-password promotion atomically rotates session ID and CSRF (`ARCHITECTURE-SPINE.md:176`). Neither the spine, design, nor ADR explicitly requires the promoted authenticated session and rotated CSRF state to be successfully stored before the success redirect is emitted (`specs/design.md:87`; `specs/adr/0001-foundation-architecture.md:55`).

**Architecture hole:** A session adapter can interpret "atomic promotion" as an in-memory request mutation and defer fallible persistence until after constructing or releasing the redirect. A save failure can then issue an authenticated-looking redirect with a cookie that has no corresponding authenticated server state, producing ambiguous login state and inconsistent retry/limiter behavior.

**Required closure:** Make successful persistence of the rotated session ID, authenticated state, and new CSRF token part of the atomic promotion boundary and a prerequisite to emitting the redirect or `Set-Cookie`. Map persistence failure to fixed retryable feedback with no authenticated cookie issued.

### SEC2-05 - LOW - HTMX provenance is gated, but static executable response handling is not

**Evidence:** AD-11 now requires exact official HTMX and `response-targets` bytes and digests to be recorded and CI-verified before use, and forbids other executable application assets (`ARCHITECTURE-SPINE.md:144,284`). Its `X-Content-Type-Options: nosniff` rule applies to login and authenticated HTML, not explicitly to the script responses themselves, and no fixed JavaScript content type is required for vendored executable assets (`ARCHITECTURE-SPINE.md:144`).

**Architecture hole:** Proven bytes may still be served through a generic static adapter with an incorrect or attacker-influenced content type and without `nosniff`. The provenance gate proves file identity but does not fully constrain how executable bytes cross the HTTP boundary.

**Required closure:** Require approved executable assets to be served only from fixed routes with a fixed JavaScript media type, `X-Content-Type-Options: nosniff`, and immutable asset-to-digest mapping. Include these response properties in the vendoring/CI acceptance gate.

## Closure Gate

SEC2-01 must be resolved before independent lifecycle and mutation implementation. SEC2-02 through SEC2-04 must be made binding before authentication and production deployment are accepted. SEC2-05 must close before HTMX assets ship.

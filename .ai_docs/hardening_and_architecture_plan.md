# Hardening & Architecture Plan — B- → A on the non-GUI code

## Context

A three-part review (July 2026) graded the non-GUI code C+ as-is, B- for
architecture. The domain design is sound — typestate `Wallet<Locked/Unlocked>`,
injected `Env` capabilities, real fakes, an offline e2e test — but it carries one
**critical cryptographic bug** and a set of layering/enforcement gaps that keep it
short of an A. This plan closes them.

The verdict was **keep, don't rewrite**: every item below is a localized patch or a
contained refactor, not a redesign. The GUI is being replaced separately (iced →
Lumen); this plan deliberately makes the domain framework-agnostic so that swap
touches nothing underneath the UI.

**Pre-1.0, no users, no deployed data.** Any stored artifact (wallets, DBs) can be
wiped and regenerated in dev. So this plan contains **no data-migration or
backward-compatibility code** — formats change outright, old data is discarded.
Version stamps are still added where noted, but purely as *forward-looking* discipline
(the first real format change after 1.0 gets a hook, and unknown versions fail closed);
they are not there to migrate anything that exists today.

Guiding principle for the "A": **dependency direction and correctness must be
enforced by the compiler and CI, not by discipline.** A solo codebase's architecture
is only as durable as what CI refuses to merge.

Commit convention (per `CLAUDE.md`): one commit per completed point, with a clear
message. Each phase lists its commit boundaries.

---

## Phase ordering & rationale

Correctness-critical work first (Phases 0–2), then the architectural refactors
(Phases 3–5). "Risk" below is risk to *this codebase's* correctness during the change,
not to users (there are none). Phases 3–5 raise the architecture grade and can follow
at any pace.

| Phase | Theme | Risk | Must-fix before real funds? |
|---|---|---|---|
| 0 | Enforcement baseline + dep pinning | low | — (enables the rest) |
| 1 | Localized security & correctness fixes | low | yes (Windows, panics) |
| 2 | Sealed crypto crate + versioned secret format | medium | **yes (GCM nonce)** |
| 3 | Dependency direction: kill `0_deps`, de-leak iced, dedup | medium | no |
| 4 | Ports to the consumer (true hexagonal) | high | no |
| 5 | Contract tests + DB schema versioning | medium | no |

---

## Phase 0 — Enforcement baseline & dependency pinning

**Goal:** put the gates in place before changing code, so every later phase is
checked. Addresses review weakness #4 (wildcard crypto deps) and improvement #7.

Changes:
- **Pin all security-critical deps** in root `Cargo.toml:81-86`: replace `ring = "*"`,
  `rand = "*"`, `tiny-bip39 = "*"`, `slip10_ed25519 = "*"`, `ed25519-dalek-fiat = "*"`
  with exact versions matching the current `Cargo.lock`. Open a tracking note to
  re-evaluate `ed25519-dalek-fiat` vs mainstream `ed25519-dalek` (Phase 2).
- **Add `deny.toml`** (cargo-deny): ban wildcard versions, enable the advisories +
  licenses + bans checks. Lumen already ships one — copy its shape.
- **CI workflow** (`.github/workflows/ci.yml`): `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo test --workspace`, `cargo deny check`, and a
  **Windows runner job** (the Windows secrets path is currently untested — see 1b).
- **`#![forbid(unsafe_code)]`** added crate-by-crate as each is cleaned. Add it now
  to every crate that is already unsafe-free; `1_types` gets it in Phase 1 after
  `unsafe_reference.rs` is deleted.
- Clear the existing warnings so `-D warnings` passes (tests currently pass *with*
  warnings).

Verification: CI green on a no-op PR; `cargo deny check` passes; `cargo build`
reproducible against pinned lock.

Commits: (0a) pin crypto deps, (0b) add deny.toml, (0c) add CI workflow incl.
Windows job, (0d) forbid(unsafe) on already-clean crates + clear warnings.

---

## Phase 1 — Localized security & correctness fixes

**Goal:** the small, self-contained bugs. Each is one file, each its own commit.

- **1a — Windows credential blob length bug.**
  `01_ports_and_adapters/secrets_store/src/os_credential_store.rs:178` writes
  `CredentialBlobSize: (blob.len() * 2)` but reads back `/2` at `:147`. Store the
  real length; drop the `*2`/`/2` dance. Covered by the new Windows CI job.
- **1b — UTF-8 truncation panics.**
  `1_types/src/crypto/password.rs:56-57,123,136` and
  `1_types/src/crypto/seedphrase.rs:120` slice on byte offsets and panic mid-codepoint
  (and silently truncate). Use `floor_char_boundary` / `char_indices`; reject
  over-length input as an error rather than truncating a password. Add multibyte-input
  tests.
- **1c — Delete `UnsafeRef`/`UnsafeSlice`/`UnsafeRefMut`.**
  `1_types/src/unsafe_reference.rs` hands raw `&HashMap` pointers into `tokio::spawn`
  (`ledger_connector/src/radix_dlt/updates.rs:86,137`). Replace with the `Arc<HashMap>`
  already in hand at `updates.rs:72`. Delete the module; add
  `#![forbid(unsafe_code)]` to `1_types`.
- **1d — Remove dead code.** Commented-out `SyncDataBase` at the end of
  `data_stores/src/database.rs`, dead methods at `password.rs:283-307`, and the
  commented no-mangle remnants. If it's not load-bearing, it's noise that misleads the
  next reader.

Verification: `cargo test --workspace` green on Linux **and** Windows CI; new
multibyte password/seedphrase tests pass; `grep -rn unsafe 1_types/` empty.

Commits: 1a, 1b, 1c, 1d (four commits).

---

## Phase 2 — Sealed crypto crate + versioned secret format (fixes the critical bug)

**Goal:** make the GCM nonce-reuse bug *inexpressible*, and establish the
versioning/migration pattern reused in Phase 5. Addresses review weakness #1 and
improvements #4 + #5.

The bug (confirmed): `encrypted_mnemonic.rs:94-102` seals the seed phrase and the
seed password with **one** `SealingKey`, and `MnemonicNonceSequence::advance()`
(`:62`) returns the **same** nonce each call → two plaintexts under one (key, nonce),
which breaks AES-GCM confidentiality and leaks the auth key. The struct persists a
single `nonce_bytes` (`:73`) and is `Serialize`d with no version tag.

Design:
- **New `crypto` crate** (workspace member; move `1_types/src/crypto/*` into it). Its
  public API is misuse-resistant:
  - `seal(key, plaintext) -> SealedBlob` generates a **fresh random nonce internally**
    and returns it bundled in the blob. There is **no** public API accepting a
    caller-supplied nonce, and `NonceSequence` is not exposed.
  - `SealedBlob` is a `#[non_exhaustive]`, versioned, self-describing struct:
    `{ format_version: u16, nonce: [u8; 12], ciphertext: Vec<u8> }`. `open` rejects an
    unknown `format_version` (fail closed). The version exists so the *next* format
    change has a hook — it is not there to read any current data.
  - No other crate may import `ring`/`aes`/`aead` — enforce via clippy
    `disallowed_types`/`disallowed_methods` in `.cargo` config or a `deny.toml` ban.
- **Rewrite `EncryptedMnemonic`** on top of `seal`: **two independent seals with two
  independent fresh nonces** (phrase, seed-password), each a `SealedBlob`. This is the
  fix for the nonce-reuse bug. No compatibility with the old single-nonce format — any
  dev wallet in the old format is discarded and regenerated.
- **Property tests + pinned vectors:** round-trip `seal`/`open`; distinct nonces across
  seals; tamper-detection (flip a ciphertext byte → open fails); unknown-version →
  error. Keep the existing pinned address vectors for derivation.
- Resolve the `ed25519-dalek-fiat` question here: confirm it is required for the Radix
  curve variant, or migrate to mainstream `ed25519-dalek`. Document the decision in
  `decision_log.md`.

Verification: new property tests green; `grep -rn "ring::aead" --include=*.rs` returns
hits only inside the `crypto` crate; the e2e login→send passes against a freshly
generated wallet.

Commits: (2a) extract crypto crate (move, no behaviour change), (2b) misuse-resistant
`seal`/`SealedBlob` + version field, (2c) rewrite EncryptedMnemonic with two nonces,
(2d) property tests, (2e) ed25519 dep decision.

**Status (2026-07-10): 2b–2e done in place; 2a (physical crate extraction)
deferred.** The security objective is met without the move: the `crypto::sealed`
module is the sole `ring::aead` user with a nonce-reuse-proof API, the
`NonceSequence`/`SealingKey` misuse pattern is gone from the codebase, and the
two remaining `ring::aead` sites (`profile_backup`, `radix_connect::relay`)
already seal with a fresh nonce per call. Extracting a standalone `crypto` crate
is blocked by a `types` ↔ crypto cycle (ed25519 derivation references
`types::Network`, while `types::account`/`persona` use crypto), so a clean split
also has to relocate `Network` and ripples through `iced_ui` (which is being
replaced). Deferred until after Phase 3 (killing `0_deps` makes per-crate deps
explicit) or the GUI swap, whichever comes first. Tracked in `decision_log.md`.

---

## Phase 3 — Dependency direction: kill `0_deps`, de-leak iced, dedup

**Goal:** make the crate graph match the architecture diagram. Addresses improvements
#2 + #3 and the review's "iced in `1_types`" secondary smell.

- **3a — Drop `iced` from the domain.** Move the `impl From<Theme> for iced::Theme`
  out of `1_types/src/theme.rs:65` into the UI layer (it's ~30 lines). `1_types` keeps
  only the framework-neutral `Theme` enum.
- **3b — Delete the duplicate `app_path`.** The `01_ports_and_adapters/app_path/`
  trait crate (with its `get() -> &'static Self` global flavour) is unused; the live
  type is `AppPathInner` in `1_types/src/app_path.rs`. Remove the crate; keep one.
- **3c — Retire `0_deps`.** Replace the everything-re-export with
  `workspace.dependencies` (already partly in use). Each crate declares only what it
  uses, restoring per-crate feature granularity and removing the mechanism that pulls
  iced/reqwest/sqlite into every crate's graph. This touches every `Cargo.toml` but is
  mechanical.

Verification (the payoff): `cargo tree -i iced -p wallet` and `-p types` return
**nothing** — the domain is provably GUI-free, rather than asserted to be. Workspace
builds; e2e still green.

Commits: 3a, 3b, 3c (three commits).

**Status (2026-07-10): 3a + 3b done; 3c (retire `0_deps`) deferred.** The
meaningful leak is closed — the domain crates (`1_types`, `5_wallet`, the
adapters) now use **zero** iced symbols (grep-verified), and the duplicate
`app_path` crate is gone. What remains is the transitive dep edge: every crate
depends on the `0_deps` mega-crate, which re-exports `iced`, so `cargo tree -i
iced` still lists the domain until `0_deps` is dissolved. Fully retiring `0_deps`
means rewriting `use deps::*` imports in ~120 files, roughly half of them in
`iced_ui`/`mercurium` (being replaced) — wasted churn on doomed code — and the
Cargo feature-unification rules defeat the cleaner "iced behind an optional
`deps` feature" trick in a whole-workspace build. Deferred and coupled to the GUI
swap: when `iced_ui` is rewritten on the new framework it will declare its own
dependencies, at which point `0_deps` can be dissolved for every crate at once
and each will declare only what it uses. Tracked in `decision_log.md`.

---

## Phase 4 — Ports to the consumer (true hexagonal)

**Goal:** invert the remaining port traits so adapters depend inward. Addresses
improvement #1. Highest effort — schedule after the security work lands.

Today the key ports live *inside adapter crates*: `SecretsStore`
(`secrets_store/src/port.rs:14`), `LedgerConnector` (`ledger_connector/src/port.rs:13`),
`TransactionGateway` (`ledger_connector/src/transaction_gateway.rs:39`), `ProfileStore`
(`data_stores/src/ports/profile_store.rs:6`). So `5_wallet` must depend on the adapter
crates just to name its dependencies. (Note: `LedgerTransport`, `RelayTransport`,
`SigningFactor` are already defined in `5_wallet` — this makes the codebase consistent,
not newly correct.)

Changes:
- Introduce a small **`ports` crate** (or a `ports` module in `5_wallet`) holding every
  port trait + its DTOs/errors. Move `SecretsStore`, `LedgerConnector`,
  `TransactionGateway`, `ProfileStore`, `SettingsStore`, `IconProvider`, `GatewayProvider`
  there.
- Adapter crates now depend on `ports` and `impl` the traits; nothing in the domain
  depends on an adapter crate. `Env` (`5_wallet/src/env.rs:19`) already injects
  `Arc<dyn Trait>` — only the trait *locations* move, wiring is unchanged.
- Do it one port at a time (trait moves, adapter re-points, build) to keep each commit
  reviewable and the tree always green.

Verification: `cargo tree` shows adapters → `ports` → (nothing back); no
`5_wallet` → adapter edges. Full test suite + e2e green after each port move.

Commits: one per port moved (≈7), plus a final `ports` crate scaffold commit.

**Status (2026-07-10): deferred, coupled to the GUI swap.** Investigation showed
the blocker is not the trait locations alone but the composition root:
`Env::production()` (in `5_wallet/src/env.rs`) constructs the concrete adapters
(`secrets_store::production`, `RadixGatewayProvider`), so `5_wallet` depends on
the adapter crates for *construction*, not just to name the traits. The wallet
typestate also legitimately holds concrete `AppDataDb`/`IconsDb` handles (a
resource, not a swappable port). So the `cargo tree` inversion (`5_wallet` with no
adapter edges) requires relocating `Env::production()` out of `5_wallet` into the
binary — which changes every `Env::production()` caller, including `iced_ui`
(being replaced). Moving only the trait definitions without that relocation
leaves `5_wallet → adapters` intact, i.e. limited value for real refactor risk.
This is the plan's highest-risk, no-mainnet-blocker item, so it is best done with
the GUI swap, when the composition root and `Env` callers are rebuilt anyway.

Recipe when picked up: (1) add a `ports` crate holding the port traits + DTOs;
(2) adapters depend on `ports` and `impl` them; (3) move `Env::production()`
(and the concrete-adapter wiring) into `mercurium` (or a `composition` crate),
leaving `5_wallet` with only `Env::new(paths, secrets, gateways)` over trait
objects; (4) `5_wallet` then depends on `ports` + `types` only. Tracked in
`decision_log.md`.

---

## Phase 5 — Contract tests + DB schema versioning

**Goal:** prove the fakes match reality and give persisted data a migration path.
Addresses improvement #6 and the DB half of #5.

- **5a — Per-port contract suites.** For each port, one generic `#[test]` suite,
  parameterized over the implementation, run against **both** the fake and the real
  adapter (real ones behind `#[cfg(feature = "integration")]` where they need a DB/net).
  Prevents `FakeGateway` from accepting a submission the real gateway would reject —
  which silently voids what the e2e test proves today.
- **5b — DB schema version stamp.** There is currently no `PRAGMA user_version`. Stamp
  both sqlcipher DBs (app-data, icons) with a `user_version`, and add the
  `migrate(from_version)` seam invoked on open — but it starts **empty** (current
  version only; unknown/older version = error, wipe, recreate). No baseline migration
  is written, because there's no data to migrate. The seam exists so the first
  post-1.0 schema change has somewhere to go. Mirrors the Phase 2 `format_version`
  discipline.

Verification: contract suite runs green against fake + real; opening a DB stamped with
an unexpected `user_version` errors cleanly rather than silently mis-reading; e2e
login→send passes.

Commits: (5a) contract-test harness + per-port suites, (5b) DB `user_version` stamp +
empty migrate seam.

---

## Out of scope (tracked elsewhere)

- GUI framework swap (iced → Lumen) — separate plan; blocked on Lumen clipboard,
  masked input, file dialogs.
- The numbered-directory naming (`0_deps`, `01_…`, `1_types`, `5_wallet`) is
  idiosyncratic but harmless; a rename is optional cosmetic churn, not part of the grade.

## Definition of done (the "A")

1. `cargo tree -i iced` empty for all domain crates; adapters depend only inward.
2. No `unsafe` outside a single documented, tested exception (if any survives).
3. All cryptography in one crate whose API cannot express nonce reuse; `ring` imports
   nowhere else.
4. Every persisted artifact (mnemonic blob, both DBs) carries a version stamp and fails
   closed on an unknown version (migration seam present but empty — no data to migrate).
5. Each port has a contract suite binding its fake to its real adapter.
6. CI enforces `clippy -D warnings`, `cargo deny`, and a Windows job — all green.

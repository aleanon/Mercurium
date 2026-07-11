# Decision Log

A running record of notable decisions taken while executing the Mercurium MVP plan
(see [MVP_REVIEW_AND_PLAN.md](./MVP_REVIEW_AND_PLAN.md)). Newest entries at the top.

Format per entry: **Date — Decision** · *Context / why* · *Consequence*.

---

## 2026-06-17 — Stage A–C "complete" = code-complete + tested; on-device verification descoped
- **Context:** Stages A–C were implemented and unit-tested (159 tests), but three items can only be
  *verified* with resources absent from a headless/CI session: a physical Ledger (raw HID exchange),
  a biometric sensor (OS call), and a live Stokenet connection (on-ledger MFA submission). The
  surrounding logic for all three is implemented and tested behind traits.
- **Decision (user-chosen — "descope to buildable"):** Define Stage A–C completion as
  **code-complete + unit-tested**. Move the three irreducible external verifications out of A–C
  scope into a separate **on-device verification backlog** (`parity_roadmap.md` §0a, items V.1–V.4),
  each being a single trait implementation against real hardware/platform/network.
- **Consequence:** Stages A–C are complete by this definition. The deferred work is isolated behind
  `LedgerTransport`, `BiometricAuthenticator`, and the manifest/aggregation seams, so supplying each
  primitive later does not alter the tested code. GUI screens compile into the binary; visual
  verification is V.4.

## 2026-06-17 — `AppLock` lives in `types`, inside `Profile.security`
- **Context:** The app-lock PIN must be part of the serializable, backed-up `Profile`
  (`SecurityPreferences`), but `AppLock` was originally in the `5_wallet` crate, which `1_types`
  cannot depend on. `AppLock` only needs `ring::pbkdf2` + `crypto::Salt`, both already in `types`.
- **Decision:** Move `AppLock` into `1_types` (`app_lock.rs`) and add
  `SecurityPreferences.app_lock: Option<AppLock>`. Enabling app lock flips a flag and reveals a PIN
  field in Settings; setting a PIN stores the PBKDF2 hash; disabling clears it.
- **Consequence:** The PIN gate is captured by Profile backup/restore for free. Biometric unlock,
  when added, layers on this same lock via a platform API.

## 2026-06-17 — Stage A–C boundary: built everything not needing device/platform/on-ledger
- **Context:** The goal was to "complete the plan up to and including Stage C." Three capabilities
  cannot be completed or verified in a non-interactive session: Ledger HID (physical device),
  biometric unlock (platform API), and on-ledger Access Controller/MFA validation (Stokenet).
- **Decision:** Build and test every other Stage A–C capability to a wired-in state — backends with
  unit tests, plus iced UI verticals (send-review, history, settings, backup export/import,
  app-lock PIN, persona-data editing) that compile into the binary. Leave clean seams
  (`LedgerFactor`, the PIN gate, `securify_account_manifest`) for the three external-dependent items.
- **Consequence:** The residual for full parity Stage C is strictly hardware/platform/on-ledger and
  live-GUI verification; all in-process logic is implemented and tested. Documented in
  [parity_roadmap.md](./parity_roadmap.md) §0.

## 2026-06-16 — Workflow: commit per completed point + PR; docs live in `.ai_docs/`
- **Context:** CLAUDE.md asks for "a clear explanation of what has been done for every point
  completed" (e.g. 0B data stores gets its own commit). The user asked to commit/PR completed
  work and keep doing so going forward, and to keep the plan + this log under `.ai_docs/`.
- **Decision:** Group commits by plan phase/point with descriptive messages, branch off `main`,
  and open a PR. The plan and this decision log are kept in `.ai_docs/`.
- **Consequence:** History is readable per-point; the plan and rationale are versioned alongside code.

## 2026-06-16 — Parity roadmap: Profile model precedes MFA/Security Shields
- **Context:** Planning the path to official-wallet parity (see
  [parity_roadmap.md](./parity_roadmap.md)). The official wallet is built around a single
  versioned `Profile` object that is cloud-backed-up without the seed; Mercurium uses ad-hoc
  `WalletData` + SQLite rows.
- **Decision:** Sequence the roadmap as: Stage A (validate/finish existing — Radix Connect live,
  tx history, recovery scan, network switch) → Stage B (adopt a versioned **Profile** + encrypted
  backup/restore + authorized-dApp management) → Stage C (app lock → factor sources → Ledger →
  Security Shields/MFA/Access Controllers). Stage D (staking/pool units/deposit settings/asset
  detail) is independent and can slot in early; Stage E (transaction V2/pre-auth, guarantees, RNS).
- **Consequence:** Backup/restore, MFA and multi-device all build on the Profile model, so it
  lands before the security-heavy work rather than after — avoiding a later re-platforming.

## 2026-06-16 — `2_store` removed; SQLite store consolidated into `data_stores`
- **Context:** Two store implementations existed: the working `2_store` (`AppDataDb`/`IconsDb`)
  used everywhere, and an *incomplete parallel* hexagonal redesign inside `data_stores`
  (`DataStore`/`WalletDataStore`/`Sqlite`) that nothing consumed (only the dead, commented-out
  `radix_official_gateway` referenced it).
- **Decision:** Relocate the working `2_store` SQLite impl into `data_stores` (re-exported at the
  crate root) and **drop the incomplete parallel scaffolding** rather than try to finish it.
  Repoint all consumers (`5_wallet`, `iced_ui`, `ledger_connector`) from `store` to `data_stores`;
  delete `2_store`. The test-generated `mock.db` is now gitignored.
- **Consequence:** One store crate (`data_stores`), keeping the proven implementation. Full
  workspace builds (incl. binary); 44 tests pass. A future clean hexagonal `WalletDataStore` port
  over `AppDataDb` can be reintroduced incrementally if desired.

## 2026-06-16 — `handles` dissolved; `statics` → `iced_ui` bootstrap (not mercurium)
- **Context:** Executing the handles-dissolution plan. `initialize_statics` was planned for the
  mercurium composition root, but `App::new` (in `iced_ui`) is invoked by the iced runtime and is
  the actual bootstrap point; branching on init failure happens there.
- **Decision:** Move `initialize_statics` into `iced_ui::bootstrap` (the plan's "tiny bootstrap
  module" option) rather than mercurium, to dissolve `handles` with zero behavior change. Also
  found `accounts_and_resources`, `app_settings`, `create_wallet`, `create_account`,
  `database_handle`, and several `image` helpers were dead — dropped rather than carried.
- **Consequence:** The `handles` crate is fully removed (7 commits, all green, 41 tests pass).
  Moving bootstrap to a true mercurium composition root remains a possible future cleanup.

## 2026-06-16 — Dissolving `handles`: module destinations (incl. `image`)
- **Context:** Planning the removal of the `3_handles` grab-bag crate (see
  [refactor_handles_plan.md](./refactor_handles_plan.md)); `image` placement was an open question.
- **Decision:** `radix_dlt` → `ledger_connector` adapter; `credentials` → `secrets_store`;
  `store/get` → `data_stores`; `wallet/*` → `5_wallet`. **`image` → a dedicated `icon_provider`
  port + adapter** (icon *acquisition* = a distinct external system from the gateway and the DB;
  persistence stays in `data_stores::icon_data_store`; `resize`/`image_extension` become internal
  helpers). **`app_settings` → its own `settings_store`** (JSON file store, separate from SQLite).
  **`update_all_accounts` stays in the ledger adapter for now** (orchestration move deferred).
- **Consequence:** `handles` is emptied and removed; `ledger_connector`'s back-dependency on
  `handles` is eliminated. Migration order: secrets_store → ledger_connector → icon_provider →
  data_stores(+settings) → 5_wallet → composition root → delete handles.

## 2026-06-16 — Radix Connect wire format reconciled to official toolkit (not guessed)
- **Context:** CAP-21 message framing and the relay API are interop-critical and easy to get
  subtly wrong.
- **Decision:** Reconcile the `cap21` serde model and the `ConnectRelay` HTTP body against the
  official `@radixdlt/radix-dapp-toolkit` schema (`packages/dapp-toolkit/src/schemas/index.ts`)
  and relay API service, rather than inventing field names. Field names, discriminators
  (`authorizedRequest`/`transaction`/`loginWithChallenge`), nesting
  (`transaction.send.{transactionManifest, version, blobs?, message?}`), metadata
  (`{version:2, networkId, dAppDefinitionAddress, origin}`) and the `{method, sessionId, data}`
  relay body all match the published schema.
- **Consequence:** The implementation is spec-accurate; the only residual is end-to-end runtime
  validation against a live relay + real dApp (a QA step needing external infrastructure).

## 2026-06-16 — Signing-session model: re-decrypt the mnemonic per signing operation
- **Context:** Open decision from the plan (§6): cache the decrypted mnemonic in `Unlocked` vs.
  re-decrypt per signature.
- **Decision:** Re-decrypt the mnemonic from the OS credential store for each signing operation
  (`handles::wallet::get_decrypted_mnemonic`) and drop it immediately; do **not** cache it in the
  unlocked wallet.
- **Consequence:** Smaller in-memory secret-exposure window (safer) at the cost of a key
  derivation per send. No restructuring of the `Unlocked` state was required.

## 2026-06-16 — Transaction V1 for the MVP
- **Context:** Plan §6 locked decision; Radix mainnet still accepts V1, V2 adds subintents.
- **Decision:** Use the `NotarizedTransactionV1` flow. Single-signer transfers use
  `notary_is_signatory = true` with the account key as notary, so one notary signature authorizes
  the withdrawals (no separate intent signature needed).
- **Consequence:** Simpler signing path; V2 (subintents/pre-auth) deferred as a fast-follow.

## 2026-06-16 — Radix crate versions pinned to 1.3.1 (match existing scrypto)
- **Context:** Version skew between `scrypto`, `radix-common`, and `radix-transactions` is the
  most common build failure.
- **Decision:** Add `radix-transactions` and `radix-common` at `1.3.1` to match the existing
  `scrypto 1.3.1`, re-exported from the `deps` crate.
- **Consequence:** Clean compile; all Radix types interoperate.

## 2026-06-16 — Hexagonal migration: complete the ledger ports first, consume gradually
- **Context:** Plan locked "finish the hexagonal migration first". The legacy
  `radix_official_gateway` read adapter was half-written and non-compiling.
- **Decision:** Implement clean, compiling `TransactionGateway` (write) and `LedgerReader` (read)
  ports + a `RadixGateway` adapter + a `radix_transaction_gateway` composition root, and route the
  wallet's transaction service and the setup/refresh flow through them. Leave the legacy
  non-compiling adapter commented out rather than resurrecting it.
- **Consequence:** Both read and write paths flow through the ports; further consumption is a
  gradual per-path swap without behavior change.

## 2026-06-16 — Removed dangling `gpui` workspace member
- **Context:** `cargo build --workspace` failed to load the workspace because the `gpui` member
  had no `Cargo.toml` on disk and was never tracked in git.
- **Decision:** Remove `gpui` from the workspace members in the root `Cargo.toml`.
- **Consequence:** The workspace builds again; this was the sole blocker, not pervasive breakage.

## 2026-07-10 — Removed `ed25519-dalek-fiat`; ed25519 via `radix_common`
- **Context:** Hardening plan Phase 2e flagged the unusual `ed25519-dalek-fiat` fork on the
  wallet's key path. Inspection showed it was used only to derive the public key from the
  derived secret bytes; signing already went through `radix_common::Ed25519PrivateKey`.
- **Decision:** Drop the fork. Store the derived secret as `Zeroizing<[u8; 32]>` and derive the
  public key via `Ed25519PrivateKey::from_bytes(..).public_key()`. No third-party ed25519
  implementation remains on the signing path.
- **Consequence:** One fewer unmaintained crypto dependency; the pinned mainnet/stokenet
  account+identity address vectors are byte-for-byte unchanged, proving derivation is identical.

## 2026-07-10 — Defer physical `crypto` crate extraction (hardening plan 2a)
- **Context:** Plan Phase 2 aimed to move all cryptography into a standalone crate so `ring` is
  importable nowhere else. The security goal (nonce reuse inexpressible) was delivered in place
  via `crypto::sealed`.
- **Decision:** Defer the physical crate move. A clean split hits a `types` ↔ crypto dependency
  cycle (ed25519 derivation uses `types::Network`; `types::account`/`persona` use crypto), so it
  would also have to relocate `Network` and would ripple through `iced_ui` (being replaced).
- **Consequence:** `ring::aead` is confined to three reviewed fresh-nonce-per-seal sites and the
  misuse-prone `NonceSequence` pattern is gone, but "ring nowhere else" is not yet compiler-
  enforced. Revisit after Phase 3 (per-crate deps) or the GUI swap.

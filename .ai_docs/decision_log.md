# Decision Log

A running record of notable decisions taken while executing the Mercurium MVP plan
(see [MVP_REVIEW_AND_PLAN.md](./MVP_REVIEW_AND_PLAN.md)). Newest entries at the top.

Format per entry: **Date — Decision** · *Context / why* · *Consequence*.

---

## 2026-06-16 — Workflow: commit per completed point + PR; docs live in `.ai_docs/`
- **Context:** CLAUDE.md asks for "a clear explanation of what has been done for every point
  completed" (e.g. 0B data stores gets its own commit). The user asked to commit/PR completed
  work and keep doing so going forward, and to keep the plan + this log under `.ai_docs/`.
- **Decision:** Group commits by plan phase/point with descriptive messages, branch off `main`,
  and open a PR. The plan and this decision log are kept in `.ai_docs/`.
- **Consequence:** History is readable per-point; the plan and rationale are versioned alongside code.

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

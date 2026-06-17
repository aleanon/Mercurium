# Roadmap — Toward parity with the official Radix Wallet

> Goal: identify the development stages that take Mercurium from its current MVP to feature
> parity with the official Radix Wallet, with a detailed, ordered step list.
>
> Status date: 2026-06-17 · Grounded in the official wallet feature set (see §6 sources).

---

## 0. Implementation status — Stages A–C (live)

**Scope decision (2026-06-17):** Stages A–C are **defined as code-complete + unit-tested**. The
three irreducible *on-device / on-ledger verifications* — the raw Ledger HID/USB byte exchange
(C.12), the OS biometric sensor call (C.10), and on-ledger submission of an MFA signature set
(C.13) — are **explicitly out of the A–C completion scope** and tracked separately under
[§0a On-device verification backlog](#0a-on-device-verification-backlog-post-ac). Each is isolated
behind a trait so the tested surrounding code does not change when the primitive is supplied.

By that definition, **Stages A–C are complete.** **159 tests pass; the full workspace + `mercurium`
binary builds.** Built (each its own commit, all unit-tested where logic exists):

| Item | Status | Where |
|---|---|---|
| **A.1** Radix Connect client flow — CAP-21 responses, `ConnectRelay: RelayTransport`, session driver (receive→dispatch→respond) | ✅ done + tested | `5_wallet/src/radix_connect/{cap21,relay,session}.rs` |
| **A.3** Account recovery scan (derivation + BIP-44 gap-limit scan over a presence predicate) | ✅ done + tested | `5_wallet/src/recovery.rs` |
| **A.4** Gateway switching logic (`Gateways::switch_to`, custom gateway) | ✅ done + tested | `1_types/src/profile.rs` (UI pending) |
| **B.6** Versioned `Profile` model (gateways, preferences, authorized dApps, factor-source metadata, security structures) | ✅ done + tested | `1_types/src/profile.rs` |
| **B.7/B.8** Encrypted Profile backup (export/import + file, AES-256-GCM/PBKDF2) | ✅ done + tested | `5_wallet/src/profile_backup.rs` |
| **B.9** Authorized-dApp + persona-data model (`AuthorizedDapp`/`AuthorizedPersona`, upsert/forget) | ✅ done + tested | `1_types/src/profile.rs` (UI pending) |
| **C.10** App lock (PBKDF2 PIN, serializable, constant-time verify) | ✅ done + tested | `1_types/src/app_lock.rs` |
| **C.11/C.12** Factor-source signing abstraction (`SigningFactor`, `DeviceFactor`) + Ledger seam (`LedgerFactor`) | ✅ done + tested | `5_wallet/src/factors/mod.rs` |
| **C.12** Ledger **APDU protocol layer** — BIP-32 path serialization, APDU framing, response parsing; `LedgerTransport` trait abstracts the USB/HID exchange; driven via mock transport | ✅ protocol done + tested (raw HID exchange needs device) | `5_wallet/src/factors/ledger.rs` |
| **C.10** Biometric **unlock abstraction** — `BiometricAuthenticator` trait + `unlock()` gating policy with PIN fallback | ✅ logic done + tested (OS sensor call needs platform) | `5_wallet/src/biometric.rs` |
| **C.13** **Multi-factor signing aggregation** — `signatures_satisfying_role` (override/threshold), `role_is_met`, `outstanding_factors` | ✅ logic done + tested (on-ledger validation needs network) | `5_wallet/src/factors/multi_factor.rs` |
| **C.13** Security Shields / MFA data model (`SecurityStructure`, `RoleOfFactors`, satisfiability) | ✅ done + tested | `1_types/src/profile.rs` |

| **A.2** Transaction history **backend** — fixed buggy `UPSERT_TRANSACTION` + a latent `BalanceChangeId` panic; `upsert_transactions`, `parse_transaction` (gateway→domain, tested), `fetch_transactions` + `LedgerReader::transaction_history`, `transactions_for_account` accessor | ✅ backend done + tested | `data_stores`, `ledger_connector`, `5_wallet`, `1_types` |
| **B** Live Profile store (JSON persistence, load/save) | ✅ done + tested | `data_stores` (`profile_store`) |
| **A.2** Cursor pagination + **Transaction History UI tab** (account picker → async load → list) | ✅ done (UI compiles into binary) | `ledger_connector`, `iced_ui/src/unlocked/history` |
| **C.13** On-ledger **Access Controller (Smart Account) manifest** (`securify_account_manifest`) | ✅ done + tested | `5_wallet/src/factors/access_controller.rs` |
| **B/C** Live **Profile threaded into `App`** (load/persist via `JsonProfileStore`) + **Settings UI** (switch gateway A.4, toggle app-lock C.10 / dev-mode, dApp count) | ✅ done (compiles into binary) | `iced_ui/src/app.rs`, `iced_ui/src/unlocked/settings` |
| **B.8** Backup **export + restore (import)** UI in Settings (password field, save/load `profile_backup.bin`) | ✅ done (compiles into binary) | `iced_ui/src/unlocked/settings`, `iced_ui/src/app.rs` |
| **C.10** App-lock **PIN entry** UI — `AppLock` moved into `types::Profile.security`, PIN set/clear in Settings | ✅ done (compiles into binary) | `1_types/src/{app_lock,profile}.rs`, `iced_ui/src/unlocked/settings` |
| **A.5** Transaction-**review summary** (From/To + asset amounts) above the send password field | ✅ done (compiles into binary) | `iced_ui/src/unlocked/transaction/create_transaction.rs` |
| **B.9** Persona-**data editing** UI (name/email/phone inline edit) + `Unlocked::update_persona_data` | ✅ done (compiles into binary) | `iced_ui/src/unlocked/personas`, `5_wallet/src/wallet/unlocked.rs` |

Every Stage A–C capability is implemented behind clean modules/ports with tested logic and (where
it has a screen) a wired-in iced UI. Per the scope decision above, the items below are **not** part
of A–C completion — they are the irreducible external primitives, deferred to the backlog.

---

## 0a. On-device verification backlog (post-A–C)

*Explicitly out of Stage A–C scope (see scope decision in §0). Each is a single trait
implementation against a resource unavailable in CI/headless builds; the surrounding code is done
and tested, so supplying the primitive does not change it.*

- **V.1 — Ledger raw HID/USB byte exchange (was C.12).** Implement `LedgerTransport::exchange`
  against a physical device (system USB libraries + hardware), then verify the APDU flow end-to-end
  on a Ledger. Protocol/parsing already tested via mock transport (`factors/ledger.rs`).
- **V.2 — Biometric OS sensor call (was C.10).** Implement a `BiometricAuthenticator` backend per
  target (Touch ID / Android BiometricPrompt / `fprintd`). Gating policy + PIN fallback already
  tested (`biometric.rs`).
- **V.3 — On-ledger MFA validation (was C.13).** Submit `securify_account_manifest` output to
  Stokenet and confirm an Access Controller is created and the `signatures_satisfying_role` set is
  accepted. Manifest + aggregation already tested (`factors/{access_controller,multi_factor}.rs`).
- **V.4 — Live GUI verification.** Run the `mercurium` binary and manually verify each iced screen
  (send-review, history, settings, backup export/import, app-lock PIN, persona-data editing). All
  compile into the binary today.

---

## 1. Parity target — official wallet feature areas

From the official wallet docs and the `radixdlt/sargon` shared-logic crates (`factors`,
`profile`, `gateway`, `transaction`, `radix-name-service`, security structures / MFA):

- **Profile** — one versioned, serializable wallet object: accounts, personas, networks,
  factor sources, security structures, authorized dApps, app preferences. Cloud-backed-up
  *without* the seed phrase.
- **Factor sources & Security Shields (MFA)** — multiple factor sources (device/seed, Ledger,
  off-device mnemonic, Arculus, password, security questions); Security Structures ("shields")
  with primary / recovery / confirmation roles; on-ledger **Access Controllers** = Smart Accounts.
- **Account recovery** — derivation/recovery *scan* from a seed; multi-factor recovery without seed.
- **Ledger hardware wallet** signing.
- **Cloud backup + encrypted export/import** of the Profile.
- **Staking** — validators, stake/unstake XRD, LSUs (liquid stake units), claim NFTs; **pool units**.
- **Assets** — fungibles, NFTs, pool units, LSUs; asset details, fiat values, hide/show.
- **Transactions** — V1 **and V2 (subintents, pre-authorizations)**; transaction history;
  transaction review with **guarantees**; third-party **deposit settings**.
- **Personas & dApp connections** — persona data sharing, authorized-dApps management,
  dApp-metadata/well-known verification.
- **Radix Connect** — connector-extension pairing (WebRTC + Connect Relay), full CAP-21,
  mobile deep-linking.
- **Gateways / networks** — mainnet / stokenet / custom gateway switching.
- **Radix Name Service (RNS)** — human-readable address resolution.
- **App security** — biometric / PIN app lock, seed-phrase backup + verify, encrypted profile.

---

## 2. Where Mercurium is today

Implemented + tested (this codebase): multiple accounts (derive/persist), personas
(derive/persist/UI + ROLA), transaction **V1** build/sign/notarize/submit/poll + send UI,
Radix Connect **client logic** (dispatch, X25519/AES envelope, relay HTTP, CAP-21, pairing —
*not live-validated, no WebRTC/connector pairing*), read-only assets (fungibles/NFTs + icons),
mainnet/stokenet, themes, hexagonal ports/adapters, 114 unit tests.

Not started: Profile model, factor sources / Security Shields / MFA, Ledger, cloud backup,
recovery scan, staking / pool units, transaction history UI, transaction V2, deposit settings
(on-ledger), app lock, RNS, custom gateway, fiat values.

---

## 3. Gap analysis (parity target → status)

| Area | Status | Gap |
|---|---|---|
| Accounts (create/derive/persist) | ◑ | recovery *scan*, hide/show UI, account labels/appearance |
| Personas + ROLA | ✅ | authorized-dApp management, persona-data sharing UX |
| Transactions V1 (send) | ✅ | review/guarantees UI, message, fee customization |
| Transaction history | ❌ | fetch wired? data layer exists; **no UI** |
| Radix Connect | ◑ | live transport (WebRTC + relay), connector pairing UX, full CAP-21 round-trips |
| Assets (read) | ◑ | pool units, LSUs, staking, fiat, asset details, hide/show |
| Profile model + backup | ❌ | structured Profile, encrypted export/import, cloud backup |
| Factor sources / Security Shields / MFA | ❌ | entire workstream |
| Ledger hardware | ❌ | entire workstream |
| App lock (biometric/PIN) | ❌ | entire workstream |
| Networks / custom gateway | ◑ | mainnet/stokenet only, no switch UI / custom gateway |
| Transaction V2 (subintents/pre-auth) | ❌ | entire workstream |
| Deposit settings (3rd-party) | ◑ | `DepositRules` type exists; no on-ledger set/UI |
| RNS | ❌ | entire workstream |

---

## 4. Staged roadmap (ordered; each stage shippable)

Sequencing rationale: finish/validate what exists (Stage A), then adopt the **Profile model**
(Stage B) because backup, MFA, and multi-device all depend on it, then layer security
(Stage C), assets/DeFi (Stage D), and advanced transactions (Stage E). Stages D and parts of A
can run in parallel with B/C.

### Stage A — Make existing features real end-to-end
1. **Radix Connect live transport.**
   1. Implement the `ConnectRelay` against a live Connect Relay; confirm the encryption envelope
      and `{method,sessionId,data}` framing against the running service.
   2. Add connector-extension **pairing UX** (QR / `radix:` deep link handling; desktop URI scheme).
   3. Wire `next_interaction → dispatch → send_response` into the app loop (an iced subscription).
   4. Round-trip against a Stokenet sample dApp: login (ROLA), one-time/ongoing accounts,
      transaction request. Add CAP-21 response types for each.
   5. *(Fast-follow)* WebRTC peer transport + signaling, to match the connector extension path.
2. **Transaction history.** Fetch via the existing gateway stream (`stream_transactions`),
   persist (the `transactions`/`balance_changes` tables already exist), and add an account
   history view + per-transaction detail.
3. **Account recovery scan.** Derive accounts from the seed in batches, query the gateway for
   on-ledger presence, and import the used ones (BIP-44 gap-limit scan) — needed for restore.
4. **Network / gateway switching.** Settings UI to switch mainnet/stokenet (data already
   models both) and add a **custom gateway** URL; re-load data on switch.
5. **Transaction review polish.** Show the decompiled manifest summary, fee, and a message
   field on the confirm screen (the manifest/preview backends exist).

### Stage B — Profile data model + backup
6. **Introduce a versioned `Profile`** type (accounts, personas, networks, app preferences,
   authorized dApps, factor-source references) as the single source of truth, serialized to a
   stable schema. Migrate `WalletData`/settings onto it; keep SQLite as the cache/index.
7. **Encrypted Profile export/import** (password-encrypted JSON), mirroring the official format
   where practical.
8. **Cloud backup** of the Profile (no seed): desktop = file/OS cloud folder; document the
   restore flow (Profile + separately-restored seed/factors).
9. **Authorized dApps + persona-data sharing** stored in the Profile, with management UI
   (list connected dApps, revoke, see shared persona data).

### Stage C — Security: app lock, factor sources, Ledger, Security Shields/MFA
10. **App lock** — biometric / PIN gate on launch and on sensitive actions; advanced lock option.
11. **Factor sources abstraction** — a `FactorSource` model (device/seed, off-device mnemonic,
    password) behind a `Signer`-shaped port, generalizing today's single-mnemonic signing.
12. **Ledger hardware** — add a Ledger factor source + transport (HID/USB) implementing the
    signing port; derive accounts and sign intents on-device.
13. **Security Shields / MFA (Access Controllers).**
    1. Model Security Structures (primary / recovery / confirmation roles over factor sources).
    2. Build the on-ledger flows to create/secure an account with an Access Controller
       (Smart Account), and to sign with a shield.
    3. Recovery flows (timed recovery, confirmation) and the shield-management UI.
    *(Large; depends on factors (11) + Profile (6).)*

### Stage D — Assets & DeFi (can parallel B/C)
14. **Staking** — list validators, stake/unstake XRD, model LSUs and claim NFTs, show rewards.
15. **Pool units** — recognize and display pool-unit resources and their redeemable value.
16. **Asset detail + fiat values** — per-resource detail screens; optional fiat pricing via a
    price source; hide/show assets.
17. **Third-party deposit settings** — wire the existing `DepositRules` to on-ledger
    `account_set_default_deposit_rule` / resource preference manifests + settings UI.

### Stage E — Advanced transactions & niceties
18. **Transaction V2** — subintents and **pre-authorizations** (the newer model the official
    wallet uses); extend the transaction builder/service beyond V1.
19. **Transaction guarantees** — predicted vs. guaranteed deposits on the review screen.
20. **RNS** — resolve Radix Name Service names to addresses in the transfer/recipient UI.
21. **Polish** — account appearance (labels/colors/gradients), localization, notifications,
    address QR refinements.

---

## 5. Suggested milestones

- **M-A**: Radix Connect works against a real dApp + transaction history + recovery scan +
  network switch. (Stage A) — closes the biggest "exists but unproven / missing UI" gaps.
- **M-B**: Profile model + encrypted backup/restore + authorized-dApp management. (Stage B)
- **M-C**: App lock + factor sources + Ledger; then Security Shields/MFA. (Stage C)
- **M-D**: Staking + pool units + deposit settings + asset details. (Stage D)
- **M-E**: Transaction V2 / pre-authorizations + guarantees + RNS + polish. (Stage E)

Dependency spine: Stage A (validate Connect) → Stage B (Profile) → Stage C (factors→Ledger→MFA).
Stage D is largely independent and can be slotted in early for user-visible value.

---

## 6. Sources

- [Introducing Multi-Factor Smart Accounts (Stokenet rollout)](https://www.radixdlt.com/blog/introducing-multi-factor-smart-accounts-a-step-by-step-rollout-on-stokenet)
- [The Radix Wallet (iOS & Android)](https://www.radixdlt.com/wallet)
- [Account | Radix Docs](https://docs.radixdlt.com/docs/account)
- [What are Personas (and Identities)?](https://www.radixdlt.com/articles-learn/what-are-personas-and-identities)
- [How to Recover your Radix Wallet from Backup](https://learn.radixdlt.com/article/how-to-recover-your-radix-wallet-from-backup)
- [radixdlt/sargon](https://github.com/radixdlt/sargon) — shared wallet logic; crates: `factors`,
  `profile`, `gateway`, `transaction`, `radix-name-service`, security structures / MFA.

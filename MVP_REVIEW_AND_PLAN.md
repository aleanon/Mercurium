# Mercurium — Repository Review & MVP Implementation Plan

> Target: a minimum-viable **desktop crypto wallet for the Radix network** that, on top of
> what already exists, can **build transaction manifests, sign/notarize them, submit them to
> the network**, manage **Radix personas (identities)**, and **connect to dApps via Radix Connect**.
>
> Status date: 2026-06-15 · Branch reviewed: `main`

---

## 1. Executive summary

Mercurium is a Rust + [Iced](https://iced.rs) desktop wallet. It already has a solid
**foundation for the read-only / key-management half of a wallet**:

- BIP-39 mnemonic generation + SLIP-10 Ed25519 key derivation on the Radix derivation path,
  with virtual-account-address derivation verified against known test vectors.
- Encrypted secret storage (OS credential store for the mnemonic + a SQLCipher-encrypted
  SQLite database for wallet data).
- A `Wallet` type-state machine (`Setup` → `Unlocked`/`Locked`).
- Gateway **read** operations (balances, NFTs, transaction history) via a `radix-gateway-sdk` fork.
- A full Iced UI for onboarding (new wallet / restore), login, account views, and a
  transaction-builder screen.

It is **missing the entire "write" half** required by the goal: manifest construction,
signing/notarization, submission + status polling, personas, and Radix Connect. The
transaction UI is a shell with no backend. The codebase is also **mid-refactor** toward a
ports-and-adapters layout, which must be stabilized first.

**Bottom line:** roughly 60–70% of the *infrastructure* exists, but **~0% of the
transaction-write, persona, and dApp-connectivity features** exist. The MVP is achievable but
the dApp-connect (Radix Connect) piece is by far the largest and riskiest workstream.

---

## 2. Architecture as it stands

### 2.1 Crate layering (numbered = build order / dependency direction)

| Crate | Role | State |
|---|---|---|
| `0_deps` | Single re-export hub for all third-party crates (`pub use scrypto, radix_gateway_sdk, …`). Everything imports `deps::*`. | Stable |
| `1_types` | Domain types: `Account`, `Network`, addresses, `crypto/*` (mnemonic, Ed25519, salts, keys), `Transaction`, assets, gateway response models. | Stable, actively edited |
| `2_store` | SQLCipher SQLite persistence: `AppDataDb` (accounts/resources/assets/tx) + `IconsDb`. | Stable |
| `3_handles` | "Handles" = side-effecting operations: gateway requests, response parsing, OS credential store, wallet create/login, image download. | Stable |
| `4_font_and_icons` | Bundled fonts + bootstrap icon set. | Stable |
| `5_wallet` | Wallet type-state machine + setup task orchestration + encryption-key bundle. | Stable |
| `iced_ui` (+ `iced_ui/widgets`) | The entire GUI (onboarding, login, accounts, the transaction-builder screen). | Stable |
| `mercurium` | Binary entry point; wires Iced app, supports hot-reload. | Stable |
| `01_ports_and_adapters/*` | **In-progress** hexagonal refactor: `app_path`, `data_stores`, `ledger_connector`, `secrets_store`. | **Incomplete** |

### 2.2 The in-flight refactor (important)

`git status` shows large deletions of `01_ports/`, `01_ports_and_adapters/src/*`, and
`1_types/src/services/*`, with new homes under `01_ports_and_adapters/{app_path,data_stores,
ledger_connector,secrets_store}`. The intent is a clean **hexagonal architecture**: domain
core + `port` traits + swappable `adapter` implementations.

- `ledger_connector` defines a `LedgerConnector` **trait that is read-only** (`get_asset_summary…`,
  `update_account…`). There is **no `submit_transaction` / `preview` / `get_epoch` method**, and
  `radix_official_gateway.rs` (the adapter) is commented out / empty.
- `data_stores` has `ports/` + `adapters/` for wallet data and icons but is partially wired.
- `secrets_store` is a near-empty `port.rs`.

> **Decision (locked):** **finish the hexagonal migration first** (Phase 0), then build all new
> MVP features against the `01_ports_and_adapters` `port`/`adapter` layer rather than bolting
> them onto `3_handles`/`5_wallet`. This costs more up front but means the transaction, persona,
> and Radix Connect code lands on the intended architecture instead of being migrated twice.

### 2.3 Key existing capabilities worth reusing

- **Key derivation** — `1_types/src/crypto/ed25519.rs::Ed25519KeyPair::new` already builds the
  exact derivation path `m/44'/1022'/<network>'/<entity>'/<keykind>'/<index>'` and exposes
  `radixdlt_public_key()` (a `scrypto::crypto::Ed25519PublicKey`). This is the foundation for
  signing — it just needs a `sign(hash)` method (the secret key is held but never used to sign yet).
- **`Bip32Entity`** already distinguishes `Account` vs `Identity` (Identity = persona) — persona
  derivation is a small extension, not new infrastructure.
- **Gateway client** — `3_handles/src/radix_dlt/gateway_requests.rs` shows the established
  pattern for talking to the gateway (`radix_gateway_sdk::Client`). Submit/preview/status follow
  the same shape.
- **Network → `NetworkDefinition`** — `Network::definition()` already yields the scrypto
  `NetworkDefinition` needed for Bech32m encoding and manifest compilation.

---

## 3. Gap analysis against the goal

| Goal capability | Exists? | Notes |
|---|---|---|
| Mnemonic + key derivation | ✅ | Verified against vectors. Needs a `sign()` method. |
| Account creation / virtual addresses | ✅ | `create_account_from_mnemonic`. |
| Encrypted secret + DB storage | ✅ | OS cred store + SQLCipher. |
| Read balances / NFTs / history | ✅ | Gateway read ops + parsing. |
| Transaction-builder **UI** | ◑ | Presentational only; no backend. |
| **Build transaction manifest** | ❌ | Needs `radix-transactions` `ManifestBuilder`. |
| **Sign + notarize transaction** | ❌ | Needs intent header, epoch, nonce, Ed25519 signing, notarization. |
| **Submit + poll status** | ❌ | Needs gateway `submit` / `status` / `preview`. |
| Fee / preview estimation | ❌ | Needs gateway `/transaction/preview` (+ optional toolkit). |
| **Personas (identities)** | ❌ | Derivation entity exists; creation/data/storage/UI do not. |
| **Radix Connect (dApp)** | ❌ | Entire workstream: relay/WebRTC, CAP-21 schema, ROLA, request handlers. |
| ROLA off-ledger auth signing | ❌ | Required by Radix Connect login/auth. |

---

## 4. What "MVP" should mean here (scope guardrails)

These constraints are **locked for the MVP** (only items marked *open* remain in §6):

1. **Stokenet first.** Do all transaction/persona/connect work on Stokenet (testnet) before
   enabling Mainnet. The code already models both networks.
2. **Transaction V1 (locked).** Use the `NotarizedTransactionV1` flow (`TransactionManifestV1`,
   `TransactionHeaderV1`, `TransactionBuilder` → `NotarizedTransactionV1`) for the MVP. Radix
   mainnet still accepts V1. Transaction V2 (subintents / pre-authorizations) is explicitly
   **out of scope for the MVP** and is a documented fast-follow.
3. **Transfer manifests only** for MVP: fungible + non-fungible withdraw/deposit between
   accounts (the existing UI already models exactly this). Generic/arbitrary manifests and
   dApp-supplied manifests come via Radix Connect.
4. **Radix Connect via the Connect Relay (HTTPS) (locked).** Use the HTTPS Connect Relay +
   QR/deep-link pairing for the MVP. Raw WebRTC + signaling server is **out of scope** and is a
   documented fast-follow.
5. **Single mnemonic / software signing only.** No Ledger hardware, no multi-factor
   (Shield/MFA) for MVP.

---

## 5. Implementation plan (phased)

Each phase lists **new/changed crates & files**, the **work**, and **exit criteria**.

### Phase 0 — Stabilize the build **and finish the hexagonal migration** (prerequisite)

**Why:** the tree is mid-refactor; nothing else is safe until `cargo build`/`cargo test` pass.
Per the locked architecture decision, the ports-and-adapters migration is completed *now* so
that every later phase targets the final layer and is never migrated twice.

**0a — Get the workspace compiling again.**
- Run `cargo build --workspace` and `cargo test --workspace`; record every error.
- Reconcile the deleted `01_ports/`, `01_ports_and_adapters/src/*`, and `1_types/src/services/*`
  modules against their new homes so the workspace compiles.

**0b — Finish the hexagonal migration (now on the critical path).**
- **`data_stores`** — complete the `wallet_data_store` + `icon_data_store` ports and their
  `sqlite` adapters; route `5_wallet`/UI persistence through the ports instead of `2_store`/
  `3_handles` directly. Migrate the read/write paths currently in `2_store` + `3_handles/store`.
- **`secrets_store`** — flesh out the `port.rs` trait (store/get/delete encrypted mnemonic + DB
  salt) and add an OS-credential-store adapter that wraps the existing
  `3_handles/src/credentials/*` logic. This becomes the home for mnemonic access at signing time.
- **`ledger_connector`** — keep the existing read methods and **extend the port** (see Phase 3)
  with `current_epoch()`, `transaction_preview(...)`, `submit_transaction(...)`,
  `transaction_status(...)`. Implement the `radix_official_gateway` adapter (currently empty)
  by moving the gateway calls from `3_handles/src/radix_dlt/gateway_requests.rs` behind it.
- **`app_path`** — already largely migrated; confirm it is the single source for paths.
- Define a small **composition root** (in `mercurium` or a new `00_app`/wiring module) that
  constructs the concrete adapters and injects them into `5_wallet`, so the wallet depends on
  `port` traits only.

**0c — Add the Radix build/sign crates.**
- Add to `0_deps` + workspace `Cargo.toml` and re-export from `deps`:
  - `radix-transactions` (manifest + transaction builders, notarization, compile/decompile).
  - `radix-engine-toolkit-common` *(optional, for manifest summary/preview analysis)*.
  - `radix-common` is already present — pin **all** radix crates to **one matching version set**
    (skew between `scrypto`, `radix-common`, and `radix-transactions` is the #1 source of build pain).

**Exit:** `cargo build --workspace` green; `5_wallet` depends only on `port` traits with the
gateway/secrets/data adapters injected at a composition root; `radix_transactions` usable
through `deps`.

---

### Phase 1 — Signing primitives

**Goal:** turn the existing keypair into something that can sign Radix intents.

- `1_types/src/crypto/ed25519.rs`
  - Add `fn sign(&self, message_hash: &Hash) -> Ed25519Signature` using `ed25519_dalek_fiat`
    (return the scrypto `Ed25519Signature` type).
  - Add `fn radix_signature_with_public_key(...) -> SignatureWithPublicKeyV1`.
- `5_wallet` — add a **signing service** that, given an `Account` (which stores its derivation
  path + network) and the decrypted mnemonic, re-derives the `Ed25519KeyPair` and signs.
  - Decrypt the mnemonic at signing time **through the `secrets_store` port** (completed in
    Phase 0b) rather than calling `3_handles/credentials` directly. Decide whether `Unlocked`
    caches the decrypted mnemonic in a `Zeroizing` buffer (faster, larger attack surface) or
    re-decrypts per signature (safer). Recommend: cache in `Unlocked` behind a `zeroize` guard
    for the session.
- Unit-test signatures against a known vector.

**Exit:** can produce a valid Ed25519 signature + public key for any account from the unlocked wallet.

---

### Phase 2 — Manifest construction

**Goal:** convert the UI's `Recipient { address, resources }` model into a Radix manifest.

- New `manifest` module — home it in a new `transaction` building block consumed by the
  `ledger_connector` adapter (keep manifest construction pure/domain-side, free of gateway I/O):
  - `fn build_transfer_manifest(from: &AccountAddress, recipients: &[Recipient], network: Network)
    -> TransactionManifestV1` using `radix_transactions::manifest::ManifestBuilder`:
    - For each resource per recipient: `withdraw_from_account` → `take_from_worktop` →
      `try_deposit_or_abort` into the recipient account. Handle fungible (amount) vs
      non-fungible (ids) separately. Use the existing `Decimal` and address types.
  - Provide a **human-readable manifest string** (decompile) for a confirmation/review screen.
- Map `iced_ui` `Recipient.resources: HashMap<ResourceAddress, (symbol, amount_string)>` into
  typed amounts (parse the `String` amounts to `Decimal`; reject invalid).
- Decide guaranteed vs estimated deposits (use `try_deposit_or_abort` for MVP simplicity).

**Exit:** unit test produces a valid manifest for a fungible + an NFT transfer; manifest
decompiles to readable RTM.

---

### Phase 3 — Build, sign, notarize, submit, poll

**Goal:** the end-to-end "send" path.

- Extend the **`ledger_connector` port** (and its `radix_official_gateway` adapter, built in
  Phase 0b) with:
  - `get_current_epoch()` (gateway `/status/gateway-status` or `/transaction/construction`).
  - `transaction_preview(manifest, signers, network)` → fee summary (gateway
    `/transaction/preview`).
  - `submit_notarized_transaction(notarized_hex)` (gateway `/transaction/submit`).
  - `transaction_status(intent_hash)` (gateway `/transaction/status`) for polling.
- New `transaction_service` in `5_wallet`:
  1. Build `TransactionManifestV1` (Phase 2).
  2. Build `TransactionHeaderV1` (network id, start/end epoch window, random nonce, notary =
     the from-account's transaction-signing public key, `notary_is_signatory = true`, tip = 0).
  3. `TransactionBuilder::new().header(...).manifest(...).sign(account_key).notarize(notary_key)`
     → `NotarizedTransactionV1`; compile to bytes/hex; compute intent hash.
  4. `submit` → store a `Pending` `Transaction` locally → poll `status` until
     `CommittedSuccess`/`Failure` → update local DB + UI notification.
- The `transaction_service` consumes the `ledger_connector` **port** (not the gateway directly),
  so the same code path later serves dApp-initiated transactions (Phase 6).
- Wire the existing **"Create transaction"** button in
  `iced_ui/src/unlocked/transaction/create_transaction.rs` to: preview (show fee) → confirm →
  submit → status toast. Reuse the existing `notification` component.

**Exit:** can send XRD + an NFT between two Stokenet accounts from the GUI, see the fee, and
watch it commit. This is the first headline deliverable.

---

### Phase 4 — Personas (identities)

**Goal:** create and manage Radix personas (on-ledger identity entities + persona data).

- `1_types`: add a `Persona` type (identity address, label, persona data fields: name,
  email(s), phone, etc.) + storage rows. Reuse `Bip32Entity::Identity` for derivation
  (`create_identity_from_mnemonic`, analogous to `create_account_from_mnemonic`).
- Identity virtual address: derive via scrypto (`ComponentAddress`/identity preallocation from
  the identity public key) — mirror `bech32_address()` but for the identity entity.
- `2_store`: add `personas` table(s) + CRUD statements (mirror `accounts`).
- Persona **on-ledger creation** is an optional manifest (`create_identity_advanced` / set
  metadata) — for MVP a persona can exist as a derived virtual identity used only for ROLA
  (no on-ledger tx needed unless metadata is set). Decide per §6 open item 1.
- `iced_ui`: a personas section (list, create, edit persona data). This data is what gets
  shared with dApps during Radix Connect login.

**Exit:** can create a persona, store its data, and produce its identity key for ROLA signing.

---

### Phase 5 — ROLA (Radix Off-Ledger Authentication)

**Goal:** prove ownership of a persona/account to a dApp (prerequisite for Connect login).

- New module `rola.rs`:
  - Construct the ROLA challenge payload (`0x52` prefix + challenge(32) + dApp definition
    address length + bech32m dApp def address + origin), hash with Blake2b-256, sign with the
    identity (or account) transaction-signing key.
  - Return `SignedChallenge { public_key, signature, curve }` in the CAP-21 shape.
- Unit-test the exact byte layout against the published ROLA spec/test vectors (byte-exactness
  is critical or dApp verification fails).

**Exit:** produce a ROLA proof a dApp backend will accept.

---

### Phase 6 — Radix Connect (dApp connectivity) — the big one

**Goal:** pair with a dApp and service wallet-interaction requests.

Recommended MVP transport: **Radix Connect Relay (HTTPS)** + QR/deep-link pairing, instead of
WebRTC + signaling server.

- **Pairing/session:** new `radix_connect` crate.
  - Generate/store a Connect password / session keys; pair via a QR code shown in-app or a
    `radix://` deep link (link handling: register the URI scheme on desktop).
  - Establish the encrypted channel to the relay; long-poll/stream for incoming requests.
- **Message schema (CAP-21 wallet interactions):** model the request/response enums:
  - `AuthorizedRequest` (login with challenge → ROLA proof + persona data),
  - `OneTimeRequest` (one-time accounts/persona data),
  - `TransactionRequest` (dApp supplies a manifest to sign+submit),
  - plus `personaData` ongoing/one-time sharing.
  - Encrypt/decrypt payloads per the Radix Connect spec.
- **Request handlers** that reuse earlier phases:
  - login/auth → Phase 5 ROLA + Phase 4 persona data picker,
  - transaction request → Phase 2/3 (parse dApp manifest → preview → user approval → sign →
    submit → return intent hash),
  - account/persona sharing → user consent UI.
- **Approval UX:** a modal (reuse `iced_ui/widgets/modal`) showing dApp metadata + requested
  data + manifest summary, with allow/deny.

**Exit:** connect Mercurium to a Stokenet sample dApp (e.g. via radix-dapp-toolkit), log in
with a persona, and approve a dApp-initiated transaction.

> **Risk:** the Radix Connect protocol (encryption, relay handshake, CAP-21 message shapes) is
> intricate and under-documented outside the official TS/Swift/Kotlin implementations. Budget
> the most time here; consider porting message structs from the official `radixdlt` repos.

---

### Phase 7 — Hardening & polish

- Pending-transaction recovery on restart; transaction-history view wired to the existing DB.
- Error surfaces: replace `unwrap_unreachable`/`.ok()` swallowing in the setup/submit paths
  with user-visible errors.
- Mainnet enablement gate + a clear network switch.
- Security pass: confirm the mnemonic is never logged, zeroized everywhere, and the session
  cache is cleared on lock/logout.
- Tests: manifest builder, signing vectors, ROLA vectors, gateway submit (mocked).

---

## 6. Decisions

### Locked
- **Architecture:** finish the `01_ports_and_adapters` hexagonal migration **first** (Phase 0b);
  build all MVP features against the `port`/`adapter` layer.
- **Transaction version:** **V1** for the MVP; V2 (subintents / pre-authorizations) is a fast-follow.
- **Radix Connect transport:** **Connect Relay (HTTPS)** + QR/deep-link pairing; WebRTC + signaling
  is a fast-follow.

### Still open (need your input)
1. **Persona on-ledger creation:** MVP creates personas as derived identities used only for
   ROLA (no on-ledger tx), or fully create the identity on-ledger with metadata? *(recommend:
   derived-only for MVP.)*
2. **Signing-session model:** cache the decrypted mnemonic in the `Unlocked` state (zeroized)
   for the session, or re-decrypt per signature? *(recommend: session cache behind a zeroize guard.)*
3. **Networks:** Stokenet-only for the MVP milestone, with Mainnet behind a flag? *(recommend: yes.)*

---

## 7. Suggested milestone ordering (headline deliverables)

1. **M0** Build green + **hexagonal migration finished** (ports/adapters wired at a composition
   root) + radix-transactions available through `deps` (Phase 0). This is a larger milestone now
   that the migration is on the critical path, but it pays for itself across every later phase.
2. **M1** "Send" works on Stokenet end-to-end (Phases 1–3). ← first big visible win
3. **M2** Personas + ROLA (Phases 4–5).
4. **M3** Radix Connect (HTTPS Relay) login + dApp transaction approval (Phase 6).
5. **M4** Hardening, history, Mainnet gate (Phase 7).

The dependency chain is strict: finish the architecture (Phase 0b) → signing (1) → manifest (2)
→ submit (3) underpins both the "send" feature and every dApp transaction; ROLA (5) underpins
Connect login (6). Personas (4) can proceed in parallel with M1 once Phase 0 is done.

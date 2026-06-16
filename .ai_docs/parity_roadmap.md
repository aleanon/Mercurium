# Roadmap — Toward parity with the official Radix Wallet

> Goal: identify the development stages that take Mercurium from its current MVP to feature
> parity with the official Radix Wallet, with a detailed, ordered step list.
>
> Status date: 2026-06-16 · Grounded in the official wallet feature set (see §6 sources).

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

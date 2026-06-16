# Plan — Dissolve the `handles` crate into ports/adapters and the wallet

> Goal: remove the `3_handles` ("handles") crate by moving each of its modules to the layer it
> belongs to — gateway/ledger and secrets and store concerns into `01_ports_and_adapters`,
> wallet orchestration into `5_wallet` — leaving no general-purpose "handles" grab-bag.
>
> Status date: 2026-06-16

---

## 1. What `handles` contains today

`3_handles/src/`:

| Module | What it does | Current consumers |
|---|---|---|
| `radix_dlt/` (`gateway_requests`, `parse_responses`, `updates`) | Radix Gateway calls + response parsing + account/resource update orchestration | `ledger_connector` (already wraps it) |
| `credentials/` (`get/store/delete`, target-name consts) | OS credential store for the encrypted mnemonic + DB salt | `5_wallet` (setup/login/unlocked) |
| `store/` (`get`) | Read helpers over `AppDataDb`/`IconsDb` (`accounts_and_resources`, `resource_icons` — resizes on read) | `5_wallet::resource_data` |
| `wallet/` (`login`, `create_wallet`, `create_account`, `get_mnemonic`) | Wallet orchestration (DB+credentials) | `5_wallet`, `iced_ui` (login) |
| `image/` (`download`, `resize`, `image_extension`) | HTTP icon download + CPU resize/encode + format util | `5_wallet::task_manager`, `iced_ui` |
| `app_settings` | Read/write `AppSettings` JSON via `AppPath` | (internal / app) |
| `statics/initialize_statics` | App bootstrap: init `AppPath`, gateway client, address regexes | `iced_ui::app` |
| `database_handle` | Struct bundling `AppDataDb` + `IconsDb` | none found (dead) |

`handles` is referenced in only ~14 places workspace-wide, and `ledger_connector` currently
depends **back** on `handles` — an awkward edge this refactor removes.

---

## 2. Target mapping

| `handles` module | Destination | Notes |
|---|---|---|
| `radix_dlt/gateway_requests` + `parse_responses` | **`ledger_connector`** adapter (`RadixGateway`) | Becomes the body of the gateway adapter; finishes what `radix_official_gateway.rs` started. |
| `radix_dlt/updates` | **`ledger_connector`** adapter | `update_account(s)` move here. Note: `update_all_accounts` *reads the DB* — that's orchestration; it can move to `5_wallet` later (it already calls a `LedgerReader`-shaped flow). |
| `credentials/` | **`secrets_store`** adapter | Implement the existing `SecretsRepository` port with the OS-credential-store + file (`mnemonic.json`/`db_salt.json`) backend. |
| `store/get` | **`data_stores`** adapter | `accounts_and_resources` → a read method on the wallet-data-store adapter; the icon resize-on-read moves to the icon provider (see §3). |
| `wallet/` (`login`, `create_wallet`, `create_account`, `get_mnemonic`) | **`5_wallet`** | These orchestrate secrets + store + ledger; they become wallet functions/methods calling the ports. |
| `image/` | **new `icon_provider` port+adapter** (recommended — see §3) | |
| `app_settings` | **own `settings_store`** data store — a JSON file store (its own port+adapter, separate from the SQLite stores) | Config persistence over `AppPath`; not SQLite, so it gets its own small store. |
| `statics/initialize_statics` | **composition root** (`mercurium` binary, or a tiny `bootstrap` module) | It's app startup wiring, not a port. |
| `database_handle` | **delete** (or fold into the `data_stores` adapter) | Appears unused. |

After this, `handles` is empty and is removed from the workspace.

---

## 3. Where should `image` go? (decided: dedicated `icon_provider`)

**Decision (confirmed): a new dedicated `icon_provider` port + adapter** under
`01_ports_and_adapters/`, separate from icon *persistence*.

Reasoning:

- **Icon acquisition is its own external concern.** Icon bytes come from arbitrary third-party
  CDNs / IPFS gateways (URLs found in resource metadata) — a *different* external system from both
  the Radix gateway and the local SQLite DB. It deserves its own swappable adapter so it can be
  mocked in tests, given a caching layer, or pointed at an IPFS gateway, without touching ledger
  or store code.
- **Keep acquisition and persistence separate.** Icon *persistence* already has a home —
  `data_stores::icon_data_store` (and the SQLite `IconsDb`). The new `icon_provider` is only
  *fetch + transform*: `download` → `resize` (small/standard) → encoded bytes. The persistence
  adapter stores what the provider returns.
- **The pure transforms aren't ports.** `resize` and `image_extension` have no I/O — they become
  *internal helpers* of the `icon_provider` adapter (a private `image` submodule), not a public
  port. This also fixes today's oddity where `store/get::resource_icons` resizes on read: that
  resize moves into the provider so the data store deals only in bytes.

Proposed port (rough shape):

```rust
// icon_provider port
#[async_trait]
pub trait IconProvider {
    /// Download an icon by URL and produce small + standard encoded variants.
    async fn fetch_icon(&self, url: &str) -> Result<IconImages, IconError>; // { small: Vec<u8>, standard: Vec<u8> }
    async fn fetch_icons(&self, urls: Vec<(ResourceId, String)>) -> HashMap<ResourceId, IconImages>;
}
```

Adapter: `HttpIconProvider` (reqwest download + `fast_image_resize`). The wallet orchestrates:
ledger update → resource icon URLs → `IconProvider::fetch_icons` → `icon_data_store` persist.

**Alternatives considered (and why not):**
- *Fold into `ledger_connector`* — conflates two unrelated external systems (gateway vs. CDN); the
  gateway only yields a URL string.
- *Fold resize into `data_stores` icon adapter and download elsewhere* — splits one cohesive
  concern across two crates.
- *Plain `image_utils` crate (no port)* — fine for `resize`/`image_extension`, but loses the
  swappable/mockable boundary for the network download, which is the part worth abstracting.

If you'd rather avoid a new crate, the acceptable second choice is: `resize`/`image_extension`
as internal helpers of the `data_stores` icon adapter, and `download` as a thin function on the
`icon_data_store` adapter. I recommend the dedicated `icon_provider` for the cleaner boundary.

---

## 4. Migration order (keep the build green at each step)

Each step is its own commit (per `CLAUDE.md`); the workspace builds after each.

1. **`secrets_store`**: flesh out the port + implement the OS-credential adapter by moving
   `credentials/` in. Repoint `5_wallet` to `secrets_store`. Delete `handles::credentials`.
2. **`ledger_connector`**: move `radix_dlt/{gateway_requests,parse_responses,updates}` into the
   adapter; `RadixGateway` calls its own code instead of `handles`. Drop `ledger_connector`'s
   dependency on `handles`. Delete `handles::radix_dlt`.
3. **`icon_provider`** (new crate): move `image/` in; wire `5_wallet`/`iced_ui` to it; move the
   resize-on-read out of `store/get`.
4. **`data_stores`**: move `store/get` read helpers into the wallet-data-store adapter; add the
   `settings_store` for `app_settings`. Repoint `5_wallet`. Delete `handles::store` + `app_settings`.
5. **`5_wallet`**: move `wallet/{login,create_wallet,create_account,get_mnemonic}` in as wallet
   functions over the ports. Delete `handles::wallet`.
6. **Composition root**: move `statics::initialize_statics` into `mercurium` (or a `bootstrap`
   module); delete `database_handle`.
7. **Remove `handles`** from the workspace `Cargo.toml` and delete the crate.

Land 1–2 first: they remove the `ledger_connector → handles` back-edge and unblock the rest.

---

## 5. Risks / watch-outs

- **Circular dependencies.** Target edges: `5_wallet → {secrets_store, data_stores,
  ledger_connector, icon_provider}`; `ledger_connector → data_stores`; adapters → `types`/`deps`.
  No adapter may depend on `5_wallet`. `update_all_accounts` (reads DB inside the ledger adapter)
  is the one place tempted to reach "up" — keep DB reads behind `data_stores`, or move that
  orchestration into `5_wallet`.
- **Statics / global init.** `initialize_statics` and any `OnceCell`/`lazy_static` (gateway
  client, address regexes, `AppPath`) must be initialized at the composition root before adapters
  are used; verify nothing else relied on `handles` triggering them.
- **`update_all_accounts` placement.** It mixes a DB read with gateway calls — decide whether it's
  a ledger-adapter convenience or wallet orchestration (recommend the latter long-term).
- **`database_handle` / commented-out code** (`radix_official_gateway.rs`, dead `image` fns):
  delete rather than carry forward.
- Keep each move behind a green `cargo build --workspace` + `cargo test`.

---

## 6. Decisions (confirmed 2026-06-16)

1. **`image`** → dedicated `icon_provider` port + adapter (fetch + resize); persistence stays in
   `data_stores::icon_data_store`; `resize`/`image_extension` become internal helpers.
2. **`app_settings`** → its own `settings_store` (a JSON file store, separate port+adapter from the
   SQLite stores).
3. **`update_all_accounts`** → stays in the ledger adapter for now (revisit moving the
   orchestration to `5_wallet` later).

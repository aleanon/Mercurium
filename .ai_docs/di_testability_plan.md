# Plan — Dependency injection for headless GUI verification

> Goal: let an automated agent verify Mercurium end-to-end, up to a full **click-through of the
> send-transaction flow**, by replacing global/ambient dependencies with **injected** ones and
> driving the real `App` through `iced_test`.
>
> Decisions (locked): **dependency injection** (not global-swap); **prove the loop first** then
> remove globals; the two process globals (`AppDataDb`, `AppPath`) are to be **deleted**, not
> wrapped. Status date: 2026-06-17.

---

## 0. Background — what `iced_test` 0.14.0 actually offers

Confirmed against the pinned source (`iced_test-0.14.0`, matches `iced 0.14.0`). Two harnesses:

1. **`Simulator`** (`iced_test::simulator(element)`) — headless, **view-only**. API: `find`,
   `click`, `point_at`, `tap_key`, `typewrite`, `simulate(events)`, `snapshot(theme)`,
   `into_messages()`. Selectors are `&str` (widget by contained text) or a `Point`. It holds one
   `view()` `Element`, drains the messages produced, and **you** feed them to `update`. It **never
   runs the runtime**, so it never executes a `Task`. Use it for: view↔message wiring, reducer
   steps, and visual-regression snapshots.

2. **`Emulator` + `.ice` + `iced_test::run(program, dir)`** — runs the **real `Program`**, including
   `update`'s `Task`s. Its own docs: *"the Emulator executes the real thing! Side effects will take
   place."* `.ice` files are a DSL: a metadata header (`viewport:`, `mode: Immediate`,
   `preset: <name>`) plus `Instruction`s — `Interact` (`Mouse` move/press/release/click on a
   `Target::Text|Point`, `Keyboard` type/typewrite) and `Expect(Text)`. Use it for: the full
   send-transaction click-through.

**Injection point:** `application(...).presets([Preset::new("name", || (State, Task))])` overrides
the **boot** strategy; `.ice` selects one via `preset:`. The preset closure is where a test
assembles the `App` with test-backed dependencies — `App::new` itself stays untouched.

**Why injection is required, not optional:** the Emulator runs the real `update`, whose `Task`s and
`tokio::spawn` blocks capture `'static` owned data. Injected deps therefore cannot be borrows
threaded down; they must be **`Arc<dyn Trait>` cloned into each async block**. That dictates the
capability-bundle design below.

---

## 1. Target design — the `Env` capability bundle

A single cheaply-clonable struct holding every previously-ambient dependency:

```rust
// new: 5_wallet/src/env.rs  (extractable to its own crate later)
#[derive(Clone)]
pub struct Env {
    pub paths:    Arc<types::AppPathInner>,                 // replaces AppPath::get()
    pub secrets:  Arc<dyn secrets_store::SecretsStore + Send + Sync>,
    pub gateways: Arc<dyn ledger_connector::GatewayProvider>,  // provider, NOT one pinned gateway
    pub profiles: Arc<dyn data_stores::ProfileStore + Send + Sync>,
    pub settings: Arc<dyn data_stores::SettingsStore + Send + Sync>,
}
```

Note what is **absent**: the open database. See §1a — it is `Unlocked` state, not a boot capability.

- **Production:** `Env::production(network)` builds the real adapters (relocated from `App::new` /
  `bootstrap::initialize_statics`).
- **Test:** `Env::test()` builds `InMemorySecretsStore`, `FakeGateway`, and points `paths` at a
  `tempfile::TempDir` so the **real** SQLCipher DB + JSON stores run, just sandboxed.

**Carrier:** add `env: Env` to `WalletData` (it is already `Clone` and is cloned across every
typestate transition). Wallet methods self-serve their deps via `self.wallet_data.env` and clone
the needed `Arc`s into spawn blocks. `App` keeps its own `Env` for boot/settings paths.
`WalletData` keeps `#[derive(Clone)]`; replace `#[derive(Debug)]` with a manual
`finish_non_exhaustive` impl (trait objects aren't `Debug`).

---

## 1a. Mutability & ownership — what is a capability vs. what is state

`Env` is shared as `Arc` and cloned into `'static` task blocks, so every trait-object method it
holds **must take `&self`** — there is no `&mut` through an `Arc<dyn _>` without a lock. The
guiding split:

- **`Env` = capabilities** (how to do I/O): read-only handles. Any mutation they perform is either
  external (OS keychain, sqlite connection pool) or interior (`Mutex` inside a test fake), always
  behind `&self`. Nothing in `Env` needs `&mut`.
- **Mutable things are application state, not `Env`:** the `Profile` (lives in `App`), `Settings`
  (in `WalletData`), login attempts (in `Locked`), and the **open database connection** (in
  `Unlocked`). These already have homes; `Env` must not duplicate them.

Two consequences that correct the naïve field list:

1. **The database is `Unlocked` state, not a boot capability.** It is opened lazily at the
   `Locked → Unlocked` transition with the decryption key, which does not exist at boot. Putting it
   in an immutable boot `Env` would force `Arc<OnceCell<…>>`/`Arc<RwLock<Option<…>>>` to make
   post-login population visible to all clones — re-creating the global we are deleting. Instead,
   the opened handle rides in `Unlocked` (which already holds `key: Key<DataBase>`), owned via
   `&mut self`, single source of truth, no locks.

2. **The gateway is a provider keyed by network, not one pinned instance.** The wallet runs two
   networks and switches; `send_transfer` uses `from_account.network`. A single
   `Arc<dyn TransactionGateway>` is bound to one network and goes silently wrong after a switch.
   Inject a provider instead (new in `ledger_connector`):

   ```rust
   pub trait GatewayProvider: Send + Sync {
       fn gateway(&self, network: Network) -> Arc<dyn TransactionGateway + Send + Sync>;
   }
   ```
   Production returns `RadixGateway::new(network)`; test returns `FakeGateway`. Network-switch is
   then a pure read (`env.gateways.gateway(net)`) with **no `Env` mutation**.

**Rule for state that genuinely changes (e.g. network switch): rebuild `Env`, don't mutate it.**
`App` owns its `Env` by value, so `self.env = self.env.with_…()`. Keeping `Env` immutable-and-
replaceable avoids the divergence bug where `App`'s clone and `WalletData`'s clone drift apart.

## 2. Phase plan (each phase compiles, tests stay green, one commit each)

### Phase 1 — Gateway DI  *(smallest; proves the offline send path)*

The `TransactionGateway` port + `RadixGateway` adapter already exist
(`ledger_connector/src/transaction_gateway.rs`). Only the call sites bypass them via the
`radix_transaction_gateway(network)` free fn (`ledger_connector/src/lib.rs:17`).

Changes in `5_wallet/src/transaction/service.rs`:

```rust
// was: gateway built inside via radix_transaction_gateway(network)
pub async fn submit_transfer(
    request: &TransferRequest,
    gateway: &dyn TransactionGateway,   // + injected
    network: Network,
    signer: &Ed25519KeyPair,
    tip_percentage: u16,
) -> Result<SubmittedTransaction, TransactionServiceError>;

pub async fn submit_transfer_with_password(
    request: TransferRequest,
    from_account: Account,
    gateway: Arc<dyn TransactionGateway + Send + Sync>,  // + injected (owned for the task)
    password: Password,
    tip_percentage: u16,
) -> Result<SubmittedTransaction, AppError>;

pub async fn transaction_status(
    gateway: &dyn TransactionGateway, intent_hash_id: &str,
) -> Result<TransactionOutcome, TransactionServiceError>;

pub async fn poll_until_settled(
    gateway: &dyn TransactionGateway, intent_hash_id: &str,
    max_attempts: u32, interval: Duration,
) -> Result<TransactionOutcome, TransactionServiceError>;
```

Caller `Wallet<Unlocked>::send_transfer` (`5_wallet/src/wallet/unlocked.rs`): for this phase
construct `Arc::new(RadixGateway::new(from_account.network))` and pass it (swapped for
`self.wallet_data.env.gateways.gateway(from_account.network)` in phase 5 — see §1a, the gateway is
provided per-network, never pinned).

New test adapter — `ledger_connector`, behind a `testing` feature/module:

```rust
pub struct FakeGateway { pub epoch: u64, pub submitted: Mutex<Vec<String>>, pub status: SubmittedStatus }
#[async_trait] impl TransactionGateway for FakeGateway { /* canned epoch; record notarized_hex; canned status */ }
```

Tests: `submit_transfer` builds + submits against `FakeGateway`, asserting the recorded
`notarized_hex` is non-empty and the returned `intent_hash_id` starts with `txid_`.

### Phase 2 — SecretsStore DI

Port exists (`secrets_store::SecretsStore`); call sites use the free fns over `OsCredentialStore`.

Thread `Arc<dyn SecretsStore>` into:
- `wallet::get_decrypted_mnemonic(secrets, password)` (`5_wallet/src/wallet.rs`)
- `WalletData::create_new_account` (drop `secrets_store::get_encrypted_mnemonic()`; capture the Arc
  into the spawn) (`5_wallet/src/wallet/wallet_data.rs`)
- `login::perform_login_check(secrets, network, password)` (`5_wallet/src/wallet/login.rs`)
- `Wallet<Unlocked>::{send_transfer, create_*_persona, create_new_account}` read
  `self.wallet_data.env.secrets`.

New test adapter — `secrets_store`, `testing` feature:
`InMemorySecretsStore` seeded with a known mnemonic + salt (reuse the test mnemonic already used in
`wallet.rs`/`factors` tests). Now login + signing run headless with a deterministic seed.

### Phase 2.5 — Boot seam + first tests  *(the payoff of "prove the loop first")*

- Add `App::new_with(env: Env) -> (Self, Task<AppMessage>)`; make `App::new` call
  `App::new_with(Env::production(settings.network))`. Relocate `bootstrap::initialize_statics`'s
  `AppPathInner::new()` into `Env::production` (return the value instead of discarding it).
- **Simulator tests** for the send screen (`iced_ui/src/unlocked/transaction/create_transaction.rs`):
  assert the review summary renders and the confirm button emits the submit message.
- **First `.ice`** under `mercurium/tests/flows/send.ice` with a `Preset::new("SendFlow", ||
  App::new_with(Env::test()))` registered on the `application(...)` builder, driving:
  `click "Send" → typewrite amount → click "Confirm" → typewrite password → expect "submitted"`.

At this point the full click-through works on top of the still-global `AppDataDb`/`AppPath`,
sandboxed via `Env::test()`'s temp dir.

### Phase 3 — Delete the `AppDataDb` global

Introduce `data_stores::AppDataStore` (trait) implemented by the existing `AppDataDb` value (already
`Clone`, already `Deref<Target = DataBase>`). Per §1a the opened handle lives in the **`Unlocked`
state** (produced at the `Locked → Unlocked` transition, beside `key`), **not** in `Env`; remove the
`MAINNET_DB`/`STOKENET_DB` `OnceCell`s. Touch points: `WalletData::{save_resource_data_to_disk,
save_resource_icons_to_disk, create_new_account}`, `wallet/unlocked.rs::{transactions_for_account,
create_persona_handle, update_persona_data, create_new_persona}`, `wallet/login.rs`,
`wallet/resource_data.rs::{load_resource_data_from_disk, save_persona}`,
`service.rs::read_account_transactions`. The DB is opened once during `Env` construction (login path)
and handed around.

### Phase 4 — Delete the `AppPath` global

Replace the `Lazy<AppPathInner>` static + `AppPath::get()` with an `Arc<AppPathInner>` constructed at
boot and carried in `Env`. Widest blast radius: every `AppPath::get()` (data_stores DB/icon paths,
`app.rs` backup export/import, settings store). `AppPathInner::new()` already does all the work;
add `AppPathInner::with_root(PathBuf)` for the temp-dir test root. This kills global #2 and makes
`Env::test()`'s sandbox first-class.

---

## 3. Determinism hazards (must address for reliable `.ice` runs)

- **Detached `tokio::spawn`.** `create_persona_handle` and `update_persona_data`
  (`wallet/unlocked.rs`) spawn background DB writes **outside** iced's `Task` system, so the
  Emulator's frame loop can't await them — they race `.ice` `expect` assertions. Send-transaction
  itself goes through a proper iced `Task`, so phase 2.5 is safe; **route persona writes through
  `Task` before writing persona `.ice` tests.**
- **`FakeGateway` settling.** Give it a deterministic status sequence (e.g. `Pending` → then
  `CommittedSuccess`) so `poll_until_settled` terminates without wall-clock waits; use
  `mode: Immediate` in the `.ice` header.
- **Snapshot goldens.** First `snapshot()` run writes the golden; a human blesses it once. Keep
  goldens per-theme and pin the viewport in the `.ice` header.

---

## 4. New crates/features/files summary

- `5_wallet/src/env.rs` — `Env`, `Env::production`, `Env::test`.
- `ledger_connector` — `testing` feature → `FakeGateway`.
- `secrets_store` — `testing` feature → `InMemorySecretsStore`.
- `data_stores` — `AppDataStore` trait (phase 3); `AppPathInner::with_root` (phase 4).
- `mercurium/tests/flows/*.ice` + a `presets([...])` registration; `iced_test` as a dev-dependency.

## 5. Sequencing recap

Phase 1 → 2 → **2.5 (first Simulator + `.ice` green)** → 3 → 4. Value lands at 2.5; phases 3–4 then
remove the globals without changing the now-passing tests (they only swap what `Env::production`/
`Env::test` build).
</content>
</invoke>

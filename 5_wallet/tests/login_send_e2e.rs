//! Headless end-to-end verification of the injected loop: create a wallet DB in a temp dir, log
//! in (`Locked → Unlocked`, opening the DB and loading the account), then build/sign/submit a
//! transfer — all offline.
//!
//! This is the runtime net for the dependency-injection work: it exercises the **injected secrets
//! store** (phase 2, in-memory), the **injected gateway** (phase 1, fake), and the **DB handle
//! living in the `Unlocked` state** (phase 3) through real code paths the hermetic unit suite does
//! not cover. The on-disk sandbox uses `AppPathInner::with_root`'s sibling — the `XDG_DATA_HOME`
//! override — until the phase-4 path injection lands. Unix-only (XDG).
#![cfg(unix)]

use std::str::FromStr;
use std::sync::Arc;

use data_stores::{AppDataDb, DataBase};
use ledger_connector::testing::{FakeGateway, FakeGatewayProvider};
use secrets_store::testing::InMemorySecretsStore;
use types::address::{AccountAddress, ResourceAddress};
use types::crypto::{
    Bip32Entity, Bip32KeyKind, Ed25519KeyPair, EncryptedMnemonic, KeySaltPair, Password,
};
use types::{Account, AppPath, Network};

use wallet::transaction::{FungibleTransfer, RecipientTransfer, TransferRequest};
use wallet::wallet::Wallet;
use wallet::{Env, Locked, LoginResponse, Settings, WalletData};

const PHRASE: &str = "toward point obtain quit degree route beauty magnet hidden cereal reform increase limb measure guide skirt nominee faint shoulder win deal april error axis";
const XRD_STOKENET: &str =
    "resource_tdx_2_1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxtfd2jc";

fn mnemonic() -> deps::bip39::Mnemonic {
    deps::bip39::Mnemonic::from_phrase(PHRASE, deps::bip39::Language::English).unwrap()
}

#[test]
fn login_then_send_transfer_end_to_end() {
    let rt = deps::tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let network = Network::Stokenet;
        let password = Password::from("correct horse battery staple");

        // Sandbox every on-disk path (DB, icon cache, config) in a fresh temp dir. Set before any
        // `AppPath::get()` so the lazy global initializes against the sandbox.
        let tmp = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("XDG_DATA_HOME", tmp.path()) };
        AppPath::get().create_directories_if_not_exists().unwrap();

        // Derive the account (public API; `create_account_from_mnemonic` is crate-private).
        let (signer, path) = Ed25519KeyPair::new(
            &mnemonic(),
            None,
            0,
            network,
            Bip32Entity::Account,
            Bip32KeyKind::TransactionSigning,
        );
        let address = AccountAddress::from_str(&signer.bech32_address()).unwrap();
        let account = Account::new(
            0,
            "Main".to_string(),
            network,
            path,
            address.clone(),
            signer.radixdlt_public_key(),
        );

        // DB key/salt derived from the password; the same salt is what login reads back.
        let db_key_salt = KeySaltPair::<DataBase>::new(password.as_str()).unwrap();
        let salt = db_key_salt.salt().clone();
        let key = db_key_salt.into_key();
        let password_hash = password.derive_db_encryption_key_hash_from_salt(&salt);

        // Encrypt the mnemonic with the password (what the secrets store would hold).
        let mnemonic_key_salt = KeySaltPair::<EncryptedMnemonic>::new(password.as_str()).unwrap();
        let encrypted_mnemonic =
            EncryptedMnemonic::new_with_key_and_salt(&mnemonic(), "", mnemonic_key_salt).unwrap();

        // Create the encrypted DB in the sandbox with the password hash + the account.
        {
            let db = AppDataDb::open(network, key.clone()).await.unwrap();
            db.upsert_password_hash(password_hash).await.unwrap();
            db.upsert_account(account.clone()).await.unwrap();
        }

        // Injected environment: in-memory secrets (seeded) + a deterministic fake gateway.
        let secrets = Arc::new(InMemorySecretsStore::seeded(encrypted_mnemonic, salt));
        let provider = FakeGatewayProvider::new(FakeGateway::committed());
        let fake = provider.gateway_ref();
        let env = Env::new(secrets, Arc::new(provider));

        let mut settings = Settings::new();
        settings.set_network(network);
        let wallet = Wallet::new(Locked::new(true), WalletData::with_env(settings, env));

        // Log in — opens the DB, verifies the hash, loads the account into the Unlocked state.
        let unlocked = match wallet.login_with_password(password.clone()).await {
            LoginResponse::Success(w, _) => w,
            LoginResponse::Failed(_, err) => panic!("login should succeed, got: {err}"),
        };
        assert!(
            unlocked.accounts().contains_key(&address),
            "the persisted account should be loaded on login"
        );

        // Build, sign and submit a 1 XRD self-transfer through the injected gateway.
        let request = TransferRequest {
            from: address.clone(),
            transfers: vec![RecipientTransfer {
                to: address.clone(),
                fungibles: vec![FungibleTransfer {
                    resource: ResourceAddress::from_str(XRD_STOKENET).unwrap(),
                    amount: deps::scrypto::math::Decimal::from(1),
                }],
                non_fungibles: vec![],
            }],
        };

        let submitted = unlocked
            .send_transfer(request, password, 0)
            .await
            .expect("send_transfer should build, sign and submit via the fake gateway");

        assert!(submitted.intent_hash_id.starts_with("txid_tdx_2_"));
        assert_eq!(fake.submissions().len(), 1, "the gateway received the transaction");
        assert!(!fake.submissions()[0].is_empty(), "notarized hex was submitted");
    });
}

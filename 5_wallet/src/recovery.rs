//! Account recovery scan — re-derive accounts from the seed and find the ones already used
//! on-ledger, so a restored wallet recovers its accounts (BIP-44 gap-limit scan).

use deps::*;

use std::future::Future;

use bip39::Mnemonic;
use types::{
    Network,
    crypto::{Bip32Entity, Bip32KeyKind, Ed25519KeyPair},
};

/// Derives a contiguous batch of account addresses `[start_index, start_index + count)`.
pub fn derive_account_addresses(
    mnemonic: &Mnemonic,
    seed_password: Option<&str>,
    network: Network,
    start_index: u32,
    count: u32,
) -> Vec<(u32, String)> {
    (start_index..start_index + count)
        .map(|index| {
            let (keypair, _) = Ed25519KeyPair::new(
                mnemonic,
                seed_password,
                index,
                network,
                Bip32Entity::Account,
                Bip32KeyKind::TransactionSigning,
            );
            (index, keypair.bech32_address())
        })
        .collect()
}

/// Scans derivation indices from 0, calling `is_used` for each derived address. Returns the used
/// `(index, address)` pairs and stops after `gap_limit` consecutive unused addresses.
///
/// `is_used` is the gateway presence check (an account is "used" if it exists on-ledger); it is
/// injected so the scan logic is testable without the network.
pub async fn scan_for_used_accounts<F, Fut>(
    mnemonic: &Mnemonic,
    seed_password: Option<&str>,
    network: Network,
    gap_limit: u32,
    is_used: F,
) -> Vec<(u32, String)>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = bool>,
{
    let mut used = Vec::new();
    let mut consecutive_unused = 0u32;
    let mut index = 0u32;

    while consecutive_unused < gap_limit {
        let (keypair, _) = Ed25519KeyPair::new(
            mnemonic,
            seed_password,
            index,
            network,
            Bip32Entity::Account,
            Bip32KeyKind::TransactionSigning,
        );
        let address = keypair.bech32_address();

        if is_used(address.clone()).await {
            used.push((index, address));
            consecutive_unused = 0;
        } else {
            consecutive_unused += 1;
        }
        index += 1;
    }

    used
}

#[cfg(test)]
mod tests {
    use super::*;

    const INDEX_0: &str = "account_rdx128ykx9agh0maq8nw6h6pzmltmaexts0xf24sledqp44x5cdec0uqjj";
    const INDEX_1: &str = "account_rdx12xn8d9ykr8pmch33v0q66vhpvt98afalg0tfgctrqenkly96sgdx5n";

    fn mnemonic() -> Mnemonic {
        use bip39::Language;
        Mnemonic::from_phrase(
            "toward point obtain quit degree route beauty magnet hidden cereal reform increase limb measure guide skirt nominee faint shoulder win deal april error axis",
            Language::English,
        )
        .unwrap()
    }

    #[test]
    fn derive_matches_known_addresses() {
        let derived = derive_account_addresses(&mnemonic(), None, Network::Mainnet, 0, 2);
        assert_eq!(derived[0], (0, INDEX_0.to_string()));
        assert_eq!(derived[1], (1, INDEX_1.to_string()));
    }

    #[test]
    fn scan_stops_after_gap_limit_and_returns_used() {
        let runtime = deps::tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            // Pretend only indices 0 and 1 are used on-ledger.
            let used_set = [INDEX_0.to_string(), INDEX_1.to_string()];
            let used = scan_for_used_accounts(&mnemonic(), None, Network::Mainnet, 3, |address| {
                let hit = used_set.contains(&address);
                async move { hit }
            })
            .await;

            assert_eq!(used.len(), 2);
            assert_eq!(used[0].0, 0);
            assert_eq!(used[1].0, 1);
            assert_eq!(used[0].1, INDEX_0);
        });
    }

    #[test]
    fn scan_returns_empty_when_nothing_used() {
        let runtime = deps::tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let used =
                scan_for_used_accounts(&mnemonic(), None, Network::Mainnet, 2, |_| async { false })
                    .await;
            assert!(used.is_empty());
        });
    }
}

use ethers::{prelude::*, signers::coins_bip39::English};
use eyre::Result;
use k256::ecdsa::SigningKey;

pub fn set_up_wallet(mnemonic: &str) -> Result<Wallet<SigningKey>> {
    let wallet: Wallet<SigningKey> = MnemonicBuilder::<English>::default()
        .phrase(mnemonic)
        .build()?;
    println!("Wallet address: {}", format!("0x{:x}", wallet.address()));
    Ok(wallet)
}

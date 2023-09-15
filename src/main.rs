use config::Config;
use ethers::abi::AbiEncode;
// use contracts::avarice;
use ethers::prelude::{Provider, SignerMiddleware};
use ethers_providers::Http;
use eyre::Result;
use once_cell::sync::Lazy;
use std::env;

use ethers::prelude::*;
use k256::ecdsa::SigningKey;

mod contracts;
mod launch;

static SETTINGS: Lazy<Config> = Lazy::new(|| {
    dotenv::dotenv().ok();
    let mut settings: Config = Config::default();
    settings.merge(config::File::with_name("Settings")).unwrap();
    settings
});

#[tokio::main]
async fn main() -> Result<()> {
    println!("*** TITAN ***\n");
    let debug: bool = SETTINGS.get_bool("debug").unwrap();
    println!("Debug: {}", debug);

    let wallet_mnemonic: String = env::var("WALLET_MNEMONIC").expect("WALLET_MNEMONIC must be set");
    if debug {
        println!("MNEMONIC: {}", wallet_mnemonic);
    }

    let wallet: Wallet<SigningKey> = launch::wallet::set_up_wallet(&wallet_mnemonic).unwrap();
    let provider: Provider<Http> = launch::client::set_up_provider().await?;

    let client: SignerMiddleware<Provider<Http>, Wallet<SigningKey>> =
        launch::client::set_up_client(provider.clone(), wallet.clone()).unwrap();

    launch::helpers::get_balance(client.provider().clone(), wallet.address()).await?;

    // avarice::run_avarice().await?;

    Ok(())
}

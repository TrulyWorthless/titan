use ethers::prelude::*;
use ethers::signers::Wallet;
use ethers_providers::Http;
use eyre::Result;
use k256::ecdsa::SigningKey;
use std::{env, time::Duration};

use crate::SETTINGS;

pub async fn set_up_provider() -> Result<Provider<Http>> {
    let rpc: String = env::var("JSON_RPC_CLIENT").expect("JSON_RPC_CLIENT must be set");
    println!("JSON RPC Client URL: {}", rpc.clone());
    let provider: Provider<Http> =
        Provider::<Http>::try_from(rpc)?.interval(Duration::from_millis(
            SETTINGS
                .get_int("rpc_poll_interval")
                .unwrap()
                .try_into()
                .unwrap(),
        ));

    let chain_id: u64 = provider.get_chainid().await?.as_u64();
    println!("Started with chain_id: {chain_id}");
    Ok(provider)
}

pub fn set_up_client(
    provider: Provider<Http>,
    wallet: Wallet<SigningKey>,
) -> Result<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> {
    let middleware: SignerMiddleware<Provider<Http>, Wallet<SigningKey>> =
        SignerMiddleware::new(provider, wallet);
    Ok(middleware)
}

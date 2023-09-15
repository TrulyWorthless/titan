use ethers::providers::Http;

use ethers::prelude::{Middleware, Provider};
use ethers::types::H160;
use eyre::Result;

pub async fn get_balance(provider: Provider<Http>, coinbase_address: H160) -> Result<()> {
    let balance = provider.get_balance(coinbase_address, None).await?;
    println!("Wallet first address balance: {}", balance);

    Ok(())
}

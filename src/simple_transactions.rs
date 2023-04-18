use std::time::Duration;

use ethers::{
    prelude::{Address, LocalWallet, Middleware, Provider, Signer, TransactionRequest, U256},
    utils::Ganache,
};
use eyre::{ContextCompat, Result};
use hex::ToHex;

#[tokio::main]
async fn main() -> Result<()>{
    let mnemonic = "pear eye alley dumb online dizzy wine turtle need biology slight frame";
    let ganache = Ganache::new().mnemonic(mnemonic).spawn();
    println!("HTTP Endpoint: {}", ganache.endpoint());

    let wallet: LocalWallet = ganache.keys()[0].clone().into();
    let first_address = wallet.address();

    println!(
        "wallet first address: {}", first_address.encode_hex::<String>()
    );

    let provider = Provider::try_from(ganache.endpoint())?.interval(Duration::from_millis(10));

    let first_balance = provider.get_balance(first_address, None).await?;
    println!("wallet first address balance: {}", first_balance);

    // let dawnstar_address_hex = "0x7FbFe2FC3c77bcCAA37fbC00EBAB2fb059Ab9bc2";
    let dawnstar_address = "0x7FbFe2FC3c77bcCAA37fbC00EBAB2fb059Ab9bc2".parse::<Address>()?;
    // let dawnstar_balance = provider.get_balance(dawnstar_address, None);

    let tx = TransactionRequest::pay(dawnstar_address, U256::from(1000u64)).from(first_address);

    let receipt = provider
        .send_transaction(tx, None)
        .await?
        .log_msg("pending transfer")
        .confirmations(1)
        .await?
        .context("Missing reciept")?;

    println!("transaction mined in block {}",
        receipt.block_number.context("cannot get block number")?
    );

    println!("balance of {} {}",
        dawnstar_address, 
        provider.get_balance(dawnstar_address, None).await?
    );

    Ok(())
}
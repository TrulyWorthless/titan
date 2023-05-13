use std::time::Duration;

use ethers::contract::Contract;
use ethers::{
    prelude::{
        BlockNumber, ConfigurableArtifacts, ContractFactory, Eip1559TransactionRequest,
        LocalWallet, Middleware, Project, ProjectCompileOutput, ProjectPathsConfig, Provider,
        Signer, SignerMiddleware, U256,
    },
    utils::Ganache,
};
use eyre::{ContextCompat, Result};
use hex::ToHex;

use ethers_solc::Artifact;

use eyre::eyre;
use std::path::PathBuf;

pub type SignerDeployedContract<T> = Contract<SignerMiddleware<Provider<T>, LocalWallet>>;

#[tokio::main]
async fn main() -> Result<()> {
    /*Set up*/
    // Spawn a ganache instance
    let mnemonic = "test test test test test test test test test test test junk";
    let ganache = Ganache::new().mnemonic(mnemonic).spawn();

    // Get the first wallet managed by ganache
    let wallet: LocalWallet = ganache.keys()[0].clone().into();
    let first_address = wallet.address();
    println!(
        "Wallet first address: {}",
        first_address.encode_hex::<String>()
    );

    // A provider is an Ethereum JsonRPC client
    let provider = Provider::try_from("http://localhost:8545")?.interval(Duration::from_millis(10));
    let chain_id = provider.get_chainid().await?.as_u64();
    println!("Ganache started with chain_id {chain_id}");

    // Get the first wallet managed by ganache
    let coinbase: LocalWallet = ganache.keys()[0].clone().into();
    let coinbase_address = coinbase.address();
    // Query the balance of our account
    let first_balance = provider.get_balance(coinbase_address, None).await?;
    println!("Wallet first address balance: {}", first_balance);

    let recipient: LocalWallet = ganache.keys()[1].clone().into();
    let recipient_address = recipient.address();

    let block = provider
        .clone()
        .get_block(BlockNumber::Latest)
        .await?
        .context("Failed to get latest block")?;

    let gas_price = block
        .next_block_base_fee()
        .context("Failed to get base fee")?;

    let tx = Eip1559TransactionRequest::new()
        .to(recipient_address)
        .value(U256::from(1))
        .from(coinbase_address)
        .max_priority_fee_per_gas(gas_price)
        .max_fee_per_gas(gas_price);

    // Send the transaction and wait for receipt
    let receipt = provider
        .send_transaction(tx, None)
        .await?
        .log_msg("Pending transfer")
        .confirmations(1) // number of confirmations required
        .await?
        .context("Missing receipt")?;

    println!(
        "TX mined in block {}",
        receipt.block_number.context("Can not get block number")?
    );

    // Compile solidity project
    let project = compile("examples/").await?;

    // Print compiled project information
    print_project(project.clone()).await?;

    let balance = provider.get_balance(wallet.address(), None).await?;

    println!(
        "Wallet first address {} balance: {}",
        wallet.address().encode_hex::<String>(),
        balance
    );

    let contract_name = "ERC20";

    // Find the contract to be deployed
    let contract = project
        .find(contract_name)
        .context("Contract not found")?
        .clone();

    // We'll create a transaction which will include code for deploying the contract
    // Get ABI and contract byte, these are required for contract deployment
    let (abi, bytecode, _) = contract.into_parts();
    let abi = abi.context("Missing abi from contract")?;
    let bytecode = bytecode.context("Missing bytecode from contract")?;

    // Create signer client
    let wallet = wallet.with_chain_id(chain_id);
    let client = SignerMiddleware::new(provider.clone(), wallet).into();

    // Deploy contract
    let factory = ContractFactory::new(abi.clone(), bytecode, client);
    // Our contract don't need any constructor arguments, so we can use an empty tuple
    let name = "TrulyWorthless".to_string();
    let symbol = "TWC".to_string();
    let mut deployer = factory.deploy((name, symbol))?;
    let block = provider
        .clone()
        .get_block(BlockNumber::Latest)
        .await?
        .context("Failed to get latest block")?;

    // Set a reasonable gas price to prevent our contract from being rejected by EVM
    let gas_price = block
        .next_block_base_fee()
        .context("Failed to get base fee")?;
    deployer
        .tx
        .as_eip1559_mut()
        .unwrap()
        .max_priority_fee_per_gas = Some(gas_price);
    deployer.tx.as_eip1559_mut().unwrap().max_fee_per_gas = Some(gas_price);

    // Create transaction and send
    let contract = deployer.clone().send().await?;

    println!(
        "Token contract address {}",
        contract.address().encode_hex::<String>()
    );

    Ok(())
}

pub async fn compile(root: &str) -> Result<ProjectCompileOutput<ConfigurableArtifacts>> {
    // Create path from string and check if the path exists
    let root = PathBuf::from(root);
    if !root.exists() {
        return Err(eyre!("Project root {root:?} does not exists!"));
    }

    // Configure `root` as our project root
    let paths = ProjectPathsConfig::builder()
        .root(&root)
        .sources(&root)
        .build()?;

    // Create a solc ProjectBuilder instance for compilation
    let project = Project::builder()
        .paths(paths)
        .set_auto_detect(true) // auto detect solc version from solidity source code
        .no_artifacts()
        .build()?;

    // Compile project
    let output = project.compile()?;

    // Check for compilation errors
    if output.has_compiler_errors() {
        Err(eyre!(
            "Compiling solidity project failed: {:?}",
            output.output().errors
        ))
    } else {
        Ok(output.clone())
    }
}

pub async fn print_project(project: ProjectCompileOutput<ConfigurableArtifacts>) -> Result<()> {
    let artifacts = project.into_artifacts();
    for (id, artifact) in artifacts {
        let name = id.name;
        let abi = artifact.abi.context("No ABI found for artificat {name}")?;

        println!("{}", "=".repeat(80));
        println!("CONTRACT: {:?}", name);

        let contract = &abi.abi;
        let functions = contract.functions();
        let functions = functions.cloned();
        let constructor = contract.constructor();

        if let Some(constructor) = constructor {
            let args = &constructor.inputs;
            println!("CONSTRUCTOR args: {args:?}");
        }

        for func in functions {
            let name = &func.name;
            let params = &func.inputs;
            println!("FUNCTION  {name} {params:?}");
        }
    }
    Ok(())
}

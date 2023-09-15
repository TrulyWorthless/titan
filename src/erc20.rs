use std::time::Duration;

use ethers::contract::Contract;
use ethers::{
    prelude::{
        abigen, BlockNumber, ConfigurableArtifacts, ContractFactory, LocalWallet, Middleware,
        Project, ProjectCompileOutput, ProjectPathsConfig, Provider, Signer, SignerMiddleware,
        U256,
    },
    utils::Ganache,
};
use eyre::{ContextCompat, Result};

use ethers_solc::Artifact;

use eyre::eyre;
use std::path::PathBuf;

use std::sync::Arc;

pub type SignerDeployedContract<T> = Contract<SignerMiddleware<Provider<T>, LocalWallet>>;

#[tokio::main]
async fn main() -> Result<()> {
    let mnemonic = "test test test test test test test test test test test junk";
    let ganache = Ganache::new().mnemonic(mnemonic).spawn();

    let provider = Provider::try_from("http://localhost:8545")?.interval(Duration::from_millis(10));
    let client = Arc::new(provider.clone());

    // Get the first wallet managed by ganache
    let coinbase: LocalWallet = ganache.keys()[0].clone().into();
    println!("{coinbase:?}");

    let recipient: LocalWallet = ganache.keys()[1].clone().into();

    let contract_name = "Avarice";
    let project = compile("contracts/").await?;
    let contract = project
        .find(contract_name)
        .context("Contract not found")?
        .clone();

    let (abi, bytecode, _) = contract.into_parts();
    let abi = abi.context("Missing abi from contract")?;
    let bytecode = bytecode.context("Missing bytecode from contract")?;

    let factory = ContractFactory::new(abi.clone(), bytecode, client);
    let name = "TrulyWorthless".to_string();
    let symbol = "TWC".to_string();
    let mut deployer = factory.deploy((name, symbol))?;
    let block = provider
        .clone()
        .get_block(BlockNumber::Latest)
        .await?
        .context("Failed to get latest block")?;

    let gas_price = block
        .next_block_base_fee()
        .context("Failed to get base fee")?;
    deployer
        .tx
        .as_eip1559_mut()
        .unwrap()
        .max_priority_fee_per_gas = Some(gas_price);
    deployer.tx.as_eip1559_mut().unwrap().max_fee_per_gas = Some(gas_price);

    let contract = deployer.clone().send().await?;

    abigen!(
        Avarice,
        r#"[
            function totalSupply() external view returns (uint256)
            function balanceOf(address account) external view returns (uint256)
            function transfer(address recipient, uint256 amount) external returns (bool)
            function allowance(address owner, address spender) external view returns (uint256)
            function approve(address spender, uint256 amount) external returns (bool)
            function transferFrom( address sender, address recipient, uint256 amount) external returns (bool)
        ]"#,
    );

    let client = Arc::new(provider);
    let contract = Avarice::new(contract.address(), client.clone());

    let amount = U256::from(100);

    let transfer = contract.transfer(recipient.address(), amount);
    let send_future = transfer.send();
    let new_tx = send_future.await?;

    println!("{new_tx:?}");

    if let Ok(total_supply) = contract.total_supply().call().await {
        println!("Avarice total supply is {total_supply:?}");
    }

    if let Ok(total_supply) = contract.balance_of(recipient.address()).call().await {
        println!("Avarice total supply is {total_supply:?}");
    }

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

pub async fn deploy_token() -> Result<()> {
    Ok(())
}

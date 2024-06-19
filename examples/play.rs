//! Example on how to interact with a deployed Rock Paper Scissors contract using the Stylus SDK.
//! This example uses the Stylus SDK to instantiate the contract and interact with it.
//! It attempts to initialize the contract, commit choices for two players, and distribute the winnings.
//! The deployed contract is fully written in Rust and compiled to WASM.

use stylus_sdk::{
    alloy_primitives::U256,
    call, msg,
    prelude::*,
};
use eyre::eyre;
use std::str::FromStr;

/// Your private key environment variable name.
const PRIV_KEY_ENV: &str = "PRIV_KEY";

/// Stylus RPC endpoint URL environment variable name.
const RPC_URL_ENV: &str = "RPC_URL";

/// Deployed contract address environment variable name.
const CONTRACT_ADDRESS_ENV: &str = "CONTRACT_ADDRESS";

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let privkey =
        std::env::var(PRIV_KEY_ENV).map_err(|_| eyre!("No {} env var set", PRIV_KEY_ENV))?;
    let rpc_url =
        std::env::var(RPC_URL_ENV).map_err(|_| eyre!("No {} env var set", RPC_URL_ENV))?;
    let contract_address = std::env::var(CONTRACT_ADDRESS_ENV)
        .map_err(|_| eyre!("No {} env var set", CONTRACT_ADDRESS_ENV))?;

    println!("RPC URL: {}", rpc_url);
    println!("Contract address: {}", contract_address);

    // Create a new Stylus client
    let client = StylusClient::new(rpc_url, privkey).await?;

    // Get the contract instance
    let rps = client.contract_instance::<RPS>(contract_address.parse()?);

    println!("Connected to contract at address: {}", contract_address);

    // Initialize the contract with a smaller bet amount
    let bet_amount = U256::from(1_000_000_000_000_000u64); // 0.001 ETH
    println!("Initializing the contract with a bet amount of {} wei", bet_amount);
    let _ = rps.new(bet_amount).send().await?;
    println!("Successfully initialized the contract");

    // Player 1 commits their choice
    let player1_choice = U256::from(1); // Rock
    println!("Player 1 committing choice: {:?}", Choice::from(player1_choice));
    let _ = rps.commit(player1_choice).value(bet_amount).send().await?;
    println!("Player 1 successfully committed their choice");

    // Player 2 commits their choice
    let player2_choice = U256::from(3); // Scissors
    println!("Player 2 committing choice: {:?}", Choice::from(player2_choice));
    let _ = rps.commit(player2_choice).value(bet_amount).send().await?;
    println!("Player 2 successfully committed their choice");

    // Distribute the winnings
    println!("Distributing the winnings");
    let _ = rps.distribute().send().await?;
    println!("Successfully distributed the winnings");

    Ok(())
}
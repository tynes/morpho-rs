//! Withdraw command implementation for V1 and V2 vaults.

use alloy_primitives::{Address, U256};
use anyhow::{Context, Result};
use morpho_rs_api::{Erc4626Client, VaultV1TransactionClient, VaultV2TransactionClient};

use crate::cli::WithdrawArgs;

/// Parse a human-readable amount string to U256 given decimals.
fn parse_amount(amount: &str, decimals: u8) -> Result<U256> {
    let parts: Vec<&str> = amount.split('.').collect();

    let (integer_part, fractional_part) = match parts.len() {
        1 => (parts[0], ""),
        2 => (parts[0], parts[1]),
        _ => anyhow::bail!("Invalid amount format: {}", amount),
    };

    // Pad or truncate the fractional part to match decimals
    let fractional_padded = if fractional_part.len() > decimals as usize {
        &fractional_part[..decimals as usize]
    } else {
        fractional_part
    };

    let fractional_padded = format!("{:0<width$}", fractional_padded, width = decimals as usize);

    let combined = format!("{}{}", integer_part, fractional_padded);
    let combined = combined.trim_start_matches('0');

    if combined.is_empty() {
        return Ok(U256::ZERO);
    }

    U256::from_str_radix(combined, 10)
        .with_context(|| format!("Failed to parse amount: {}", amount))
}

/// Run the withdraw command for a V1 vault.
pub async fn run_v1_withdraw(args: &WithdrawArgs) -> Result<()> {
    let vault: Address = args.vault.parse().context("Invalid vault address")?;

    println!("Connecting to RPC...");
    let client = VaultV1TransactionClient::new(&args.rpc_url, &args.private_key)?;

    println!("Fetching vault asset...");
    let asset = client.get_asset(vault).await?;

    println!("Fetching token decimals...");
    let decimals = client.get_decimals(asset).await?;

    let amount = parse_amount(&args.amount, decimals)?;
    let signer = client.signer_address();

    println!("\nTransaction submitted: withdrawing {} from vault...", args.amount);
    println!("Waiting for confirmation...\n");

    let receipt = client.withdraw(vault, amount, signer, signer).send().await?;

    println!("Transaction confirmed!");
    println!("  Tx Hash:   {:#x}", receipt.transaction_hash);
    println!("  Block:     {}", receipt.block_number.unwrap_or_default());
    println!("  Gas Used:  {}", format_gas(receipt.gas_used));
    println!(
        "  Status:    {}",
        if receipt.status() { "Success" } else { "Failed" }
    );

    Ok(())
}

/// Run the withdraw command for a V2 vault.
pub async fn run_v2_withdraw(args: &WithdrawArgs) -> Result<()> {
    let vault: Address = args.vault.parse().context("Invalid vault address")?;

    println!("Connecting to RPC...");
    let client = VaultV2TransactionClient::new(&args.rpc_url, &args.private_key)?;

    println!("Fetching vault asset...");
    let asset = client.get_asset(vault).await?;

    println!("Fetching token decimals...");
    let decimals = client.get_decimals(asset).await?;

    let amount = parse_amount(&args.amount, decimals)?;
    let signer = client.signer_address();

    println!("\nTransaction submitted: withdrawing {} from vault...", args.amount);
    println!("Waiting for confirmation...\n");

    let receipt = client.withdraw(vault, amount, signer, signer).send().await?;

    println!("Transaction confirmed!");
    println!("  Tx Hash:   {:#x}", receipt.transaction_hash);
    println!("  Block:     {}", receipt.block_number.unwrap_or_default());
    println!("  Gas Used:  {}", format_gas(receipt.gas_used));
    println!(
        "  Status:    {}",
        if receipt.status() { "Success" } else { "Failed" }
    );

    Ok(())
}

/// Format gas with thousands separators.
fn format_gas(gas: u64) -> String {
    let s = gas.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

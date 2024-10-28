mod contracts;
mod helper;

use crate::contracts::load_contracts;
use alloy::{
    network::EthereumWallet,
    providers::{Provider as EthProvider, ProviderBuilder},
    signers::{
        k256::{ecdsa::SigningKey, SecretKey as SepoliaPrivateKey},
        local::PrivateKeySigner,
    },
};
use fuels::{
    accounts::{provider::Provider as FuelProvider, wallet::WalletUnlocked},
    crypto::SecretKey as FuelPrivateKey,
};
use helper::write_demo_run_to_file;
use std::env;
use std::str::FromStr;

// Demo cases:
// 1. Bidirectional message sending - done
// 2. Bidirectional token sending
// 3. Receive IGP payments
// 4. All ISMS working

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let sepolia_http_url =
        env::var("SEPOLIA_HTTP_RPC_URL").expect("SEPOLIA_HTTP_RPC_URL must be set");
    let fuel_provider = FuelProvider::connect("testnet.fuel.network").await.unwrap();

    let sepolia_pk = SepoliaPrivateKey::from_slice(
        &hex::decode(env::var("SEPOLIA_PRIVATE_KEY").expect("SEPOLIA_HTTP_RPC_URL must be set"))
            .unwrap(),
    )
    .unwrap();
    let sepolia_pk = SigningKey::from(sepolia_pk);
    let signer = PrivateKeySigner::from_signing_key(sepolia_pk);
    let eth_wallet = EthereumWallet::from(signer);
    let sepolia_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(eth_wallet)
        .on_builtin(&sepolia_http_url)
        .await?;

    let fuel_block_number = fuel_provider.latest_block_height().await.unwrap();
    let sepolia_block_number = sepolia_provider.get_block_number().await.unwrap();
    println!("Latest fuel block number: {}", fuel_block_number);
    println!("Latest sepolia block number: {}", sepolia_block_number);

    let secret_key = FuelPrivateKey::from_str(
        &env::var("FUEL_PRIVATE_KEY").expect("FUEL_PRIVATE_KEY must be set"),
    )
    .unwrap();
    let fuel_wallet = WalletUnlocked::new_from_private_key(secret_key, Some(fuel_provider.clone()));

    let contracts = load_contracts(fuel_wallet.clone(), sepolia_provider.clone()).await;

    ///////////////////////////////////////////////
    // Case 1: Send message from Sepolia to Fuel //
    ///////////////////////////////////////////////

    let (sent_to_fuel_msg_id, sent_to_fuel_tx) = contracts.sepolia_send_dispatch().await;
    println!("Sent to Fuel Message ID: {:?}", sent_to_fuel_msg_id);

    contracts
        .monitor_fuel_for_delivery(sent_to_fuel_msg_id)
        .await;

    ///////////////////////////////////////////////
    // Case 2: Send message from Fuel to Sepolia //
    ///////////////////////////////////////////////

    let (sent_to_sepolia_msg_id, sent_to_sepolia_tx) = contracts.fuel_send_dispatch(false).await;
    println!("Sent to Sepolia Message ID: {:?}", sent_to_sepolia_msg_id);

    let delivered_to_sepolia_tx = contracts.monitor_sepolia_for_delivery().await;

    write_demo_run_to_file(vec![
        format!(
            "Sent message id {:?} to Fuel - tx {:?}",
            sent_to_fuel_msg_id, sent_to_fuel_tx
        ),
        format!(
            "Sent message id 0x{:?} to Sepolia - tx {:?}",
            sent_to_sepolia_msg_id.as_str(),
            sent_to_sepolia_tx
        ),
        format!("Delivered to Sepolia - tx {:?}", delivered_to_sepolia_tx),
    ]);

    Ok(())

    ////////////////////////////////////////////////////////////////////////////////////
    // ⬇️ TODO move to clean case, actually check if we send/claim the right amount ⬇️ //
    ////////////////////////////////////////////////////////////////////////////////////

    // let gas_payment_quote = contracts.fuel_quote_dispatch().await;
    // let wallet_balance_before = get_native_balance(&fuel_provider, fuel_wallet.address()).await;
    // let wallet_balance_after = get_native_balance(&fuel_provider, fuel_wallet.address()).await;

    // // Wallet balance after should be more than gas_payment_quote
    // if wallet_balance_before - wallet_balance_after < gas_payment_quote {
    //     panic!("Wallet balance difference is less than gas payment quote");
    // }

    // let sepolia_ws_url = env::var("SEPOLIA_WS_RPC_URL").expect("SEPOLIA_WS_RPC_URL must be set");
    // let sepolia_provider = ProviderBuilder::new().on_builtin(&sepolia_ws_url).await?;

    // let mailbox_address = address!("c2E0b1526E677EA0a856Ec6F50E708502F7fefa9");
    // let filter = Filter::new()
    //     .address(mailbox_address)
    //     .event("ReceivedMessage(uint32,bytes32,uint256,string)")
    //     .from_block(BlockNumberOrTag::Latest);

    // let sub = sepolia_provider.subscribe_logs(&filter).await?;
    // let mut stream = sub.into_stream();

    // while let Some(log) = stream.next().await {
    //     println!("Mailbox logs: {log:?}");
    // }

    // Ok(())
}

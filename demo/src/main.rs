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
use contracts::DispatchType;
use fuels::{
    accounts::{provider::Provider as FuelProvider, wallet::WalletUnlocked},
    crypto::SecretKey as FuelPrivateKey,
};
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
        &hex::decode(env::var("SEPOLIA_PRIVATE_KEY").expect("SEPOLIA_PRIVATE_KEY must be set"))
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

    ////////////////////
    // Pre Demo Setup //
    ////////////////////

    contracts.set_sepolia_ism_to_test_ism().await;

    ///////////////////////////////////////////////
    // Case 1: Send message from Sepolia to Fuel //
    ///////////////////////////////////////////////

    let (sent_to_fuel_msg_id, sent_to_fuel_tx) = contracts
        .sepolia_send_dispatch(DispatchType::WithNoHook)
        .await;
    println!("Sent to Fuel Message ID: {:?}", sent_to_fuel_msg_id);
    println!("Transaction ID on Sepolia: {:?}", sent_to_fuel_tx);

    contracts
        .monitor_fuel_for_delivery(sent_to_fuel_msg_id)
        .await;

    ///////////////////////////////////////////////
    // Case 2: Send message from Fuel to Sepolia //
    ///////////////////////////////////////////////

    let (sent_to_sepolia_msg_id, sent_to_sepolia_tx) =
        contracts.fuel_send_dispatch(DispatchType::WithNoHook).await;
    println!("Sent to Sepolia Message ID: {:?}", sent_to_sepolia_msg_id);
    println!("Transaction ID on Fuel: {:?}", sent_to_sepolia_tx);

    let delivered_to_sepolia_tx = contracts.monitor_sepolia_for_delivery().await;
    println!(
        "Delivered to Sepolia Transaction ID: {:?}",
        delivered_to_sepolia_tx
    );

    ///////////////////////////////////////////////////////////////////////////
    // Case 3: Send message from Sepolia to Fuel, verify with different ISMs //
    ///////////////////////////////////////////////////////////////////////////

    // Aggregation ISM
    contracts.set_fuel_ism_to_aggregation().await;
    let (sent_to_fuel_msg_id, sent_to_fuel_tx) = contracts
        .sepolia_send_dispatch(DispatchType::WithNoHook)
        .await;

    println!("Sent to Fuel Message ID: {:?}", sent_to_fuel_msg_id);
    println!("Transaction ID on Sepolia: {:?}", sent_to_fuel_tx);

    contracts
        .monitor_fuel_for_delivery(sent_to_fuel_msg_id)
        .await;

    // Domain routing ISM
    contracts.set_fuel_ism_to_domain_routing().await;

    let (sent_to_fuel_msg_id, sent_to_fuel_tx) = contracts
        .sepolia_send_dispatch(DispatchType::WithNoHook)
        .await;

    println!("Sent to Fuel Message ID: {:?}", sent_to_fuel_msg_id);
    println!("Transaction ID on Sepolia: {:?}", sent_to_fuel_tx);

    contracts
        .monitor_fuel_for_delivery(sent_to_fuel_msg_id)
        .await;

    // Default Fallback Domain Routing ISM
    contracts.set_fuel_mailbox_ism_to_test_ism().await;
    contracts.set_fuel_ism_to_fallback_domain_routing().await;

    let (sent_to_fuel_msg_id, sent_to_fuel_tx) = contracts
        .sepolia_send_dispatch(DispatchType::WithNoHook)
        .await;

    println!("Sent to Fuel Message ID: {:?}", sent_to_fuel_msg_id);
    println!("Transaction ID on Sepolia: {:?}", sent_to_fuel_tx);

    contracts
        .monitor_fuel_for_delivery(sent_to_fuel_msg_id)
        .await;

    // Message ID Multisig ISM
    contracts.set_fuel_ism_to_message_id_multisig().await;

    let (sent_to_fuel_msg_id, sent_to_fuel_tx) = contracts
        .sepolia_send_dispatch(DispatchType::WithMerkleTreeHook)
        .await;

    println!("Sent to Fuel Message ID: {:?}", sent_to_fuel_msg_id);
    println!("Transaction ID on Sepolia: {:?}", sent_to_fuel_tx);

    contracts
        .monitor_fuel_for_delivery(sent_to_fuel_msg_id)
        .await;

    // Merkle Root Multisig ISM
    contracts.set_fuel_ism_to_merkle_root_multisig().await;

    let (sent_to_fuel_msg_id, sent_to_fuel_tx) = contracts
        .sepolia_send_dispatch(DispatchType::WithMerkleTreeHook)
        .await;

    println!("Sent to Fuel Message ID: {:?}", sent_to_fuel_msg_id);
    println!("Transaction ID on Sepolia: {:?}", sent_to_fuel_tx);

    contracts
        .monitor_fuel_for_delivery(sent_to_fuel_msg_id)
        .await;

    ///////////////////////////////////////////////////////////////////////////////////////////////////////
    // Case 4: Send message from Fuel to Sepolia, make sure Fuel MerkleTreeHook can get indexed properly //
    ///////////////////////////////////////////////////////////////////////////////////////////////////////

    // Validator indexes MerkleHook for Message ID Multisig ISM
    contracts.set_sepolia_ism_to_message_id_multisig().await;

    // Send 3 messages
    for _ in 0..3 {
        let (msg_id, tx_id) = contracts
            .fuel_send_dispatch(DispatchType::WithMerkleTreeHook)
            .await;

        println!("Message ID: {:?}", msg_id);
        println!("Transaction ID on Fuel: {:?}", tx_id);

        contracts.monitor_sepolia_for_delivery().await;
    }

    // Validator indexes MerkleHook for MerkleRoot Multisig ISM
    contracts.set_sepolia_ism_to_merkle_root_multisig().await;

    // Send message 3 times
    for _ in 0..3 {
        let (msg_id, tx_id) = contracts
            .fuel_send_dispatch(DispatchType::WithMerkleTreeHook)
            .await;

        println!("Message ID: {:?}", msg_id);
        println!("Transaction ID on Fuel: {:?}", tx_id);

        contracts.monitor_sepolia_for_delivery().await;
    }

    Ok(())
}

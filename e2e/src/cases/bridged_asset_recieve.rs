use crate::{
    cases::TestCase,
    setup::{
        abis::{Mailbox, MsgRecipient, WarpRoute},
        get_loaded_wallet,
    },
    utils::{
        _test_message,
        constants::{TEST_RECIPIENT, TEST_REMOTE_DOMAIN},
        local_contracts::get_contract_address_from_yaml,
        token::get_balance,
    },
};
use fuels::types::{transaction_builders::VariableOutputPolicy, Address, Bits256, Bytes};
use std::str::FromStr;
use test_utils::get_revert_reason;
use tokio::time::Instant;

async fn bridged_asset_recieve() -> Result<f64, String> {
    let start = Instant::now();

    let wallet = get_loaded_wallet().await;
    let warp_route_id = get_contract_address_from_yaml("warpRouteBridged");
    let mailbox_id = get_contract_address_from_yaml("mailbox");
    let msg_recipient = get_contract_address_from_yaml("testRecipient");

    let warp_route_instance = WarpRoute::new(warp_route_id, wallet.clone());
    let mailbox_instance = Mailbox::new(mailbox_id, wallet.clone());
    let msg_recipient_instance = MsgRecipient::new(msg_recipient, wallet.clone());

    let amount = 100_000u64;
    let recipient_address = Address::from_str(TEST_RECIPIENT).unwrap();

    //get token info
    let token_metadata = warp_route_instance
        .methods()
        .get_token_info()
        .call()
        .await
        .map_err(|e| format!("Failed to get token info: {:?}", e))?;

    let asset_id = token_metadata.value.asset_id;

    let initial_balance = get_balance(
        wallet.provider().unwrap(),
        &recipient_address.into(),
        asset_id,
    )
    .await
    .unwrap();

    let message = _test_message(
        &mailbox_instance,
        msg_recipient_instance.contract_id(),
        amount,
    );

    let _ = warp_route_instance
        .methods()
        .handle(
            TEST_REMOTE_DOMAIN,
            Bits256(Address::from(wallet.address()).into()),
            Bytes(message.clone().body),
        )
        .with_variable_output_policy(VariableOutputPolicy::Exactly(5))
        .call()
        .await
        .map_err(|e| format!("Handle message failed: {:?}", e))?;

    let final_balance = get_balance(
        wallet.provider().unwrap(),
        &recipient_address.into(),
        asset_id,
    )
    .await
    .map_err(|e| format!("Failed to get final balance: {:?}", e))?;

    //ensure recipient balance is increased by amount
    if final_balance != initial_balance + amount {
        return Err(format!(
            "Final balance mismatch. Expected: {}, Got: {}",
            initial_balance + amount,
            final_balance
        ));
    }

    //ensure circulating supply is increased by amount
    let token_metadata_final = warp_route_instance
        .methods()
        .get_token_info()
        .call()
        .await
        .map_err(|e| format!("Failed to get token metadata: {:?}", e))?;

    if token_metadata_final.value.total_supply != token_metadata.value.total_supply + amount {
        return Err(format!(
            "Circulating supply mismatch. Expected: {}, Got: {}",
            token_metadata.value.total_supply + amount,
            token_metadata_final.value.total_supply
        ));
    }

    //Attempt to handle amount greater than total supply should fail

    let new_message = _test_message(
        &mailbox_instance,
        msg_recipient_instance.contract_id(),
        100_000_000_000_000, // greater than total supply
    );

    let should_return_error = warp_route_instance
        .methods()
        .handle(
            TEST_REMOTE_DOMAIN,
            Bits256(Address::from(wallet.address()).into()),
            Bytes(new_message.clone().body),
        )
        .with_variable_output_policy(VariableOutputPolicy::Exactly(5))
        .call()
        .await;

    assert_eq!(
        get_revert_reason(should_return_error.err().unwrap()),
        "MaxMinted"
    );

    let token_metadata_final_2 = warp_route_instance
        .methods()
        .get_token_info()
        .call()
        .await
        .map_err(|e| format!("Failed to get token metadata: {:?}", e))?;

    assert!(
        token_metadata_final_2.value.total_supply - token_metadata_final.value.total_supply == 0,
        "Total Supply should stay the same since handle is reverted"
    );

    Ok(start.elapsed().as_secs_f64())
}

pub fn test() -> TestCase {
    TestCase::new("bridged_asset_recieve", bridged_asset_recieve)
}

pub mod constants;
pub mod local_contracts;
pub mod monitor;
pub mod token;

use crate::cases::FailedTestCase;
use crate::setup::abis::Mailbox;

use constants::*;
use fuels::{
    accounts::wallet::WalletUnlocked,
    types::{bech32::Bech32ContractId, Bits256, Bytes, U256},
};
use hyperlane_core::{HyperlaneMessage, H256};
use tokio::time::Instant;

pub fn summary(test_amount: usize, failed: Vec<FailedTestCase>, start: Instant) {
    println!("\nRan {} test cases", test_amount);
    println!("- Successful: {}", test_amount - failed.len());
    println!("- Failed: {}", failed.len());
    if !failed.is_empty() {
        failed.iter().for_each(|case| case.log());
    }
    println!("Total time: {:.3} sec", start.elapsed().as_secs_f64());
}

pub fn _test_message(
    mailbox: &Mailbox<WalletUnlocked>,
    recipient: &Bech32ContractId,
    amount: u64,
) -> HyperlaneMessage {
    let hash = mailbox.account().address().hash();
    let sender = hash.as_slice();

    let recipient_user = Bits256::from_hex_str(TEST_RECIPIENT).unwrap();
    let message_body = build_message_body(recipient_user, amount);

    HyperlaneMessage {
        version: 3u8,
        nonce: 0u32,
        origin: TEST_LOCAL_DOMAIN,
        sender: H256::from_slice(sender),
        destination: TEST_REMOTE_DOMAIN,
        recipient: H256::from_slice(recipient.hash().as_slice()),
        body: message_body.into(),
    }
}

fn build_message_body(recipient: Bits256, amount: u64) -> Bytes {
    let mut buffer = Vec::new();

    let amount_u256 = U256::from(amount);
    let mut amount_bytes = [0u8; 32];
    amount_u256.to_big_endian(&mut amount_bytes);

    buffer.extend(&recipient.0);
    buffer.extend(&amount_bytes);

    Bytes(buffer)
}

library;

use std::convert::Into;
use std::bytes::{Bytes,};
use std_lib_extended::bytes::*;

/// A Hyperlane message.
/// This struct is not intended to be used within smart contracts directly
/// and is included to be used off-chain using SDKs. EncodedMessage is preferred
/// for usage by smart contracts.
pub struct Message {
    version: u8,
    nonce: u32,
    origin: u32,
    sender: b256,
    destination: u32,
    recipient: b256,
    body: Bytes,
}

/// A heap-allocated tightly packed Hyperlane message.
/// Byte layout:
///   version:     [0:1]
///   nonce:       [1:5]
///   origin:      [5:9]
///   sender:      [9:41]
///   destination: [41:45]
///   recipient:   [45:77]
///   body:        [77:??]
///
/// See https://github.com/hyperlane-xyz/hyperlane-monorepo/blob/main/solidity/contracts/libs/Message.sol
/// for the reference implementation.
pub struct EncodedMessage {
    pub bytes: Bytes,
}

/// Byte offets of Message properties in an EncodedMessage.
/// VERSION_BYTE_OFFSET = 0;
const NONCE_BYTE_OFFSET: u64 = 1u64;
const ORIGIN_BYTE_OFFSET: u64 = 5u64;
const SENDER_BYTE_OFFSET: u64 = 9u64;
const DESTINATION_BYTE_OFFSET: u64 = 41u64;
const RECIPIENT_BYTE_OFFSET: u64 = 45u64;
const BODY_BYTE_OFFSET: u64 = 77u64;

impl EncodedMessage {
    /// Creates a new EncodedMessage from a Bytes object.
    ///
    /// ### Arguments
    ///
    /// * `bytes`: [Bytes] - The bytes to create the EncodedMessage from.
    ///
    /// ### Returns
    ///
    /// * [EncodedMessage] - The EncodedMessage.
    pub fn from_bytes(bytes: Bytes) -> Self {
        let (mut data, body) = bytes.split_at(BODY_BYTE_OFFSET);
        let buffer = Buffer::new();
        let buffer = body.abi_encode(buffer);
        let body_with_len = Bytes::from(buffer.as_raw_slice());
        data.append(body_with_len);
        Self { bytes: data }
    }

    /// Creates a new EncodedMessage from all of its properties.
    ///
    /// ### Arguments
    ///
    /// * `version`: [u8] - The Hyperlane message version.
    /// * `nonce`: [u32] - The message nonce.
    /// * `origin`: [u32] - The message origin domain.
    /// * `sender`: [b256] - The message sender.
    /// * `destination`: [u32] - The message destination domain.
    /// * `recipient`: [b256] - The message recipient.
    /// * `body`: [Bytes] - The message body.
    ///
    /// ### Returns
    ///
    /// * [EncodedMessage] - The EncodedMessage.
    pub fn new(
        version: u8,
        nonce: u32,
        origin: u32,
        sender: b256,
        destination: u32,
        recipient: b256,
        body: Bytes,
    ) -> Self {
        let buffer = Buffer::new();
        let buffer = version.abi_encode(buffer);
        let buffer = nonce.abi_encode(buffer);
        let buffer = origin.abi_encode(buffer);
        let buffer = sender.abi_encode(buffer);
        let buffer = destination.abi_encode(buffer);
        let buffer = recipient.abi_encode(buffer);
        let buffer = body.abi_encode(buffer);
        let bytes = Bytes::from(buffer.as_raw_slice());

        Self { bytes }
    }

    /// Returns an EncodedMessage with the body length encoding removed.
    ///
    /// ### Returns
    ///
    /// * [EncodedMessage] - The EncodedMessage with the body length encoding removed.
    pub fn message_clean(self) -> EncodedMessage {
        let body_len_size = 8;
        let bytes = self.bytes.clone();
        let (mut data, body_with_len) = bytes.split_at(BODY_BYTE_OFFSET);
        let body = body_with_len.split_at(body_len_size).1;
        data.append(body);
        EncodedMessage { bytes: data }
    }

    /// Calculates the message's ID.
    ///
    /// ### Returns
    ///
    /// * [b256] - The message ID.
    pub fn id(self) -> b256 {
        let body_len_size = 8;
        let bytes = self.bytes.clone();
        let (mut data, body_with_len) = bytes.split_at(BODY_BYTE_OFFSET);
        let body = body_with_len.split_at(body_len_size).1;
        data.append(body);
        data.keccak256()
    }

    /// Gets the message's version.
    ///
    /// ### Returns
    ///
    /// * [u8] - The message version.
    pub fn version(self) -> u8 {
        let bytes = self.bytes.clone();
        let data = bytes.split_at(NONCE_BYTE_OFFSET).0;
        BufferReader::from_parts(data.ptr(), data.len()).decode()
    }

    /// Gets the message's nonce.
    ///
    /// ### Returns
    ///
    /// * [u32] - The message nonce.
    pub fn nonce(self) -> u32 {
        let bytes = self.bytes.clone();
        bytes.read_u32(NONCE_BYTE_OFFSET)
    }

    /// Gets the message's origin domain.
    ///
    /// ### Returns
    ///
    /// * [u32] - The message origin domain.
    pub fn origin(self) -> u32 {
        let bytes = self.bytes.clone();
        let data = bytes.split_at(ORIGIN_BYTE_OFFSET).1.split_at(SENDER_BYTE_OFFSET - ORIGIN_BYTE_OFFSET).0;
        BufferReader::from_parts(data.ptr(), data.len()).decode()
    }

    /// Gets the message's sender.
    ///
    /// ### Returns
    ///
    /// * [b256] - The message sender.
    pub fn sender(self) -> b256 {
        let bytes = self.bytes.clone();
        bytes.read_b256(SENDER_BYTE_OFFSET)
    }

    /// Gets the message's destination domain.
    ///
    /// ### Returns
    ///
    /// * [u32] - The message destination domain.
    pub fn destination(self) -> u32 {
        let bytes = self.bytes.clone();
        let data = bytes.split_at(DESTINATION_BYTE_OFFSET).1.split_at(RECIPIENT_BYTE_OFFSET - DESTINATION_BYTE_OFFSET).0;
        BufferReader::from_parts(data.ptr(), data.len()).decode()
    }

    /// Gets the message's recipient.
    ///
    /// ### Returns
    ///
    /// * [b256] - The message recipient.
    pub fn recipient(self) -> b256 {
        let bytes = self.bytes.clone();
        let data = bytes.split_at(RECIPIENT_BYTE_OFFSET).1.split_at(BODY_BYTE_OFFSET - RECIPIENT_BYTE_OFFSET).0;
        BufferReader::from_parts(data.ptr(), data.len()).decode()
    }

    /// Gets the message's body.
    ///
    /// ### Returns
    ///
    /// * [Bytes] - The message body.
    pub fn body(self) -> Bytes {
        let body_len = self.bytes.len() - BODY_BYTE_OFFSET;
        if body_len > 0 {
            let bytes = self.bytes.clone();
            let data = bytes.split_at(BODY_BYTE_OFFSET).1;
            BufferReader::from_parts(data.ptr(), data.len()).decode()
        } else {
            Bytes::new()
        }
    }
}

impl From<Message> for EncodedMessage {
    fn from(message: Message) -> Self {
        Self::new(
            message
                .version,
            message
                .nonce,
            message
                .origin,
            message
                .sender,
            message
                .destination,
            message
                .recipient,
            message
                .body,
        )
    }
}

impl From<EncodedMessage> for Message {
    fn from(msg: EncodedMessage) -> Message {
        Message {
            version: msg.version(),
            nonce: msg.nonce(),
            origin: msg.origin(),
            sender: msg.sender(),
            destination: msg.destination(),
            recipient: msg.recipient(),
            body: msg.body(),
        }
    }
}

impl Into<Message> for EncodedMessage {
    fn into(self) -> Message {
        Message {
            version: self.version(),
            nonce: self.nonce(),
            origin: self.origin(),
            sender: self.sender(),
            destination: self.destination(),
            recipient: self.recipient(),
            body: self.body(),
        }
    }
}

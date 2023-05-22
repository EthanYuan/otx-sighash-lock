use crate::helper::{add_prefix, new_blake2b, verify_pubkey_hash};
use crate::types::SIGHASH_ALL_SIGNATURE_SIZE;
use crate::{error::Error, types::SighashMode};

use ckb_lib_secp256k1::LibSecp256k1;

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::{vec, vec::Vec};

// Import CKB syscalls and structures
// https://docs.rs/ckb-std/
use ckb_std::{
    ckb_constants::Source,
    ckb_types::bytes::Bytes,
    ckb_types::prelude::*,
    high_level::{load_transaction, load_witness_args},
};

pub(crate) fn validate_sighash_single_anyonecanpay(
    lib: &LibSecp256k1,
    index: usize,
    signature: &[u8; SIGHASH_ALL_SIGNATURE_SIZE],
    expected_pubkey_hash: &[u8],
) -> Result<(), Error> {
    let tx = load_transaction()?.raw();

    // input
    let input = tx.inputs().get(index).ok_or(Error::Encoding)?;
    let input_len = input.as_slice().len() as u64;

    // output
    let output = tx.outputs().get(index).ok_or(Error::Encoding)?;
    let output_len = output.as_slice().len() as u64;

    // witness
    let witness = load_witness_args(index, Source::GroupInput)?;
    let zero_lock: Bytes = {
        let buf: Vec<_> = vec![0u8; 1 + SIGHASH_ALL_SIGNATURE_SIZE];
        buf.into()
    };
    let witness_for_digest = witness
        .clone()
        .as_builder()
        .lock(Some(zero_lock).pack())
        .build();
    let witness_len = witness_for_digest.as_bytes().len() as u64;

    // hash
    let mut message = [0u8; 32];
    let mut blake2b = new_blake2b();
    blake2b.update(&input_len.to_le_bytes());
    blake2b.update(input.as_slice());
    blake2b.update(&output_len.to_le_bytes());
    blake2b.update(output.as_slice());
    blake2b.update(&witness_len.to_le_bytes());
    blake2b.update(&witness_for_digest.as_bytes());
    blake2b.finalize(&mut message);

    // add prefix
    add_prefix(SighashMode::SingleAnyoneCanPay as u8, &mut message);

    verify_pubkey_hash(lib, &message, signature, expected_pubkey_hash)
}
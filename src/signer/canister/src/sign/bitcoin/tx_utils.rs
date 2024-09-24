use crate::{
    derivation_path::Schema,
    sign::{
        bitcoin::bitcoin_utils::transform_network,
        ecdsa_api::{ecdsa_pubkey_of, get_ecdsa_signature},
        ecdsa_utils::sec1_to_der,
    },
};
use bitcoin::consensus::serialize;
use bitcoin::{
    absolute::LockTime, hashes::Hash, script::PushBytesBuf, sighash::SighashCache,
    transaction::Version, Address, AddressType, Amount, EcdsaSighashType, OutPoint, ScriptBuf,
    Sequence, Transaction, TxIn, TxOut, Txid, Witness,
};
use candid::Principal;
use ic_cdk::api::management_canister::bitcoin::{BitcoinNetwork, Utxo};
use ic_chain_fusion_signer_api::types::bitcoin::BtcTxOutput;
use std::str::FromStr;

const ECDSA_SIG_HASH_TYPE: EcdsaSighashType = EcdsaSighashType::All;

pub async fn build_p2wpkh_transaction(
    source_address: String,
    network: BitcoinNetwork,
    utxos_to_spend: &[Utxo],
    fee: u64,
    request_outputs: Vec<BtcTxOutput>,
) -> Result<Transaction, String> {
    // Assume that any amount below this threshold is dust.
    const DUST_THRESHOLD: u64 = 1_000;

    let own_address = Address::from_str(&source_address)
        .unwrap()
        .require_network(transform_network(network))
        .expect("Network check failed");

    assert_eq!(
        own_address.address_type(),
        Some(AddressType::P2wpkh),
        "Address must be of type p2wpkh."
    );

    let inputs: Vec<TxIn> = utxos_to_spend
        .iter()
        .map(|utxo| TxIn {
            previous_output: OutPoint {
                txid: Txid::from_raw_hash(Hash::from_slice(&utxo.outpoint.txid).unwrap()),
                vout: utxo.outpoint.vout,
            },
            sequence: Sequence(0xFFFF_FFFF),
            witness: Witness::new(),
            script_sig: ScriptBuf::new(),
        })
        .collect();

    let total_spent: u64 = utxos_to_spend.iter().map(|u| u.value).sum();

    let mut outputs: Vec<TxOut> = request_outputs
        .iter()
        .map(|output| TxOut {
            script_pubkey: Address::from_str(&output.destination_address)
                .unwrap()
                .require_network(transform_network(network))
                .map(|address| address.script_pubkey())
                .expect("Failed decoding destination address"),
            value: Amount::from_sat(output.sent_satoshis),
        })
        .collect();

    let sent_amount: u64 = outputs.iter().map(|u| u.value.to_sat()).sum();
    // The fee is set with leaving that amount of difference between the inputs and outputs values.
    // For example, if the inputs sum 200 and the fee is 20, then the outputs should sum 180.
    let remaining_amount = total_spent - sent_amount - fee;

    if remaining_amount >= DUST_THRESHOLD {
        outputs.push(TxOut {
            script_pubkey: own_address.script_pubkey(),
            value: Amount::from_sat(remaining_amount),
        });
    }

    Ok(Transaction {
        input: inputs,
        output: outputs,
        lock_time: LockTime::ZERO,
        version: Version::TWO,
    })
}

fn get_input_value(input: &TxIn, outputs: &[Utxo]) -> Option<Amount> {
    // The `previous_output` field in `TxIn` contains the `OutPoint`, which includes
    // the TXID and the output vout that this input is spending from.
    outputs
        .iter()
        .find(|output| output.outpoint.vout == input.previous_output.vout)
        .map(|output| Amount::from_sat(output.value))
}

pub struct SignedTransaction {
    pub signed_transaction_bytes: Vec<u8>,
    pub txid: String,
}

pub async fn btc_sign_transaction(
    principal: &Principal,
    mut transaction: Transaction,
    utxos: &[Utxo],
    source_address: String,
    key_name: String,
    network: BitcoinNetwork,
) -> Result<SignedTransaction, String> {
    let derivation_path = Schema::Btc.derivation_path(principal);
    let txclone = transaction.clone();
    let user_public_key = ecdsa_pubkey_of(key_name.clone(), derivation_path.clone()).await?;
    let own_address = Address::from_str(&source_address)
        .unwrap()
        .require_network(transform_network(network))
        .expect("Network check failed");
    for (index, input) in transaction.input.iter_mut().enumerate() {
        let value = get_input_value(input, utxos).expect("input value not found in passed utxos");
        let sighash = SighashCache::new(&txclone)
            .p2wpkh_signature_hash(
                index,
                &own_address.script_pubkey(),
                value,
                ECDSA_SIG_HASH_TYPE,
            )
            .unwrap();

        let signature = get_ecdsa_signature(
            key_name.clone(),
            derivation_path.clone(),
            sighash.as_byte_array().to_vec(),
        )
        .await?;

        // Convert signature to DER.
        let der_signature = sec1_to_der(&signature);

        let mut sig_with_hashtype: Vec<u8> = der_signature;
        sig_with_hashtype.push(u8::try_from(ECDSA_SIG_HASH_TYPE.to_u32()).expect("Error converting ECDSA_SIG_HASH_TYPE"));

        let sig_with_hashtype_push_bytes = PushBytesBuf::try_from(sig_with_hashtype).unwrap();
        let own_public_key_push_bytes = PushBytesBuf::try_from(user_public_key.clone()).unwrap();
        let mut witness = Witness::new();
        witness.push(sig_with_hashtype_push_bytes.as_bytes());
        witness.push(own_public_key_push_bytes.as_bytes());
        input.witness = witness;
    }

    let signed_transaction_bytes = serialize(&transaction);

    let txid = transaction.compute_txid().to_string();

    Ok(SignedTransaction {
        signed_transaction_bytes,
        txid,
    })
}

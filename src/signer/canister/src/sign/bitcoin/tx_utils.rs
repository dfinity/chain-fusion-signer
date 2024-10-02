use crate::{
    derivation_path::Schema,
    sign::{
        bitcoin::bitcoin_utils::transform_network,
        ecdsa_api::{ecdsa_pubkey_of, get_ecdsa_signature},
    },
};
use bitcoin::consensus::serialize;
use bitcoin::{
    absolute::LockTime, hashes::Hash, script::PushBytesBuf, sighash::SighashCache,
    transaction::Version, Address, AddressType, Amount, EcdsaSighashType, ScriptBuf,
    Sequence, Transaction, TxIn, TxOut, Txid, Witness, OutPoint as BitcoinOutPoint,
};
use candid::Principal;
use ic_cdk::api::management_canister::bitcoin::{BitcoinNetwork, Outpoint as IcCdkOutPoint, Utxo};
use ic_chain_fusion_signer_api::types::bitcoin::{BtcTxOutput, BuildP2wpkhTxError};
use std::str::FromStr;

const ECDSA_SIG_HASH_TYPE: EcdsaSighashType = EcdsaSighashType::All;

// TODO: Add testing - https://dfinity.atlassian.net/browse/GIX-3013
/// Converts a SEC1 ECDSA signature to the DER format.
/// [Reference Bitcoin Example](https://github.com/dfinity/examples/blob/aac0602139a2b3b9c509a126ee707ac9316912b0/rust/basic_bitcoin/src/basic_bitcoin/src/bitcoin_wallet/p2pkh.rs#L229)
fn sec1_to_der(sec1_signature: &[u8]) -> Vec<u8> {
    let r: Vec<u8> = if sec1_signature[0] & 0x80 != 0 {
        // r is negative. Prepend a zero byte.
        let mut tmp = vec![0x00];
        tmp.extend(sec1_signature[..32].to_vec());
        tmp
    } else {
        // r is positive.
        sec1_signature[..32].to_vec()
    };

    let s: Vec<u8> = if sec1_signature[32] & 0x80 != 0 {
        // s is negative. Prepend a zero byte.
        let mut tmp = vec![0x00];
        tmp.extend(sec1_signature[32..].to_vec());
        tmp
    } else {
        // s is positive.
        sec1_signature[32..].to_vec()
    };

    let r_len = u8::try_from(r.len()).expect("Failed to convert r length to u8");
    let s_len = u8::try_from(s.len()).expect("Failed to convert s length to u8");

    // Convert signature to DER.
    vec![
        vec![0x30, 4 + r_len + s_len, 0x02, r_len],
        r,
        vec![0x02, s_len],
        s,
    ]
    .into_iter()
    .flatten()
    .collect()
}

pub async fn build_p2wpkh_transaction(
    source_address: String,
    network: BitcoinNetwork,
    utxos_to_spend: &[Utxo],
    fee: u64,
    request_outputs: Vec<BtcTxOutput>,
) -> Result<Transaction, BuildP2wpkhTxError> {
    // Assume that any amount below this threshold is dust.
    const DUST_THRESHOLD: u64 = 1_000;

    let own_address = Address::from_str(&source_address)
        .map_err(|_| BuildP2wpkhTxError::InvalidSourceAddress {
            address: source_address.clone(),
        })?
        .require_network(transform_network(network))
        .map_err(|_| BuildP2wpkhTxError::WrongBitcoinNetwork)?;

    assert_eq!(
        own_address.address_type(),
        Some(AddressType::P2wpkh),
        "Address must be of type p2wpkh."
    );

    let inputs: Vec<TxIn> = utxos_to_spend
        .iter()
        .map(|utxo| TxIn {
            previous_output: BitcoinOutPoint {
                txid: Txid::from_raw_hash(Hash::from_slice(&utxo.outpoint.txid).unwrap()),
                vout: utxo.outpoint.vout,
            },
            sequence: Sequence(0xFFFF_FFFF),
            witness: Witness::new(),
            script_sig: ScriptBuf::new(),
        })
        .collect();

    let total_spent: u64 = utxos_to_spend.iter().map(|u| u.value).sum();

    let outputs_result: Result<Vec<TxOut>, BuildP2wpkhTxError> = request_outputs
        .iter()
        .map(|output| {
            let address = Address::from_str(&output.destination_address).map_err(|_| {
                BuildP2wpkhTxError::InvalidDestinationAddress {
                    address: output.destination_address.clone(),
                }
            })?; // Convert from ParseError to BuildP2wpkhError

            let address = address
                .require_network(transform_network(network))
                .map_err(|_| BuildP2wpkhTxError::WrongBitcoinNetwork)?; // Convert from ParseError to BuildP2wpkhError

            Ok(TxOut {
                script_pubkey: address.script_pubkey(),
                value: Amount::from_sat(output.sent_satoshis),
            })
        })
        .collect();

    match outputs_result {
        Ok(mut outputs) => {
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
        Err(e) => Err(e),
    }
}

fn is_same_outpoint(txin_outpoint: &BitcoinOutPoint, utxo_outpout: &IcCdkOutPoint) -> bool {
    txin_outpoint.vout == utxo_outpout.vout && txin_outpoint.txid == Txid::from_raw_hash(Hash::from_slice(&utxo_outpout.txid).unwrap())
}

fn get_input_value(input: &TxIn, outputs: &[Utxo]) -> Option<Amount> {
    // The `previous_output` field in `TxIn` contains the `OutPoint`, which includes
    // the TXID and the output vout that this input is spending from.
    outputs
        .iter()
        .find(|output| is_same_outpoint(&input.previous_output, &output.outpoint))
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
    network: BitcoinNetwork,
) -> Result<SignedTransaction, String> {
    let derivation_path = Schema::Btc.derivation_path(principal);
    let txclone = transaction.clone();
    let user_public_key = ecdsa_pubkey_of(derivation_path.clone()).await?;
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

        let signature =
            get_ecdsa_signature(derivation_path.clone(), sighash.as_byte_array().to_vec()).await?;

        // Convert signature to DER.
        let der_signature = sec1_to_der(&signature);

        let mut sig_with_hashtype: Vec<u8> = der_signature;
        sig_with_hashtype.push(
            u8::try_from(ECDSA_SIG_HASH_TYPE.to_u32())
                .expect("Error converting ECDSA_SIG_HASH_TYPE"),
        );

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

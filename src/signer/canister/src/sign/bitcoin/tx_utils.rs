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
    transaction::Version, Address, AddressType, Amount, EcdsaSighashType,
    OutPoint as BitcoinOutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Witness,
};
use candid::Principal;
use ic_cdk::api::management_canister::bitcoin::{BitcoinNetwork, Outpoint as IcCdkOutPoint, Utxo};
use ic_chain_fusion_signer_api::types::bitcoin::{BtcTxOutput, BuildP2wpkhTxError};
use std::str::FromStr;

const ECDSA_SIG_HASH_TYPE: EcdsaSighashType = EcdsaSighashType::All;
// Assume that any amount below this threshold is dust.
const DUST_THRESHOLD: u64 = 1_000;

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

/// The fee is set with leaving that amount of difference between the inputs and outputs values.
/// For example, if the inputs sum 200 and the fee is 20, then the outputs should sum 180.
/// This function calculates the remaining amount that should be sent to the source address.
/// The function returns successfully if `utxos_amount >= sent_amount + fee`
/// The function returns an error if `sent_amount + fee > utxos_amount`
fn calculate_remaining_amount(
    utxos_amount: u64,
    sent_amount: u64,
    fee: u64,
) -> Result<u64, BuildP2wpkhTxError> {
    if let Some(remaining_amount) = utxos_amount
        .checked_sub(sent_amount)
        .and_then(|res| res.checked_sub(fee))
    {
        Ok(remaining_amount)
    } else {
        Err(BuildP2wpkhTxError::NotEnoughFunds {
            required: sent_amount + fee,
            available: utxos_amount,
        })
    }
}

pub fn build_p2wpkh_transaction(
    source_address: &str,
    network: BitcoinNetwork,
    utxos_to_spend: &[Utxo],
    fee: u64,
    request_outputs: &[BtcTxOutput],
) -> Result<Transaction, BuildP2wpkhTxError> {
    let own_address = Address::from_str(source_address)
        .map_err(|_| BuildP2wpkhTxError::InvalidSourceAddress {
            address: source_address.to_string(),
        })?
        .require_network(transform_network(network))
        .map_err(|_| BuildP2wpkhTxError::WrongBitcoinNetwork)?;

    if own_address.address_type() != Some(AddressType::P2wpkh) {
        return Err(BuildP2wpkhTxError::NotP2WPKHSourceAddress);
    }

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

    let utxos_amount: u64 = utxos_to_spend.iter().map(|u| u.value).sum();

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
            let remaining_amount = calculate_remaining_amount(utxos_amount, sent_amount, fee)?;

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
    txin_outpoint.vout == utxo_outpout.vout
        && txin_outpoint.txid.as_byte_array()[..] == utxo_outpout.txid[..]
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitcoin::{
        hashes::Hash, OutPoint as BitcoinOutPoint, ScriptBuf, Sequence, TxIn, Txid, Witness,
    };
    use ic_cdk::api::management_canister::bitcoin::{
        BitcoinNetwork, Outpoint as IcCdkOutPoint, Utxo,
    };
    use ic_chain_fusion_signer_api::types::bitcoin::{BtcTxOutput, BuildP2wpkhTxError};

    use super::{build_p2wpkh_transaction, get_input_value, DUST_THRESHOLD};

    const TXID1: &str = "36f3a7fcb6b5ebd9fa4041928da89cd423662f9c5c12e41c80e07a6559d178ef";
    const TXID2: &str = "d3f71b58d539fd97d2122f112d52dadb6a479ad3c47464978b3b0ce0046c1b50";
    const TXID3: &str = "62791113aa4bf339e72afca37c99960e0e29240916b65e2b245a8a7b9effcdeb";
    const TXID4: &str = "bc1ac3cb81e3f261c9c7eae460f7fe6c51583db5b130b4928ffb77c602e3f48e";

    #[derive(Clone)]
    struct UtxoWrapper {
        pub utxo: Utxo,
        pub txid: Txid,
    }

    fn get_mock_utxos() -> Vec<UtxoWrapper> {
        let txid1 = Txid::from_str(TXID1).unwrap();
        let txid2 = Txid::from_str(TXID2).unwrap();
        let txid3 = Txid::from_str(TXID3).unwrap();
        let txid4 = Txid::from_str(TXID4).unwrap();
        let utxo1: Utxo = Utxo {
            outpoint: IcCdkOutPoint {
                txid: txid1.as_byte_array().to_vec(),
                vout: 0,
            },
            value: 1000,
            height: 100,
        };
        let utxo2: Utxo = Utxo {
            outpoint: IcCdkOutPoint {
                txid: txid2.as_byte_array().to_vec(),
                vout: 1,
            },
            value: 2000,
            height: 200,
        };
        let utxo3: Utxo = Utxo {
            outpoint: IcCdkOutPoint {
                txid: txid3.as_byte_array().to_vec(),
                vout: 2,
            },
            value: 3000,
            height: 300,
        };
        let utxo4: Utxo = Utxo {
            outpoint: IcCdkOutPoint {
                txid: txid4.as_byte_array().to_vec(),
                vout: 3,
            },
            value: 4000,
            height: 400,
        };
        vec![
            UtxoWrapper {
                utxo: utxo1,
                txid: txid1,
            },
            UtxoWrapper {
                utxo: utxo2,
                txid: txid2,
            },
            UtxoWrapper {
                utxo: utxo3,
                txid: txid3,
            },
            UtxoWrapper {
                utxo: utxo4,
                txid: txid4,
            },
        ]
    }

    #[test]
    fn test_get_input_value_returns_expected_value() {
        let mock_utxos = get_mock_utxos();
        let first_mock = mock_utxos[0].clone();
        let input = TxIn {
            previous_output: BitcoinOutPoint {
                txid: first_mock.txid,
                vout: first_mock.utxo.outpoint.vout,
            },
            sequence: Sequence(0xFFFF_FFFF),
            witness: Witness::new(),
            script_sig: ScriptBuf::new(),
        };

        let utxos: Vec<Utxo> = mock_utxos
            .iter()
            .map(|wrapper| wrapper.utxo.clone())
            .collect();
        let value = get_input_value(&input, &utxos);
        assert_eq!(value.unwrap().to_sat(), first_mock.utxo.value);
    }

    #[test]
    fn test_get_input_value_returns_none_if_no_value() {
        let mut mock_utxos = get_mock_utxos();
        // Pop the first one so that it's not in the list anymore.
        let first_mock = mock_utxos.pop().unwrap();
        // Use the popped value to create the input.
        let input = TxIn {
            previous_output: BitcoinOutPoint {
                txid: first_mock.txid,
                vout: first_mock.utxo.outpoint.vout,
            },
            sequence: Sequence(0xFFFF_FFFF),
            witness: Witness::new(),
            script_sig: ScriptBuf::new(),
        };

        let utxos: Vec<Utxo> = mock_utxos
            .iter()
            .map(|wrapper| wrapper.utxo.clone())
            .collect();
        let value = get_input_value(&input, &utxos);
        assert!(value.is_none());
    }

    #[test]
    fn test_build_p2wpkh_transaction_not_enough_funds() {
        let source_address = "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh"; // Valid source address

        let mock_utxos = get_mock_utxos();
        let first_mock = mock_utxos.first().unwrap();
        let utxos = vec![first_mock.utxo.clone()];
        let tx_fee = 500;

        let result = build_p2wpkh_transaction(
            source_address,
            BitcoinNetwork::Mainnet,
            &utxos,
            tx_fee,
            &vec![BtcTxOutput {
                destination_address: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string(),
                sent_satoshis: first_mock.utxo.value,
            }],
        );

        match result {
            Err(BuildP2wpkhTxError::NotEnoughFunds {
                required,
                available,
            }) => {
                assert_eq!(required, first_mock.utxo.value + tx_fee);
                assert_eq!(available, first_mock.utxo.value);
            }
            _ => panic!("Expected NotEnoughFunds error"),
        }
    }

    #[test]
    fn test_build_p2wpkh_transaction_invalid_source_address() {
        let invalid_address = "invalid_address";

        let result =
            build_p2wpkh_transaction(invalid_address, BitcoinNetwork::Mainnet, &[], 10, &vec![]);

        match result {
            Err(BuildP2wpkhTxError::InvalidSourceAddress { address }) => {
                assert_eq!(address, invalid_address);
            }
            _ => panic!("Expected InvalidSourceAddress error"),
        }
    }

    #[test]
    fn test_build_p2wpkh_transaction_wrong_bitcoin_network() {
        let source_address = "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh"; // Valid mainnet P2wpkh address

        let result = build_p2wpkh_transaction(
            source_address,
            BitcoinNetwork::Testnet, // Incorrect network for the address
            &[],
            10,
            &vec![],
        );

        match result {
            Err(BuildP2wpkhTxError::WrongBitcoinNetwork) => {}
            _ => panic!("Expected WrongBitcoinNetwork error"),
        }
    }

    #[test]
    fn test_build_p2wpkh_transaction_invalid_destination_address() {
        let source_address = "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh";
        let invalid_address = "invalid_destination".to_string();

        let result = build_p2wpkh_transaction(
            source_address,
            BitcoinNetwork::Mainnet,
            &[],
            10,
            &vec![BtcTxOutput {
                destination_address: invalid_address.clone(),
                sent_satoshis: 1000,
            }],
        );

        match result {
            Err(BuildP2wpkhTxError::InvalidDestinationAddress { address }) => {
                assert_eq!(address, invalid_address);
            }
            _ => panic!("Expected InvalidDestinationAddress error"),
        }
    }

    #[test]
    fn test_build_p2wpkh_transaction_not_p2wpkh_source_address() {
        let source_address = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"; // This is a legacy P2PKH address, not P2WPKH

        let result =
            build_p2wpkh_transaction(source_address, BitcoinNetwork::Mainnet, &[], 10, &vec![]);

        match result {
            Err(BuildP2wpkhTxError::NotP2WPKHSourceAddress) => {} // Success if this error is returned
            _ => panic!("Expected NotP2WPKHSourceAddress error"),
        }
    }

    #[test]
    fn test_build_p2wpkh_transaction_successful_transaction() {
        let source_address = "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh";

        let wrapped_utxos = get_mock_utxos();
        let utxos: Vec<Utxo> = wrapped_utxos
            .iter()
            .map(|wrapper| wrapper.utxo.clone())
            .collect();
        let utxos_amount: u64 = utxos.iter().map(|utxo| utxo.value).sum();
        let tx_fee = 400;
        let remaining = DUST_THRESHOLD * 2;
        // Leave some amount to be sent to the source address.
        let amount_sent = utxos_amount - tx_fee - remaining;

        let request_outputs = vec![BtcTxOutput {
            destination_address: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string(),
            sent_satoshis: amount_sent,
        }];

        let result = build_p2wpkh_transaction(
            source_address,
            BitcoinNetwork::Mainnet,
            &utxos,
            tx_fee,
            &request_outputs,
        );

        // Assert success
        match result {
            Ok(tx) => {
                assert_eq!(tx.input.len(), utxos.len());
                assert_eq!(tx.output.len(), 2); // 2 outputs (one for the destination, one for the change)

                // Check that the first output matches the sent amount
                assert_eq!(tx.output[0].value.to_sat(), amount_sent);

                // Check that the second output is the change
                assert_eq!(tx.output[1].value.to_sat(), remaining);
            }
            Err(e) => panic!("Expected successful transaction, got error: {:?}", e),
        }
    }
}

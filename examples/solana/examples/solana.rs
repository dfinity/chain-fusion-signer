//! Attempt to write a client
//!
//! # Bib
//! - <https://solana.com/docs/clients/rust>
//! - <https://medium.com/geekculture/create-a-solana-client-with-rust-62b35a59511d>
//! - <https://github.com/anmoldh121/solana-counter/tree/main>
//! - <https://dev.to/swaroopmaddu/calling-anchor-program-from-rust-1ee2>
//! - [Off chain message signing](https://github.com/solana-labs/solana/blob/master/docs/src/proposals/off-chain-message-signing.md)

use std::{thread::sleep, time::Duration};

use solana_client::{
    client_error::ClientError,
    rpc_client::{RpcClient, SerializableTransaction},
    rpc_config::RpcSendTransactionConfig,
    rpc_request::RpcError,
};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    hash::Hash,
    message::Message,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::{Signer, SignerError},
    system_instruction,
    transaction::Transaction,
};

fn main() {
    println!("Hello, world!");
    let rpc_client = RpcClient::new("http://localhost:8899");
    let pubkey = Pubkey::from(signer_cli::sync::pub_key());

    // Get an airdrop
    let lamports = 123456789101112;
    let sig = rpc_client
        .request_airdrop(&pubkey, lamports)
        .expect("Failed to request airdrop");
    loop {
        let confirmed = rpc_client
            .confirm_transaction(&sig)
            .expect("Failed to check whether the airdrop was confirmed");
        if confirmed {
            break;
        }
    }
    println!("Received an airdrop");
    // Check balance
    let balance = rpc_client
        .get_balance(&pubkey)
        .expect("Failed to get balance");
    println!("Balance is: {}", balance);
    // Send some funds to another account
    let recipient = Keypair::new();
    send_and_confirm_transaction(
        &rpc_client,
        &transfer(
            &recipient.pubkey(),
            123456789,
            rpc_client
                .get_latest_blockhash()
                .expect("Could not get latest blockhash"),
        ),
    )
    .expect("Failed to send transaction");
    let balance = rpc_client
        .get_balance(&pubkey)
        .expect("Failed to get balance");
    println!("Balance is: {}", balance);

    // Call canister
}

pub fn transfer(to: &Pubkey, lamports: u64, recent_blockhash: Hash) -> Transaction {
    let from_pubkey = Pubkey::from(signer_cli::sync::pub_key());
    let instruction = system_instruction::transfer(&from_pubkey, to, lamports);
    let message = Message::new(&[instruction], Some(&from_pubkey));
    {
        let mut tx = Transaction::new_unsigned(message);
        let keypairs = &[()]; // Previously: keypairs.  Could specify the derivation path and other parameters for use
                              // with the chain fusion signer.
        let pubkeys = [from_pubkey.clone()];
        {
            // try_partial_sign
            let positions = tx.get_signing_keypair_positions(&pubkeys).unwrap();
            if positions.iter().any(|pos| pos.is_none()) {
                panic!("Keypair pubkey mismatch");
            }
            let positions: Vec<usize> = positions.iter().map(|pos| pos.unwrap()).collect();

            {
                // Try partial sign unchecked
                // if you change the blockhash, you're re-signing...
                // In our case we haven't started yet, so this initializes all the signatures.
                /* if recent_blockhash != tx.message.recent_blockhash */
                {
                    println!("Re-signing");
                    tx.message.recent_blockhash = recent_blockhash;
                    tx.signatures
                        .iter_mut()
                        .for_each(|signature| *signature = Signature::default());
                }

                let signatures: Result<Vec<Signature>, SignerError> = {
                    let message = tx.message_data();
                    keypairs
                        .into_iter()
                        .map(|_keypair| {
                            //keypair.try_sign_message(&message)
                            // Try a bad signature:
                            // Ok(Signature::from([6u8; 64]))
                            Ok(Signature::from(signer_cli::sync::sign(&message)))
                        })
                        .collect()
                };
                let signatures = signatures.unwrap();

                for i in 0..positions.len() {
                    tx.signatures[positions[i]] = signatures[i];
                }
            }
        }
        tx
    }
}

pub fn send_and_confirm_transaction(
    rpc_client: &RpcClient,
    transaction: &impl SerializableTransaction,
) -> Result<Signature, ClientError> {
    const SEND_RETRIES: usize = 1;
    const GET_STATUS_RETRIES: usize = usize::MAX;

    'sending: for _ in 0..SEND_RETRIES {
        let signature = rpc_client.send_transaction(transaction)?;

        let recent_blockhash = if transaction.uses_durable_nonce() {
            let (recent_blockhash, ..) =
                rpc_client.get_latest_blockhash_with_commitment(CommitmentConfig::processed())?;
            recent_blockhash
        } else {
            *transaction.get_recent_blockhash()
        };

        for status_retry in 0..GET_STATUS_RETRIES {
            match rpc_client.get_signature_status(&signature)? {
                Some(Ok(_)) => return Ok(signature),
                Some(Err(e)) => return Err(e.into()),
                None => {
                    if !rpc_client
                        .is_blockhash_valid(&recent_blockhash, CommitmentConfig::processed())?
                    {
                        // Block hash is not found by some reason
                        break 'sending;
                    } else if cfg!(not(test))
                        // Ignore sleep at last step.
                        && status_retry < GET_STATUS_RETRIES
                    {
                        // Retry twice a second
                        sleep(Duration::from_millis(500));
                        continue;
                    }
                }
            }
        }
    }

    Err(RpcError::ForUser(
        "unable to confirm transaction. \
         This can happen in situations such as transaction expiration \
         and insufficient fee-payer funds"
            .to_string(),
    )
    .into())
}

pub fn send_transaction(
    rpc_client: &RpcClient,
    transaction: &impl SerializableTransaction,
) -> Result<Signature, ClientError> {
    rpc_client.send_transaction_with_config(
        transaction,
        RpcSendTransactionConfig {
            preflight_commitment: Some(rpc_client.commitment().commitment),
            ..RpcSendTransactionConfig::default()
        },
    )
}

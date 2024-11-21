//! Attempt to write a client
//!
//! # Bib
//! - <https://solana.com/docs/clients/rust>
//! - <https://medium.com/geekculture/create-a-solana-client-with-rust-62b35a59511d>
//! - <https://github.com/anmoldh121/solana-counter/tree/main>
//! - <https://dev.to/swaroopmaddu/calling-anchor-program-from-rust-1ee2>
//! - [Off chain message signing](https://github.com/solana-labs/solana/blob/master/docs/src/proposals/off-chain-message-signing.md)

use std::{fs::File, io::BufReader, thread::sleep, time::Duration};

use signer_cli::{args::SignerCliArgs, SignerCli};
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
    signers::Signers,
    system_instruction,
    transaction::Transaction,
};
use tokio::runtime::{Builder, Runtime};

fn main() {
    println!("Hello, world!");
    doitall();
}

fn runtime() -> Runtime {
    // The IC client
    Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Unable to create a runtime")
}

fn ic_cli() -> SignerCli {
    runtime().block_on(async {
        SignerCli::new(SignerCliArgs {
            network: None,
            identity: None,
            verbose: 0,
            quiet: 0,
        })
        .await
        .expect("failed to create signer cli")
    })
}

fn doitall() {
    // IC client
    let ic_cli_instance = ic_cli();
    let pubkey: [u8;32] = runtime().block_on(async {
         ic_cli_instance
        .schnorr_public_key()
        .await
        .expect("failed to get schnorr public key")
        .public_key
    }).try_into().expect("Public key has wrong length");
    println!(
        "IC public key: {:?}  ({} bytes)",
        &pubkey,
        pubkey.len()
    );
    let pubkey = Pubkey::from(pubkey);
    //
    let rpc_client = RpcClient::new("localhost:8899");

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
            &ic_cli_instance,
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

pub fn transfer(
    signer_cli: &SignerCli,
    to: &Pubkey,
    lamports: u64,
    recent_blockhash: Hash,
) -> Transaction {
    let from_pubkey = {
        let pub_key = runtime().block_on(async {
            signer_cli
            .schnorr_public_key()
            .await
            .expect("Failed to get schnorr public key")
            .public_key
        });
        let pub_key: [u8;32] = pub_key.try_into().expect("Public key has wrong length");
        Pubkey::from(pub_key)
    };
    let instruction = system_instruction::transfer(&from_pubkey, to, lamports);
    let message = Message::new(&[instruction], Some(&from_pubkey));
    {
        let mut tx = Transaction::new_unsigned(message);
        {
            // try_partial_sign
            let positions: Vec<Option<usize>> = vec![Some(0)]; 
            if positions.iter().any(|pos| pos.is_none()) {
                panic!("Keypair pubkey mismatch");
            }
            let positions: Vec<usize> = positions.iter().map(|pos| pos.unwrap()).collect();

            {
                // Try partial sign unchecked
                // if you change the blockhash, you're re-signing...
                // In our case we haven't started yet, so this initializes all the signatures.
                /*if recent_blockhash != tx.message.recent_blockhash*/
                {
                    println!("Re-signing");
                    tx.message.recent_blockhash = recent_blockhash;
                    tx.signatures
                        .iter_mut()
                        .for_each(|signature| *signature = Signature::default());
                }

                let signatures: Result<Vec<Signature>, SignerError> = {
                    let message = tx.message_data();
                    let signature_bytes = runtime().block_on(async {
                        SignerCli::new(SignerCliArgs::default()).await.unwrap()
                        .schnorr_sign(&message)
                        .await
                        .expect("Failed to sign")
                    });
                    let signature_bytes: [u8;64] = signature_bytes.try_into().expect("Signature has wrong length");
                    Ok(vec![Signature::from(signature_bytes)])
                    /*
                                        keypairs
                                            .into_iter()
                                            .map(async |keypair| {
                                                //keypair.try_sign_message(&message)
                                                // Try a bad signature:
                                                // Ok(Signature::from([6u8; 64]))
                                                let signature_bytes = SignerCli::new(SignerCliArgs::default())
                                                    .schnorr_sign(&message).await.expect("Failed to sign");
                                                Ok(Signature::new(&signature_bytes))
                                            })
                                            .collect()
                    */
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

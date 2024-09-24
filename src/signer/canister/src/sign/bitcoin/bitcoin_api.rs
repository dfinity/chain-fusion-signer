use ic_cdk::api::management_canister::bitcoin::{
    bitcoin_get_balance, bitcoin_get_current_fee_percentiles, bitcoin_send_transaction,
    BitcoinNetwork, GetBalanceRequest, GetCurrentFeePercentilesRequest, MillisatoshiPerByte,
    SendTransactionRequest,
};

/// Returns the balance of the given bitcoin address.
///
/// Relies on the `bitcoin_get_balance` endpoint.
/// See [Bitcoin API](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-bitcoin_get_balance)
pub async fn get_balance(network: BitcoinNetwork, address: String) -> Result<u64, String> {
    let min_confirmations = None;
    let balance_res = bitcoin_get_balance(GetBalanceRequest {
        address,
        network,
        min_confirmations,
    })
    .await
    .map_err(|err| err.1)?;

    Ok(balance_res.0)
}

/// Returns the 100 fee percentiles measured in millisatoshi/byte.
/// Percentiles are computed from the last 10,000 transactions (if available).
///
/// Relies on the `bitcoin_get_current_fee_percentiles` endpoint.
/// See [Bitcoin API](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-bitcoin_get_current_fee_percentiles)
async fn get_current_fee_percentiles(
    network: BitcoinNetwork,
) -> Result<Vec<MillisatoshiPerByte>, String> {
    let res = bitcoin_get_current_fee_percentiles(GetCurrentFeePercentilesRequest { network })
        .await
        .map_err(|err| err.1)?;

    Ok(res.0)
}

/// Returns the 50th percentile for sending fees.
pub async fn get_fee_per_byte(network: BitcoinNetwork) -> Result<u64, String> {
    // Get fee percentiles from previous transactions to estimate our own fee.
    let fee_percentiles = get_current_fee_percentiles(network).await?;

    if fee_percentiles.is_empty() {
        // There are no fee percentiles. This case can only happen on a regtest
        // network where there are no non-coinbase transactions. In this case,
        // we use a default of 2000 millisatoshis/byte (i.e. 2 satoshi/byte)
        Ok(2000)
    } else {
        Ok(fee_percentiles[50])
    }
}

/// Sends a (signed) transaction to the bitcoin network.
///
/// Relies on the `bitcoin_send_transaction` endpoint.
/// See [Bitcoin API](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-bitcoin_send_transaction)
pub async fn send_transaction(network: BitcoinNetwork, transaction: Vec<u8>) -> Result<(), String> {
    bitcoin_send_transaction(SendTransactionRequest {
        transaction,
        network,
    })
    .await
    .map_err(|err| err.1)?;

    Ok(())
}

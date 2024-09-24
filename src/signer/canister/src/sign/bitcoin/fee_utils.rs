use ic_cdk::api::management_canister::bitcoin::{BitcoinNetwork, Utxo};
use super::bitcoin_api;

/// Functions stolen from ckBTC Minter: https://github.com/dfinity/ic/blob/285a5db07da50a4e350ec43bf3b488cc6fe36102/rs/bitcoin/ckbtc/minter/src/lib.rs#L1258

// One output for the caller and one for the change.
const DEFAULT_OUTPUT_COUNT: u64 = 2;

/// Computes an estimate for the size of transaction (in vbytes) with the given number of inputs and outputs.
fn tx_vsize_estimate(input_count: u64, output_count: u64) -> u64 {
  // See
  // https://github.com/bitcoin/bips/blob/master/bip-0141.mediawiki
  // for the transaction structure and
  // https://bitcoin.stackexchange.com/questions/92587/calculate-transaction-fee-for-external-addresses-which-doesnt-belong-to-my-loca/92600#92600
  // for transaction size estimate.
  const INPUT_SIZE_VBYTES: u64 = 68;
  const OUTPUT_SIZE_VBYTES: u64 = 31;
  const TX_OVERHEAD_VBYTES: u64 = 11;

  input_count * INPUT_SIZE_VBYTES + output_count * OUTPUT_SIZE_VBYTES + TX_OVERHEAD_VBYTES
}

/// Computes an estimate for the retrieve_btc fee.
///
/// Arguments:
///   * `available_utxos` - the list of UTXOs available to the minter.
///   * `maybe_amount` - the withdrawal amount.
///   * `median_fee_millisatoshi_per_vbyte` - the median network fee, in millisatoshi per vbyte.
///
/// Functions stolen from ckBTC Minter: https://github.com/dfinity/ic/blob/285a5db07da50a4e350ec43bf3b488cc6fe36102/rs/bitcoin/ckbtc/minter/src/lib.rs#L1258
pub fn estimate_fee(selected_utxos: &[Utxo], median_fee_millisatoshi_per_vbyte: u64) -> u64 {
  let input_count = selected_utxos.len() as u64;

  let vsize = tx_vsize_estimate(input_count, DEFAULT_OUTPUT_COUNT);
  // We subtract one from the outputs because the minter's output
  // does not participate in fees distribution.
  let bitcoin_fee =
      vsize * median_fee_millisatoshi_per_vbyte / 1000 / (DEFAULT_OUTPUT_COUNT - 1).max(1);
  bitcoin_fee
}

async fn get_default_fee(utxos: &Vec<Utxo>, network: BitcoinNetwork) -> Result<u64, String> {
  let fee_per_byte = bitcoin_api::get_fee_per_byte(network).await?;
  Ok(estimate_fee(utxos, fee_per_byte))
}

pub async fn calculate_fee(maybe_fee: Option<u64>, utxos: &Vec<Utxo>, network: BitcoinNetwork) -> Result<u64, String> {
  match maybe_fee {
      Some(fee) => Ok(fee),
      None => get_default_fee(utxos, network).await,
  }
}
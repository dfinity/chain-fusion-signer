# Chain Fusion Signer ❤️ Solana


## Test environment
* Install [the Solana CLI](https://solana.com/docs/intro/installation).
* Start both dfx and Solana:
  * `dfx start --clean`
  * `solana-test-validator --reset`
* Deploy the chain fusion signer and all the other canisters you might want to use for testing:
  * `cd` to the root of this repository.
  * `dfx deploy`
* Verify that the chain fusion signer is installed and is able to sign:
  * `cargo run --bin signer-cli`
* Verify that the chain fusion signer can sign Solana transactions:
  * `cargo run --bin solana-cli`
# Pricing Report

**Date: March 2026**

## Pricing policy

Some costs are not paid by PAPI, such as the cost of failed API calls.

API calls are therefore charged at cost plus a margin to cover otherwise unpaid expenditures, that would otherwise cause the canister to eventually run out of cycles.

The margin is currently set at 40% over a typical API call for each method. Note that the most expensive call possible for each method is higher than the typical call.

Fees are rounded, to make them simpler to use and remember.

## How to update pricing

Run the check-pricing script against the `beta` canister:

```
scripts/check-pricing beta
```

The script will:
1. Measure the actual cycle cost of each API method
2. Compare against the current fees in `src/signer/api/src/methods.rs`
3. Calculate recommended fees (cost + 40% margin, rounded)
4. Print results and update this report

To re-analyse an existing measurement without re-running canister calls:

```
scripts/check-pricing --analyze <jsonl-file>
```

## Current fees

The fee in cycles charged for each method is:

```
            SignerMethods::BtcCallerAddress => 79_000_000,
            SignerMethods::BtcCallerBalance => 113_000_000,
            SignerMethods::BtcCallerSend => 132_000_000_000,
            SignerMethods::BtcCallerSign => 132_000_000_000,
            SignerMethods::EthAddress | SignerMethods::EthAddressOfCaller => 77_000_000,
            SignerMethods::EthPersonalSign => 37_000_000_000,
            SignerMethods::EthSignPrehash => 37_000_000_000,
            SignerMethods::EthSignTransaction => 37_000_000_000,
            SignerMethods::GenericCallerEcdsaPublicKey => 77_000_000,
            SignerMethods::GenericSignWithEcdsa => 37_000_000_000,
            SignerMethods::SchnorrPublicKey => 77_000_000,
            SignerMethods::SchnorrSign => 37_000_000_000,
```

## Check results

```
OK: Signer balance rose by 18721905 for: schnorr_public_key
OK: Signer balance rose by 10787834615 for: schnorr_sign
OK: Signer balance rose by 20002986 for: btc_caller_address
OK: Signer balance rose by 29482092 for: btc_caller_balance
OK: Signer balance rose by 53389886961 for: btc_caller_sign
OK: Signer balance rose by 37954522981 for: btc_caller_send
OK: Signer balance rose by 19210845 for: eth_address
OK: Signer balance rose by 10747829457 for: eth_personal_sign
OK: Signer balance rose by 10723111175 for: eth_sign_prehash
OK: Signer balance rose by 10746203674 for: eth_sign_transaction
OK: Signer balance rose by 18898161 for: generic_caller_ecdsa_public_key
OK: Signer balance rose by 10787893521 for: generic_sign_with_ecdsa
```

## Analysis

```
{
  "method_name": "btc_caller_address",
  "fee": 79000000,
  "cycles_balance_before": 11705572451724,
  "cycles_balance_after": 11705592454710,
  "diff": 20002986,
  "typical_cost": 58997014,
  "cost_plus": 82595819.6,
  "rounding": 1000000,
  "recommended_fee": 83000000,
  "recommended_change": 4000000,
  "fee_usd": 0.00010559219,
  "recommended_fee_usd": 0.00011093863
}
{
  "method_name": "btc_caller_balance",
  "fee": 113000000,
  "cycles_balance_before": 11705587370403,
  "cycles_balance_after": 11705616852495,
  "diff": 29482092,
  "typical_cost": 83517908,
  "cost_plus": 116925071.19999999,
  "rounding": 1000000,
  "recommended_fee": 117000000,
  "recommended_change": 4000000,
  "fee_usd": 0.00015103693,
  "recommended_fee_usd": 0.00015638337
}
{
  "method_name": "btc_caller_send",
  "fee": 132000000000,
  "cycles_balance_before": 11758996570842,
  "cycles_balance_after": 11796951093823,
  "diff": 37954522981,
  "typical_cost": 94045477019,
  "cost_plus": 131663667826.59999,
  "rounding": 1000000000,
  "recommended_fee": 132000000000,
  "recommended_change": 0,
  "fee_usd": 0.17643252,
  "recommended_fee_usd": 0.17643252
}
{
  "method_name": "btc_caller_sign",
  "fee": 132000000000,
  "cycles_balance_before": 11705611768188,
  "cycles_balance_after": 11759001655149,
  "diff": 53389886961,
  "typical_cost": 78610113039,
  "cost_plus": 110054158254.59999,
  "rounding": 1000000000,
  "recommended_fee": 111000000000,
  "recommended_change": -21000000000,
  "fee_usd": 0.17643252,
  "recommended_fee_usd": 0.14836371
}
{
  "method_name": "eth_address",
  "fee": 77000000,
  "cycles_balance_before": 11796946009516,
  "cycles_balance_after": 11796965220361,
  "diff": 19210845,
  "typical_cost": 57789155,
  "cost_plus": 80904817,
  "rounding": 1000000,
  "recommended_fee": 81000000,
  "recommended_change": 4000000,
  "fee_usd": 0.00010291897,
  "recommended_fee_usd": 0.00010826541000000001
}
{
  "method_name": "eth_personal_sign",
  "fee": 37000000000,
  "cycles_balance_before": 11796960136054,
  "cycles_balance_after": 11807707965511,
  "diff": 10747829457,
  "typical_cost": 26252170543,
  "cost_plus": 36753038760.2,
  "rounding": 1000000000,
  "recommended_fee": 37000000000,
  "recommended_change": 0,
  "fee_usd": 0.04945457,
  "recommended_fee_usd": 0.04945457
}
{
  "method_name": "eth_sign_prehash",
  "fee": 37000000000,
  "cycles_balance_before": 11807702849354,
  "cycles_balance_after": 11818425960529,
  "diff": 10723111175,
  "typical_cost": 26276888825,
  "cost_plus": 36787644355,
  "rounding": 1000000000,
  "recommended_fee": 37000000000,
  "recommended_change": 0,
  "fee_usd": 0.04945457,
  "recommended_fee_usd": 0.04945457
}
{
  "method_name": "eth_sign_transaction",
  "fee": 37000000000,
  "cycles_balance_before": 11818420844372,
  "cycles_balance_after": 11829167048046,
  "diff": 10746203674,
  "typical_cost": 26253796326,
  "cost_plus": 36755314856.399994,
  "rounding": 1000000000,
  "recommended_fee": 37000000000,
  "recommended_change": 0,
  "fee_usd": 0.04945457,
  "recommended_fee_usd": 0.04945457
}
{
  "method_name": "generic_caller_ecdsa_public_key",
  "fee": 77000000,
  "cycles_balance_before": 11829161963739,
  "cycles_balance_after": 11829180861900,
  "diff": 18898161,
  "typical_cost": 58101839,
  "cost_plus": 81342574.6,
  "rounding": 1000000,
  "recommended_fee": 82000000,
  "recommended_change": 5000000,
  "fee_usd": 0.00010291897,
  "recommended_fee_usd": 0.00010960202000000001
}
{
  "method_name": "generic_sign_with_ecdsa",
  "fee": 37000000000,
  "cycles_balance_before": 11829175777593,
  "cycles_balance_after": 11839963671114,
  "diff": 10787893521,
  "typical_cost": 26212106479,
  "cost_plus": 36696949070.6,
  "rounding": 1000000000,
  "recommended_fee": 37000000000,
  "recommended_change": 0,
  "fee_usd": 0.04945457,
  "recommended_fee_usd": 0.04945457
}
{
  "method_name": "schnorr_public_key",
  "fee": 77000000,
  "cycles_balance_before": 11694776095668,
  "cycles_balance_after": 11694794817573,
  "diff": 18721905,
  "typical_cost": 58278095,
  "cost_plus": 81589333,
  "rounding": 1000000,
  "recommended_fee": 82000000,
  "recommended_change": 5000000,
  "fee_usd": 0.00010291897,
  "recommended_fee_usd": 0.00010960202000000001
}
{
  "method_name": "schnorr_sign",
  "fee": 37000000000,
  "cycles_balance_before": 11694789701416,
  "cycles_balance_after": 11705577536031,
  "diff": 10787834615,
  "typical_cost": 26212165385,
  "cost_plus": 36697031539,
  "rounding": 1000000000,
  "recommended_fee": 37000000000,
  "recommended_change": 0,
  "fee_usd": 0.04945457,
  "recommended_fee_usd": 0.04945457
}
```

### Conclusion

Fees that should be **increased**: `btc_caller_address`, `btc_caller_balance`, `eth_address`, `generic_caller_ecdsa_public_key`, `schnorr_public_key`.

Fees that can be **reduced**: `btc_caller_sign`.

No change needed: `btc_caller_send`, `eth_personal_sign`, `eth_sign_prehash`, `eth_sign_transaction`, `generic_sign_with_ecdsa`, `schnorr_sign`.

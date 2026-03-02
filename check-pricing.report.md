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

To re-analyze an existing measurement without re-running canister calls:

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
OK: Signer balance rose by 18_699_807 for: schnorr_public_key
OK: Signer balance rose by 10_787_823_584 for: schnorr_sign
OK: Signer balance rose by 20_007_839 for: btc_caller_address
OK: Signer balance rose by 29_532_659 for: btc_caller_balance
OK: Signer balance rose by 53_389_820_640 for: btc_caller_sign
OK: Signer balance rose by 37_994_481_882 for: btc_caller_send
OK: Signer balance rose by 19_222_125 for: eth_address
OK: Signer balance rose by 10_722_728_398 for: eth_personal_sign
OK: Signer balance rose by 10_748_162_886 for: eth_sign_prehash
OK: Signer balance rose by 10_721_011_065 for: eth_sign_transaction
OK: Signer balance rose by 18_862_233 for: generic_caller_ecdsa_public_key
OK: Signer balance rose by 10_787_898_370 for: generic_sign_with_ecdsa
```

## Analysis

```
{
  "method_name": "btc_caller_address",
  "fee": 79000000,
  "cycles_balance_before": 11963718370286,
  "cycles_balance_after": 11963738378125,
  "diff": 20007839,
  "typical_cost": 58992161,
  "cost_plus": 82589025.39999999,
  "rounding": 1000000,
  "recommended_fee": 83000000,
  "recommended_change": 4000000,
  "fee_usd": 0.00010559219,
  "recommended_fee_usd": 0.00011093863
}
{
  "method_name": "btc_caller_balance",
  "fee": 113000000,
  "cycles_balance_before": 11963733293818,
  "cycles_balance_after": 11963762826477,
  "diff": 29532659,
  "typical_cost": 83467341,
  "cost_plus": 116854277.39999999,
  "rounding": 1000000,
  "recommended_fee": 117000000,
  "recommended_change": 4000000,
  "fee_usd": 0.00015103693,
  "recommended_fee_usd": 0.00015638337
}
{
  "method_name": "btc_caller_send",
  "fee": 132000000000,
  "cycles_balance_before": 12017142478503,
  "cycles_balance_after": 12055136960385,
  "diff": 37994481882,
  "typical_cost": 94005518118,
  "cost_plus": 131607725365.2,
  "rounding": 1000000000,
  "recommended_fee": 132000000000,
  "recommended_change": 0,
  "fee_usd": 0.17643252,
  "recommended_fee_usd": 0.17643252
}
{
  "method_name": "btc_caller_sign",
  "fee": 132000000000,
  "cycles_balance_before": 11963757742170,
  "cycles_balance_after": 12017147562810,
  "diff": 53389820640,
  "typical_cost": 78610179360,
  "cost_plus": 110054251104,
  "rounding": 1000000000,
  "recommended_fee": 111000000000,
  "recommended_change": -21000000000,
  "fee_usd": 0.17643252,
  "recommended_fee_usd": 0.14836371
}
{
  "method_name": "eth_address",
  "fee": 77000000,
  "cycles_balance_before": 12055131876078,
  "cycles_balance_after": 12055151098203,
  "diff": 19222125,
  "typical_cost": 57777875,
  "cost_plus": 80889025,
  "rounding": 1000000,
  "recommended_fee": 81000000,
  "recommended_change": 4000000,
  "fee_usd": 0.00010291897,
  "recommended_fee_usd": 0.00010826541000000001
}
{
  "method_name": "eth_personal_sign",
  "fee": 37000000000,
  "cycles_balance_before": 12055145982046,
  "cycles_balance_after": 12065868710444,
  "diff": 10722728398,
  "typical_cost": 26277271602,
  "cost_plus": 36788180242.799995,
  "rounding": 1000000000,
  "recommended_fee": 37000000000,
  "recommended_change": 0,
  "fee_usd": 0.04945457,
  "recommended_fee_usd": 0.04945457
}
{
  "method_name": "eth_sign_prehash",
  "fee": 37000000000,
  "cycles_balance_before": 12065863626137,
  "cycles_balance_after": 12076611789023,
  "diff": 10748162886,
  "typical_cost": 26251837114,
  "cost_plus": 36752571959.6,
  "rounding": 1000000000,
  "recommended_fee": 37000000000,
  "recommended_change": 0,
  "fee_usd": 0.04945457,
  "recommended_fee_usd": 0.04945457
}
{
  "method_name": "eth_sign_transaction",
  "fee": 37000000000,
  "cycles_balance_before": 12076606704716,
  "cycles_balance_after": 12087327715781,
  "diff": 10721011065,
  "typical_cost": 26278988935,
  "cost_plus": 36790584509,
  "rounding": 1000000000,
  "recommended_fee": 37000000000,
  "recommended_change": 0,
  "fee_usd": 0.04945457,
  "recommended_fee_usd": 0.04945457
}
{
  "method_name": "generic_caller_ecdsa_public_key",
  "fee": 77000000,
  "cycles_balance_before": 12087322631474,
  "cycles_balance_after": 12087341493707,
  "diff": 18862233,
  "typical_cost": 58137767,
  "cost_plus": 81392873.8,
  "rounding": 1000000,
  "recommended_fee": 82000000,
  "recommended_change": 5000000,
  "fee_usd": 0.00010291897,
  "recommended_fee_usd": 0.00010960202000000001
}
{
  "method_name": "generic_sign_with_ecdsa",
  "fee": 37000000000,
  "cycles_balance_before": 12087336409400,
  "cycles_balance_after": 12098124307770,
  "diff": 10787898370,
  "typical_cost": 26212101630,
  "cost_plus": 36696942282,
  "rounding": 1000000000,
  "recommended_fee": 37000000000,
  "recommended_change": 0,
  "fee_usd": 0.04945457,
  "recommended_fee_usd": 0.04945457
}
{
  "method_name": "schnorr_public_key",
  "fee": 77000000,
  "cycles_balance_before": 11952922047359,
  "cycles_balance_after": 11952940747166,
  "diff": 18699807,
  "typical_cost": 58300193,
  "cost_plus": 81620270.19999999,
  "rounding": 1000000,
  "recommended_fee": 82000000,
  "recommended_change": 5000000,
  "fee_usd": 0.00010291897,
  "recommended_fee_usd": 0.00010960202000000001
}
{
  "method_name": "schnorr_sign",
  "fee": 37000000000,
  "cycles_balance_before": 11952935662859,
  "cycles_balance_after": 11963723486443,
  "diff": 10787823584,
  "typical_cost": 26212176416,
  "cost_plus": 36697046982.399994,
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

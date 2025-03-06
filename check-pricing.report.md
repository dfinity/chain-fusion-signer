# Pricing Report

**Date: March 2025**

## Motivation
The cost of operating a canister has increased and some prices no longer cover the cost of a typical API call:

```
WARNING: signer balance fell by -35_707_210 for: btc_caller_address
WARNING: signer balance fell by -40_206_857 for: btc_caller_balance
OK: Signer balance rose by 35_977_643_522 for: btc_caller_send
OK: Signer balance rose by 945_542_196 for: eth_address
OK: Signer balance rose by 13_715_468_661 for: eth_personal_sign
OK: Signer balance rose by 13_746_201_113 for: eth_sign_prehash
OK: Signer balance rose by 13_744_199_562 for: eth_sign_transaction
OK: Signer balance rose by 945_211_461 for: generic_caller_ecdsa_public_key
OK: Signer balance rose by 13_791_213_266 for: generic_sign_with_ecdsa
```

## Current pricing
The cost in cycles for each method is:
```
            SignerMethods::GenericCallerEcdsaPublicKey
            | SignerMethods::SchnorrPublicKey
            | SignerMethods::EthAddress
            | SignerMethods::EthAddressOfCaller => 1_000_000_000,
            SignerMethods::GenericSignWithEcdsa
            | SignerMethods::SchnorrSign
            | SignerMethods::EthSignTransaction
            | SignerMethods::EthPersonalSign
            | SignerMethods::EthSignPrehash => 40_000_000_000,
            SignerMethods::BtcCallerAddress => 20_000_000,
            SignerMethods::BtcCallerBalance => 40_000_000,
            SignerMethods::BtcCallerSend => 130_000_000_000,
```

## Pricing policy
Some costs are not paid by PAPI, such as the cost of failed API calls.

API calls are therefore charged at cost plus a margin to cover otherwise unpaid expenditures, that would otherwise cause the canister to eventually run out of cycles.

The margin is currently set at 40% over a typical API call for each method.  Note that the most expensive call possible for each method is higher than the typical call.

Fees are rounded, to make them simpler to use and remember.

## Pricing calculation
Prices for typical calls are determined by running `scripts/check-pricing` against the `beta` canister.

Adding the current prices to the output JSON we get:


```
cat check-pricing.beta.2025-03-06T11\:16\:56+01\:00.fees.jsonl | jq '.typical_cost = .fee - .diff | .cost_plus = .typical_cost * 1.4 | .recommended_fee = (.cost_plus | if . <1000000 then (. / 1000000 | ceil) * 1000000 else (. / 1000000000 | ceil) * 1000000000 end) | .recommended_change = (.recommended_fee - .fee)'
{
  "method_name": "btc_caller_address",
  "fee": 20000000,
  "cycles_balance_before": 10953343142984,
  "cycles_balance_after": 10953307435774,
  "diff": -35707210,
  "typical_cost": 55707210,
  "cost_plus": 77990094,
  "recommended_fee": 1000000000,
  "recommended_change": 980000000
}
{
  "method_name": "btc_caller_balance",
  "fee": 40000000,
  "cycles_balance_before": 10953303993928,
  "cycles_balance_after": 10953263787071,
  "diff": -40206857,
  "typical_cost": 80206857,
  "cost_plus": 112289599.8,
  "recommended_fee": 1000000000,
  "recommended_change": 960000000
}
{
  "method_name": "btc_caller_send",
  "fee": 130000000000,
  "cycles_balance_before": 10953260345225,
  "cycles_balance_after": 10989237988747,
  "diff": 35977643522,
  "typical_cost": 94022356478,
  "cost_plus": 131631299069.2,
  "recommended_fee": 132000000000,
  "recommended_change": 2000000000
}
{
  "method_name": "eth_address",
  "fee": 1000000000,
  "cycles_balance_before": 10989234546901,
  "cycles_balance_after": 10990180089097,
  "diff": 945542196,
  "typical_cost": 54457804,
  "cost_plus": 76240925.6,
  "recommended_fee": 1000000000,
  "recommended_change": 0
}
{
  "method_name": "eth_personal_sign",
  "fee": 40000000000,
  "cycles_balance_before": 10990176647251,
  "cycles_balance_after": 11003892115912,
  "diff": 13715468661,
  "typical_cost": 26284531339,
  "cost_plus": 36798343874.6,
  "recommended_fee": 37000000000,
  "recommended_change": -3000000000
}
{
  "method_name": "eth_sign_prehash",
  "fee": 40000000000,
  "cycles_balance_before": 11003888674066,
  "cycles_balance_after": 11017634875179,
  "diff": 13746201113,
  "typical_cost": 26253798887,
  "cost_plus": 36755318441.799995,
  "recommended_fee": 37000000000,
  "recommended_change": -3000000000
}
{
  "method_name": "eth_sign_transaction",
  "fee": 40000000000,
  "cycles_balance_before": 11017631433333,
  "cycles_balance_after": 11031375632895,
  "diff": 13744199562,
  "typical_cost": 26255800438,
  "cost_plus": 36758120613.2,
  "recommended_fee": 37000000000,
  "recommended_change": -3000000000
}
{
  "method_name": "generic_caller_ecdsa_public_key",
  "fee": 1000000000,
  "cycles_balance_before": 11031372191049,
  "cycles_balance_after": 11032317402510,
  "diff": 945211461,
  "typical_cost": 54788539,
  "cost_plus": 76703954.6,
  "recommended_fee": 1000000000,
  "recommended_change": 0
}
{
  "method_name": "generic_sign_with_ecdsa",
  "fee": 40000000000,
  "cycles_balance_before": 11032313890556,
  "cycles_balance_after": 11046105103822,
  "diff": 13791213266,
  "typical_cost": 26208786734,
  "cost_plus": 36692301427.6,
  "recommended_fee": 37000000000,
  "recommended_change": -3000000000
}
```

### Conclusion
Ethereum signing prices can be reduced slightly.  `btc_caller_address` and `btc_caller_balance` prices need to be increased.
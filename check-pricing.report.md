# Pricing Report

**Date: March 2025**

## Motivation

The cost of operating a canister has increased and some prices no longer cover the cost of a typical API call:

```
OK: Signer balance rose by 18_721_905 for: schnorr_public_key
OK: Signer balance rose by 10_787_834_615 for: schnorr_sign
OK: Signer balance rose by 20_002_986 for: btc_caller_address
OK: Signer balance rose by 29_482_092 for: btc_caller_balance
OK: Signer balance rose by 53_389_886_961 for: btc_caller_sign
OK: Signer balance rose by 37_954_522_981 for: btc_caller_send
OK: Signer balance rose by 19_210_845 for: eth_address
OK: Signer balance rose by 10_747_829_457 for: eth_personal_sign
OK: Signer balance rose by 10_723_111_175 for: eth_sign_prehash
OK: Signer balance rose by 10_746_203_674 for: eth_sign_transaction
OK: Signer balance rose by 18_898_161 for: generic_caller_ecdsa_public_key
OK: Signer balance rose by 10_787_893_521 for: generic_sign_with_ecdsa
```

## Current pricing

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

## Pricing policy

Some costs are not paid by PAPI, such as the cost of failed API calls.

API calls are therefore charged at cost plus a margin to cover otherwise unpaid expenditures, that would otherwise cause the canister to eventually run out of cycles.

The margin is currently set at 40% over a typical API call for each method. Note that the most expensive call possible for each method is higher than the typical call.

Fees are rounded, to make them simpler to use and remember.

## Pricing calculation

Prices for typical calls are determined by running `scripts/check-pricing` against the `beta` canister.

Adding the current prices to the output JSON we get:

```
BASE="check-pricing.beta.2026-03-02T13:12:44+01:00"

jq -s '. | map({key: .method_name, value: .}) | from_entries' \
  "fees.jsonl" > "fees.json"

jq -s '. | map({key: .method_name, value: .}) | from_entries' \
  "${BASE}.jsonl" > "${BASE}.json"

jq -s '.[0] * .[1] | to_entries | .[].value' \
  "fees.json" \
  "${BASE}.json" \
  > "${BASE}.fees.json"
```

```
$ cat check-pricing.beta.2026-02-27T14:13:09+01:00.fees.json | jq '.typical_cost = .fee - .diff | .cost_plus = .typical_cost * 1.4 | .rounding = if .cost_plus <1000000000 then 1000000 else 1000000000 end | .recommended_fee = ((.cost_plus / .rounding | ceil) * .rounding) | .recommended_change = (.recommended_fee - .fee) | .fee_usd = .fee / 1000000000000 * 1.336610 | .recommended_fee_usd = .recommended_fee /
  1000000000000 * 1.336610'

{
  "method_name": "schnorr_public_key",
  "fee": 1000000000,
  "cycles_balance_before": 11245238972675,
  "cycles_balance_after": 11246184000028,
  "diff": 945027353,
  "typical_cost": 54972647,
  "cost_plus": 76961705.8,
  "rounding": 1000000,
  "recommended_fee": 77000000,
  "recommended_change": -923000000,
  "fee_usd": 0.00133661,
  "recommended_fee_usd": 0.00010291897
}
{
  "method_name": "schnorr_sign",
  "fee": 40000000000,
  "cycles_balance_before": 11246180558182,
  "cycles_balance_after": 11259971620565,
  "diff": 13791062383,
  "typical_cost": 26208937617,
  "cost_plus": 36692512663.799995,
  "rounding": 1000000000,
  "recommended_fee": 37000000000,
  "recommended_change": -3000000000,
  "fee_usd": 0.0534644,
  "recommended_fee_usd": 0.04945457
}
{
  "method_name": "btc_caller_address",
  "fee": 20000000,
  "cycles_balance_before": 11259968178719,
  "cycles_balance_after": 11259932450151,
  "diff": -35728568,
  "typical_cost": 55728568,
  "cost_plus": 78019995.19999999,
  "rounding": 1000000,
  "recommended_fee": 79000000,
  "recommended_change": 59000000,
  "fee_usd": 2.6732200000000005e-05,
  "recommended_fee_usd": 0.00010559219
}
{
  "method_name": "btc_caller_balance",
  "fee": 40000000,
  "cycles_balance_before": 11259928976438,
  "cycles_balance_after": 11259888728172,
  "diff": -40248266,
  "typical_cost": 80248266,
  "cost_plus": 112347572.39999999,
  "rounding": 1000000,
  "recommended_fee": 113000000,
  "recommended_change": 73000000,
  "fee_usd": 5.346440000000001e-05,
  "recommended_fee_usd": 0.00015103693
}
{
  "method_name": "btc_caller_send",
  "fee": 130000000000,
  "cycles_balance_before": 11259885286326,
  "cycles_balance_after": 11295882862880,
  "diff": 35997576554,
  "typical_cost": 94002423446,
  "cost_plus": 131603392824.4,
  "rounding": 1000000000,
  "recommended_fee": 132000000000,
  "recommended_change": 2000000000,
  "fee_usd": 0.1737593,
  "recommended_fee_usd": 0.17643252
}
{
  "method_name": "eth_address",
  "fee": 1000000000,
  "cycles_balance_before": 11295879421034,
  "cycles_balance_after": 11296824922006,
  "diff": 945500972,
  "typical_cost": 54499028,
  "cost_plus": 76298639.19999999,
  "rounding": 1000000,
  "recommended_fee": 77000000,
  "recommended_change": -923000000,
  "fee_usd": 0.00133661,
  "recommended_fee_usd": 0.00010291897
}
{
  "method_name": "eth_personal_sign",
  "fee": 40000000000,
  "cycles_balance_before": 11296821448293,
  "cycles_balance_after": 11310567310332,
  "diff": 13745862039,
  "typical_cost": 26254137961,
  "cost_plus": 36755793145.399994,
  "rounding": 1000000000,
  "recommended_fee": 37000000000,
  "recommended_change": -3000000000,
  "fee_usd": 0.0534644,
  "recommended_fee_usd": 0.04945457
}
{
  "method_name": "eth_sign_prehash",
  "fee": 40000000000,
  "cycles_balance_before": 11310563868486,
  "cycles_balance_after": 11324279796589,
  "diff": 13715928103,
  "typical_cost": 26284071897,
  "cost_plus": 36797700655.799995,
  "rounding": 1000000000,
  "recommended_fee": 37000000000,
  "recommended_change": -3000000000,
  "fee_usd": 0.0534644,
  "recommended_fee_usd": 0.04945457
}
{
  "method_name": "eth_sign_transaction",
  "fee": 40000000000,
  "cycles_balance_before": 11324276322876,
  "cycles_balance_after": 11337990255349,
  "diff": 13713932473,
  "typical_cost": 26286067527,
  "cost_plus": 36800494537.799995,
  "rounding": 1000000000,
  "recommended_fee": 37000000000,
  "recommended_change": -3000000000,
  "fee_usd": 0.0534644,
  "recommended_fee_usd": 0.04945457
}
{
  "method_name": "generic_caller_ecdsa_public_key",
  "fee": 1000000000,
  "cycles_balance_before": 11337986813503,
  "cycles_balance_after": 11338931963452,
  "diff": 945149949,
  "typical_cost": 54850051,
  "cost_plus": 76790071.39999999,
  "rounding": 1000000,
  "recommended_fee": 77000000,
  "recommended_change": -923000000,
  "fee_usd": 0.00133661,
  "recommended_fee_usd": 0.00010291897
}
{
  "method_name": "generic_sign_with_ecdsa",
  "fee": 40000000000,
  "cycles_balance_before": 11338928521606,
  "cycles_balance_after": 11352719682700,
  "diff": 13791161094,
  "typical_cost": 26208838906,
  "cost_plus": 36692374468.399994,
  "rounding": 1000000000,
  "recommended_fee": 37000000000,
  "recommended_change": -3000000000,
  "fee_usd": 0.0534644,
  "recommended_fee_usd": 0.04945457
}
```

### Conclusion

Ethereum signing prices can be reduced slightly. `btc_caller_address` and `btc_caller_balance` prices need to be increased.

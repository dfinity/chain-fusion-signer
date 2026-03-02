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
OK: Signer balance rose by 18_602_713 for: schnorr_public_key
OK: Signer balance rose by 10_787_751_732 for: schnorr_sign
OK: Signer balance rose by 19_932_548 for: btc_caller_address
OK: Signer balance rose by 29_480_767 for: btc_caller_balance
OK: Signer balance rose by 53_389_816_759 for: btc_caller_sign
OK: Signer balance rose by 37_994_462_267 for: btc_caller_send
OK: Signer balance rose by 19_215_047 for: eth_address
OK: Signer balance rose by 10_747_815_577 for: eth_personal_sign
OK: Signer balance rose by 10_723_063_451 for: eth_sign_prehash
OK: Signer balance rose by 10_721_078_417 for: eth_sign_transaction
OK: Signer balance rose by 18_845_835 for: generic_caller_ecdsa_public_key
OK: Signer balance rose by 10_787_866_426 for: generic_sign_with_ecdsa
```

## Analysis

```
{
  "method_name": "btc_caller_address",
  "fee": 79000000,
  "cycles_balance_before": 12108903984691,
  "cycles_balance_after": 12108923917239,
  "diff": 19932548,
  "typical_cost": 59067452,
  "cost_plus": 82694432.8,
  "rounding": 1000000,
  "recommended_fee": 83000000,
  "recommended_change": 4000000,
  "fee_usd": 0.00010559219,
  "recommended_fee_usd": 0.00011093863
}
{
  "method_name": "btc_caller_balance",
  "fee": 113000000,
  "cycles_balance_before": 12108918832932,
  "cycles_balance_after": 12108948313699,
  "diff": 29480767,
  "typical_cost": 83519233,
  "cost_plus": 116926926.19999999,
  "rounding": 1000000,
  "recommended_fee": 117000000,
  "recommended_change": 4000000,
  "fee_usd": 0.00015103693,
  "recommended_fee_usd": 0.00015638337
}
{
  "method_name": "btc_caller_send",
  "fee": 132000000000,
  "cycles_balance_before": 12162327961844,
  "cycles_balance_after": 12200322424111,
  "diff": 37994462267,
  "typical_cost": 94005537733,
  "cost_plus": 131607752826.2,
  "rounding": 1000000000,
  "recommended_fee": 132000000000,
  "recommended_change": 0,
  "fee_usd": 0.17643252,
  "recommended_fee_usd": 0.17643252
}
{
  "method_name": "btc_caller_sign",
  "fee": 132000000000,
  "cycles_balance_before": 12108943229392,
  "cycles_balance_after": 12162333046151,
  "diff": 53389816759,
  "typical_cost": 78610183241,
  "cost_plus": 110054256537.4,
  "rounding": 1000000000,
  "recommended_fee": 111000000000,
  "recommended_change": -21000000000,
  "fee_usd": 0.17643252,
  "recommended_fee_usd": 0.14836371
}
{
  "method_name": "eth_address",
  "fee": 77000000,
  "cycles_balance_before": 12200317339804,
  "cycles_balance_after": 12200336554851,
  "diff": 19215047,
  "typical_cost": 57784953,
  "cost_plus": 80898934.19999999,
  "rounding": 1000000,
  "recommended_fee": 81000000,
  "recommended_change": 4000000,
  "fee_usd": 0.00010291897,
  "recommended_fee_usd": 0.00010826541000000001
}
{
  "method_name": "eth_personal_sign",
  "fee": 37000000000,
  "cycles_balance_before": 12200331438694,
  "cycles_balance_after": 12211079254271,
  "diff": 10747815577,
  "typical_cost": 26252184423,
  "cost_plus": 36753058192.2,
  "rounding": 1000000000,
  "recommended_fee": 37000000000,
  "recommended_change": 0,
  "fee_usd": 0.04945457,
  "recommended_fee_usd": 0.04945457
}
{
  "method_name": "eth_sign_prehash",
  "fee": 37000000000,
  "cycles_balance_before": 12211074138114,
  "cycles_balance_after": 12221797201565,
  "diff": 10723063451,
  "typical_cost": 26276936549,
  "cost_plus": 36787711168.6,
  "rounding": 1000000000,
  "recommended_fee": 37000000000,
  "recommended_change": 0,
  "fee_usd": 0.04945457,
  "recommended_fee_usd": 0.04945457
}
{
  "method_name": "eth_sign_transaction",
  "fee": 37000000000,
  "cycles_balance_before": 12221792085408,
  "cycles_balance_after": 12232513163825,
  "diff": 10721078417,
  "typical_cost": 26278921583,
  "cost_plus": 36790490216.2,
  "rounding": 1000000000,
  "recommended_fee": 37000000000,
  "recommended_change": 0,
  "fee_usd": 0.04945457,
  "recommended_fee_usd": 0.04945457
}
{
  "method_name": "generic_caller_ecdsa_public_key",
  "fee": 77000000,
  "cycles_balance_before": 12232508047668,
  "cycles_balance_after": 12232526893503,
  "diff": 18845835,
  "typical_cost": 58154165,
  "cost_plus": 81415831,
  "rounding": 1000000,
  "recommended_fee": 82000000,
  "recommended_change": 5000000,
  "fee_usd": 0.00010291897,
  "recommended_fee_usd": 0.00010960202000000001
}
{
  "method_name": "generic_sign_with_ecdsa",
  "fee": 37000000000,
  "cycles_balance_before": 12232521774161,
  "cycles_balance_after": 12243309640587,
  "diff": 10787866426,
  "typical_cost": 26212133574,
  "cost_plus": 36696987003.6,
  "rounding": 1000000000,
  "recommended_fee": 37000000000,
  "recommended_change": 0,
  "fee_usd": 0.04945457,
  "recommended_fee_usd": 0.04945457
}
{
  "method_name": "schnorr_public_key",
  "fee": 77000000,
  "cycles_balance_before": 12098107798860,
  "cycles_balance_after": 12098126401573,
  "diff": 18602713,
  "typical_cost": 58397287,
  "cost_plus": 81756201.8,
  "rounding": 1000000,
  "recommended_fee": 82000000,
  "recommended_change": 5000000,
  "fee_usd": 0.00010291897,
  "recommended_fee_usd": 0.00010960202000000001
}
{
  "method_name": "schnorr_sign",
  "fee": 37000000000,
  "cycles_balance_before": 12098121317266,
  "cycles_balance_after": 12108909068998,
  "diff": 10787751732,
  "typical_cost": 26212248268,
  "cost_plus": 36697147575.2,
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

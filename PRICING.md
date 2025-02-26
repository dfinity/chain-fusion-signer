# Pricing

## Costs

- [How canister pricing works](https://internetcomputer.org/docs/current/developer-docs/gas-cost)
- The [production canister](https://dashboard.internetcomputer.org/canister/grghe-syaaa-aaaar-qabyq-cai) is on a 34 node subnet.
- Management canister signing costs are documented here: https://internetcomputer.org/docs/current/references/t-sigs-how-it-works#api-fees
  - Note: `dfx` pricing is not described.
  - Signing costs depend on the size of the signing subnet, not the size of the canister subnet.  In our case, the canister and signing are both done on a fiduciary subnet, and the subnet sizes are the same.
- Cycle costs are not quite linear in the number of nodes in the subnet, however we will assume that linearity provides an adequate approximation.
- We will assume that an up-to-date dfx has cycle costs equivalent to a single node subnet.
- We will set pricing assuming that messages to be signed are half [the maximum message ingress size](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/maintain/resource-limits/).
- We will include a margin to cover the cost of failed API calls, storage and other fees not covered by API calls.
- Pricing will be verified against canisters on the IC, as a final check that pricing is correct.

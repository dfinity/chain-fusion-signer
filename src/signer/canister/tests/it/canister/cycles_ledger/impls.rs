use candid::Nat;

use super::{Account, ApproveArgs};

impl ApproveArgs {
    /// Simple approval arguments.
    pub fn new(spender: Account, amount: Nat) -> Self {
        Self {
            fee: None,
            memo: None,
            from_subaccount: None,
            created_at_time: None,
            amount,
            expected_allowance: None,
            expires_at: None,
            spender,
        }
    }
}

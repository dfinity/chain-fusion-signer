use crate::utils::{
    cycles_depositor::{self, CyclesDepositorPic},
    cycles_ledger::{
        Account, ApproveArgs, CyclesLedgerPic, InitArgs as LedgerInitArgs, LedgerArgs,
    },
    pic_canister::{cargo_wasm_path, dfx_wasm_path, PicCanisterBuilder, PicCanisterTrait},
    signer::{Arg, InitArg, SignerPic},
};
use candid::{encode_one, CandidType, Nat, Principal};
use ic_papi_api::{cycles::cycles_ledger_canister_id, PaymentError};
use pocket_ic::{PocketIc, PocketIcBuilder};
use std::sync::Arc;

pub const LEDGER_FEE: u128 = 100_000_000; // The documented fee: https://internetcomputer.org/docs/current/developer-docs/defi/cycles/cycles-ledger#fees

/// Methods protected by PAPI.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PaidMethods {
    Cost1bIcrc2Cycles,
    CallerPays1bIcrc2Tokens,
    Cost1b,
}
impl PaidMethods {
    pub fn name(&self) -> &str {
        match self {
            Self::Cost1bIcrc2Cycles => "caller_pays_1b_icrc2_cycles",
            Self::CallerPays1bIcrc2Tokens => "caller_pays_1b_icrc2_tokens",
            Self::Cost1b => "cost_1b",
        }
    }
    pub fn cost(&self) -> u128 {
        match self {
            Self::Cost1bIcrc2Cycles => 1_000_000_000,
            Self::CallerPays1bIcrc2Tokens => 1_000_000_000,
            Self::Cost1b => 1_000_000_000,
        }
    }
}

pub struct TestSetup {
    /// The PocketIC instance.
    #[allow(dead_code)]
    // The Arc is used; this makes it accessible without having to refer to a specific canister.
    pub pic: Arc<PocketIc>,
    /// The canister providing the API.
    pub signer: SignerPic,
    /// ICRC2 ledger
    pub ledger: CyclesLedgerPic,
    /// User
    pub user: Principal,
    /// Another user
    pub user2: Principal,
    /// A crowd
    pub users: [Principal; 5],
    /// Unauthorized user
    pub unauthorized_user: Principal,
    /// A canister used to deposit cycles into the ledger.
    pub cycles_depositor: CyclesDepositorPic,
}
impl Default for TestSetup {
    fn default() -> Self {
        let pic = Arc::new(
            PocketIcBuilder::new()
                .with_fiduciary_subnet()
                .with_system_subnet()
                .with_application_subnet()
                .with_ii_subnet()
                .with_nns_subnet()
                .build(),
        );
        let cycles_ledger_canister_id = pic
            .create_canister_with_id(None, None, cycles_ledger_canister_id())
            .unwrap();

        // Would like to create this with the cycles ledger canister ID but currently this yields an error.
        let ledger = CyclesLedgerPic::from(
            PicCanisterBuilder::default()
                .with_canister(cycles_ledger_canister_id)
                .with_wasm(&dfx_wasm_path("cycles_ledger"))
                .with_arg(
                    encode_one(LedgerArgs::Init(LedgerInitArgs {
                        index_id: None,
                        max_blocks_per_request: 999,
                    }))
                    .expect("Failed to encode ledger init arg"),
                )
                .deploy_to(pic.clone()),
        );
        let signer = SignerPic::from(
            PicCanisterBuilder::default()
                .with_wasm(&cargo_wasm_path("signer"))
                .with_arg(
                    encode_one(Arg::Init(InitArg {
                        ecdsa_key_name: format!("test_key_1"),
                        ic_root_key_der: None,
                        cycles_ledger: None,
                    }))
                    .unwrap(),
                )
                .deploy_to(pic.clone()),
        );
        let user =
            Principal::from_text("xzg7k-thc6c-idntg-knmtz-2fbhh-utt3e-snqw6-5xph3-54pbp-7axl5-tae")
                .unwrap();
        let user2 =
            Principal::from_text("jwhyn-xieqy-drmun-h7uci-jzycw-vnqhj-s62vl-4upsg-cmub3-vakaq-rqe")
                .unwrap();
        let users = [
            Principal::from_text("s2xin-cwqnw-sjvht-gp553-an54g-2rhlc-z4c5d-xz5iq-irnbi-sadik-qae")
                .unwrap(),
            Principal::from_text("dmvof-2tilt-3xmvh-c7tbj-n3whk-k2i6b-2s2ge-xoo3d-wjuw3-ijpuw-eae")
                .unwrap(),
            Principal::from_text("kjerd-nj73t-u3hhp-jcj4d-g7w56-qlrvb-gguta-45yve-336zs-sunxa-zqe")
                .unwrap(),
            Principal::from_text("zxhav-yshtx-vhzs2-nvuu3-jrq66-bidn2-put3y-ulwcf-2gb2o-ykfco-sae")
                .unwrap(),
            Principal::from_text("nggqm-p5ozz-i5hfv-bejmq-2gtow-4dtqw-vjatn-4b4yw-s5mzs-i46su-6ae")
                .unwrap(),
        ];
        let unauthorized_user =
            Principal::from_text("rg3gz-22tjp-jh7hl-migkq-vb7in-i2ylc-6umlc-dtbug-v6jgc-uo24d-nqe")
                .unwrap();
        let cycles_depositor = PicCanisterBuilder::default()
            .with_wasm(&dfx_wasm_path("cycles_depositor"))
            .with_controllers(vec![user])
            .with_arg(
                encode_one(cycles_depositor::InitArg {
                    ledger_id: ledger.canister_id,
                })
                .unwrap(),
            )
            .deploy_to(pic.clone())
            .into();

        let ans = Self {
            pic,
            signer,
            ledger,
            user,
            user2,
            users,
            unauthorized_user,
            cycles_depositor,
        };
        ans.fund_user(Self::USER_INITIAL_BALANCE);
        ans
    }
}
impl TestSetup {
    /// The user's initial balance.
    pub const USER_INITIAL_BALANCE: u128 = 100_000_000_000;
    /// Deposit 100 * the ledger fee in the user's ledger wallet. That should be enough to be getting on with.
    pub fn fund_user(&self, megasquigs: u128) {
        let initial_balance = self.user_balance();
        // .. Magic cycles into existence (test only - not IRL).
        let deposit = megasquigs + LEDGER_FEE;
        self.pic
            .add_cycles(self.cycles_depositor.canister_id, deposit);
        // .. Send cycles to the cycles ledger.
        self.cycles_depositor
            .deposit(
                self.user,
                &cycles_depositor::DepositArg {
                    to: cycles_depositor::Account {
                        owner: self.user,
                        subaccount: None,
                    },
                    memo: None,
                    cycles: candid::Nat::from(deposit),
                },
            )
            .expect("Failed to deposit funds in the ledger");
        // .. That should have cost one fee.
        let expected_balance = initial_balance.clone() + megasquigs;
        self.assert_user_balance_eq(expected_balance.clone(), format!("Expected user balance to be the initial balance ({initial_balance}) plus the requested sum ({megasquigs}) = {expected_balance}"));
    }
    /// Gets the user balance
    pub fn user_balance(&self) -> Nat {
        self.ledger
            .icrc_1_balance_of(
                self.user,
                &Account {
                    owner: self.user,
                    subaccount: None,
                },
            )
            .expect("Could not get user balance")
    }
    /// Asserts that the user's ledger balance is a certain value.
    pub fn assert_user_balance_eq<T>(&self, expected_balance: T, message: String)
    where
        T: Into<Nat>,
    {
        assert_eq!(self.user_balance(), expected_balance.into(), "{}", message);
    }
    /// User sends an ICRC2 approval with the paid service as spender.
    pub fn user_approves_payment_for_paid_service<T>(&self, amount: T)
    where
        T: Into<Nat>,
    {
        self.ledger
            .icrc_2_approve(
                self.user,
                &ApproveArgs {
                    spender: Account {
                        owner: self.signer.canister_id(),
                        subaccount: None,
                    },
                    amount: amount.into(),
                    ..ApproveArgs::default()
                },
            )
            .expect("Failed to call the ledger to approve")
            .expect("Failed to approve the paid service to spend the user's ICRC-2 tokens");
    }
    /// Calls a paid service.
    pub fn call_paid_service(
        &self,
        caller: Principal,
        method: PaidMethods,
        arg: impl CandidType,
    ) -> Result<String, PaymentError> {
        self.signer
            .update(caller, method.name(), arg)
            .expect("Failed to call the paid service")
    }
}

#[test]
fn icrc2_test_setup_works() {
    let _setup = TestSetup::default();
}

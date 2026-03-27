use candid::{CandidType, Deserialize};

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum GenericSigningError {
    /// Payment failed.
    PaymentError(ic_papi_api::PaymentError),
    /// An inter-canister call error from the threshold signature API.
    SigningError(String),
}

impl From<ic_papi_api::PaymentError> for GenericSigningError {
    fn from(e: ic_papi_api::PaymentError) -> Self {
        Self::PaymentError(e)
    }
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum GenericCallerEcdsaPublicKeyError {
    /// Payment failed.
    PaymentError(ic_papi_api::PaymentError),
    /// An inter-canister call error from the threshold signature API.
    SigningError(String),
}
impl From<ic_papi_api::PaymentError> for GenericCallerEcdsaPublicKeyError {
    fn from(e: ic_papi_api::PaymentError) -> Self {
        Self::PaymentError(e)
    }
}
impl From<ic_cdk::call::Error> for GenericCallerEcdsaPublicKeyError {
    fn from(e: ic_cdk::call::Error) -> Self {
        Self::SigningError(e.to_string())
    }
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum GenericSignWithEcdsaError {
    /// Payment failed.
    PaymentError(ic_papi_api::PaymentError),
    /// An inter-canister call error from the threshold signature API.
    SigningError(String),
}
impl From<ic_papi_api::PaymentError> for GenericSignWithEcdsaError {
    fn from(e: ic_papi_api::PaymentError) -> Self {
        Self::PaymentError(e)
    }
}
impl From<ic_cdk_management_canister::SignCallError> for GenericSignWithEcdsaError {
    fn from(e: ic_cdk_management_canister::SignCallError) -> Self {
        Self::SigningError(e.to_string())
    }
}

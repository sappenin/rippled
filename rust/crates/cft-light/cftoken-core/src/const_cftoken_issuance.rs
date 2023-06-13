use xrpl_rust_sdk_core::core::types::{AccountId, Currency, Hash160, Hash256};
use plugin_transactor::{ConstSLE, SField, SLE};
use plugin_transactor::transactor::{ConstLedgerObject, LedgerObject};
use rippled_bridge::Keylet;
use crate::{CFTokenFields};

pub struct ConstCFTokenIssuance {
    sle: ConstSLE,
}

impl From<ConstSLE> for ConstCFTokenIssuance {
    fn from(value: ConstSLE) -> Self {
        Self { sle: value }
    }
}

impl ConstLedgerObject for ConstCFTokenIssuance {
    fn get_sle(&self) -> &ConstSLE {
        &self.sle
    }
}

impl ConstCFTokenIssuance {
    pub fn is_frozen(&self) -> bool {
        // TODO: Once we implement issuance freezing, check if flag is set
        false
    }

    pub fn issuer(&self) -> AccountId {
        self.sle.get_account_id(&SField::sf_issuer())
    }
}
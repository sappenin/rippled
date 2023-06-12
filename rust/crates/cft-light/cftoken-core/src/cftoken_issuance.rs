use std::str::FromStr;
use xrpl_rust_sdk_core::core::crypto::ToFromBase58;
use xrpl_rust_sdk_core::core::types::{AccountId, Currency, Hash160, Hash256};
use plugin_transactor::{SField, SLE};
use plugin_transactor::transactor::{LedgerObject};
use rippled_bridge::Keylet;
use crate::{CFTokenFields};

pub const CFT_ISSUANCE_TYPE: u16 = 0x007Eu16;

pub type CFTokenIssuanceID = Hash256;

pub struct CFTokenIssuance {
    sle: SLE,
}

impl From<SLE> for CFTokenIssuance {
    fn from(value: SLE) -> Self {
        Self { sle: value }
    }
}

impl LedgerObject for CFTokenIssuance {
    fn get_sle(&self) -> &SLE {
        &self.sle
    }
}

impl CFTokenIssuance {
    pub fn new(keylet: &Keylet) -> CFTokenIssuance {
        CFTokenIssuance { sle: SLE::from(keylet) }
    }

    pub fn issuer(&self) -> AccountId {
        self.sle.get_account_id(&SField::sf_issuer())
    }

    pub fn outstanding_amount(&self) -> u64 {
        self.sle.get_field_uint64(&SField::sf_outstanding_amount())
    }
    pub fn set_transfer_fee(mut self, fee: u16) -> Self {
        if fee != 0 {
            self.sle.set_field_u16(&SField::sf_transfer_fee(), fee);
        }
        self
    }

    pub fn set_flags(mut self, flags: u32) -> Self {
        self.sle.set_field_u32(&SField::sf_flags(), flags);
        self
    }

    pub fn set_maximum_amount(mut self, maximum_amount: u64) -> Self {
        self.sle.set_field_u64(&SField::sf_maximum_amount(), maximum_amount);
        self
    }

    pub fn set_outstanding_amount(mut self, amount: u64) -> Self {
        self.sle.set_field_u64(&SField::sf_outstanding_amount(), amount);
        self
    }

    pub fn set_locked_amount(mut self, amount: u64) -> Self {
        self.sle.set_field_u64(&SField::sf_locked_amount(), amount);
        self
    }

    pub fn set_owner_node(mut self, owner_node: u64) -> Self {
        self.sle.set_field_u64(&SField::sf_owner_node(), owner_node);
        self
    }

    pub fn set_cft_metadata(mut self, metadata: &[u8]) -> Self {
        self.sle.set_field_blob2(&SField::sf_cft_metadata(), metadata);
        self
    }

    pub fn set_issuer(mut self, issuer: &AccountId) -> Self {
        self.sle.set_field_account(&SField::sf_issuer(), issuer);
        self
    }

    pub fn set_asset_scale(mut self, scale: u8) -> Self {
        self.sle.set_field_u8(&SField::sf_asset_scale(), scale);
        self
    }

    pub fn set_asset_code(mut self, code: &Hash160) -> Self {
        self.sle.set_field_h160(&SField::sf_asset_code(), code);
        self
    }

    pub fn is_frozen(&self) -> bool {
        // TODO: Once we implement issuance freezing, check if flag is set
        false
    }
}

pub fn keylet(issuer: &AccountId, asset_code: &Hash160) -> Keylet {
    Keylet::builder(CFT_ISSUANCE_TYPE as i16, CFT_ISSUANCE_TYPE)
        .key(issuer)
        .key(asset_code)
        .build()
}

pub fn keylet_from_currency(issuer: &AccountId, asset_code: &Currency) -> Keylet {
    Keylet::builder(CFT_ISSUANCE_TYPE as i16, CFT_ISSUANCE_TYPE)
        .key(issuer)
        .key(asset_code)
        .build()
}

#[test]
fn keylet_and_keylet_from_currency_same() {
    let k1 = keylet(&AccountId::from_base58("rMZHjgNPYHEXkQNMHrMye6qYzVMeTWo4tg").unwrap(), &Hash160::try_from_hex("0000000000000000000000005553440000000000").unwrap());
    let k2 = keylet_from_currency(&AccountId::from_base58("rMZHjgNPYHEXkQNMHrMye6qYzVMeTWo4tg").unwrap(), &Currency::try_from(hex::decode("0000000000000000000000005553440000000000").unwrap().as_slice()).unwrap());

    assert_eq!(k1, k2)
}
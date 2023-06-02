use xrpl_rust_sdk_core::core::types::{AccountId, Hash160};
use plugin_transactor::{SField, SLE};
use plugin_transactor::transactor::{LedgerObject};
use rippled_bridge::Keylet;

const CFT_ISSUANCE_TYPE: u16 = 0x007Eu16;

pub struct CFTokenIssuance {
    sle: SLE,
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
        self.sle.set_field_u64(&SField::get_plugin_field(3, 20), maximum_amount);
        self
    }

    pub fn set_outstanding_amount(mut self, amount: u64) -> Self {
        self.sle.set_field_u64(&SField::get_plugin_field(3, 21), amount);
        self
    }

    pub fn set_locked_amount(mut self, amount: u64) -> Self {
        self.sle.set_field_u64(&SField::get_plugin_field(3, 22), amount);
        self
    }

    pub fn set_owner_node(mut self, owner_node: u64) -> Self {
        self.sle.set_field_u64(&SField::sf_owner_node(), owner_node);
        self
    }

    pub fn set_cft_metadata(mut self, metadata: &[u8]) -> Self {
        self.sle.set_field_blob2(&SField::get_plugin_field(7, 22), metadata);
        self
    }

    pub fn set_issuer(mut self, issuer: &AccountId) -> Self {
        self.sle.set_field_account(&SField::sf_issuer(), issuer);
        self
    }

    pub fn set_asset_scale(mut self, scale: u8) -> Self {
        self.sle.set_field_u8(&SField::get_plugin_field(16, 19), scale);
        self
    }

    pub fn set_asset_code(mut self, code: &Hash160) -> Self {
        self.sle.set_field_h160(&SField::get_plugin_field(17, 5), code);
        self
    }
}

pub fn keylet(issuer: &AccountId, asset_code: &Hash160) -> Keylet {
    Keylet::builder(CFT_ISSUANCE_TYPE as i16, CFT_ISSUANCE_TYPE)
        .key(issuer)
        .key(asset_code)
        .build()
}
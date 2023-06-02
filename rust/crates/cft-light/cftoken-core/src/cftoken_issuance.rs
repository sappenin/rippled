use xrpl_rust_sdk_core::core::types::{AccountId, Hash160};
use plugin_transactor::{SField, SLE};
use plugin_transactor::transactor::WriteToSle;
use rippled_bridge::Keylet;

pub const CFT_ISSUANCE_TYPE: u16 = 0x007Eu16;

pub struct CFTokenIssuance<'a> {
    pub transfer_fee: Option<u16>,
    pub flags: u32,
    pub maximum_amount: u64,
    pub outstanding_amount: Option<u64>,
    pub locked_amount: Option<u64>,
    pub owner_node: Option<u64>,
    pub cft_metadata: Option<&'a [u8]>,
    pub issuer: AccountId,
    pub asset_scale: u8,
    pub asset_code: Hash160,
}

impl WriteToSle for CFTokenIssuance<'_> {
    fn write_to_sle(&self, sle: &mut SLE) {
        sle.set_field_u32(&SField::sf_flags(), self.flags); // sfFlags
        sle.set_field_account(&SField::sf_issuer(), &self.issuer); // sfIssuer
        sle.set_field_h160(&SField::get_plugin_field(17, 5), &self.asset_code); // sfAssetCode
        sle.set_field_u8(&SField::get_plugin_field(16, 19), self.asset_scale); // sfAssetScale
        sle.set_field_u64(&SField::get_plugin_field(3, 20), self.maximum_amount); // sfMaximumAmount

        if let Some(tf) = self.transfer_fee {
            sle.set_field_u16(&SField::sf_transfer_fee(), tf);
        }

        if let Some(meta) = self.cft_metadata {
            sle.set_field_blob2(&SField::get_plugin_field(7, 22), meta);
        }
    }
}

pub fn keylet(issuer: &AccountId, asset_code: &Hash160) -> Keylet {
    Keylet::builder(CFT_ISSUANCE_TYPE as i16, CFT_ISSUANCE_TYPE)
        .key(issuer)
        .key(asset_code)
        .build()
}
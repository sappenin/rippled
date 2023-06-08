use xrpl_rust_sdk_core::core::types::{AccountId, Currency, Hash160};
use plugin_transactor::STAmount;

pub struct CFTAmount {
    value: u64,
    issuer: AccountId,
    asset_code: Currency
}

impl<'a> TryFrom<STAmount<'a>> for CFTAmount {
    type Error = ();

    fn try_from(st_amount: STAmount<'a>) -> Result<Self, Self::Error> {
        if st_amount.is_cft() {
            let value = st_amount.mantissa();
            let issuer = st_amount.issuer();
            let asset_code = st_amount.currency();
            Ok(CFTAmount {
                value,
                issuer,
                asset_code
            })
        } else {
            Err(())
        }
    }
}

impl CFTAmount {
    pub fn value(&self) -> u64 {
        self.value
    }

    pub fn issuer(&self) -> &AccountId {
        &self.issuer
    }

    pub fn asset_code(&self) -> &Currency {
        &self.asset_code
    }
}
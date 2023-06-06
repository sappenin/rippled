use std::hash::Hash;
use xrpl_rust_sdk_core::core::types::Hash256;
use plugin_transactor::{SField, STObject};
use crate::cftoken_issuance::CFTokenIssuanceID;
use crate::CFTokenFields;

pub type CFTokenID = CFTokenIssuanceID;

pub struct CFToken<'a> {
    pub(crate) inner: &'a STObject<'a>,
    id: CFTokenID
}

impl <'a>AsRef<STObject<'a>> for CFToken<'a> {
    fn as_ref(&self) -> &STObject<'a> {
        self.inner
    }
}

impl CFToken<'_> {
    pub fn issuance_id(&self) -> CFTokenIssuanceID {
        self.inner.get_h256(&SField::sf_issuance_id())
    }

    pub fn token_id(&self) -> CFTokenID {
        self.issuance_id()
    }

    pub fn low_64(&self) -> &[u8] {
        &self.id.as_ref()[24..32]
    }

    pub fn amount(&self) -> u64 {
        self.inner.get_uint64(&SField::sf_cft_amount())
    }

    pub fn locked_amount(&self) -> u64 {
        self.inner.get_uint64(&SField::sf_locked_amount())
    }

    pub fn flags(&self) -> u32 {
        self.inner.get_uint32(&SField::sf_flags())
    }
}
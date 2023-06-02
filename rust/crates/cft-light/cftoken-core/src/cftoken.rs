use std::hash::Hash;
use xrpl_rust_sdk_core::core::types::Hash256;
use plugin_transactor::{SField, STObject};
use crate::cftoken_issuance::CFTokenIssuanceID;
use crate::CFTokenFields;

pub type CFTokenID = CFTokenIssuanceID;

pub struct CFToken<'a> {
    inner: &'a STObject<'a>,
}

impl CFToken<'_> {
    pub fn issuance_id(&self) -> Hash256 {
        self.inner.get_h256(&SField::sf_issuance_id())
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
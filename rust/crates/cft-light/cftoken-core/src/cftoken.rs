use std::hash::Hash;
use xrpl_rust_sdk_core::core::types::Hash256;
use plugin_transactor::{SField, STObject};

pub struct CFToken<'a> {
    inner: &'a STObject<'a>,
}

impl CFToken<'_> {
    pub fn issuance_id(&self) -> Hash256 {
        self.inner.get_h256(&SField::get_plugin_field(5, 28))
    }

    pub fn amount(&self) -> u64 {
        self.inner.get_uint64(&SField::get_plugin_field(3, 23))
    }

    pub fn locked_amount(&self) -> u64 {
        self.inner.get_uint64(&SField::get_plugin_field(3, 22))
    }

    pub fn flags(&self) -> u32 {
        self.inner.get_uint32(&SField::sf_flags())
    }
}
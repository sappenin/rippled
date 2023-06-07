use std::cmp::Ordering;
use std::hash::Hash;
use xrpl_rust_sdk_core::core::types::Hash256;
use plugin_transactor::{SField, STObject};
use crate::cftoken_issuance::CFTokenIssuanceID;
use crate::cftoken_page::CFTokens;
use crate::CFTokenFields;

pub type CFTokenID = CFTokenIssuanceID;

pub struct CFToken<'a> {
    pub(crate) inner: STObject<'a>,
    id: CFTokenID
}

impl <'a> Eq for CFToken<'a> {}

impl<'a> Ord for CFToken<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        // The sort of CFTokens needs to be fully deterministic, but the sort
        // is weird because we sort on the low 64-bits first. But if the low
        // 64-bits are identical we still need a fully deterministic sort.
        // So we sort on the low 64-bits first. If those are equal we sort on
        // the whole thing.
        match self.low_64().cmp(other.low_64()) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.issuance_id().cmp(&other.issuance_id()),
            Ordering::Greater => Ordering::Greater
        }
    }
}

impl<'a> PartialEq<Self> for CFToken<'a> {
    fn eq(&self, other: &Self) -> bool {
        match self.cmp(other) {
            Ordering::Equal => true,
            _ => false,
        }
    }
}

impl<'a> PartialOrd<Self> for CFToken<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl <'a>AsRef<STObject<'a>> for CFToken<'a> {
    fn as_ref(&self) -> &STObject<'a> {
        &self.inner
    }
}

impl <'a> CFToken<'a> {
    pub fn new(inner: STObject<'a>) -> CFToken<'a> {
        let id = inner.get_h256(&SField::sf_issuance_id());
        CFToken { inner, id }
    }

    pub fn new_with_mem_leak() -> CFToken<'a> {
        CFToken::new(STObject::new_inner(SField::sf_cf_token()))
    }

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
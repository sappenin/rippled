use std::cmp::Ordering;
use std::hash::Hash;
use xrpl_rust_sdk_core::core::types::Hash256;
use plugin_transactor::{SField, ConstSTObject, STObject};
use crate::cftoken_issuance::CFTokenIssuanceID;
use crate::cftoken_page::CFTokens;
use crate::CFTokenFields;

pub type CFTokenID = CFTokenIssuanceID;

pub struct ConstCFToken<'a> {
    pub(crate) inner: ConstSTObject<'a>,
    id: CFTokenID
}

impl <'a> Eq for ConstCFToken<'a> {}

impl<'a> Ord for ConstCFToken<'a> {
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

impl<'a> PartialEq<Self> for ConstCFToken<'a> {
    fn eq(&self, other: &Self) -> bool {
        match self.cmp(other) {
            Ordering::Equal => true,
            _ => false,
        }
    }
}

impl<'a> PartialOrd<Self> for ConstCFToken<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl <'a>AsRef<ConstSTObject<'a>> for ConstCFToken<'a> {
    fn as_ref(&self) -> &ConstSTObject<'a> {
        &self.inner
    }
}

impl <'a> ConstCFToken<'a> {
    pub fn new(inner: ConstSTObject<'a>) -> ConstCFToken<'a> {
        let id = inner.get_h256(&SField::sf_issuance_id());
        ConstCFToken { inner, id }
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


///////// Mutable variant
pub struct CFToken {
    pub(crate) inner: STObject,
    id: CFTokenID
}

impl Eq for CFToken {}

impl Ord for CFToken {
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

impl PartialEq<Self> for CFToken {
    fn eq(&self, other: &Self) -> bool {
        match self.cmp(other) {
            Ordering::Equal => true,
            _ => false,
        }
    }
}

impl PartialOrd<Self> for CFToken {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl AsRef<STObject> for CFToken {
    fn as_ref(&self) -> &STObject {
        &self.inner
    }
}

impl From<STObject> for CFToken {
    fn from(value: STObject) -> Self {
        let id = value.get_field_h256(&SField::sf_issuance_id());
        CFToken { inner: value, id }
    }
}
impl CFToken {
    pub fn new() -> CFToken {
        CFToken::from(STObject::new_inner(SField::sf_cf_token()))
    }

    pub fn issuance_id(&self) -> CFTokenIssuanceID {
        self.inner.get_field_h256(&SField::sf_issuance_id())
    }

    pub fn token_id(&self) -> CFTokenID {
        self.issuance_id()
    }

    pub fn low_64(&self) -> &[u8] {
        &self.id.as_ref()[24..32]
    }

    pub fn amount(&self) -> u64 {
        self.inner.get_field_uint64(&SField::sf_cft_amount())
    }

    pub fn locked_amount(&self) -> u64 {
        self.inner.get_field_uint64(&SField::sf_locked_amount())
    }

    pub fn flags(&self) -> u32 {
        self.inner.get_field_uint32(&SField::sf_flags())
    }

    pub fn set_issuance_id(&mut self, id: CFTokenIssuanceID) {
        self.id = id;
        self.inner.set_field_h256(&SField::sf_issuance_id(), &self.id);
    }

    pub fn set_amount(&mut self, amount: u64) {
        self.inner.set_field_u64(&SField::sf_cft_amount(), amount);
    }

    pub fn set_locked_amount(&mut self, locked_amount: u64) {
        self.inner.set_field_u64(&SField::sf_locked_amount(), locked_amount);
    }

    pub fn set_flags(&mut self, flags: u32) {
        self.inner.set_field_u32(&SField::sf_flags(), flags);
    }
}


use std::pin::Pin;
use std::slice::Iter;
use xrpl_rust_sdk_core::core::types::Hash256;
use plugin_transactor::{SField, SLE, STArray};
use plugin_transactor::transactor::LedgerObject;
use rippled_bridge::Keylet;
use crate::cftoken::CFToken;

pub const CFTOKEN_PAGE_TYPE: u16 = 0x0033;

pub type CFTokenPageID = Hash256;

pub struct CFTokenPage {
    sle: SLE
}

impl From<SLE> for CFTokenPage {
    fn from(value: SLE) -> Self {
        Self { sle: value }
    }
}

impl LedgerObject for CFTokenPage {
    fn get_sle(&self) -> &SLE {
        &self.sle
    }
}

impl CFTokenPage {
    pub fn new(sle: SLE) -> Self {
        CFTokenPage { sle }
    }

    pub fn get_tokens<'a>(&self) -> CFTokens<'a> {
        // self.sle.get_field_array(&SField::sf_cftokens())
        todo!()
    }

    pub fn get_previous_page_min(&self) -> Option<CFTokenPageID> {
        todo!()
    }

    pub fn set_tokens(&mut self, tokens: CFTokens) {
        // self.sle.set_field_array
        todo!()
    }

    pub fn set_next_page_min(&mut self, id: &CFTokenPageID) {
        // self.sle.set_field_h256(id)
        todo!()
    }

    pub fn set_previous_page_min(&mut self, id: &CFTokenPageID) {
        todo!()
    }
}

impl From<&Keylet> for CFTokenPage {
    fn from(value: &Keylet) -> Self {
        CFTokenPage::new(SLE::from(value))
    }
}

pub struct CFTokenIter<'a>(CFTokens<'a>);

impl <'a>Iterator for CFTokenIter<'a> {
    type Item = CFToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

pub struct CFTokens<'a> {
    pub(crate) tokens: Vec<CFToken<'a>>
}

impl <'a> CFTokens<'a> {

    pub fn new(tokens: Vec<CFToken<'a>>) -> Self {
        Self { tokens }
    }

    pub fn new_empty() -> Self {
        CFTokens {
            tokens: vec![]
        }
    }

    pub fn size(&self) -> usize {
        todo!()
    }

    pub fn insert_sorted(&self, cftoken: CFToken) {
        todo!()
    }

    pub fn get(&self, index: usize) -> Option<CFToken> {
        todo!()
    }

    pub fn to_st_array(&self) -> STArray {
        let mut array = STArray::new_empty();
        self.tokens
            .iter()
            .for_each(|token| array.push_back(token));
        array
    }

    pub fn iter(&self) -> Iter<'_, CFToken<'a>> {
        self.tokens.iter()
    }

    pub fn push_back(&'a mut self, cf_token: CFToken<'a>) {
        self.tokens.push(cf_token);
    }
}

pub mod keylet {
    use xrpl_rust_sdk_core::core::types::AccountId;
    use rippled_bridge::Keylet;
    use crate::cftoken::CFTokenID;

    pub fn cftpage_min(owner: &AccountId) -> Keylet {
        todo!()
    }

    pub fn cftpage(base: &Keylet, cftoken_id: &CFTokenID) -> Keylet {
        todo!()
    }

    pub fn cftpage_max(owner: &AccountId) -> Keylet {
        todo!()
    }
}
use std::slice::Iter;
use plugin_transactor::{ConstSLE, ConstSTArray, SField};
use plugin_transactor::transactor::ConstLedgerObject;
use crate::cftoken::ConstCFToken;
use crate::cftoken_page::{CFTokenPageID, CFTokens};
use crate::CFTokenFields;

pub struct ConstCFTokenPage<'a> {
    sle: ConstSLE<'a>
}

impl<'a> ConstLedgerObject<'a> for ConstCFTokenPage<'a> {
    fn get_sle(&self) -> &ConstSLE {
        &self.sle
    }

    fn from(sle: ConstSLE<'a>) -> ConstCFTokenPage<'a> {
        ConstCFTokenPage { sle }
    }
}

impl<'a> ConstCFTokenPage<'a> {
    pub fn new(sle: ConstSLE<'a>) -> Self {
        ConstCFTokenPage { sle }
    }

    pub fn get_tokens(&'a self) -> ConstCFTokens<'a> {
        let st_array = self.sle.get_field_array(&SField::sf_cf_tokens());
        let mut tokens = vec![];
        for i in 0..st_array.size() {
            tokens.push(ConstCFToken::new(st_array.get(i).unwrap()));
        }

        ConstCFTokens::new(tokens)
    }

    pub fn get_previous_page_min(&self) -> Option<CFTokenPageID> {
        if self.sle.is_field_present(&SField::sf_previous_page_min()) {
            Some(self.sle.get_field_h256(&SField::sf_previous_page_min()))
        } else {
            None
        }
    }
}

pub struct ConstCFTokens<'a> {
    pub(crate) tokens: Vec<ConstCFToken<'a>>
}

impl <'a>ConstCFTokens<'a> {

    pub fn new(tokens: Vec<ConstCFToken<'a>>) -> Self {
        Self { tokens }
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn get(&self, index: usize) -> Option<&ConstCFToken<'a>> {
        self.tokens.get(index)
    }

    pub fn iter(&self) -> Iter<'_, ConstCFToken<'a>> {
        self.tokens.iter()
    }

}
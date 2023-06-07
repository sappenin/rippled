use plugin_transactor::{ConstSLE, SField};
use plugin_transactor::transactor::ConstLedgerObject;
use crate::cftoken::CFToken;
use crate::cftoken_page::{CFTokenPageID, CFTokens};
use crate::CFTokenFields;

pub struct ConstCFTokenPage {
    sle: ConstSLE
}

impl ConstLedgerObject for ConstCFTokenPage {
    fn get_sle(&self) -> &ConstSLE {
        &self.sle
    }
}

impl From<ConstSLE> for ConstCFTokenPage {
    fn from(value: ConstSLE) -> Self {
        Self { sle: value }
    }
}

impl ConstCFTokenPage {
    pub fn new(sle: ConstSLE) -> Self {
        ConstCFTokenPage { sle }
    }

    pub fn get_tokens<'a>(&self) -> CFTokens<'a> {
        let st_array = self.sle.get_field_array(&SField::sf_cf_tokens());
        let mut tokens = vec![];
        for i in 0..st_array.size() {
            tokens.push(CFToken::new(st_array.get(i).unwrap()));
        }

        CFTokens::new(tokens)
    }

    pub fn get_previous_page_min(&self) -> Option<CFTokenPageID> {
        if self.sle.is_field_present(&SField::sf_previous_page_min()) {
            Some(self.sle.get_field_h256(&SField::sf_previous_page_min()))
        } else {
            None
        }
    }
}
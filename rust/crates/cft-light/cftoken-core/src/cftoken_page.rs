use std::pin::Pin;
use std::slice::Iter;
use xrpl_rust_sdk_core::core::types::Hash256;
use plugin_transactor::{SField, SLE, STArray};
use plugin_transactor::transactor::LedgerObject;
use rippled_bridge::Keylet;
use crate::cftoken::{CFToken, ConstCFToken};
use crate::CFTokenFields;

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
        let mut st_array = self.sle.peek_field_array(&SField::sf_cf_tokens());
        let mut tokens = vec![];
        for i in 0..st_array.size() {
            tokens.push(CFToken::from(st_array.get(i).unwrap()));
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

    pub fn set_tokens(&mut self, tokens: CFTokens) {
        self.sle.set_field_array(&SField::sf_cf_tokens(), tokens.to_st_array());
    }

    pub fn set_next_page_min(&mut self, id: &CFTokenPageID) {
        self.sle.set_field_h256(&SField::sf_next_page_min(), id);
    }

    pub fn set_previous_page_min(&mut self, id: &CFTokenPageID) {
        self.sle.set_field_h256(&SField::sf_previous_page_min(), id);
    }
}

impl From<&Keylet> for CFTokenPage {
    fn from(value: &Keylet) -> Self {
        CFTokenPage::new(SLE::from(value))
    }
}

pub struct CFTokens<'a> {
    pub(crate) tokens: Vec<CFToken<'a>>
}

impl<'a> CFTokens<'a> {

    pub fn new(tokens: Vec<CFToken<'a>>) -> CFTokens<'a> {
        Self { tokens }
    }

    pub fn new_empty() -> Self {
        CFTokens {
            tokens: vec![]
        }
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn insert_sorted(&mut self, cftoken: CFToken<'a>) {
        match self.tokens.binary_search(&cftoken) {
            Ok(_) => {}
            Err(index) => self.tokens.insert(index, cftoken)
        };

    }

    pub fn get(&self, index: usize) -> Option<&CFToken<'a>> {
        self.tokens.get(index)
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

    pub fn push_back(&mut self, cf_token: CFToken<'a>) {
        self.tokens.push(cf_token);
    }
}

pub mod keylet {
    use std::any::Any;
    use xrpl_rust_sdk_core::core::types::AccountId;
    use rippled_bridge::{Keylet, UInt256};
    use crate::cftoken::CFTokenID;
    use crate::cftoken_issuance::CFT_ISSUANCE_TYPE;
    use crate::cftoken_page::CFTOKEN_PAGE_TYPE;
    use crate::cftoken_utils::PAGE_MASK;

    pub fn cftpage_min(owner: &AccountId) -> Keylet {
        let mut key_arr = [0; 32];
        // We construct a Keylet so we can get the SHA512-Half of 0x007E concatenated with the owner
        // AccountId so we can set the high 192 bits of the page keylet to the high 192 bits
        // of holder_id_keylet's key
        let holder_id_keylet = get_holder_id(owner);
        key_arr[..24].copy_from_slice(&holder_id_keylet.key.as_ref()[..24]);
        Keylet::new(CFTOKEN_PAGE_TYPE as i16, UInt256::new(key_arr))
    }

    pub fn cftpage(base: &Keylet, cftoken_id: &CFTokenID) -> Keylet {
        if base.r#type != CFTOKEN_PAGE_TYPE as i16 {
            panic!("Attempted to construct cftpage Keylet based on non-CFTPage base keylet.");
        }

        // The C++ code for NFTs does this: (k.key & ~nft::pageMask) + (token & nft::pageMask)
        // (k.key & ~nft::pageMask) sets the high-192 bits to the high-192 bits of the base keylet
        //   and the low bits to 0.
        // (token & nft::pageMask) sets the low-96 (low-64 for CFT) bits to the low-96 bits
        //   of the NFT ID, and the high-192 bits to 0.
        // +'ing them together combines the high-192 bits from (k.key & ~nft::pageMask) and the
        //   low-96 bits from (token & nft::pageMask) into one 256 bit number.
        //
        // In our case, we simply start with a copy of base's 256 bit key, then set the
        // low-64 bits of that copy to the low-64 bits of the CFT ID.
        let mut key_arr = base.key.data();
        key_arr[24..32].copy_from_slice(&cftoken_id.as_ref()[24..32]);
        Keylet::new(CFTOKEN_PAGE_TYPE as i16, UInt256::new(key_arr))
    }

    pub fn cftpage_max(owner: &AccountId) -> Keylet {
        let mut key_arr = PAGE_MASK;
        let holder_id_keylet = get_holder_id(owner);
        key_arr[..24].copy_from_slice(&holder_id_keylet.key.as_ref()[..24]);
        Keylet::new(CFTOKEN_PAGE_TYPE as i16, UInt256::new(key_arr))
    }

    fn get_holder_id(owner: &AccountId) -> Keylet {
        // We construct a Keylet so we can get the SHA512-Half of 0x007E concatenated with the owner
        // AccountId so we can set the high 192 bits of the page keylet to the high 192 bits
        // of holder_id_keylet's key
        let holder_id_keylet = Keylet::builder(CFT_ISSUANCE_TYPE as i16, CFT_ISSUANCE_TYPE)
            .key(owner)
            .build();
        holder_id_keylet
    }
}